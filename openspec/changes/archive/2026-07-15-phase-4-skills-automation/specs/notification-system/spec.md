## ADDED Requirements

### Requirement: Skill output notifications

The system SHALL create notifications from skill execution results:
- Skills with notify approval mode create notification on completion
- Notification includes skill name, output preview, and link to full results
- Severity based on skill configuration

#### Scenario: Skill completion notification
- **WHEN** skill completes with approval mode "notify"
- **THEN** system creates notification with type "skill_completed"
- **AND** body contains output summary (first 200 chars)

#### Scenario: Skill failure notification
- **WHEN** skill execution fails
- **THEN** system creates notification with type "skill_failed"
- **AND** body contains error message

### Requirement: Approval pending notifications

The system SHALL notify users of skills awaiting approval:
- Skills with approval_pending status create notification
- Notification includes action preview and approve/reject buttons
- Clicking notification opens skill approval modal

#### Scenario: Approval notification created
- **WHEN** skill enters approval_pending status
- **THEN** notification created with type "skill_approval_needed"
- **AND** notification links to skill run details

#### Scenario: Approval notification cleared
- **WHEN** user approves or rejects skill
- **THEN** related notification is dismissed

## MODIFIED Requirements

### Requirement: Notification Data Model

The system SHALL store notifications with: id, type, title, body, task_id (optional), project_id (optional), skill_run_id (optional), is_read, and created_at.

#### Scenario: Notification types
- **WHEN** notification is created
- **THEN** type indicates category: sync_complete, task_due, import_ready, skill_completed, skill_failed, skill_approval_needed, etc.

#### Scenario: Link to entity
- **WHEN** notification relates to task, project, or skill run
- **THEN** notification stores reference ID for navigation
