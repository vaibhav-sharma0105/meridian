## ADDED Requirements

### Requirement: Observation recording

The system SHALL record user actions as structured observations in the `pattern_observations` table with fields: id, pattern_type, observation (JSON), context (JSON), created_at.

#### Scenario: Task completion recorded
- **WHEN** user marks a task as complete
- **THEN** system records observation with pattern_type "task_completion" containing task_id, project_id, previous_status, time_to_complete, and tags

#### Scenario: Task assignment recorded
- **WHEN** user assigns a task to someone
- **THEN** system records observation with pattern_type "task_assignment" containing task_id, assignee, task_title_keywords, priority, and project_id

#### Scenario: Draft edit recorded
- **WHEN** user modifies AI-generated draft text before using it
- **THEN** system records observation with pattern_type "draft_edit" containing original_text, edited_text, edit_distance, and context (task_id or message_type)

### Requirement: Observation types

The system SHALL support the following observation pattern_types: task_completion, task_creation, task_assignment, draft_edit, suggestion_accepted, suggestion_dismissed, priority_change, workflow_sequence.

#### Scenario: All observation types stored
- **WHEN** any supported action type occurs
- **THEN** system records observation with appropriate pattern_type and structured JSON payload

### Requirement: Observation context

Each observation SHALL include context containing: timestamp, project_id (if applicable), session_id, and preceding_action (last action taken within 5 minutes).

#### Scenario: Context captures preceding action
- **WHEN** user completes task B within 5 minutes of completing task A
- **THEN** observation for task B includes preceding_action referencing task A completion

### Requirement: Observation pruning

The system SHALL automatically prune observations older than 90 days that have been processed into pattern models.

#### Scenario: Old processed observations removed
- **WHEN** daily maintenance job runs
- **THEN** observations older than 90 days with processed_at not null are deleted
- **AND** unprocessed observations are retained regardless of age
