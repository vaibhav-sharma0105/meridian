## ADDED Requirements

### Requirement: Bundled Embedding Model

The system SHALL ship with MiniLM-L6-v2 ONNX model (~80MB) for offline embedding generation without external dependencies.

#### Scenario: Generate embeddings offline
- **WHEN** user uploads a document with no internet connection and no Ollama running
- **THEN** system generates embeddings using the bundled MiniLM-L6-v2 model

#### Scenario: Model loads on demand
- **WHEN** first embedding request is made
- **THEN** system loads ONNX model into memory (one-time ~2-3 second delay)
- **AND** subsequent requests use cached model instance

### Requirement: ONNX Runtime Integration

The system SHALL use ONNX Runtime for cross-platform inference of the bundled embedding model.

#### Scenario: Cross-platform compatibility
- **WHEN** application runs on macOS, Windows, or Linux
- **THEN** ONNX Runtime executes model inference without platform-specific code

#### Scenario: Memory-efficient inference
- **WHEN** generating embeddings for large documents
- **THEN** system processes text in batches to limit memory usage below 500MB

### Requirement: Embedding Provider Selection

The system SHALL allow users to choose their embedding provider: bundled model, Ollama, OpenAI, or Anthropic.

#### Scenario: Default to bundled model
- **WHEN** user has not configured an embedding provider
- **THEN** system uses bundled MiniLM-L6-v2 model

#### Scenario: Switch to Ollama
- **WHEN** user selects Ollama as embedding provider and Ollama is running
- **THEN** system uses Ollama for embedding generation

#### Scenario: Fallback chain
- **WHEN** selected provider is unavailable (Ollama offline, API error)
- **THEN** system falls back to: configured provider → Ollama → bundled model
- **AND** logs warning about fallback

### Requirement: Embedding Dimensions Compatibility

The system SHALL normalize embeddings to a consistent dimension for Qdrant storage.

#### Scenario: MiniLM embeddings
- **WHEN** using bundled MiniLM-L6-v2
- **THEN** system generates 384-dimensional embeddings

#### Scenario: OpenAI embeddings
- **WHEN** using OpenAI text-embedding-3-small
- **THEN** system stores embeddings at their native 1536 dimensions
- **AND** Qdrant collection is configured for the appropriate dimension
