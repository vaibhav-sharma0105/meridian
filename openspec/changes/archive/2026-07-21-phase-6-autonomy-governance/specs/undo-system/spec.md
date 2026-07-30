## ADDED Requirements

### Requirement: Action history tracking
The system SHALL track all executed agent actions with sufficient detail to support undo.

#### Scenario: Record action for undo
- **WHEN** agent action executes successfully
- **THEN** system records action_id, action_type, entity_type, entity_id, before_state, after_state, timestamp
- **AND** system stores in action_history table

#### Scenario: Record non-undoable action
- **WHEN** agent executes external_send action (Slack message, GitHub comment)
- **THEN** system records action with undoable=false flag
- **AND** before_state is null (cannot be restored)

### Requirement: Instant undo for last action
The system SHALL provide instant undo for the most recent action.

#### Scenario: Show undo bar
- **WHEN** agent action completes
- **THEN** system displays undo bar at bottom of screen for 10 seconds
- **AND** bar shows "Action: [summary] — Undo"

#### Scenario: Execute instant undo
- **WHEN** user clicks "Undo" within 10 seconds
- **THEN** system reverses the action
- **AND** system restores entity to before_state
- **AND** system records undo in audit log

#### Scenario: Undo bar dismissal
- **WHEN** 10 seconds elapse OR user clicks dismiss OR user performs new action
- **THEN** undo bar disappears
- **AND** action can still be undone via action history

### Requirement: Action history view
The system SHALL provide a scrollable history of recent agent actions.

#### Scenario: View action history
- **WHEN** user opens action history panel
- **THEN** system displays list of recent actions (last 100)
- **AND** each item shows: timestamp, action summary, undoable status, undo button

#### Scenario: Filter action history
- **WHEN** user filters history by action_type or date range
- **THEN** system displays filtered list of actions

### Requirement: Selective undo from history
The system SHALL allow undoing any undoable action from history.

#### Scenario: Undo historical action
- **WHEN** user clicks "Undo" on action in history
- **THEN** system shows confirmation with warning about intermediate changes
- **AND** on confirm, system creates reversal action
- **AND** system executes reversal

#### Scenario: Undo blocked by dependencies
- **WHEN** action has dependent subsequent actions
- **THEN** system shows warning "This action has N dependent actions. Undoing may cause inconsistencies."
- **AND** user can choose to proceed or cancel

### Requirement: Reversal execution
The system SHALL execute undo as a reversal action (not true rollback).

#### Scenario: Undo create action
- **WHEN** user undoes "create_task" action
- **THEN** system executes "delete_task" with the created entity_id
- **AND** records as "undo_create" in audit log

#### Scenario: Undo update action
- **WHEN** user undoes "update_task" action
- **THEN** system executes "update_task" with before_state values
- **AND** records as "undo_update" in audit log

#### Scenario: Undo delete action
- **WHEN** user undoes "delete_task" action
- **THEN** system executes "create_task" with before_state values
- **AND** may generate new entity_id
- **AND** records as "undo_delete" in audit log

### Requirement: Non-undoable action marking
The system SHALL clearly mark actions that cannot be undone.

#### Scenario: Display non-undoable indicator
- **WHEN** action is external_send (Slack message, GitHub issue, email)
- **THEN** system displays "Cannot undo" badge on action
- **AND** undo button is disabled with tooltip "External actions cannot be undone"

#### Scenario: Warning before non-undoable action
- **WHEN** agent is about to execute non-undoable action
- **THEN** approval dialog includes warning "This action cannot be undone"

### Requirement: Undo confirmation
The system SHALL require confirmation for undo actions that may cause data loss.

#### Scenario: Confirm undo of create
- **WHEN** user undoes create action
- **THEN** system shows "This will delete [entity]. Continue?"

#### Scenario: Confirm undo with warnings
- **WHEN** undo may affect linked entities
- **THEN** system shows warning about affected entities before proceeding

### Requirement: Undo audit trail
The system SHALL maintain complete audit trail of undo operations.

#### Scenario: Log undo operation
- **WHEN** undo operation completes
- **THEN** audit log entry includes: original_action_id, undo_action_id, user_initiated=true
- **AND** original action is linked to its undo
