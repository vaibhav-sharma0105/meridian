## Why

Documents are uploaded but not meaningfully searchable. The current FTS5 keyword search misses semantic matches, and embeddings require manual Ollama setup that most users skip. Without automatic embeddings and intelligent retrieval, uploaded documents become digital filing cabinets rather than accessible knowledge that AI can leverage.

## What Changes

- **Bundled embedding model**: Ship MiniLM-L6-v2 (~80MB) with the app for zero-config embeddings
- **Auto-embed pipeline**: Automatically queue documents for embedding on upload with background processing
- **Multi-provider embeddings**: Support bundled model, Ollama, OpenAI, and Anthropic with graceful fallback
- **Hybrid semantic search**: Combine Qdrant vector similarity with FTS5 keyword search using RRF fusion
- **Document parsing improvements**: Proper XLSX support, improved PDF extraction, document preview
- **Bulk upload**: Support uploading multiple documents at once

## Capabilities

### New Capabilities
- `bundled-embeddings`: Ship and run MiniLM-L6-v2 via ONNX runtime for offline embedding generation
- `auto-embed-pipeline`: Background job that processes embedding queue with progress tracking and retry logic
- `hybrid-search`: Reciprocal Rank Fusion combining semantic and keyword search results

### Modified Capabilities
- `document-management`: Add XLSX parsing, improved PDF extraction, document preview, bulk upload support

## Impact

- **Cargo.toml**: Add ort (ONNX Runtime), pdf-extract dependencies
- **App bundle size**: +80MB for MiniLM-L6-v2 ONNX model
- **Background daemon**: New embedding job type in job queue
- **AI chat**: Automatically retrieves relevant document chunks via hybrid search
- **Settings UI**: Embedding provider selection (bundled/Ollama/OpenAI/Anthropic)
- **Document UI**: Progress indicator during embedding, preview component
