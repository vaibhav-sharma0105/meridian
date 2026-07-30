## ADDED Requirements

### Requirement: MCP Tools - Write Operations
The system SHALL expose write tools for external agents to create and modify Meridian data.

#### Scenario: create_task tool
- **WHEN** agent calls create_task with title, project_id, and optional fields
- **THEN** server creates task and returns created task details

#### Scenario: update_task tool
- **WHEN** agent calls update_task with task_id and fields to update
- **THEN** server updates task and returns updated task details

#### Scenario: create_meeting_note tool
- **WHEN** agent calls create_meeting_note with project_id, title, and content
- **THEN** server creates meeting record and returns meeting details

#### Scenario: run_skill tool
- **WHEN** agent calls run_skill with skill_id
- **THEN** server queues skill execution and returns run_id

### Requirement: MCP Permission System
The system SHALL enforce configurable permissions for MCP write operations.

#### Scenario: Permission check on write
- **WHEN** agent calls write tool
- **THEN** server checks if operation is allowed by user configuration
- **THEN** if denied, server returns permission error

#### Scenario: Configure MCP permissions
- **WHEN** user opens MCP settings
- **THEN** user can enable/disable: create_task, update_task, create_meeting_note, run_skill

### Requirement: MCP Audit Logging
The system SHALL log all MCP write operations to the audit log.

#### Scenario: Log MCP write
- **WHEN** agent successfully executes write operation
- **THEN** system logs action with agent context and operation details

### Requirement: MCP Rate Limiting
The system SHALL rate-limit MCP operations to prevent abuse.

#### Scenario: Exceed rate limit
- **WHEN** agent exceeds 100 operations per minute
- **THEN** server returns rate limit error with retry-after header

## MODIFIED Requirements

### Requirement: MCP Database Access
The system SHALL access the same SQLite database as the main Meridian app with read-write capability when permissions allow.

#### Scenario: Shared database
- **WHEN** MCP server starts
- **THEN** server connects to ~/.meridian/meridian.db

#### Scenario: Write access when permitted
- **WHEN** MCP server has write permissions configured
- **THEN** operations can modify database via write tools

#### Scenario: Read-only fallback
- **WHEN** MCP server has no write permissions
- **THEN** write tool calls return permission denied error
