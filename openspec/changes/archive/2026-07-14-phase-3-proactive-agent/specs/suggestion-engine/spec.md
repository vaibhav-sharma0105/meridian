## ADDED Requirements

### Requirement: Suggestion storage

The system SHALL store suggestions in a `suggestions` table with fields: id, type, title, description, reasoning, action_config (JSON), severity, status, created_at, acted_at.

#### Scenario: Suggestion created
- **WHEN** suggestion engine detects actionable opportunity
- **THEN** suggestion is stored with status "pending" and severity level

### Requirement: Overdue task suggestions

The system SHALL generate suggestions for tasks overdue by more than 24 hours.

#### Scenario: Overdue task detected
- **WHEN** task due_date is more than 24 hours in the past
- **AND** task status is not "done" or "cancelled"
- **THEN** system creates suggestion with type "overdue_task" and severity "warning"

### Requirement: Stale task suggestions

The system SHALL generate suggestions for tasks with no updates in 7+ days.

#### Scenario: Stale task detected
- **WHEN** task updated_at is more than 7 days ago
- **AND** task status is "in_progress"
- **THEN** system creates suggestion with type "stale_task" and severity "info"

### Requirement: Meeting follow-up suggestions

The system SHALL generate suggestions for meetings without follow-up tasks after 24 hours.

#### Scenario: Meeting without follow-up
- **WHEN** meeting was imported more than 24 hours ago
- **AND** no tasks are linked to the meeting
- **THEN** system creates suggestion with type "meeting_followup" and severity "info"

### Requirement: Pattern-based suggestions

The system SHALL integrate workflow sequence patterns into suggestion generation.

#### Scenario: Workflow sequence suggestion
- **WHEN** task completion matches a learned workflow trigger
- **AND** confidence >= 0.5
- **THEN** system creates suggestion with type "workflow_sequence"
- **AND** includes reasoning referencing the learned pattern

### Requirement: Daily suggestion limit

The system SHALL respect a configurable max suggestions per day (default: 10).

#### Scenario: Limit reached
- **WHEN** suggestion count for today >= max_suggestions_per_day
- **THEN** new suggestions are queued for next day
- **AND** high-severity suggestions bypass the limit

### Requirement: Suggestion job scheduling

The system SHALL run suggestion generation as a daemon job every 30 minutes.

#### Scenario: Scheduled generation
- **WHEN** 30 minutes have passed since last run
- **THEN** daemon job analyzes all projects for suggestion opportunities
