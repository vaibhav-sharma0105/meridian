## ADDED Requirements

### Requirement: Reciprocal Rank Fusion

The system SHALL combine semantic (Qdrant) and keyword (FTS5) search results using Reciprocal Rank Fusion.

#### Scenario: Combine search results
- **WHEN** user searches documents with query "Q123 budget review"
- **THEN** system runs both semantic search (Qdrant) and keyword search (FTS5)
- **AND** merges results using RRF formula: `score = Σ 1/(k + rank)` where k=60

#### Scenario: Handle empty semantic results
- **WHEN** Qdrant returns no results (embeddings not ready)
- **THEN** system falls back to FTS5-only results

#### Scenario: Handle empty keyword results
- **WHEN** FTS5 returns no matches
- **THEN** system returns semantic-only results

### Requirement: Semantic Search via Qdrant

The system SHALL query Qdrant for semantically similar document chunks.

#### Scenario: Query embeddings
- **WHEN** search request is made
- **THEN** system embeds query text using configured provider
- **AND** queries Qdrant for top-N similar vectors (default N=20)

#### Scenario: Filter by project
- **WHEN** search is scoped to specific project
- **THEN** Qdrant query filters by project_id payload field

#### Scenario: Score threshold
- **WHEN** Qdrant returns results
- **THEN** system excludes results with similarity score below 0.3

### Requirement: Keyword Search via FTS5

The system SHALL maintain FTS5 index for fast keyword lookup.

#### Scenario: Full-text index
- **WHEN** document is uploaded or text extracted
- **THEN** system updates FTS5 index with document content

#### Scenario: Query FTS5
- **WHEN** search request is made
- **THEN** system queries FTS5 with tokenized query
- **AND** returns ranked results by BM25 score

#### Scenario: Phrase matching
- **WHEN** query contains quoted phrase "exact match"
- **THEN** FTS5 matches exact phrase

### Requirement: Search Result Deduplication

The system SHALL deduplicate results that appear in both semantic and keyword results.

#### Scenario: Same chunk in both
- **WHEN** same document chunk appears in both Qdrant and FTS5 results
- **THEN** system keeps highest combined RRF score
- **AND** marks result as "matched both"

#### Scenario: Adjacent chunks
- **WHEN** multiple adjacent chunks from same document match
- **THEN** system merges into single result with expanded context

### Requirement: Search Result UI

The system SHALL display search results with relevance indicators.

#### Scenario: Show match type
- **WHEN** displaying search result
- **THEN** UI shows badge: "Semantic", "Keyword", or "Both"

#### Scenario: Highlight matches
- **WHEN** keyword match is displayed
- **THEN** matching terms are highlighted in result snippet

#### Scenario: Show relevance score
- **WHEN** displaying search result
- **THEN** UI shows relevance indicator (high/medium/low or percentage)

### Requirement: Search Performance

The system SHALL return search results within acceptable latency.

#### Scenario: Typical search
- **WHEN** searching across 100 documents
- **THEN** results return within 500ms

#### Scenario: Large corpus
- **WHEN** searching across 1000+ documents
- **THEN** results return within 2 seconds
- **AND** UI shows loading state
