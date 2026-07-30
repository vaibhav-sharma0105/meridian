## ADDED Requirements

### Requirement: Desktop notification flag
The system SHALL support a `desktop` flag on notifications to trigger OS-level notifications.

#### Scenario: Create notification with desktop flag
- **WHEN** notification is created with `desktop: true`
- **THEN** system sends both in-app and OS notification

#### Scenario: Desktop flag respects user preference
- **WHEN** user has disabled desktop notifications in settings
- **THEN** desktop flag is ignored, only in-app notification shown

### Requirement: Notification severity
The system SHALL support severity levels (info, warning, critical) affecting notification presentation.

#### Scenario: Set notification severity
- **WHEN** notification is created with severity "critical"
- **THEN** notification displays with critical styling
- **THEN** if desktop enabled, OS notification includes sound

### Requirement: Integration notifications
The system SHALL create notifications for integration events (sync complete, sync failed, external updates).

#### Scenario: Integration sync notification
- **WHEN** integration sync completes with new items
- **THEN** system creates notification with type "integration_sync"
- **THEN** body includes count of new items per type

#### Scenario: Integration error notification
- **WHEN** integration sync fails
- **THEN** system creates notification with type "integration_error"
- **THEN** severity set to "warning"

## MODIFIED Requirements

### Requirement: Notification Data Model
The system SHALL store notifications with: id, type, title, body, severity (info/warning/critical), desktop (boolean), task_id (optional), project_id (optional), skill_run_id (optional), integration_id (optional), is_read, and created_at.

#### Scenario: Notification types
- **WHEN** notification is created
- **THEN** type indicates category: sync_complete, task_due, import_ready, skill_completed, skill_failed, skill_approval_needed, integration_sync, integration_error, etc.

#### Scenario: Link to entity
- **WHEN** notification relates to task, project, skill run, or integration
- **THEN** notification stores reference ID for navigation

#### Scenario: Default severity
- **WHEN** notification is created without severity
- **THEN** severity defaults to "info"
