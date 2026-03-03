<h1 align="center">QEX</h1>

<p align="center">
  <strong>Lightweight MCP server for semantic code search</strong>
</p>

<p align="center">
  BM25 + optional dense vectors + tree-sitter chunking
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-AGPL--3.0-blue.svg" alt="License: AGPL-3.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2021_edition-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <strong>English</strong> | <a href="README.zh-CN.md">中文</a>
</p>

---

QEX is a high-performance MCP server for semantic code search built in Rust. It combines BM25 full-text search with optional dense vector embeddings for hybrid retrieval — delivering Cursor-quality search from a single ~19 MB binary. Tree-sitter parsing understands code structure (functions, classes, methods), Merkle DAG change detection enables incremental indexing, and everything runs locally with zero cloud dependencies.

## What's New

- **Hybrid Search** — BM25 + dense vector search with Reciprocal Rank Fusion for 48% better accuracy than dense-only retrieval
- **10 Language Support** — Python, JavaScript, TypeScript, Rust, Go, Java, C, C++, C#, Markdown via tree-sitter
- **Incremental Indexing** — Merkle DAG change detection, only re-indexes what changed
- **Optional Dense Vectors** — snowflake-arctic-embed-s (33 MB, 384-dim, INT8 quantized) via ONNX Runtime
- **MCP Native** — plugs directly into Claude Code as a tool server via stdio

## Why QEX?

Claude Code uses grep + glob for code search — effective but token-hungry and lacks semantic understanding. Cursor uses vector embeddings with cloud indexing (~3.5 GB stack). **QEX** is the middle ground:

