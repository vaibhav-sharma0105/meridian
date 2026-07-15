## Tasks

- [x] **1. Add ONNX Runtime dependencies** — Add `ort` crate and tokenizers to Cargo.toml for ONNX model inference. Files: `src-tauri/Cargo.toml`. Acceptance: Cargo builds successfully with new dependencies.

- [x] **2. Bundle MiniLM-L6-v2 ONNX model** — Download and bundle MiniLM-L6-v2 ONNX model (~80MB) and tokenizer in `src-tauri/resources/models/all-MiniLM-L6-v2/`. Update `tauri.conf.json` to include resources.

- [x] **3. Implement BundledEmbedder struct** — Create `src-tauri/src/ai/embeddings.rs` with ONNX inference. Load model lazily via OnceCell, implement `embed(text) -> Vec<f32>` with mean pooling and L2 normalization. Returns 384-dim vectors.

- [x] **4. Create EmbeddingProvider trait** — Define trait in `embeddings.rs` with `embed()`, `dimensions()`, `provider_name()`. Implement for BundledEmbedder and OllamaEmbedder. Add provider selection function based on settings.

- [x] **5. Add embedding provider to settings UI** — Add provider dropdown to `AISettings.tsx`: "Bundled (offline)", "Ollama", "OpenAI". Show Ollama status indicator. Warning when switching providers.

- [x] **6. Add embeddings_ready column to documents** (already exists in schema) — New migration `v008_embeddings_ready.rs`: add `embeddings_ready BOOLEAN`, `embedding_provider TEXT`, `chunks_count INTEGER` to documents table.

- [x] **7. Implement document text chunking** — Create `src-tauri/src/ai/chunking.rs` with `chunk_text(text, chunk_size, overlap)`. Default 500 tokens, 50 overlap. Find natural break points.

- [x] **8. Create embed_document daemon job** — Add new job type in daemon: read document, chunk text, generate embeddings, store in Qdrant, update document status. Track progress percentage.

- [x] **9. Queue embedding job on document upload** — In `commands/documents.rs`, create `embed_document` job after successful upload. Set priority based on active project.

- [x] **10. Show embedding status on document card** — Update `DocumentCard.tsx`: spinner during embedding, checkmark when ready, warning on failure, info icon when no provider.

- [x] **11. Implement hybrid search function** — Create `src-tauri/src/ai/search.rs` with RRF fusion (k=60). Query Qdrant for semantic, FTS5 for keyword, merge with RRF, deduplicate by document_id.

- [x] **12. Update AI chat to use hybrid search** — Replace keyword-only search in `extractor.rs`. Include top-5 chunks in system prompt with source citations.

- [x] **13. Add search result UI with match badges** — Update document search to show match type badges: semantic (purple), keyword (blue), both (indigo). Highlight keyword matches.

- [x] **14. Add XLSX parser using calamine** — Add `calamine` crate. Create `src-tauri/src/documents/parsers/xlsx.rs`. Extract text from all sheets as markdown tables, limit 10K rows.

- [x] **15. Add PDF parser using pdf-extract** — Add `pdf-extract` crate. Create `pdf.rs` parser. Extract text with paragraph preservation. Warn for scanned PDFs.

- [x] **16. Implement document preview modal** — Create `DocumentPreview.tsx` component. Modal with rendered content (markdown/table). First page for PDF. Close on Escape/backdrop.

- [x] **17. Support bulk document upload** — Update `DocumentUpload.tsx` for multiple files. Add `BulkUploadProgress.tsx`. Drag-and-drop multiple. Skip unsupported with warning.

- [x] **18. Migrate existing documents** — On app start, check for documents with `embeddings_ready = false`. Queue embedding jobs with low priority. Show "Indexing..." banner.

- [x] **19. Ensure Qdrant collection per dimension** — Create collections named `documents_384`, `documents_1536` etc. Query correct collection based on provider dimensions.

- [x] **20. Handle provider switch re-embedding** — On provider change in settings, show confirmation dialog. If confirmed, queue re-embedding for all documents.

- [x] **21. Add embedding fallback UI** — In document search and AI chat, show "Using keyword search only" when no embeddings. Link to embedding settings.

- [x] **22. Update CLAUDE.md and ARCHITECTURE.md** — Document embedding provider selection, chunking strategy, hybrid search (RRF), new daemon job type.

- [x] **23. Update Playwright tests** — Add E2E tests for embedding status, document preview, bulk upload, search badges. Update tauri-mock.ts with new commands.
