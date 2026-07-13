## ADDED Requirements

### Requirement: Qdrant Embedded Integration

The system SHALL use Qdrant in embedded mode for vector storage, replacing sqlite-vec. Qdrant data SHALL be stored locally in `~/.meridian/qdrant/`.

#### Scenario: Initialize Qdrant
- **WHEN** application starts
- **THEN** system initializes Qdrant embedded instance at configured path

#### Scenario: Qdrant data isolation
- **WHEN** Qdrant stores vectors
- **THEN** data is stored in `~/.meridian/qdrant/` directory, separate from SQLite

### Requirement: Collection Management

The system SHALL organize vectors into collections: one global collection and one per project.

#### Scenario: Create global collection
- **WHEN** first document is embedded without project association
- **THEN** system creates `global` collection if not exists

#### Scenario: Create project collection
- **WHEN** first document is embedded for a project
- **THEN** system creates `project_{id}` collection if not exists

#### Scenario: Delete project collection
- **WHEN** project is deleted
- **THEN** system deletes corresponding Qdrant collection

### Requirement: Vector Operations

The system SHALL support insert, search, and delete operations on vectors.

#### Scenario: Insert vectors
- **WHEN** document chunks are embedded
- **THEN** system inserts vectors with payload (document_id, chunk_index, chunk_text)

#### Scenario: Search vectors
- **WHEN** semantic search is performed
- **THEN** system returns top-k similar vectors with scores and payloads

#### Scenario: Delete vectors
- **WHEN** document is deleted
- **THEN** system removes all vectors associated with document_id

### Requirement: Search Performance

The system SHALL complete vector search within 100ms for collections up to 100,000 vectors.

#### Scenario: Performance benchmark
- **WHEN** searching collection with 100K vectors
- **THEN** results return in under 100ms

#### Scenario: Scale gracefully
- **WHEN** collection exceeds 100K vectors
- **THEN** system continues to function with degraded but acceptable performance

### Requirement: Migration from sqlite-vec

The system SHALL migrate existing sqlite-vec embeddings to Qdrant on upgrade.

#### Scenario: Detect sqlite-vec data
- **WHEN** app starts and finds document_embeddings table with data
- **THEN** system queues migration job

#### Scenario: Migrate embeddings
- **WHEN** migration job runs
- **THEN** system reads embeddings from SQLite, inserts into Qdrant, marks migration complete

#### Scenario: Migration progress
- **WHEN** migration is in progress
- **THEN** UI shows progress indicator with vector count

### Requirement: Qdrant Encryption

The system SHALL encrypt Qdrant data using the same encryption key as SQLite database.

#### Scenario: Encrypted Qdrant storage
- **WHEN** vectors are stored
- **THEN** Qdrant data files are encrypted at rest

#### Scenario: Decrypt on access
- **WHEN** vector search is performed
- **THEN** system decrypts data transparently using stored key
