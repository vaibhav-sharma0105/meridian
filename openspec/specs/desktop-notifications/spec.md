## ADDED Requirements

### Requirement: Native OS notifications
The system SHALL send native operating system notifications using the Tauri notification plugin.

#### Scenario: Send desktop notification
- **WHEN** notification with desktop flag is created
- **THEN** system sends native OS notification with title and body

### Requirement: Notification severity levels
The system SHALL support three severity levels (info, warning, critical) with different presentation.

#### Scenario: Info notification
- **WHEN** info-level notification is triggered
- **THEN** system updates badge count only (no toast)

#### Scenario: Warning notification
- **WHEN** warning-level notification is triggered
- **THEN** system shows toast notification without sound

#### Scenario: Critical notification
- **WHEN** critical-level notification is triggered
- **THEN** system shows toast notification with sound

### Requirement: Sound for critical notifications
The system SHALL play a notification sound for critical-level notifications.

#### Scenario: Play notification sound
- **WHEN** critical notification is sent
- **THEN** system plays system notification sound

### Requirement: Notification click handling
The system SHALL handle notification clicks by focusing the app and navigating to relevant context.

#### Scenario: Click notification to navigate
- **WHEN** user clicks desktop notification about a task
- **THEN** app comes to foreground
- **THEN** app navigates to the task detail

### Requirement: User notification preferences
The system SHALL allow users to configure notification preferences (enable/disable desktop, sound, per-type settings).

#### Scenario: Disable desktop notifications
- **WHEN** user disables desktop notifications in settings
- **THEN** system stops sending OS notifications
- **THEN** in-app notifications continue

#### Scenario: Disable sound
- **WHEN** user disables notification sounds
- **THEN** critical notifications show toast without sound

### Requirement: Do Not Disturb respect
The system SHALL respect OS-level Do Not Disturb settings.

#### Scenario: DND mode active
- **WHEN** OS DND mode is enabled
- **THEN** system defers to OS handling (silent/queued)

### Requirement: Notification grouping
The system SHALL group multiple notifications when several arrive in quick succession.

#### Scenario: Group rapid notifications
- **WHEN** 5 notifications arrive within 10 seconds
- **THEN** system shows grouped notification with count

### Requirement: Notification permission request
The system SHALL request notification permissions on first use with clear explanation.

#### Scenario: First notification attempt
- **WHEN** system needs to send first desktop notification
- **THEN** system requests OS notification permission if not granted
