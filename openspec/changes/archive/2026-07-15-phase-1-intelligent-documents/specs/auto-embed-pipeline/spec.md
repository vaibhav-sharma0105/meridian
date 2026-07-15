## ADDED Requirements

### Requirement: Automatic Embedding Queue

The system SHALL automatically queue documents for embedding upon upload without manual user intervention.

#### Scenario: Queue on upload
- **WHEN** user uploads a document
- **THEN** system creates embedding job in daemon queue with status "pending"
- **AND** document card shows "Embedding..." indicator

#### Scenario: Queue on text paste
- **WHEN** user pastes text content as a document
- **THEN** system queues the content for embedding

### Requirement: Background Embedding Processing

The system SHALL process embedding jobs in the background daemon without blocking the UI.

#### Scenario: Process embedding job
- **WHEN** daemon picks up pending embedding job
- **THEN** system chunks document text (500 tokens, 50 overlap)
- **AND** generates embedding for each chunk
- **AND** stores embeddings in Qdrant collection

#### Scenario: Update progress
- **WHEN** embedding job is processing
- **THEN** system updates job progress (0-100%)
- **AND** document card reflects current progress

#### Scenario: Job completion
- **WHEN** all chunks are embedded successfully
- **THEN** system marks document as "embeddings_ready = true"
- **AND** document card shows checkmark indicator

### Requirement: Embedding Retry Logic

The system SHALL retry failed embedding jobs with exponential backoff.

#### Scenario: Transient failure
- **WHEN** embedding fails due to temporary error (network, rate limit)
- **THEN** system retries after delay: 1min, 5min, 30min
- **AND** marks job as "failed" after 3 retries

#### Scenario: Permanent failure
- **WHEN** embedding fails due to permanent error (invalid content, unsupported format)
- **THEN** system marks job as "failed" immediately
- **AND** stores error message in job details

### Requirement: Embedding Job Priority

The system SHALL prioritize embedding jobs based on document recency and user activity.

#### Scenario: Recent documents first
- **WHEN** multiple documents are queued for embedding
- **THEN** system processes most recently uploaded documents first

#### Scenario: Active project priority
- **WHEN** user is viewing a project with pending embeddings
- **THEN** system prioritizes that project's documents

### Requirement: Re-embed on Model Change

The system SHALL re-embed documents when the embedding provider changes.

#### Scenario: Provider switch
- **WHEN** user changes embedding provider in settings
- **THEN** system queues all documents for re-embedding with new provider
- **AND** shows confirmation dialog with document count

#### Scenario: Incremental re-embed
- **WHEN** re-embedding is in progress
- **THEN** system continues serving old embeddings until new ones are ready
- **AND** switches atomically per document
