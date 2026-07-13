## ADDED Requirements

### Requirement: MCP Server Binary

The system SHALL provide a separate MCP server binary (meridian-mcp) that exposes Meridian data to external AI agents via Model Context Protocol.

#### Scenario: Build MCP server
- **WHEN** running npm run build:mcp
- **THEN** system produces meridian-mcp binary in src-tauri/target/release/

#### Scenario: Run MCP server
- **WHEN** external agent connects via stdio
- **THEN** MCP server accepts JSON-RPC 2.0 requests

### Requirement: MCP Protocol Compliance

The system SHALL implement MCP protocol with JSON-RPC 2.0 over stdio transport.

#### Scenario: Handle initialize
- **WHEN** agent sends initialize request
- **THEN** server responds with capabilities and protocol version

#### Scenario: Handle tool calls
- **WHEN** agent calls a tool
- **THEN** server executes tool and returns result

#### Scenario: Handle resource reads
- **WHEN** agent reads a resource
- **THEN** server returns resource content

### Requirement: MCP Tools - Read Operations

The system SHALL expose read-only tools for querying Meridian data.

#### Scenario: list_projects tool
- **WHEN** agent calls list_projects
- **THEN** server returns all projects with task counts

#### Scenario: list_tasks tool
- **WHEN** agent calls list_tasks with optional filters
- **THEN** server returns tasks matching filters (status, priority, assignee, due_date, text_search)

#### Scenario: get_task tool
- **WHEN** agent calls get_task with task_id
- **THEN** server returns detailed task info with project and meeting context

#### Scenario: list_meetings tool
- **WHEN** agent calls list_meetings with optional project_id
- **THEN** server returns meetings, optionally filtered by project

#### Scenario: get_meeting tool
- **WHEN** agent calls get_meeting with meeting_id
- **THEN** server returns full meeting details including transcript and extracted tasks

#### Scenario: get_task_context tool
- **WHEN** agent calls get_task_context with task_id
- **THEN** server returns rich context: task + project + source meeting excerpt

### Requirement: MCP Resources

The system SHALL expose Meridian data as MCP resources for agent browsing.

#### Scenario: Projects resource
- **WHEN** agent reads meridian://projects
- **THEN** server returns project list

#### Scenario: Tasks resource
- **WHEN** agent reads meridian://tasks
- **THEN** server returns all tasks

#### Scenario: Meetings resource
- **WHEN** agent reads meridian://meetings
- **THEN** server returns all meetings

### Requirement: MCP Configuration

The system SHALL provide MCP configuration file for Claude Code integration.

#### Scenario: MCP config file
- **WHEN** .mcp.json exists in project root
- **THEN** Claude Code can discover and connect to meridian-mcp server

### Requirement: MCP Database Access

The system SHALL access the same SQLite database as the main Meridian app.

#### Scenario: Shared database
- **WHEN** MCP server starts
- **THEN** server connects to ~/.meridian/meridian.db

#### Scenario: Read-only access
- **WHEN** MCP server accesses database
- **THEN** operations are read-only (no writes currently supported)
