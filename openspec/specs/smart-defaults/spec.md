# smart-defaults Specification

## Purpose
TBD - created by archiving change phase-2-pattern-learning. Update Purpose after archive.
## Requirements
### Requirement: Priority pattern learning

The system SHALL learn task priority patterns based on: task_title_keywords, project_id, and source (meeting vs manual creation).

#### Scenario: Keyword-based priority
- **WHEN** tasks containing "bug" are consistently marked high/critical priority
- **THEN** new tasks with "bug" in title default to learned priority

#### Scenario: Project-based priority
- **WHEN** tasks in Project X are typically medium priority
- **THEN** new tasks in Project X default to medium priority

### Requirement: Assignee pattern learning

The system SHALL learn assignee patterns based on: task_title_keywords, project_id, and task_type.

#### Scenario: Keyword-based assignee
- **WHEN** tasks containing "frontend" are consistently assigned to "Alice"
- **THEN** new tasks with "frontend" suggest "Alice" as assignee

#### Scenario: Project-based assignee
- **WHEN** user is the primary assignee for Project Y
- **THEN** new tasks in Project Y default to user as assignee

### Requirement: Default model structure

Smart defaults model_data SHALL contain: priority_patterns (keyword → priority mappings), assignee_patterns (keyword → assignee mappings), project_defaults (project_id → {default_priority, default_assignee}).

#### Scenario: Multiple pattern types combined
- **WHEN** task matches both keyword pattern and project pattern
- **THEN** keyword pattern takes precedence (more specific)

### Requirement: Default application

The system SHALL apply smart defaults to new task creation when pattern confidence >= 0.5, pre-filling fields with learned values.

#### Scenario: Defaults pre-filled
- **WHEN** user creates new task matching learned patterns
- **THEN** priority and assignee fields are pre-filled
- **AND** fields show subtle indicator: "Suggested based on past tasks"

#### Scenario: User override respected
- **WHEN** user changes pre-filled default value
- **THEN** observation is recorded as potential pattern update
- **AND** override value is used

### Requirement: Default transparency

Pre-filled defaults SHALL be visually distinguished from user-entered values, with option to clear suggestions.

#### Scenario: Clear suggestion
- **WHEN** user clicks "clear" on suggested assignee
- **THEN** field returns to empty state
- **AND** no observation is recorded (neutral action)

