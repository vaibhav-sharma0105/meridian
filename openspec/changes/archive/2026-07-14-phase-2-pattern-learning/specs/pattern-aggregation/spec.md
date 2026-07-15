## ADDED Requirements

### Requirement: Pattern model storage

The system SHALL store aggregated patterns in the `pattern_models` table with fields: id, pattern_type, model_data (JSON), confidence (0.0-1.0), observation_count, last_updated.

#### Scenario: Model created from observations
- **WHEN** aggregation job processes observations for a new pattern_type
- **THEN** system creates pattern_model entry with aggregated statistics and confidence score

#### Scenario: Model updated with new observations
- **WHEN** aggregation job processes new observations for existing pattern_type
- **THEN** system updates model_data, increments observation_count, recalculates confidence, and updates last_updated

### Requirement: Aggregation job execution

The system SHALL run pattern aggregation as a background daemon job that processes unprocessed observations every 15 minutes.

#### Scenario: Scheduled aggregation runs
- **WHEN** 15 minutes have passed since last aggregation
- **THEN** daemon job processes all observations where processed_at is null
- **AND** marks processed observations with processed_at timestamp

#### Scenario: Aggregation handles empty queue
- **WHEN** no unprocessed observations exist
- **THEN** job completes successfully without modifying pattern_models

### Requirement: Confidence scoring

The system SHALL calculate confidence scores based on: observation_count (more = higher), recency (recent observations weighted more), and consistency (similar observations = higher confidence).

#### Scenario: Low observation count yields low confidence
- **WHEN** pattern has fewer than 5 observations
- **THEN** confidence score is capped at 0.3

#### Scenario: High consistency yields high confidence
- **WHEN** 80%+ of observations follow the same pattern
- **THEN** confidence score is at least 0.7

### Requirement: Pattern decay

The system SHALL reduce confidence for patterns with no recent observations, decaying by 10% per month of inactivity.

#### Scenario: Inactive pattern decays
- **WHEN** pattern has no new observations for 30 days
- **THEN** confidence is reduced by 10% on next aggregation run

### Requirement: Pattern export

The system SHALL support exporting all pattern_models as JSON for backup or transfer.

#### Scenario: Export patterns
- **WHEN** user requests pattern export
- **THEN** system generates JSON file containing all pattern_models with their observation_counts and confidence scores
