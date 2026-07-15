## Context

Meridian is a Tauri v2 desktop app with React frontend and Rust backend. Current document state:

- **Storage**: Documents stored in `~/.meridian/documents/` with metadata in SQLite
- **Search**: FTS5 keyword search only — misses semantic matches
- **Embeddings**: Requires manual Ollama setup; most users don't configure it
- **AI Context**: Chat uses keyword search to find relevant chunks

Phase 0 established encrypted SQLite (SQLCipher) and Qdrant for vector storage. Phase 1 builds on this foundation to make documents automatically searchable without external dependencies.

**Constraints:**
- Must work completely offline (bundled model, no cloud required)
- Must not block UI during embedding operations
- Must support existing documents (migration path)
- Binary size increase should be reasonable (~100MB max for model)

## Goals / Non-Goals

**Goals:**
- Ship bundled embedding model for zero-config semantic search
- Automatically embed documents on upload via daemon
- Combine semantic and keyword search for best results
- Improve document parsing (XLSX, PDF) and UX (preview, bulk upload)

**Non-Goals:**
- OCR for scanned PDFs (requires large models, adds significant binary size)
- Document editing (Meridian is for ingestion and retrieval, not authoring)
- Cloud-based embedding APIs as primary (local-first principle)
- Real-time collaborative editing

## Decisions

### Decision 1: Embedding Model Selection

**Choice:** MiniLM-L6-v2 via ONNX Runtime

**Alternatives Considered:**
- **BGE-Small**: Better quality but ~140MB, slower inference
- **all-MiniLM-L12**: Higher quality but 2x latency
- **TinyBERT**: Smaller but significantly worse quality
- **GTE-Tiny**: Newer, untested in production

**Rationale:** MiniLM-L6-v2 is the industry standard for lightweight embeddings. ~80MB model size, 384 dimensions, excellent quality/speed tradeoff. ONNX format ensures cross-platform compatibility.

### Decision 2: Model Distribution

**Choice:** Bundle ONNX model in app resources

**Alternatives Considered:**
- **Download on first use**: Requires internet, poor first-run experience
- **Separate installer**: Friction for users, harder to update
- **WebAssembly ONNX**: Significantly slower than native

**Implementation:**
```
src-tauri/resources/
  └── models/
      └── all-MiniLM-L6-v2/
          ├── model.onnx (~80MB)
          └── tokenizer.json (~500KB)
```

Model loads on-demand (first embedding request), stays in memory for session.

**Rationale:** Bundling ensures offline capability from first launch. Binary size increase is acceptable for modern desktop apps.

### Decision 3: ONNX Runtime Integration

**Choice:** `ort` crate (official Rust bindings) with dynamic linking

**Alternatives Considered:**
- **tract**: Pure Rust but significantly slower
- **candle**: Hugging Face's Rust ML, newer/less stable
- **static linking**: Larger binary, harder to update

**Implementation:**
```rust
// src-tauri/src/ai/embeddings.rs
use ort::{Environment, Session, Value};

pub struct BundledEmbedder {
    session: Session,
    tokenizer: Tokenizer,
}

impl BundledEmbedder {
    pub fn new() -> Result<Self, Error> {
        let model_path = app_handle.path().resource_dir()?.join("models/all-MiniLM-L6-v2/model.onnx");
        let session = Session::builder()?.with_model_from_file(model_path)?;
        // ...
    }
    
    pub fn embed(&self, text: &str) -> Vec<f32> {
        // Tokenize, run inference, mean pooling, normalize
    }
}
```

**Rationale:** `ort` is the official ONNX Runtime binding, well-maintained, handles cross-platform complexity.

### Decision 4: Provider Selection Architecture

**Choice:** Trait-based provider abstraction with runtime selection

**Implementation:**
```rust
// src-tauri/src/ai/embeddings.rs
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, Error>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}

pub struct BundledEmbedder { ... }  // MiniLM-L6-v2 via ONNX
pub struct OllamaEmbedder { ... }   // Ollama API
pub struct OpenAIEmbedder { ... }   // OpenAI API
pub struct AnthropicEmbedder { ... } // Anthropic API (if/when available)

pub fn get_embedding_provider(settings: &AppSettings) -> Box<dyn EmbeddingProvider> {
    match settings.embedding_provider.as_deref() {
        Some("ollama") if ollama_available() => Box::new(OllamaEmbedder::new()),
        Some("openai") if has_openai_key() => Box::new(OpenAIEmbedder::new()),
        _ => Box::new(BundledEmbedder::new()),  // Default fallback
    }
}
```

**Rationale:** Clean abstraction allows adding providers without modifying consumers. Bundled model is always available as fallback.

