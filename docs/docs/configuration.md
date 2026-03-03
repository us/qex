# Configuration

## CLI Arguments

```
qex [OPTIONS]

Options:
  -v, --verbose    Enable verbose debug logging
  -h, --help       Print help
  -V, --version    Print version
```

qex has minimal configuration. It runs as a stdio MCP server with no config files. All behavior is controlled through MCP tool parameters.

## Feature Flags

Build-time feature flags control which search engines are available:

| Flag | Default | Binary Size | Description |
|------|---------|-------------|-------------|
| (none) | on | ~19 MB | BM25-only search via Tantivy |
| `dense` | off | ~36 MB | BM25 + dense vector search via ONNX + HNSW |

```
# BM25-only (recommended)
cargo build --release

# BM25 + dense
cargo build --release --features dense
```

## Storage

All data is stored under `~/.qex/`:

```
~/.qex/
├── projects/           # Per-project index data
│   └── {name}_{hash}/
│       ├── project_info.json
│       ├── tantivy/
│       ├── dense/          # Only with dense feature
│       ├── snapshot.json
│       ├── snapshot_metadata.json
│       └── stats.json
└── models/             # Embedding models (dense feature only)
    └── arctic-embed-s/
```

Project directories use a hash suffix derived from the absolute path, preventing collisions when multiple projects share the same directory name.

## Default Ignore Patterns

qex respects `.gitignore` rules and also applies 50+ built-in ignore patterns:

**Directories:**
`node_modules`, `target`, `build`, `dist`, `out`, `.next`, `.nuxt`, `vendor`, `third_party`, `.venv`, `__pycache__`, `.git`, `.svn`, `.hg`, `.idea`, `.vscode`, `.vs`, `coverage`, `.cache`, `.turbo`, `tmp`, `.bundle`, `.gradle`, `pods`

**Files:**
Images (`.png`, `.jpg`, `.gif`, `.svg`, `.ico`, `.webp`), fonts (`.woff`, `.woff2`, `.ttf`, `.eot`), archives (`.zip`, `.tar`, `.gz`), compiled objects (`.o`, `.so`, `.dylib`, `.dll`), lock files, minified files (`.min.js`, `.min.css`), source maps (`.map`)

## Embedding Model

When using the `dense` feature, the embedding model is:

| Property | Value |
|----------|-------|
| Model | Snowflake Arctic Embed S |
| Format | ONNX (INT8 quantized) |
| Size | 33 MB |
| Dimensions | 384 |
| Location | `~/.qex/models/arctic-embed-s/` |
| Download | Via `download_model` MCP tool or `scripts/download-model.sh` |

## Merkle Snapshot TTL

Snapshots older than **5 minutes** trigger a re-check on the next `index_codebase` or `search_code` call. This ensures the index stays reasonably fresh without constant filesystem scanning.

## Dependencies

### BM25 Mode (Default)

| Crate | Version | Purpose |
|-------|---------|---------|
| tantivy | 0.22 | BM25 full-text search |
| tree-sitter | 0.24 | Code parsing (10+ language grammars) |
| rmcp | 0.17 | MCP server framework |
| rusqlite | 0.32 | SQLite metadata (bundled) |
| ignore | 0.4 | Gitignore-compatible file walking |
| rayon | 1.10 | Parallel chunking |
| sha2 | 0.10 | Merkle DAG hashing |
| clap | 4 | CLI argument parsing |

### Dense Mode (Additional)

| Crate | Version | Purpose |
|-------|---------|---------|
| ort | 2.0.0-rc.11 | ONNX Runtime |
| ndarray | 0.16 | Tensor operations |
| usearch | 2.24 | HNSW vector index |
| tokenizers | 0.22 | HuggingFace tokenizer |
