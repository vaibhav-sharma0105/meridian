## ADDED Requirements

### Requirement: Daemon Process

The system SHALL provide a background daemon process for executing scheduled jobs and async tasks. The daemon SHALL be a separate Rust binary using tokio runtime.

#### Scenario: Daemon startup with app
- **WHEN** main application starts
- **THEN** system launches daemon process if not already running

#### Scenario: Daemon standalone mode
- **WHEN** user enables "Keep running when app closed" setting
- **THEN** daemon continues running after main app exits

#### Scenario: Daemon shutdown
- **WHEN** main app exits and standalone mode is disabled
- **THEN** daemon shuts down gracefully after completing active jobs

### Requirement: Inter-Process Communication

The system SHALL use Unix sockets (macOS/Linux) or named pipes (Windows) for IPC between main app and daemon.

#### Scenario: Send job to daemon
- **WHEN** main app needs to schedule a job
- **THEN** app sends job request via IPC socket

#### Scenario: Receive job result
- **WHEN** daemon completes a job
- **THEN** daemon sends result via IPC to main app (if running)

#### Scenario: Store result for offline app
- **WHEN** daemon completes job while main app is closed
- **THEN** daemon persists result to database for app to read on next launch

### Requirement: Job Queue

The system SHALL maintain a persistent job queue for scheduled and pending jobs.

#### Scenario: Queue job
- **WHEN** job is submitted
- **THEN** system adds to queue with priority, scheduled_at, and retry_count

#### Scenario: Process job
- **WHEN** daemon picks job from queue
- **THEN** system marks job as in_progress and executes

#### Scenario: Job completion
- **WHEN** job completes successfully
- **THEN** system marks job as completed and stores result

#### Scenario: Job failure with retry
- **WHEN** job fails and retry_count < max_retries
- **THEN** system increments retry_count and reschedules with backoff

#### Scenario: Job failure final
- **WHEN** job fails and retry_count >= max_retries
- **THEN** system marks job as failed and notifies user

### Requirement: Cron Scheduling

The system SHALL support cron-style scheduling for recurring jobs.

#### Scenario: Register cron job
- **WHEN** skill with schedule trigger is enabled
- **THEN** system registers cron entry with daemon

#### Scenario: Execute cron job
- **WHEN** cron schedule matches current time
- **THEN** daemon queues job for execution

#### Scenario: Missed cron execution
- **WHEN** daemon was not running at scheduled time
- **THEN** system executes missed job on next startup (catch-up mode)

### Requirement: System Scheduler Integration

The system SHALL integrate with OS-level schedulers as fallback for daemon.

#### Scenario: Register with launchd (macOS)
- **WHEN** user enables system scheduler integration
- **THEN** system creates launchd plist to start daemon at login

#### Scenario: Register with Task Scheduler (Windows)
- **WHEN** user enables system scheduler integration on Windows
- **THEN** system creates scheduled task to start daemon at login

#### Scenario: Wake app for missed jobs
- **WHEN** system scheduler starts daemon and jobs were missed
- **THEN** daemon executes missed jobs (wake-on-launch mode)

### Requirement: Daemon Status

The system SHALL display daemon status in UI.

#### Scenario: Show daemon running
- **WHEN** daemon is running
- **THEN** UI shows green status indicator with "Background service active"

#### Scenario: Show daemon stopped
- **WHEN** daemon is not running
- **THEN** UI shows gray indicator with "Background service inactive"

#### Scenario: Show active jobs
- **WHEN** jobs are in progress
- **THEN** UI shows job count and current job description

### Requirement: Job Types

The system SHALL support multiple job types: embedding, sync, skill_execution, notification, cleanup.

#### Scenario: Embedding job
- **WHEN** document needs embedding
- **THEN** daemon processes embedding in background

#### Scenario: Sync job
- **WHEN** scheduled sync time arrives
- **THEN** daemon executes integration sync

#### Scenario: Skill execution job
- **WHEN** skill schedule triggers
- **THEN** daemon executes skill and stores output

#### Scenario: Cleanup job
- **WHEN** daily cleanup time arrives
- **THEN** daemon prunes old audit logs, expired data
