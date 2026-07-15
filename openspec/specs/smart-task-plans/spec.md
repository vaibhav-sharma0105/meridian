## ADDED Requirements

### Requirement: Task complexity evaluation

The system SHALL evaluate task complexity on creation using AI.

#### Scenario: Complexity assessed
- **WHEN** task is created
- **THEN** AI evaluates complexity as "simple", "medium", or "complex"
- **AND** stores evaluation in task.plan_complexity field

### Requirement: Simple task handling

The system SHALL auto-generate draft action for simple tasks.

#### Scenario: Simple task plan
- **WHEN** task complexity is "simple"
- **AND** task appears actionable (contains action verbs)
- **THEN** system generates draft message or action suggestion
- **AND** stores in task.plan_data field

### Requirement: Medium task handling

The system SHALL suggest subtasks for medium complexity tasks.

#### Scenario: Medium task plan
- **WHEN** task complexity is "medium"
- **THEN** AI suggests 2-5 subtasks to break down the work
- **AND** stores suggestions in task.plan_data as pending subtasks

### Requirement: Complex task handling

The system SHALL flag complex tasks for human decomposition.

#### Scenario: Complex task flagged
- **WHEN** task complexity is "complex"
- **THEN** system creates suggestion of type "decompose_task"
- **AND** task.plan_data contains AI reasoning about complexity factors

### Requirement: Plan UI in task detail

The system SHALL show Plan section in task detail when plan_data exists.

#### Scenario: View plan
- **WHEN** user opens task with plan_data
- **THEN** Plan section shows complexity indicator
- **AND** shows suggested subtasks or draft action

### Requirement: Plan acceptance

The system SHALL allow users to accept plan suggestions.

#### Scenario: Accept subtasks
- **WHEN** user clicks "Create subtasks" on plan
- **THEN** system creates tasks from suggested subtasks
- **AND** links them to parent task

### Requirement: Plan editing

The system SHALL allow users to edit plan suggestions before accepting.

#### Scenario: Edit plan
- **WHEN** user modifies suggested subtask text
- **THEN** edited version is used when accepting
- **AND** original AI suggestion is preserved for learning

### Requirement: Plan feedback learning

The system SHALL record user corrections to plans as observations.

#### Scenario: User corrects plan
- **WHEN** user modifies or rejects plan suggestion
- **THEN** system records observation of type "plan_correction"
- **AND** includes original suggestion and user action
