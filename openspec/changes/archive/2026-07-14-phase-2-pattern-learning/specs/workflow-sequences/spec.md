## ADDED Requirements

### Requirement: Sequence pattern detection

The system SHALL detect workflow sequences by analyzing task_completion observations where preceding_action references another task completion within the same project.

#### Scenario: Sequence detected from completions
- **WHEN** user completes task B within 5 minutes of completing task A, and this happens 3+ times
- **THEN** system records sequence pattern: A → B with occurrence count

#### Scenario: Meeting to task sequence
- **WHEN** user creates tasks within 30 minutes of meeting import, with similar keywords
- **THEN** system records meeting_type → task_types pattern

### Requirement: Sequence suggestion

The system SHALL suggest next actions based on detected workflow sequences with confidence >= 0.5.

#### Scenario: Next task suggested
- **WHEN** user completes a task that is the "A" in a learned A → B sequence
- **AND** sequence confidence >= 0.5
- **THEN** system displays suggestion: "Last time after [A], you did [B]. Create similar task?"

#### Scenario: Low confidence suppresses suggestion
- **WHEN** sequence confidence < 0.5
- **THEN** no suggestion is displayed

### Requirement: Sequence model structure

Workflow sequence model_data SHALL contain: sequences array (each with: trigger_action, follow_action, occurrence_count, avg_delay_minutes), and negative_sequences (rejected suggestions).

#### Scenario: Model captures timing
- **WHEN** sequence A → B occurs with varying delays
- **THEN** model stores avg_delay_minutes computed from observations

### Requirement: Negative sequence learning

The system SHALL track dismissed sequence suggestions to reduce future suggestion frequency.

#### Scenario: Dismissed suggestion reduces frequency
- **WHEN** user dismisses sequence suggestion 3 times
- **THEN** sequence is moved to negative_sequences and no longer suggested
- **AND** system may ask "Stop suggesting this?" on 4th occurrence

### Requirement: Sequence scope

Workflow sequences SHALL be learned at project level, not globally, to capture project-specific workflows.

#### Scenario: Project-scoped learning
- **WHEN** sequence A → B occurs in Project X
- **THEN** suggestion only appears in Project X context
- **AND** does not affect behavior in other projects
