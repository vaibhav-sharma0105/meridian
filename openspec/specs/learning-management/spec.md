# learning-management Specification

## Purpose
TBD - created by archiving change phase-2-pattern-learning. Update Purpose after archive.
## Requirements
### Requirement: Learning dashboard

The system SHALL provide a Learning section in Settings showing all learned pattern categories with their confidence scores and observation counts.

#### Scenario: View all patterns
- **WHEN** user opens Settings > Learning
- **THEN** system displays list of pattern types: Workflow Sequences, Communication Style, Priority Defaults, Assignee Defaults
- **AND** each shows confidence score and "X observations" count

### Requirement: Pattern detail view

The system SHALL allow users to view detailed patterns within each category.

#### Scenario: View workflow sequences
- **WHEN** user clicks on "Workflow Sequences" category
- **THEN** system shows list of learned sequences: "After [A], you usually do [B]" with occurrence counts

#### Scenario: View priority patterns
- **WHEN** user clicks on "Priority Defaults"
- **THEN** system shows keyword → priority mappings and project defaults

### Requirement: Per-category reset

The system SHALL allow users to reset learning for individual pattern categories without affecting others.

#### Scenario: Reset workflow sequences
- **WHEN** user clicks "Reset" on Workflow Sequences
- **AND** confirms reset dialog
- **THEN** workflow sequence pattern_model is deleted
- **AND** related observations are marked for re-processing
- **AND** other pattern categories remain unchanged

### Requirement: Master reset

The system SHALL allow users to reset all learned patterns with a single action.

#### Scenario: Master reset
- **WHEN** user clicks "Reset All Learning"
- **AND** confirms with additional confirmation (type "RESET")
- **THEN** all pattern_models are deleted
- **AND** all observations are deleted
- **AND** system returns to "no learned patterns" state

### Requirement: Pattern export/import

The system SHALL allow exporting learned patterns as JSON and importing patterns from a JSON file.

#### Scenario: Export patterns
- **WHEN** user clicks "Export Learning Data"
- **THEN** system downloads JSON file containing all pattern_models

#### Scenario: Import patterns
- **WHEN** user uploads valid patterns JSON file
- **THEN** system imports pattern_models with merge strategy: imported patterns replace existing for same pattern_type
- **AND** observation counts are preserved from imported data

### Requirement: Stop learning option

The system SHALL allow users to mark individual patterns as "don't learn" to prevent future learning on that pattern.

#### Scenario: Stop learning sequence
- **WHEN** user selects a workflow sequence and clicks "Stop learning this"
- **THEN** sequence is moved to negative_patterns
- **AND** no suggestions or learning occurs for this sequence
- **AND** user can later remove from negative_patterns to resume learning

