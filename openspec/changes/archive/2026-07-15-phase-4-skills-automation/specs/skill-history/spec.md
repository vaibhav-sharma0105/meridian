## ADDED Requirements

### Requirement: Skill runs table

The system SHALL store execution history in `skill_runs` table:
- `id`: Unique run identifier
- `skill_id`: FK to skills table
- `status`: Execution status
- `trigger_type`: What triggered the run (schedule, event, manual)
- `trigger_context`: JSON payload of trigger data
- `output`: Generated output (text/JSON)
- `error`: Error message if failed
- `started_at`, `completed_at`: Timing
- `duration_ms`: Execution duration
- `approval_decision`: approved/rejected/timeout (if applicable)
- `created_at`

#### Scenario: Successful run recorded
- **WHEN** skill completes successfully
- **THEN** run record includes output, duration, and completed_at

#### Scenario: Failed run recorded
- **WHEN** skill execution fails
- **THEN** run record includes error message and partial output

### Requirement: Query execution history

The system SHALL support querying skill runs with filters:
- By skill_id
- By status
- By date range
- By trigger_type

#### Scenario: Get runs for skill
- **WHEN** user requests runs for skill-123
- **THEN** system returns all runs ordered by created_at DESC

#### Scenario: Filter by status
- **WHEN** user requests failed runs
- **THEN** system returns only runs with status "failed"

### Requirement: History retention

The system SHALL manage history retention:
- Default retention: 90 days
- Configurable per-skill: 7-365 days
- Daemon prunes expired runs daily

#### Scenario: Prune old runs
- **WHEN** daemon runs daily cleanup
- **THEN** runs older than retention period are deleted
- **AND** associated outputs are removed

### Requirement: Run details access

The system SHALL provide detailed run information:
- Full output text
- Trigger payload
- Step-by-step execution log (for multi-step actions)
- Token usage statistics
- Approval decision and timestamp

#### Scenario: View run details
- **WHEN** user clicks on a run in history
- **THEN** modal shows full output and execution details

### Requirement: Run statistics

The system SHALL compute per-skill statistics:
- Total runs (last 7/30/90 days)
- Success rate
- Average duration
- Common error types

#### Scenario: Get skill stats
- **WHEN** user views skill details
- **THEN** UI shows success rate and average duration

#### Scenario: Error frequency
- **WHEN** skill has multiple failed runs with same error
- **THEN** stats group errors by type with count
