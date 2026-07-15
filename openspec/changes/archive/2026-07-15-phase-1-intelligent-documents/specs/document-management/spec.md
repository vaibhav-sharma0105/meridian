## MODIFIED Requirements

### Requirement: XLSX File Support

The system SHALL parse XLSX files and extract text content for indexing.

#### Scenario: Upload XLSX
- **WHEN** user uploads an XLSX file
- **THEN** system extracts text from all sheets
- **AND** preserves table structure as best-effort markdown

#### Scenario: Multi-sheet workbook
- **WHEN** XLSX contains multiple sheets
- **THEN** system extracts each sheet with sheet name as heading

#### Scenario: Large spreadsheet
- **WHEN** XLSX exceeds 10,000 rows
- **THEN** system truncates with warning: "Large file - first 10,000 rows indexed"

### Requirement: Improved PDF Extraction

The system SHALL use pdf-extract crate for better PDF text extraction.

#### Scenario: Standard PDF
- **WHEN** user uploads text-based PDF
- **THEN** system extracts text with paragraph preservation

#### Scenario: Scanned PDF
- **WHEN** PDF contains images without text layer
- **THEN** system shows warning: "Scanned PDF - text extraction limited"
- **AND** extracts any available metadata

#### Scenario: PDF with tables
- **WHEN** PDF contains tables
- **THEN** system extracts table content (best-effort, may lose structure)

### Requirement: Document Preview

The system SHALL provide inline preview for supported document types.

#### Scenario: Text preview
- **WHEN** user clicks preview on text/markdown document
- **THEN** system shows rendered content in modal

#### Scenario: PDF preview
- **WHEN** user clicks preview on PDF
- **THEN** system shows first page thumbnail or "Preview not available"

#### Scenario: XLSX preview
- **WHEN** user clicks preview on XLSX
- **THEN** system shows first sheet as rendered table (first 50 rows)

### Requirement: Bulk Document Upload

The system SHALL support uploading multiple documents at once.

#### Scenario: Drag multiple files
- **WHEN** user drags multiple files to document area
- **THEN** system queues all files for upload
- **AND** shows progress for batch (e.g., "Uploading 5 of 12")

#### Scenario: File picker multi-select
- **WHEN** user clicks upload button
- **THEN** file picker allows multiple file selection

#### Scenario: Mixed valid/invalid files
- **WHEN** batch contains unsupported file types
- **THEN** system uploads valid files and shows warning for skipped files

### Requirement: Document Embedding Status

The system SHALL display embedding status on document cards.

#### Scenario: Pending embedding
- **WHEN** document is uploaded and queued for embedding
- **THEN** document card shows "Embedding..." with progress spinner

#### Scenario: Embedding complete
- **WHEN** all chunks are embedded successfully
- **THEN** document card shows checkmark icon

#### Scenario: Embedding failed
- **WHEN** embedding fails after retries
- **THEN** document card shows warning icon with tooltip explaining failure

#### Scenario: No embeddings (provider unavailable)
- **WHEN** no embedding provider is configured/available
- **THEN** document card shows info icon: "Keyword search only"

### Requirement: Document Search Integration

The system SHALL integrate documents with the AI chat context via hybrid search.

#### Scenario: Chat with documents
- **WHEN** user asks question in AI chat panel
- **THEN** system runs hybrid search across project documents
- **AND** includes top-5 relevant chunks in AI context

#### Scenario: Reference documents in response
- **WHEN** AI response uses document content
- **THEN** response includes citations with document name and click-to-view link
