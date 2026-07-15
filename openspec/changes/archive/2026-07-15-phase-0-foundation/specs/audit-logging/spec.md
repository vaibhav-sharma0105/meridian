## ADDED Requirements

### Requirement: Action Logging

The system SHALL log all significant actions with: timestamp, action_type, entity_type, entity_id, details (JSON), agent_initiated flag, autonomy_mode, and risk_level.

#### Scenario: Log user action
- **WHEN** user performs action (create, update, delete task/meeting/etc.)
- **THEN** system creates audit log entry with agent_initiated=false

#### Scenario: Log agent action
- **WHEN** agent performs action
- **THEN** system creates audit log entry with agent_initiated=true and reasoning in details

#### Scenario: Log external action
- **WHEN** action affects external system (Slack message, GitHub comment)
- **THEN** system logs with action_type indicating external target

### Requirement: Audit Log Schema

The system SHALL store audit logs in `audit_log` table with required fields.

#### Scenario: Create audit entry
- **WHEN** action is logged
- **THEN** entry contains: id, timestamp, action_type, entity_type, entity_id, details, agent_initiated, autonomy_mode, risk_level, created_at

#### Scenario: Action types
- **WHEN** logging action
- **THEN** action_type is one of: create, update, delete, send, approve, reject, sync, login

#### Scenario: Entity types
- **WHEN** logging action
- **THEN** entity_type is one of: task, meeting, project, document, skill, message, integration, settings

### Requirement: Retention Policy

The system SHALL retain audit logs for 2 years and auto-prune older entries.

#### Scenario: Prune old logs
- **WHEN** daily cleanup job runs
- **THEN** system deletes audit entries older than 2 years

#### Scenario: Retention configuration
- **WHEN** user views audit settings
- **THEN** retention period is displayed (not configurable - fixed at 2 years)

### Requirement: Audit Log Viewer

The system SHALL provide UI for viewing audit logs in Settings > Advanced panel.

#### Scenario: View audit log
- **WHEN** user opens audit log viewer
- **THEN** system displays paginated log entries, newest first

#### Scenario: Filter by action type
- **WHEN** user selects action type filter
- **THEN** system shows only entries matching filter

#### Scenario: Filter by entity
- **WHEN** user selects entity type and/or entity_id
- **THEN** system shows only entries for that entity

#### Scenario: Filter by agent actions
- **WHEN** user toggles "Agent actions only"
- **THEN** system shows only entries with agent_initiated=true

#### Scenario: Filter by date range
- **WHEN** user selects date range
- **THEN** system shows only entries within range

### Requirement: Audit Log Export

The system SHALL support exporting audit logs for compliance purposes.

#### Scenario: Export to JSON
- **WHEN** user clicks export with current filters
- **THEN** system downloads JSON file with filtered entries

#### Scenario: Export to CSV
- **WHEN** user selects CSV format
- **THEN** system downloads CSV file with filtered entries

### Requirement: Risk Level Classification

The system SHALL classify actions by risk level: low, medium, high, critical.

#### Scenario: Classify read actions
- **WHEN** action is read-only (view, list)
- **THEN** risk_level is "low"

#### Scenario: Classify internal writes
- **WHEN** action modifies local data only
- **THEN** risk_level is "medium"

#### Scenario: Classify external writes
- **WHEN** action affects external system
- **THEN** risk_level is "high"

#### Scenario: Classify destructive actions
- **WHEN** action deletes data or affects many records
- **THEN** risk_level is "critical"

### Requirement: Immediate Failure Notification

The system SHALL immediately notify user of failed agent actions with user-friendly message.

#### Scenario: Agent action fails
- **WHEN** agent action fails
- **THEN** system shows toast notification with friendly error summary

#### Scenario: View failure details
- **WHEN** user clicks on failure notification
- **THEN** system opens audit log filtered to that entry with full error details

### Requirement: Audit Context Preservation

The system SHALL preserve context for audit entries to enable investigation.

#### Scenario: Store reasoning
- **WHEN** agent takes action
- **THEN** details JSON includes reasoning, confidence, and inputs used

#### Scenario: Store request/response
- **WHEN** external API call is made
- **THEN** details JSON includes sanitized request and response (no secrets)

### Requirement: Audit Log UX Enhancements

The system SHALL provide user-friendly defaults and navigation for audit log viewing.

#### Scenario: Default date filter
- **WHEN** user opens audit log viewer
- **THEN** system defaults to showing last 7 days of entries

#### Scenario: Quick date filters
- **WHEN** user views filter options
- **THEN** UI shows quick filter buttons: Today, 7 days, 30 days, All

#### Scenario: Entry count display
- **WHEN** entries are loaded
- **THEN** UI shows "Showing X of Y entries" with total count

#### Scenario: Pagination with loading state
- **WHEN** user clicks "Load more"
- **THEN** UI shows loading spinner and appends new entries

#### Scenario: Fullscreen mode
- **WHEN** user clicks fullscreen button in audit log viewer
- **THEN** system displays audit log in 90% viewport modal with all functionality
- **AND** user can exit by clicking backdrop or minimize button
