## MODIFIED Requirements

### Requirement: Observation types

The system SHALL support the following observation pattern_types: task_completion, task_creation, task_assignment, draft_edit, suggestion_accepted, suggestion_dismissed, priority_change, workflow_sequence, skill_correction, skill_usage, **time_of_day_productivity**, **task_estimation**, **focus_session**.

#### Scenario: All observation types stored
- **WHEN** any supported action type occurs
- **THEN** system records observation with appropriate pattern_type and structured JSON payload

#### Scenario: Time-of-day productivity observation
- **WHEN** user completes a task
- **THEN** system records observation with pattern_type 'time_of_day_productivity'
- **AND** observation includes: hour_of_day, day_of_week, task_type, time_spent

#### Scenario: Task estimation observation
- **WHEN** user creates task with estimate OR completes estimated task
- **THEN** system records observation with pattern_type 'task_estimation'
- **AND** observation includes: estimated_duration, actual_duration (if completed), task_keywords, assignee, priority

#### Scenario: Focus session observation
- **WHEN** user works on tasks without context switching for 30+ minutes
- **THEN** system records observation with pattern_type 'focus_session'
- **AND** observation includes: start_hour, duration_minutes, task_types_worked

## ADDED Requirements

### Requirement: Productivity pattern observation
The system SHALL observe user productivity patterns by time.

#### Scenario: Record peak productivity detection
- **WHEN** pattern aggregation processes time_of_day_productivity observations
- **THEN** system computes: hours with highest completion rate, average task completion by hour
- **AND** stores in pattern_models with pattern_type 'productivity_profile'

#### Scenario: Record work rhythm patterns
- **WHEN** user has 7+ days of focus_session observations
- **THEN** system identifies: preferred focus hours, meeting-free preferences
- **AND** stores rhythm patterns in pattern_models

### Requirement: Estimation pattern observation
The system SHALL observe estimation accuracy patterns.

#### Scenario: Record estimation accuracy by category
- **WHEN** pattern aggregation processes task_estimation observations
- **THEN** system computes: accuracy by assignee, accuracy by priority, accuracy by keyword
- **AND** stores in pattern_models with pattern_type 'estimation_accuracy'

#### Scenario: Detect systematic estimation bias
- **WHEN** user consistently over or underestimates by >30%
- **THEN** system records bias_factor in pattern_models
- **AND** applies correction factor to future suggestions
