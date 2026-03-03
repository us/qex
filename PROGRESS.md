# 📋 Progress Plan: Phase 5 — Polish, Bug Fixes, Optimization & Testing

> Created: 2026-03-03 | Status: 🔄 In Progress | Completed: 0/14

## 🎯 Objective
Phase 4 sonrası dense search entegrasyonundaki bug'ları düzeltmek, performansı
optimize etmek, gerçek projede end-to-end test yapmak ve tüm kodu git'e commit etmek.

## 📊 Progress Overview
- Total tasks: 14
- Completed: 3
- In Progress: 1
- Remaining: 10

---

## Tasks

### Phase 1: Bug Fixes

- [x] **Task 1.1**: Fix incremental dense index rebuild — lost vectors bug
  - Files: `crates/code-context-core/src/index/mod.rs`
  - Details: Incremental index'te dosya silinince `dense.clear()` tüm vektörleri siliyor ama sadece yeni chunk'ları ekliyor — değişmemiş dosyaların vektörleri kayboluyor. Fix: clear sonrası tüm proje chunk'larını yeniden embed etmek yerine, dosya bazında silme yapıp (remove_by_file) sadece değişen dosyaları yeniden embed etmek.
  - Tests: Unit test — index 3 files, modify 1, verify all 3 files' vectors still exist

- [x] **Task 1.2**: Fix compiler warning — dead code `force` field without dense
  - Files: `crates/code-context-mcp/src/tools.rs`
  - Details: `DownloadModelParams.force` field is dead code when dense feature disabled. Add `#[allow(dead_code)]` or cfg gate.
  - Tests: `cargo build` without dense — no warnings

- [x] **Task 1.3**: Fix dense-only search results missing from RRF output
  - Files: `crates/code-context-core/src/index/mod.rs`
  - Details: Hybrid search'te dense-only sonuçlar (BM25'te olmayan) RRF'te chunk_id ile map'lenemiyor çünkü BM25 map'inde yoklar. Dense-only sonuçları BM25'ten ayrıca çekmek lazım.
  - Tests: Verify dense-only results appear in hybrid output

### Phase 2: Performance Optimization

- [x] **Task 2.1**: Optimize embedding threading
  - Files: `crates/code-context-core/src/search/dense.rs`
  - Details: add_chunks batch_size=32 ile sıralı çalışıyor. Multiple batches'ı parallel embed et (model thread-safe değil, ama batch'ler arası IO wait'i overlap edebiliriz). Alternatif: ort intra_threads'i 4'ten optimize et.
  - Tests: Benchmark — 397 chunk embedding time before/after

- [x] **Task 2.2**: Skip re-embedding unchanged chunks on full_index (SKIPPED — low priority, ~20s acceptable for 400 chunks)
  - Files: `crates/code-context-core/src/index/mod.rs`
  - Details: full_index force=true ile bile, eğer dense index mevcutsa ve chunk_id'ler aynıysa, embed'i skip et. chunk_id hash'i content'e bağlı olduğundan güvenli.
  - Tests: full_index twice — second should be much faster

- [x] **Task 2.3**: Reduce binary size (SKIPPED — strip=true already in profile, 36MB acceptable)
  - Files: `Cargo.toml`
  - Details: Release profile'da `strip = true` zaten var. ort'un download ettiği dylib'leri kontrol et. ORT_DYLIB_PATH ile shared lib kullanma seçeneği araştır.
  - Tests: `ls -lh target/release/code-context`

### Phase 3: End-to-End Testing on Real Projects

- [x] **Task 3.1**: Clone FastAPI and test hybrid search
  - Details: `git clone https://github.com/tiangolo/fastapi /tmp/fastapi-test`. Index with dense. Test problem queries: "OpenAPI schema generation", "Starlette integration", "dependency injection container", "middleware authentication"
  - Tests: Compare BM25-only vs hybrid results for each query

- [x] **Task 3.2**: Test on code-context itself — semantic queries (tested in Phase 4, "embedding model" query found correct files)
  - Details: Test queries that require semantic understanding: "how does the system detect file changes", "what handles tokenization", "cosine similarity search"
  - Tests: Verify results are semantically relevant, not just keyword matches

- [x] **Task 3.3**: Test edge cases (empty query, special chars, unicode, long query — all pass, fixed special char crash)
  - Details: Empty query, very long query (>512 tokens), special chars, Unicode, single-file project, 0-result queries
  - Tests: No crashes, graceful error handling

### Phase 4: Code Quality & Git

- [x] **Task 4.1**: Add .gitignore entries for runtime files
  - Files: `.gitignore`
  - Details: Add entries for `.code-context/`, `*.onnx`, IDE files, `.env`
  - Tests: `git status` shows clean set of files

- [x] **Task 4.2**: Update .claude/CLAUDE.md with dense search info
  - Files: `.claude/CLAUDE.md`
  - Details: Document dense feature flag, model download, hybrid search behavior
  - Tests: N/A

- [🔄] **Task 4.3**: Initial git commit
  - Details: Stage all source files, create initial commit with full project
  - Tests: `git log` shows commit, `git status` clean

### Phase 5: Verification

- [ ] **Task 5.1**: Full test suite — both modes
  - Tests: `cargo test` (41 pass), `cargo test --features dense` (46 pass), zero warnings

- [ ] **Task 5.2**: Release build verification
  - Tests: `cargo build --release --features dense`, binary size check, MCP server starts

---

## 📝 Notes & Decisions
| # | Note | Date |
|---|------|------|
| - | - | - |

## 🐛 Issues Encountered
| # | Issue | Status | Resolution |
|---|-------|--------|------------|
| - | - | - | - |

## ➕ Added Tasks (discovered during execution)
