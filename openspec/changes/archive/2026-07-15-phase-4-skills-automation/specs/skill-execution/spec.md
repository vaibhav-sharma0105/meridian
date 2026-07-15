## ADDED Requirements

### Requirement: Background execution via daemon

The system SHALL execute scheduled and event-triggered skills via the daemon:
- Daemon polls for pending scheduled skills every 60 seconds
- Event-triggered skills queue immediately when event fires
- Execution creates `skill_runs` record with status tracking

#### Scenario: Scheduled skill fires
- **WHEN** daemon detects skill with next_run_at <= now
- **THEN** system queues execution job
- **AND** updates next_run_at to next scheduled time

#### Scenario: Event skill fires
- **WHEN** registered event occurs matching skill filter
- **THEN** system queues execution job immediately
- **AND** includes event payload in job context

### Requirement: Job queue integration

The system SHALL use the existing daemon_jobs table for skill execution:
- Job type: "execute_skill"
- Job payload includes skill_id and trigger context
- Jobs follow existing priority and retry logic

#### Scenario: Skill job queued
- **WHEN** skill execution is requested
- **THEN** system creates daemon_jobs record with type "execute_skill"
- **AND** sets priority based on skill configuration (default 5)

#### Scenario: Failed job retry
- **WHEN** skill execution fails with retryable error
- **THEN** job is retried up to 3 times with exponential backoff

### Requirement: Execution status tracking

The system SHALL track skill run status:
- `pending`: Queued but not started
- `running`: Currently executing
- `completed`: Finished successfully
- `failed`: Execution error
- `partial_failure`: Some steps completed, some failed
- `cancelled`: Cancelled by user or system
- `approval_pending`: Waiting for user approval

#### Scenario: Status progression
- **WHEN** skill starts executing
- **THEN** status changes pending → running → completed/failed

#### Scenario: Approval hold
- **WHEN** skill requires approval and action has side effects
- **THEN** status changes to approval_pending
- **AND** creates notification for user

### Requirement: Timeout handling

The system SHALL enforce execution timeouts:
- Default timeout: 60 seconds
- Configurable per-skill: 10-300 seconds
- Timeout triggers cancellation and error recording

#### Scenario: Skill times out
- **WHEN** execution exceeds timeout
- **THEN** system cancels execution
- **AND** records timeout error in run details
- **AND** status set to "failed"

### Requirement: Approval workflow

The system SHALL support approval modes following progressive disclosure:
- `auto`: Execute immediately, no approval needed
- `notify`: Execute and notify user of results
- `approve_first`: Require approval before executing side effects
- `approve_always`: Require approval for every execution

#### Scenario: Auto approval mode
- **WHEN** skill has approval "auto"
- **THEN** executes immediately without user interaction

#### Scenario: Approve first mode
- **WHEN** skill has approval "approve_first" and action creates tasks
- **THEN** system shows preview of tasks to create
- **AND** waits for user approval or rejection
- **AND** only creates tasks if approved

#### Scenario: Approval timeout
- **WHEN** approval_pending exceeds 24 hours
- **THEN** system marks run as cancelled
- **AND** notifies user of timeout
