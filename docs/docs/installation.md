# Installation

## From Source

```
git clone https://github.com/us/qex
cd qex
cargo build --release
```

Binary will be at `target/release/qex` (~19 MB).

### With Dense Vector Search

```
cargo build --release --features dense
```

Binary will be ~36 MB. Requires downloading the embedding model:

```
./scripts/download-model.sh
```

Or via MCP tool:

```json
{"tool": "download_model", "arguments": {"force": false}}
```

The model (Arctic Embed S, 33 MB INT8 quantized) is stored at `~/.qex/models/arctic-embed-s/`.

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `dense` | off | Enables dense vector search (ONNX Runtime + HNSW). Adds `ort`, `ndarray`, `usearch`, `tokenizers` dependencies |

**BM25-only mode** (default) is recommended for most use cases. Dense search adds ~50ms latency per query and requires the embedding model download.

## Claude Code Setup

Add to `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "qex": {
      "command": "/path/to/qex",
      "args": []
    }
  }
}
```

Or if installed via `cargo install`:

```json
{
  "mcpServers": {
    "qex": {
      "command": "qex",
      "args": []
    }
  }
}
```

For verbose debug logging:

```json
{
  "mcpServers": {
    "qex": {
      "command": "qex",
      "args": ["--verbose"]
    }
  }
}
```

## Requirements

- Rust 1.75+ (for building from source)
- Docker is **not** required
- No external services — everything runs locally
