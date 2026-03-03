//! ONNX-based text embedding for dense vector search.
//!
//! Uses snowflake-arctic-embed-s (33MB quantized, 384-dim, 512 token max).
//! Only compiled when the `dense` feature is enabled.

use anyhow::{Context, Result};
use ndarray::Array2;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::info;

/// Query prefix for arctic-embed models (asymmetric retrieval)
const QUERY_PREFIX: &str = "Represent this sentence for searching relevant passages: ";

/// Maximum sequence length for the model
const MAX_SEQ_LEN: usize = 512;

/// Embedding model backed by ONNX Runtime
pub struct EmbeddingModel {
    session: Session,
    tokenizer: Tokenizer,
    dimensions: usize,
}

impl EmbeddingModel {
    /// Load model from a directory containing model.onnx and tokenizer.json
    pub fn load(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        anyhow::ensure!(model_path.exists(), "model.onnx not found in {}", model_dir.display());
        anyhow::ensure!(tokenizer_path.exists(), "tokenizer.json not found in {}", model_dir.display());

        // Use available CPU cores for ONNX parallelism
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let session = Session::builder()
            .context("Failed to create ONNX session builder")?
            .with_intra_threads(num_threads)
            .context("Failed to set thread count")?
            .commit_from_file(&model_path)
            .context("Failed to load ONNX model")?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        info!("Loaded embedding model from {}", model_dir.display());

        Ok(Self {
            session,
            tokenizer,
            dimensions: 384, // arctic-embed-s output dimension
        })
    }

    /// Get the default model directory
    pub fn default_model_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir().context("Cannot determine home directory")?;
        Ok(home.join(".qex/models/arctic-embed-s"))
    }

    /// Check if the model is downloaded
    pub fn is_available() -> bool {
        Self::default_model_dir()
            .map(|d| d.join("model.onnx").exists() && d.join("tokenizer.json").exists())
            .unwrap_or(false)
    }

    /// Embedding dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Encode a single text into an embedding vector
    pub fn encode(&mut self, text: &str) -> Result<Vec<f32>> {
        let results = self.encode_batch(&[text])?;
        Ok(results.into_iter().next().unwrap())
    }

    /// Encode a query (adds the asymmetric retrieval prefix)
    pub fn encode_query(&mut self, query: &str) -> Result<Vec<f32>> {
        let prefixed = format!("{}{}", QUERY_PREFIX, query);
        self.encode(&prefixed)
    }

    /// Encode a batch of texts into embedding vectors
    pub fn encode_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize all texts
        let encodings: Vec<_> = texts
            .iter()
            .map(|t| {
                self.tokenizer
                    .encode(*t, true)
                    .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        // Find max length (capped at MAX_SEQ_LEN)
        let max_len = encodings
            .iter()
            .map(|e| e.get_ids().len().min(MAX_SEQ_LEN))
            .max()
            .unwrap_or(0);

        let batch_size = texts.len();

        // Build padded tensors
        let mut input_ids = Array2::<i64>::zeros((batch_size, max_len));
        let mut attention_mask = Array2::<i64>::zeros((batch_size, max_len));
        let mut token_type_ids = Array2::<i64>::zeros((batch_size, max_len));

        for (i, encoding) in encodings.iter().enumerate() {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            let types = encoding.get_type_ids();
            let len = ids.len().min(max_len);

            for j in 0..len {
                input_ids[[i, j]] = ids[j] as i64;
                attention_mask[[i, j]] = mask[j] as i64;
                token_type_ids[[i, j]] = types[j] as i64;
            }
        }

        // Run inference
        let input_ids_tensor = Tensor::from_array(input_ids)?;
        let attention_mask_tensor = Tensor::from_array(attention_mask.clone())?;
        let token_type_ids_tensor = Tensor::from_array(token_type_ids)?;

        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
            "token_type_ids" => token_type_ids_tensor,
        ])?;

        // Extract output: [batch_size, seq_len, hidden_dim]
        let (shape, raw_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract output tensor")?;

        let hidden_dim = *shape.last().unwrap_or(&384) as usize;
        let seq_len_out = if shape.len() >= 2 { shape[1] as usize } else { max_len };

        // Mean pooling with attention mask over raw tensor data
        // raw_data layout: [batch_size * seq_len_out * hidden_dim]
        let mut embeddings = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let mut pooled = vec![0.0f32; hidden_dim];
            let mut count = 0.0f32;

            for j in 0..seq_len_out {
                let mask_val = attention_mask[[i, j.min(max_len - 1)]] as f32;
                if mask_val > 0.0 {
                    let offset = (i * seq_len_out + j) * hidden_dim;
                    for k in 0..hidden_dim {
                        pooled[k] += raw_data[offset + k] * mask_val;
                    }
                    count += mask_val;
                }
            }

            // Divide by count
            if count > 0.0 {
                for v in &mut pooled {
                    *v /= count;
                }
            }

            // L2 normalize
            let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut pooled {
                    *v /= norm;
                }
            }

            embeddings.push(pooled);
        }

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model_load_and_encode() {
        let model_dir = EmbeddingModel::default_model_dir().unwrap();
        if !model_dir.join("model.onnx").exists() {
            eprintln!("Skipping test: model not downloaded. Run scripts/download-model.sh");
            return;
        }

        let mut model = EmbeddingModel::load(&model_dir).unwrap();
        assert_eq!(model.dimensions(), 384);

        // Test single encode
        let embedding = model.encode("authentication middleware").unwrap();
        assert_eq!(embedding.len(), 384);

        // Verify L2 normalized (norm ≈ 1.0)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "norm={}", norm);

        // Test query encode (with prefix)
        let query_emb = model.encode_query("authentication").unwrap();
        assert_eq!(query_emb.len(), 384);

        // Test batch encode
        let batch = model
            .encode_batch(&["hello world", "code search", "database connection"])
            .unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].len(), 384);

        // Verify different texts produce different embeddings
        assert_ne!(batch[0], batch[1]);
        assert_ne!(batch[1], batch[2]);
    }
}