### Decision 5: Embedding Job Processing

**Choice:** Extend existing daemon job queue with new `embed_document` job type

**Schema extension:**
```sql
-- daemon_jobs.job_type = 'embed_document'
-- daemon_jobs.payload contains:
{
  "document_id": "uuid",
  "project_id": "uuid",
  "file_path": "/path/to/file",
  "provider": "bundled"  // or "ollama", "openai"
}

-- Add to documents table:
ALTER TABLE documents ADD COLUMN embeddings_ready BOOLEAN DEFAULT FALSE;
ALTER TABLE documents ADD COLUMN embedding_provider TEXT;
ALTER TABLE documents ADD COLUMN chunks_count INTEGER DEFAULT 0;
```

**Daemon processing:**
```rust
// src-tauri/src/daemon/jobs/embed.rs
pub async fn process_embed_job(job: &DaemonJob) -> Result<(), Error> {
    let payload: EmbedJobPayload = serde_json::from_str(&job.payload)?;
    
    // 1. Read document content
    let content = read_document_content(&payload.file_path)?;
    
    // 2. Chunk text (500 tokens, 50 overlap)
    let chunks = chunk_text(&content, 500, 50);
    
    // 3. Embed each chunk
    let provider = get_embedding_provider(&settings);
    for (i, chunk) in chunks.iter().enumerate() {
        let embedding = provider.embed(chunk).await?;
        
        // 4. Store in Qdrant
        qdrant_client.upsert_point(
            "documents",
            &format!("{}_{}", payload.document_id, i),
            embedding,
            json!({
                "document_id": payload.document_id,
                "project_id": payload.project_id,
                "chunk_index": i,
                "text": chunk
            })
        ).await?;
        
        // 5. Update progress
        update_job_progress(job.id, (i + 1) * 100 / chunks.len());
    }
    
    // 6. Mark document as embedded
    mark_document_embedded(payload.document_id, chunks.len());
}
```

**Rationale:** Reuses Phase 0 daemon infrastructure. Job queue provides retry logic, progress tracking, and survives app restarts.

### Decision 6: Hybrid Search Implementation

**Choice:** Reciprocal Rank Fusion (RRF) combining Qdrant and FTS5

**Algorithm:**
```rust
// src-tauri/src/ai/search.rs
pub async fn hybrid_search(
    query: &str,
    project_id: &str,
    limit: usize
) -> Vec<SearchResult> {
    // 1. Semantic search
    let query_embedding = embedder.embed(query).await?;
    let semantic_results = qdrant_client.search(
        "documents",
        query_embedding,
        limit * 2,  // Oversample for RRF
        Filter::must(vec![FieldCondition::match_keyword("project_id", project_id)])
    ).await?;
    
    // 2. Keyword search
    let keyword_results = fts5_search(conn, query, project_id, limit * 2)?;
    
    // 3. RRF fusion (k=60 is standard)
    let k = 60.0;
    let mut scores: HashMap<String, f32> = HashMap::new();
    
    for (rank, result) in semantic_results.iter().enumerate() {
        let id = &result.payload["document_id"];
        *scores.entry(id.clone()).or_default() += 1.0 / (k + rank as f32 + 1.0);
    }
    
    for (rank, result) in keyword_results.iter().enumerate() {
        *scores.entry(result.document_id.clone()).or_default() += 1.0 / (k + rank as f32 + 1.0);
    }
    
    // 4. Sort by combined score, take top N
    let mut results: Vec<_> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results.truncate(limit);
    
    // 5. Hydrate with full content
    hydrate_search_results(results)
}
```

**Rationale:** RRF is proven effective for combining heterogeneous rankings. k=60 is the standard constant that balances contribution from both sources.

### Decision 7: Document Chunking Strategy

**Choice:** Fixed-size token chunking with overlap

**Parameters:**
- Chunk size: 500 tokens (~2000 characters)
- Overlap: 50 tokens (10%)
- Separator priority: paragraph break → sentence end → word boundary

**Implementation:**
```rust
// src-tauri/src/ai/chunking.rs
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let tokens = tokenize(text);
    let mut chunks = Vec::new();
    let mut start = 0;
    
    while start < tokens.len() {
        let end = (start + chunk_size).min(tokens.len());
        let chunk_tokens = &tokens[start..end];
        
        // Find natural break point near end
        let break_point = find_natural_break(chunk_tokens, end - start);
        let chunk = detokenize(&chunk_tokens[..break_point]);
        
        chunks.push(chunk);
        start += break_point - overlap;
    }
    
    chunks
}
```

**Rationale:** 500 tokens balances context preservation with retrieval precision. Overlap ensures no context is lost at boundaries.

