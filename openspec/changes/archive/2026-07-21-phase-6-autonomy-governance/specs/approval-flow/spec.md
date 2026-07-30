## ADDED Requirements

### Requirement: Pending approvals queue
The system SHALL maintain a queue of actions pending user approval.

#### Scenario: Queue action for approval
- **WHEN** agent action requires approval based on autonomy mode and risk level
- **THEN** system creates pending_approval record with action details
- **AND** system displays approval in notification panel
- **AND** action does NOT execute until approved

#### Scenario: View pending approvals
- **WHEN** user opens approval queue
- **THEN** system displays list of pending actions sorted by creation time
- **AND** each item shows: action summary, risk level, source (skill/suggestion), context

### Requirement: Approval with context
The system SHALL display full context and reasoning for each pending approval.

#### Scenario: View approval detail
- **WHEN** user clicks on pending approval item
- **THEN** system displays: full action description, risk classification breakdown, source trigger, affected entities
- **AND** system displays "Why this needs approval" explanation

#### Scenario: Show related history
- **WHEN** user views approval detail
- **THEN** system shows previous similar actions and their outcomes

### Requirement: Approve action
The system SHALL execute approved actions immediately.

#### Scenario: Approve single action
- **WHEN** user clicks "Approve" on pending action
- **THEN** system executes the action
- **AND** system records approval in audit log
- **AND** system removes action from pending queue
- **AND** system shows success notification

#### Scenario: Approve with modification
- **WHEN** user clicks "Approve with Edit" on pending action
- **THEN** system shows action details in editable form
- **AND** user can modify action parameters
- **AND** system executes modified action on confirm

### Requirement: Reject action
The system SHALL cancel rejected actions without execution.

#### Scenario: Reject single action
- **WHEN** user clicks "Reject" on pending action
- **THEN** system does NOT execute the action
- **AND** system records rejection in audit log
- **AND** system removes action from pending queue
- **AND** system optionally prompts for rejection reason

#### Scenario: Reject with feedback
- **WHEN** user rejects with reason "Never suggest this"
- **THEN** system records negative pattern observation
- **AND** similar actions are suppressed in future

### Requirement: Bulk approval actions
The system SHALL support bulk approve/reject for multiple pending actions.

#### Scenario: Bulk approve
- **WHEN** user selects multiple pending actions and clicks "Approve All"
- **THEN** system executes all selected actions in sequence
- **AND** system records each approval in audit log

#### Scenario: Bulk reject
- **WHEN** user selects multiple pending actions and clicks "Reject All"
- **THEN** system rejects all selected actions
- **AND** system records each rejection in audit log

### Requirement: Approval timeout
The system SHALL support configurable timeout for pending approvals.

#### Scenario: Configure timeout
- **WHEN** user sets approval timeout to 30 minutes in settings
- **THEN** pending approvals older than 30 minutes are auto-archived

#### Scenario: Timeout reached
- **WHEN** pending approval reaches timeout
- **THEN** system archives the action (does NOT execute)
- **AND** system creates notification "Action expired: [summary]"
- **AND** action is retrievable from archive

### Requirement: Archived actions retrieval
The system SHALL allow users to retrieve and execute archived (timed-out) actions.

#### Scenario: View archived actions
- **WHEN** user opens "Archived" tab in approval queue
- **THEN** system displays list of timed-out actions

#### Scenario: Execute archived action
- **WHEN** user clicks "Execute Now" on archived action
- **THEN** system shows warning "This action was created [time] ago. Context may have changed."
- **AND** on confirm, system executes the action
- **AND** system records execution with "archived_execution" flag in audit log

### Requirement: Approval notifications
The system SHALL notify users of pending approvals based on severity.

#### Scenario: Critical approval notification
- **WHEN** critical-risk action is queued for approval
- **THEN** system sends desktop notification with sound
- **AND** shows prominent badge in UI

#### Scenario: High-risk approval notification
- **WHEN** high-risk action is queued for approval
- **THEN** system sends desktop notification (no sound)
- **AND** increments approval badge count

#### Scenario: Medium-risk approval notification
- **WHEN** medium-risk action is queued for approval
- **THEN** system increments approval badge count only
