# qex

Lightweight Rust MCP server for semantic code search using BM25 (tantivy) + optional dense vector search (ONNX) + tree-sitter chunking.

- **GitHub**: https://github.com/us/qex
- **crates.io**: https://crates.io/crates/qex-mcp / https://crates.io/crates/qex-core

## Architecture

- `crates/qex-core/` - Core library: chunking, merkle, search, indexing
- `crates/qex-mcp/` - MCP server binary (stdio transport via rmcp)

## Build & Test

```bash
cargo test                              # BM25-only tests (41 tests)
cargo test --features dense             # Full tests including dense search (46 tests)
cargo build --release                   # BM25-only binary (~19 MB)
cargo build --release --features dense  # With dense vector search (~36 MB)
```

## Dense Vector Search (optional)

Uses snowflake-arctic-embed-s (33MB ONNX INT8, 384-dim, 512 token max) for semantic embeddings.

```bash
# Download model
scripts/download-model.sh
# Or via MCP tool: download_model

# Model stored at: ~/.qex/models/arctic-embed-s/
```

When model is available, search automatically uses hybrid BM25 + dense with Reciprocal Rank Fusion. Falls back to BM25-only if model not downloaded.

### Dense feature deps
- `ort 2.0.0-rc.11` - ONNX Runtime (requires ndarray 0.17 to match ort's version)
- `usearch 2.24` - HNSW vector index
- `tokenizers 0.22` - HuggingFace tokenizer (needs `fancy-regex` feature)

## MCP Integration

```bash
claude mcp add qex --scope user -- ~/.local/bin/qex
```

## Key Dependencies

- `tantivy 0.22` - BM25 full-text search
- `tree-sitter 0.24` - Code parsing (10 languages)
- `rmcp 0.17` - MCP server framework
- `ignore 0.4` - Gitignore-compatible file walking
- `rusqlite 0.32` - SQLite metadata (bundled)

## Conventions

- All logs go to stderr (stdout reserved for MCP JSON-RPC)
- Storage: `~/.qex/projects/{name}_{hash}/`
- Dense index: `~/.qex/projects/{name}_{hash}/dense/`
- rmcp uses `Parameters<T>` wrapper for tool params, `ErrorData` for errors
- schemars must use `rmcp::schemars::JsonSchema` (v1, not standalone 0.8)
- `#[cfg(feature = "dense")]` for all dense-search code paths
- `session.run()` in ort 2.0 requires `&mut self` — encode methods are `&mut self`
