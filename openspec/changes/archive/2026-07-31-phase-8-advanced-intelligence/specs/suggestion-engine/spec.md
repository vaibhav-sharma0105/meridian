## MODIFIED Requirements

### Requirement: Suggestion triggers skill execution

The system SHALL support triggering skills from suggestion acceptance with autonomy-aware routing:
- Suggestion action_config can include skill_id
- Accepting suggestion queues skill execution
- Skill receives suggestion context as input
- Execution is routed through approval flow based on autonomy mode and risk level
- **NEW**: Support cross-project and predictive suggestion types

#### Scenario: Accept suggestion runs skill
- **WHEN** user accepts suggestion with action_config.skill_id
- **AND** resulting action is within autonomy mode allowance
- **THEN** system queues skill execution
- **AND** passes suggestion data as trigger context

#### Scenario: Accept suggestion requires approval
- **WHEN** user accepts suggestion with action_config containing high-risk action
- **AND** autonomy mode requires approval for high-risk
- **THEN** system queues action in pending_approvals
- **AND** notifies user that approval is needed for execution

#### Scenario: Cross-project suggestion routing
- **WHEN** suggestion involves tasks from multiple projects
- **THEN** system displays affected projects in suggestion detail
- **AND** accepting creates/updates entities in correct projects

#### Scenario: Predictive suggestion with confidence
- **WHEN** predictive action generates a suggestion
- **THEN** suggestion includes prediction_confidence score
- **AND** UI displays confidence indicator

## ADDED Requirements

### Requirement: Cross-project suggestion types
The system SHALL generate suggestions based on cross-project analysis.

#### Scenario: Generate blocker alert suggestion
- **WHEN** cross-project analysis detects a blocking relationship
- **AND** blocked task has upcoming due date
- **THEN** system creates suggestion: type='cross_project_blocker', severity='warning'
- **AND** suggestion includes: blocking task details, blocked task details, suggested actions

#### Scenario: Generate consolidation suggestion
- **WHEN** cross-project analysis detects duplicate tasks or meetings
- **THEN** system creates suggestion: type='consolidation', severity='info'
- **AND** suggestion includes: items to consolidate, merge preview

#### Scenario: Generate velocity alert suggestion
- **WHEN** cross-project velocity analysis detects declining trend
- **THEN** system creates suggestion: type='velocity_alert', severity='info'
- **AND** suggestion includes: affected projects, trend data, possible causes

### Requirement: Predictive suggestion types
The system SHALL generate suggestions based on predictive analysis.

#### Scenario: Generate deadline risk suggestion
- **WHEN** predictive analysis identifies deadline at risk
- **THEN** system creates suggestion: type='deadline_risk', severity='warning'
- **AND** suggestion includes: task at risk, blockers, suggested actions (reassign, extend, escalate)

#### Scenario: Generate workload suggestion
- **WHEN** predictive analysis identifies workload imbalance
- **THEN** system creates suggestion: type='workload_rebalance', severity='info'
- **AND** suggestion includes: overloaded assignee, tasks to reassign, recommended assignees

#### Scenario: Generate pre-meeting suggestion
- **WHEN** predictive analysis identifies meeting needing preparation
- **THEN** system creates suggestion: type='meeting_prep', severity='info'
- **AND** suggestion includes: meeting details, suggested agenda draft, relevant documents

### Requirement: Suggestion source attribution
The system SHALL indicate the source of each suggestion.

#### Scenario: Display suggestion source
- **WHEN** suggestion is displayed in UI
- **THEN** system shows source: 'pattern' (from workflow learning), 'prediction' (from predictive engine), 'cross_project' (from cross-project analysis), 'rule' (from static rules)

#### Scenario: Filter suggestions by source
- **WHEN** user views suggestion list
- **THEN** user can filter by source type
- **AND** counts are shown per source type