- **BM25 + Dense Hybrid**: 48% better accuracy than dense-only retrieval ([Superlinked 2025](https://superlinked.com/vectorhub/articles/optimizing-rag-with-hybrid-search-reranking))
- **Tree-sitter Chunking**: Understands code structure — functions, classes, methods — not just lines
- **Incremental Indexing**: Merkle DAG change detection, only re-indexes what changed
- **Zero Cloud Dependencies**: Everything runs locally via ONNX Runtime
- **MCP Native**: Plugs directly into Claude Code as a tool server

## Quick Start

```bash
# Build (BM25-only, ~19 MB)
cargo build --release

# Or with dense vector search (~36 MB)
cargo build --release --features dense

# Install
cp target/release/qex ~/.local/bin/

# Add to Claude Code
claude mcp add qex --scope user -- ~/.local/bin/qex
```

That's it. Claude will now have access to `search_code` and `index_codebase` tools.

### Enable Dense Search (Optional)

Dense search adds semantic understanding — finding "authentication middleware" even when the code says `verify_token`.

```bash
# Download the embedding model (~33 MB)
./scripts/download-model.sh

# Or via MCP tool (after adding to Claude)
# Claude: "download the embedding model"
```

**Model**: [snowflake-arctic-embed-s](https://huggingface.co/Snowflake/snowflake-arctic-embed-s) — 384-dim, INT8 quantized, 512 token max.

When the model is present, search automatically switches to hybrid mode. No configuration needed.

## Architecture

```
Claude Code ──(stdio/JSON-RPC)──▶ qex
                                      │
                      ┌───────────────┼───────────────┐
                      ▼               ▼               ▼
                 tree-sitter      tantivy        ort + usearch
                  Chunking         BM25         Dense Vectors
                 (11 langs)       (<1ms)         (optional)
                      │               │               │
                      └───────┬───────┘               │
                              ▼                       │
                      Ranking Engine ◄────────────────┘
                    (RRF + multi-factor)
                              │
                              ▼
                      Ranked Results
```

### How Search Works

1. **Query Analysis** — Tokenization, stop-word removal, intent detection
2. **BM25 Search** — Full-text search via tantivy with field boosts (name, content, tags, path)
3. **Dense Search** _(optional)_ — Embed query → HNSW cosine similarity → top-k vectors
4. **Reciprocal Rank Fusion** — Merge BM25 and dense results: `score = Σ 1/(k + rank)`
5. **Multi-factor Ranking** — Re-rank by chunk type, name match, path relevance, tags, docstring presence
6. **Test Penalty** — Down-rank test files (0.7×) to prioritize implementation code

### How Indexing Works

1. **File Walking** — Respects `.gitignore`, filters by extension
2. **Tree-sitter Parsing** — Language-aware AST traversal, extracts functions/classes/methods
3. **Chunk Enrichment** — Tags (async, auth, database...), complexity score, docstrings, decorators
4. **BM25 Indexing** — 14-field tantivy schema with per-field boosts
5. **Dense Indexing** _(optional)_ — Batch embedding (64 chunks/batch) → HNSW index
6. **Merkle Snapshot** — SHA-256 DAG for incremental change detection

## MCP Tools

### `index_codebase`
Index a project for semantic search.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Absolute path to project directory |
| `force` | boolean | no | Force full re-index (default: false) |
| `extensions` | string[] | no | Only index specific extensions, e.g. `["py", "rs"]` |

Returns file count, chunk count, detected languages, and timing.

### `search_code`
Search the indexed codebase with natural language or keywords.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Absolute path to project directory |
| `query` | string | yes | Search query (natural language or keywords) |
| `limit` | integer | no | Max results (default: 10) |
| `extension_filter` | string | no | Filter by extension, e.g. `"py"` |

Auto-indexes if needed. Returns ranked results with code snippets, file paths, line numbers, and relevance scores.

### `get_indexing_status`
Check if a project is indexed and get stats.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Absolute path to project directory |

Returns index status, file/chunk counts, languages, and whether dense search is available.

### `clear_index`
Delete all index data for a project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Absolute path to project directory |

### `download_model`
Download the embedding model for dense search. Requires the `dense` feature.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `force` | boolean | no | Re-download even if exists (default: false) |

## Supported Languages

| Language | Extensions | Chunk Types |
|----------|------------|-------------|
| Python | `.py`, `.pyi` | function, method, class, module-level, imports |
| JavaScript | `.js` | function, method, class, module-level |
| TypeScript | `.ts`, `.tsx` | function, method, class, interface, module-level |
| Rust | `.rs` | function, method, struct, enum, trait, impl, macro |
| Go | `.go` | function, method, struct, interface |
| Java | `.java` | method, class, interface, enum |
| C | `.c`, `.h` | function, struct |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp` | function, method, class, struct, namespace |
| C# | `.cs` | method, class, struct, interface, enum, namespace |
| Markdown | `.md` | section, document |

## Project Structure

```
qex/
├── Cargo.toml                        # Workspace root
├── scripts/
│   └── download-model.sh             # Model download script
├── crates/
│   ├── qex-core/            # Core library
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── chunk/                # Tree-sitter chunking engine
│   │       │   ├── tree_sitter.rs    # AST traversal
│   │       │   ├── multi_language.rs # Language dispatcher
│   │       │   └── languages/        # 11 language implementations
│   │       ├── search/               # Search engines
│   │       │   ├── bm25.rs           # Tantivy BM25 index
│   │       │   ├── dense.rs          # HNSW vector index (optional)
│   │       │   ├── embedding.rs      # ONNX embeddings (optional)
│   │       │   ├── hybrid.rs         # Reciprocal Rank Fusion (optional)
│   │       │   ├── ranking.rs        # Multi-factor re-ranking
│   │       │   └── query.rs          # Query analysis
│   │       ├── index/                # Incremental indexer
│   │       │   ├── mod.rs            # Main indexing logic
│   │       │   └── storage.rs        # Project storage layout
│   │       ├── merkle/               # Change detection
│   │       │   ├── mod.rs            # Merkle DAG
│   │       │   ├── change_detector.rs
│   │       │   └── snapshot.rs
│   │       └── ignore.rs             # Gitignore-aware file walking
│   │
│   └── qex-mcp/            # MCP server binary
│       └── src/
│           ├── main.rs               # Entry point, stdio transport
│           ├── server.rs             # Tool handlers
│           ├── tools.rs              # Parameter schemas
│           └── config.rs             # CLI args
│
└── tests/fixtures/                   # Test source files
```

## Storage

All data is stored locally under `~/.qex/`:

```
~/.qex/
├── projects/
│   └── {name}_{hash}/         # Per-project index
│       ├── tantivy/           # BM25 index
│       ├── dense/             # Vector index (optional)
│       ├── snapshot.json      # Merkle DAG
│       └── stats.json         # Index stats
│
└── models/
    └── arctic-embed-s/        # Embedding model (optional)
        ├── model.onnx         # 33 MB, INT8 quantized
        └── tokenizer.json
```

## Build & Test

```bash
# Run tests (BM25-only)
cargo test                              # 41 tests

# Run tests (with dense search)
cargo test --features dense             # 46 tests

# Build for release
cargo build --release                   # ~19 MB binary
cargo build --release --features dense  # ~36 MB binary
```

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tantivy | 0.22 | BM25 full-text search |
| tree-sitter | 0.24 | Code parsing (11 languages) |
| rmcp | 0.17 | MCP server framework (stdio) |
| rusqlite | 0.32 | SQLite metadata (bundled) |
| ignore | 0.4 | Gitignore-compatible file walking |
| rayon | 1.10 | Parallel chunking |
| ort | 2.0.0-rc.11 | ONNX Runtime _(optional, dense)_ |
| usearch | 2.24 | HNSW vector index _(optional, dense)_ |
| tokenizers | 0.22 | HuggingFace tokenizer _(optional, dense)_ |

## Performance

Benchmarked on an Apple Silicon Mac:

| Metric | Value |
|--------|-------|
| Full index (400 chunks) | ~20s with dense, ~2s BM25-only |
| Incremental index (no changes) | <100ms |
| BM25 search | <5ms |
| Hybrid search | ~50ms (includes embedding) |
| Binary size | 19 MB (BM25) / 36 MB (dense) |
| Model size | 33 MB (INT8 quantized) |

## License

[AGPL-3.0](LICENSE)