### Decision 8: XLSX Parsing

**Choice:** `calamine` crate for XLSX reading

**Implementation:**
```rust
// src-tauri/src/documents/parsers/xlsx.rs
use calamine::{Reader, open_workbook, Xlsx};

pub fn parse_xlsx(path: &Path) -> Result<String, Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let mut output = String::new();
    
    for sheet_name in workbook.sheet_names().to_owned() {
        output.push_str(&format!("## {}\n\n", sheet_name));
        
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows().take(10_000) {  // Limit rows
                let row_text: Vec<String> = row.iter()
                    .map(|cell| cell.to_string())
                    .collect();
                output.push_str(&format!("| {} |\n", row_text.join(" | ")));
            }
        }
        output.push_str("\n");
    }
    
    Ok(output)
}
```

**Rationale:** `calamine` is the standard Rust crate for spreadsheet parsing, supports xlsx/xls/ods.

### Decision 9: PDF Extraction

**Choice:** `pdf-extract` crate for text extraction

**Implementation:**
```rust
// src-tauri/src/documents/parsers/pdf.rs
use pdf_extract::extract_text;

pub fn parse_pdf(path: &Path) -> Result<String, Error> {
    extract_text(path).map_err(|e| Error::ParseError(e.to_string()))
}
```

**Rationale:** `pdf-extract` handles most PDFs well. For scanned PDFs without text layer, extraction will be minimal but we clearly communicate this limitation.

### Decision 10: Qdrant Collection Configuration

**Choice:** Separate collection per embedding dimension

**Implementation:**
```rust
// On startup, ensure collection exists
pub async fn ensure_collection(client: &QdrantClient, dimension: usize) -> Result<(), Error> {
    let collection_name = format!("documents_{}", dimension);
    
    if !client.has_collection(&collection_name).await? {
        client.create_collection(&CollectionConfig {
            name: collection_name,
            vector_size: dimension as u64,
            distance: Distance::Cosine,
            ..Default::default()
        }).await?;
    }
}
```

**Rationale:** Different providers produce different dimensions (MiniLM: 384, OpenAI: 1536). Separate collections avoid dimension mismatch errors.

## Risks / Trade-offs

**[Risk: Binary size increase]** → Accept: ~100MB for ONNX model + runtime. Modern apps routinely exceed this.

**[Risk: First embedding latency]** → Mitigation: Model loads lazily on first request. Show loading indicator. Cache model in memory for session.

**[Risk: Qdrant dimension mismatch]** → Mitigation: Separate collections per dimension. When switching providers, warn user that existing embeddings use old provider.

**[Risk: ONNX Runtime compatibility]** → Mitigation: Use dynamic linking. Test on all platforms in CI.

**[Trade-off: Chunking vs context]** → Accept: 500-token chunks lose some context. Overlap partially compensates. Alternative (larger chunks) reduces retrieval precision.

**[Trade-off: RRF vs learned fusion]** → Accept: RRF is simple and effective. Learned fusion requires training data we don't have.

## Migration Plan

### Phase 1a: Bundled Embeddings
1. Add `ort` and tokenizer dependencies to Cargo.toml
2. Bundle MiniLM-L6-v2 ONNX model in resources
3. Implement `BundledEmbedder` struct
4. Add embedding provider selection to settings
5. Add provider fallback chain

### Phase 1b: Auto-Embed Pipeline
1. Add `embeddings_ready` column to documents table
2. Create `embed_document` job type in daemon
3. Implement document chunking
4. Queue embedding job on document upload
5. Show embedding progress in document card

### Phase 1c: Hybrid Search
1. Implement RRF fusion algorithm
2. Integrate with existing FTS5 search
3. Update AI chat context to use hybrid search
4. Add search result UI with match type badges

### Phase 1d: Document Improvements
1. Add `calamine` for XLSX parsing
2. Add `pdf-extract` for better PDF extraction
3. Implement document preview modal
4. Add bulk upload support

### Existing Document Migration
- On app start, check for documents with `embeddings_ready = false`
- Queue embedding jobs for all un-embedded documents
- Process in background with low priority
- Show "Indexing documents..." banner during migration

## Open Questions (Resolved)

1. **What happens when user switches embedding provider?** ✅ Show confirmation dialog, offer to re-embed all documents with new provider.

2. **How to handle very large documents?** ✅ Chunk as usual, but cap at 100 chunks per document (~50K tokens). Show warning for larger files.

3. **What if Qdrant is unavailable?** ✅ Fall back to FTS5-only search. Show warning in search results.

4. **Memory usage for ONNX model?** ✅ ~200MB when loaded. Acceptable for desktop app. Consider lazy unloading after 10 minutes of inactivity.
