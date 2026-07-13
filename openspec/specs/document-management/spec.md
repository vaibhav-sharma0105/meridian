# document-management Specification

## Purpose
TBD - created by archiving change document-existing-system. Update Purpose after archive.
## Requirements
### Requirement: Document Upload

The system SHALL support uploading documents via file picker, drag-and-drop, URL fetch, and text paste. Supported formats: PDF, DOCX, PPTX, TXT, MD, CSV, VTT, SRT.

#### Scenario: Upload via file picker
- **WHEN** user clicks upload and selects file
- **THEN** system parses file, extracts text, chunks content, and stores in database

#### Scenario: Upload via drag-and-drop
- **WHEN** user drags file into upload zone
- **THEN** system processes file same as file picker upload

#### Scenario: Upload via URL
- **WHEN** user enters URL and clicks fetch
- **THEN** system fetches content, strips HTML tags, and stores as document

#### Scenario: Upload via text paste
- **WHEN** user pastes text into text area
- **THEN** system stores text as .txt document

#### Scenario: Reject unsupported format
- **WHEN** user attempts to upload unsupported file type
- **THEN** system shows error message and does not create document

### Requirement: Document Text Extraction

The system SHALL extract text content from uploaded documents using format-specific parsers.

#### Scenario: Extract PDF text
- **WHEN** PDF document is uploaded
- **THEN** system extracts text using BT/ET markers (basic extraction)

#### Scenario: Extract DOCX text
- **WHEN** DOCX document is uploaded
- **THEN** system extracts text from document.xml within ZIP structure

#### Scenario: Extract PPTX text
- **WHEN** PPTX document is uploaded
- **THEN** system extracts text from slide XML files within ZIP structure

#### Scenario: Extract subtitle text
- **WHEN** VTT or SRT file is uploaded
- **THEN** system strips timestamps and extracts dialogue text

### Requirement: Document Chunking

The system SHALL split document text into chunks for embedding and search. Default chunk size: 2048 characters with 200 character overlap.

#### Scenario: Chunk document
- **WHEN** document text is extracted
- **THEN** system splits into overlapping chunks respecting sentence boundaries where possible

#### Scenario: Store chunks
- **WHEN** chunking completes
- **THEN** system stores chunk text and index for each chunk

### Requirement: Full-Text Search

The system SHALL support keyword search across document content using FTS5 virtual tables.

#### Scenario: Search by keyword
- **WHEN** user enters search query
- **THEN** system returns documents and chunks matching query

#### Scenario: Project-scoped search
- **WHEN** user searches within a project
- **THEN** results include only documents linked to that project

### Requirement: Document Embeddings

The system SHALL support generating vector embeddings for document chunks using Ollama embedding models. Embedding is manually triggered per document.

#### Scenario: Generate embeddings
- **WHEN** user clicks "Embed" button on document
- **THEN** system generates embeddings for all chunks via Ollama API

#### Scenario: Show embedding progress
- **WHEN** embedding generation is in progress
- **THEN** system shows progress bar with chunk count

#### Scenario: Mark embeddings ready
- **WHEN** all chunks are embedded
- **THEN** system sets embeddings_ready flag on document

### Requirement: Semantic Search

The system SHALL support semantic search using cosine similarity between query embedding and document chunk embeddings. Requires Ollama with embedding model.

#### Scenario: Semantic search
- **WHEN** user searches with semantic search enabled
- **THEN** system embeds query, finds similar chunks via cosine similarity

#### Scenario: Hybrid search
- **WHEN** semantic search is enabled
- **THEN** system combines FTS5 keyword results with semantic results using rank fusion

### Requirement: Document Project Association

The system SHALL associate documents with projects. Documents can be project-scoped or global.

#### Scenario: Upload to project
- **WHEN** user uploads document within project context
- **THEN** document is linked to that project

#### Scenario: Global document
- **WHEN** document has no project association
- **THEN** document is available across all projects

### Requirement: Document Management

The system SHALL support listing and deleting documents per project.

#### Scenario: List project documents
- **WHEN** user views project documents tab
- **THEN** system displays all documents linked to project

#### Scenario: Delete document
- **WHEN** user deletes document
- **THEN** system removes document record, chunks, embeddings, and file from filesystem

