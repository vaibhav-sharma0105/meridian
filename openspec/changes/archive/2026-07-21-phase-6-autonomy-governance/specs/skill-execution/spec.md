## MODIFIED Requirements

### Requirement: Approval workflow

The system SHALL support approval modes integrated with the autonomy controller:
- `auto`: Execute immediately, no approval needed
- `notify`: Execute and notify user of results
- `approve_first`: Require approval before executing side effects
- `approve_always`: Require approval for every execution
- **NEW**: Approval mode is overridden by autonomy controller when autonomy mode is more restrictive

#### Scenario: Auto approval mode
- **WHEN** skill has approval "auto"
- **AND** autonomy mode allows automatic execution for this risk level
- **THEN** executes immediately without user interaction

#### Scenario: Autonomy overrides skill approval
- **WHEN** skill has approval "auto"
- **AND** global autonomy mode is "Manual"
- **THEN** system requires approval regardless of skill setting
- **AND** action is queued in pending_approvals

#### Scenario: Approve first mode
- **WHEN** skill has approval "approve_first" and action creates tasks
- **THEN** system shows preview of tasks to create
- **AND** waits for user approval or rejection
- **AND** only creates tasks if approved

#### Scenario: Risk-based approval routing
- **WHEN** skill action is classified as high-risk or critical-risk
- **THEN** system routes to approval queue regardless of skill approval setting
- **AND** action is not executed until approved

#### Scenario: Approval timeout
- **WHEN** approval_pending exceeds configured timeout (default 24 hours)
- **THEN** system archives the action (does not execute)
- **AND** notifies user of timeout
- **AND** action is retrievable from archived approvals

## ADDED Requirements

### Requirement: Skill autonomy override
The system SHALL support per-skill autonomy mode override.

#### Scenario: Skill-level autonomy setting
- **WHEN** user sets skill autonomy to "Autonomous"
- **AND** global autonomy is "Manual"
- **THEN** skill can execute low/medium-risk actions automatically
- **AND** high-risk actions still require approval

#### Scenario: Skill inherits autonomy
- **WHEN** skill autonomy is set to "Inherit"
- **THEN** skill uses integration autonomy (if targeting integration) or global autonomy

### Requirement: Skill action risk classification
The system SHALL classify skill actions by risk level before execution.

#### Scenario: Classify skill action
- **WHEN** skill is about to execute action
- **THEN** system classifies action using risk-classification engine
- **AND** routes to approval or executes based on autonomy mode and risk level

#### Scenario: Multi-action skill risk
- **WHEN** skill performs multiple actions
- **THEN** system uses highest risk level among all actions
- **AND** if any action requires approval, all actions are held pending approval
