## ADDED Requirements

### Requirement: Task lifecycle event tracking
The system SHALL track task lifecycle events for estimation learning.

#### Scenario: Record task creation
- **WHEN** a task is created
- **THEN** system logs event: task_id, 'created', timestamp
- **AND** if user provides estimate, system logs: task_id, 'estimate_set', estimate_value

#### Scenario: Record task started
- **WHEN** task status changes to 'in_progress'
- **THEN** system logs event: task_id, 'started', timestamp

#### Scenario: Record task completed
- **WHEN** task status changes to 'done'
- **THEN** system logs event: task_id, 'completed', timestamp
- **AND** system computes actual_duration from started to completed

#### Scenario: Record estimate changes
- **WHEN** user modifies task estimate
- **THEN** system logs event: task_id, 'estimate_changed', new_estimate, timestamp

### Requirement: Estimation accuracy computation
The system SHALL compute estimation accuracy for completed tasks.

#### Scenario: Calculate accuracy for single task
- **WHEN** a task with estimate is completed
- **THEN** system computes: estimated_duration, actual_duration, accuracy_ratio (estimated/actual)
- **AND** system stores accuracy with task metadata

#### Scenario: Aggregate accuracy by category
- **WHEN** computing estimation insights
- **THEN** system groups by: assignee, priority, project, keywords
- **AND** system computes average accuracy per group

#### Scenario: Handle missing start time
- **WHEN** task is completed but was never marked 'in_progress'
- **THEN** system uses created_at as proxy for started_at
- **AND** system flags accuracy as "approximate"

### Requirement: Smart estimation suggestions
The system SHALL suggest estimates for new tasks based on similar completed tasks.

#### Scenario: Suggest estimate for new task
- **WHEN** user creates a new task
- **AND** system has 10+ completed tasks with estimates
- **THEN** system finds similar tasks by: title keywords, assignee, priority, project
- **AND** system suggests estimate based on weighted average of similar task durations

#### Scenario: Show estimation confidence
- **WHEN** displaying estimate suggestion
- **THEN** system shows: suggested estimate, confidence level (high/medium/low), basis ("Based on N similar tasks")

#### Scenario: Explain estimate derivation
- **WHEN** user clicks "Why this estimate?"
- **THEN** system shows: list of similar tasks used, their actual durations, weighting factors applied

#### Scenario: No suggestion when insufficient data
- **WHEN** fewer than 10 completed tasks exist
- **OR** no similar tasks found
- **THEN** system does not suggest estimate
- **AND** system shows "Not enough data for suggestion"

### Requirement: Estimation accuracy feedback loop
The system SHALL use actual outcomes to improve future estimates.

#### Scenario: Weight recent completions higher
- **WHEN** computing estimate suggestion
- **THEN** system applies recency weighting: tasks from last 30 days weighted 2x, 30-90 days weighted 1x, older weighted 0.5x

#### Scenario: Learn from systematic over/under estimation
- **WHEN** user consistently overestimates (accuracy > 1.5) or underestimates (accuracy < 0.7)
- **THEN** system applies correction factor to future suggestions
- **AND** system shows: "Your estimates tend to be [over/under] by ~X%"

#### Scenario: Track estimation improvement over time
- **WHEN** user views Estimation section in Analytics
- **THEN** system shows: accuracy trend over time, whether user is improving at estimation

### Requirement: Estimation insights display
The system SHALL display estimation insights in analytics.

#### Scenario: View estimation accuracy dashboard
- **WHEN** user opens Estimation section in Analytics
- **THEN** system displays: overall accuracy ratio, accuracy by project/assignee/priority, estimation improvement trend

#### Scenario: Identify estimation problem areas
- **WHEN** displaying estimation insights
- **THEN** system highlights: task types with worst accuracy, assignees who need estimation calibration, projects with estimation drift

#### Scenario: Export estimation data
- **WHEN** user clicks export in Estimation section
- **THEN** system exports: task estimation log, accuracy metrics, similar task groupings
