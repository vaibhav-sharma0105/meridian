## ADDED Requirements

### Requirement: Risk level in audit entries
The system SHALL include risk_level in all audit log entries for agent actions.

#### Scenario: Log action with risk level
- **WHEN** agent action is executed
- **THEN** audit log entry includes risk_level field (low/medium/high/critical)
- **AND** risk_level is queryable for filtering

#### Scenario: Query by risk level
- **WHEN** user filters audit log by risk_level="high"
- **THEN** system returns only high-risk action entries

### Requirement: Approval linkage in audit
The system SHALL link audit entries to approval records when applicable.

#### Scenario: Log approved action
- **WHEN** action is executed after approval
- **THEN** audit entry includes approval_id reference
- **AND** approval record is retrievable from audit entry

#### Scenario: Log rejected action
- **WHEN** action is rejected
- **THEN** audit entry records rejection with approval_id
- **AND** includes rejection reason if provided

### Requirement: Autonomy mode in audit
The system SHALL record the effective autonomy mode for each agent action.

#### Scenario: Log autonomy context
- **WHEN** agent action is executed
- **THEN** audit entry includes autonomy_mode (Manual/Supervised/Autonomous)
- **AND** includes autonomy_source (global/integration/skill)

### Requirement: Undo tracking in audit
The system SHALL link original actions to their undo operations.

#### Scenario: Log undo operation
- **WHEN** user undoes an action
- **THEN** audit entry for undo includes original_action_id
- **AND** original action entry is updated with undo_action_id

#### Scenario: Query undo history
- **WHEN** user queries action with undo_action_id set
- **THEN** system can retrieve the complete undo chain

### Requirement: Governance metrics from audit
The system SHALL support efficient queries for governance dashboard metrics.

#### Scenario: Query approval rates
- **WHEN** dashboard queries approval metrics
- **THEN** system efficiently counts approved/rejected/archived by time period

#### Scenario: Query risk distribution
- **WHEN** dashboard queries risk distribution
- **THEN** system efficiently aggregates actions by risk_level for period

#### Scenario: Query by autonomy mode
- **WHEN** dashboard queries autonomy breakdown
- **THEN** system efficiently counts actions by autonomy_mode and outcome
