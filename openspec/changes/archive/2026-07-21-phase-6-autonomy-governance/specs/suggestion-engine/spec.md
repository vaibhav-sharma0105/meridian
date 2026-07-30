## MODIFIED Requirements

### Requirement: Suggestion triggers skill execution

The system SHALL support triggering skills from suggestion acceptance with autonomy-aware routing:
- Suggestion action_config can include skill_id
- Accepting suggestion queues skill execution
- Skill receives suggestion context as input
- **NEW**: Execution is routed through approval flow based on autonomy mode and risk level

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

## ADDED Requirements

### Requirement: Suggestion risk classification
The system SHALL classify suggestions by risk level of their associated action.

#### Scenario: Classify suggestion risk
- **WHEN** suggestion is created with action_config
- **THEN** system evaluates action and assigns risk_level
- **AND** risk_level is stored with suggestion

#### Scenario: Display risk indicator
- **WHEN** suggestion is displayed in UI
- **THEN** risk level badge is shown (low=green, medium=yellow, high=orange, critical=red)

### Requirement: Autonomy-aware suggestion acceptance
The system SHALL route accepted suggestions through the autonomy controller.

#### Scenario: Low-risk suggestion in Supervised mode
- **WHEN** user accepts low-risk suggestion
- **AND** autonomy mode is "Supervised"
- **THEN** action executes immediately without additional approval

#### Scenario: High-risk suggestion in Supervised mode
- **WHEN** user accepts high-risk suggestion
- **AND** autonomy mode is "Supervised"
- **THEN** action is queued in pending_approvals
- **AND** user sees "Action queued for approval" message

#### Scenario: Any suggestion in Manual mode
- **WHEN** user accepts any suggestion
- **AND** autonomy mode is "Manual"
- **THEN** action is queued in pending_approvals
- **AND** requires explicit approval to execute

### Requirement: Suggestion batch approval
The system SHALL support batch actions on suggestions routed to approval.

#### Scenario: Multiple suggestions to approval
- **WHEN** user accepts multiple suggestions in batch
- **AND** any require approval based on autonomy/risk
- **THEN** all are queued in pending_approvals as a batch
- **AND** user can approve/reject batch or individual items
