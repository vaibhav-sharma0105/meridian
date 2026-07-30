# team-roster Specification

## Purpose
TBD - created by archiving change phase-7-team-sync. Update Purpose after archive.
## Requirements
### Requirement: Team member storage
The system SHALL maintain a local database of team members with deduplication across sources.

#### Scenario: Store team member
- **WHEN** team member is added (manual or sync)
- **THEN** system stores: id, name, email, avatar_url, source, source_id, role, expertise, metadata
- **AND** system generates unique ID if not provided
- **AND** system sets created_at timestamp

#### Scenario: Deduplicate across sources
- **WHEN** member exists with same (source, source_id) combination
- **THEN** system updates existing record instead of creating duplicate
- **AND** system updates last_synced_at timestamp

### Requirement: Manual team member entry
The system SHALL allow users to manually add, edit, and remove team members.

#### Scenario: Add team member manually
- **WHEN** user enters name and optional email in team settings
- **THEN** system creates team member with source="manual"
- **AND** member appears in team roster list

#### Scenario: Edit team member
- **WHEN** user edits team member details
- **THEN** system updates name, email, role, expertise fields
- **AND** system preserves source and source_id

#### Scenario: Remove team member
- **WHEN** user removes team member
- **THEN** system deletes member from team_members table
- **AND** member no longer appears in assignee suggestions

### Requirement: Slack workspace sync
The system SHALL populate team roster from connected Slack workspace.

#### Scenario: Sync Slack members
- **WHEN** Slack integration is connected
- **AND** user triggers team sync (manual or scheduled)
- **THEN** system fetches workspace members via Slack API
- **AND** system creates/updates team_members with source="slack"
- **AND** system stores Slack user ID in source_id

#### Scenario: Handle Slack sync errors
- **WHEN** Slack API returns error during sync
- **THEN** system preserves existing Slack-sourced members
- **AND** system shows error notification to user
- **AND** system logs error details

### Requirement: Google Workspace sync
The system SHALL populate team roster from connected Google Workspace (if integration exists).

#### Scenario: Sync Google members
- **WHEN** Google integration is connected with directory scope
- **AND** user triggers team sync
- **THEN** system fetches workspace members via Google Directory API
- **AND** system creates/updates team_members with source="google"

### Requirement: Team roster UI
The system SHALL provide a settings UI for managing the team roster.

#### Scenario: View team roster
- **WHEN** user opens Team settings section
- **THEN** system displays list of all team members
- **AND** each member shows: name, email, source badge, role
- **AND** members are grouped or filterable by source

#### Scenario: Sync button
- **WHEN** user clicks "Sync from Slack" or "Sync from Google"
- **THEN** system triggers sync for that source
- **AND** system shows sync progress indicator
- **AND** system displays count of added/updated members

### Requirement: Workload tracking
The system SHALL track current workload for each team member.

#### Scenario: Compute workload
- **WHEN** workload computation job runs (daily or on-demand)
- **THEN** system counts open tasks assigned to each member
- **AND** system computes workload_score (0-1, 1 = overloaded)
- **AND** system stores score in team_members.workload_score

#### Scenario: Display workload
- **WHEN** user views team roster
- **THEN** system shows workload indicator per member
- **AND** high workload (>0.8) shows warning color

