## ADDED Requirements

### Requirement: Schedule trigger type

The system SHALL support cron-based schedule triggers with:
- `cron`: Standard cron expression (5 or 6 field format)
- `timezone`: IANA timezone for execution (defaults to system timezone)
- `next_run_at`: Computed next execution time

#### Scenario: Create schedule trigger with daily cron
- **WHEN** user creates a skill with trigger type "schedule" and cron "0 9 * * *"
- **THEN** system validates cron syntax
- **AND** computes next_run_at based on timezone
- **AND** stores the trigger configuration

#### Scenario: Invalid cron expression
- **WHEN** user creates a skill with invalid cron expression
- **THEN** system rejects with specific syntax error

#### Scenario: Cron with timezone
- **WHEN** user creates a skill with cron "0 17 * * 5" and timezone "America/New_York"
- **THEN** system schedules execution for 5pm Friday in Eastern time

### Requirement: Event trigger type

The system SHALL support event-based triggers fired when specific actions occur:
- `event_type`: Event name (task_created, task_completed, meeting_imported, etc.)
- `filter`: Optional conditions to match (project_id, priority, etc.)

#### Scenario: Trigger on task creation
- **WHEN** a task is created matching skill's event filter
- **THEN** system queues the skill for execution
- **AND** passes event payload as context

#### Scenario: Event filter with project scope
- **WHEN** skill has event trigger with filter { project_id: "proj-1" }
- **AND** task is created in proj-1
- **THEN** skill fires

#### Scenario: Event filter excludes non-matching events
- **WHEN** skill has event trigger with filter { priority: "critical" }
- **AND** task is created with priority "low"
- **THEN** skill does NOT fire

### Requirement: Manual trigger type

The system SHALL support manual triggers invoked on-demand by the user.

#### Scenario: Manually trigger skill
- **WHEN** user clicks "Run" on a manual skill
- **THEN** system immediately queues the skill for execution

#### Scenario: Manual trigger with input
- **WHEN** manual skill defines input parameters
- **THEN** UI prompts for required inputs before execution

### Requirement: Supported event types

The system SHALL fire events for:
- `task_created`: When a new task is created
- `task_completed`: When task status changes to "done"
- `task_overdue`: When task passes due date (daemon check)
- `meeting_imported`: When a meeting is imported from any source
- `suggestion_accepted`: When user accepts a suggestion
- `daily_start`: Daemon event at configured start-of-day time
- `weekly_start`: Daemon event at start of configured work week

#### Scenario: Task completed event payload
- **WHEN** task status changes to "done"
- **THEN** system fires task_completed event
- **AND** payload includes task_id, project_id, title, assignee, completed_at

#### Scenario: Meeting imported event payload
- **WHEN** meeting is imported via Zoom or Sheets Relay
- **THEN** system fires meeting_imported event
- **AND** payload includes meeting_id, project_id, title, platform, attendees
