# notification-system Specification

## Purpose
TBD - created by archiving change document-existing-system. Update Purpose after archive.
## Requirements
### Requirement: In-App Notifications

The system SHALL provide in-app notification system for displaying alerts, updates, and pending actions.

#### Scenario: Create notification
- **WHEN** system event occurs (sync complete, task due, etc.)
- **THEN** notification record is created with type, title, body, and timestamp

#### Scenario: Display notifications
- **WHEN** user opens notification center
- **THEN** system displays notifications sorted by date descending

#### Scenario: Mark notification read
- **WHEN** user views notification
- **THEN** system updates is_read flag

#### Scenario: Mark all read
- **WHEN** user clicks "Mark All Read"
- **THEN** system marks all notifications as read

### Requirement: Notification Center UI

The system SHALL provide a slide-out notification center panel accessible from the app header.

#### Scenario: Open notification center
- **WHEN** user clicks bell icon
- **THEN** notification panel slides in from right

#### Scenario: Close notification center
- **WHEN** user clicks outside panel or close button
- **THEN** panel closes

#### Scenario: Notification badge
- **WHEN** unread notifications exist
- **THEN** bell icon shows badge with unread count

### Requirement: Pending Imports Display

The system SHALL display pending meeting imports in the notification center for user review.

#### Scenario: Show pending imports
- **WHEN** pending imports exist
- **THEN** notification center shows "N New Meetings Found" section at top

#### Scenario: Pending import card
- **WHEN** pending import is displayed
- **THEN** card shows meeting title, source, preview, project selector, and import buttons

#### Scenario: Approve from notification center
- **WHEN** user selects project and clicks Import
- **THEN** system creates meeting and extracts tasks

#### Scenario: Dismiss from notification center
- **WHEN** user clicks dismiss on pending import
- **THEN** system marks import as dismissed

### Requirement: Notification Dismiss

The system SHALL allow users to dismiss individual notifications.

#### Scenario: Dismiss notification
- **WHEN** user clicks dismiss button on notification
- **THEN** notification is removed from list

### Requirement: Notification Data Model

The system SHALL store notifications with: id, type, title, body, task_id (optional), project_id (optional), is_read, and created_at.

#### Scenario: Notification types
- **WHEN** notification is created
- **THEN** type indicates category: sync_complete, task_due, import_ready, etc.

#### Scenario: Link to entity
- **WHEN** notification relates to task or project
- **THEN** notification stores reference ID for navigation

### Requirement: Notification Settings (Stub)

The system SHALL have settings for notification preferences including desktop notifications and email digests. Note: These settings exist but are not yet wired to actual functionality.

#### Scenario: Settings exist
- **WHEN** user views notification settings
- **THEN** toggles for desktop notifications and email digest are visible

#### Scenario: Settings not functional
- **WHEN** settings are changed
- **THEN** values are saved but do not affect behavior (pending implementation)

### Requirement: Sync Result Notifications

The system SHALL display toast notifications after sync operations complete.

#### Scenario: New imports found
- **WHEN** sync discovers new meetings
- **THEN** toast shows "N new meetings found"

#### Scenario: Duplicates skipped
- **WHEN** sync skips duplicates
- **THEN** toast shows "N duplicates skipped"

#### Scenario: Sync errors
- **WHEN** sync encounters errors
- **THEN** toast shows error summary

