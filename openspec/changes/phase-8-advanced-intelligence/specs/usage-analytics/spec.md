## ADDED Requirements

### Requirement: Activity metrics tracking
The system SHALL track activity metrics for tasks, meetings, and documents.

#### Scenario: Track task activity
- **WHEN** tasks are created, updated, or completed
- **THEN** system records daily aggregate: tasks_created, tasks_completed, tasks_updated
- **AND** system records breakdown by project

#### Scenario: Track meeting activity
- **WHEN** meetings are imported or created
- **THEN** system records daily aggregate: meetings_imported, by source (zoom, sheets, manual, mcp)
- **AND** system records tasks_extracted count

#### Scenario: Track document activity
- **WHEN** documents are uploaded or queried
- **THEN** system records daily aggregate: documents_uploaded, documents_queried, embeddings_generated

### Requirement: AI usage tracking
The system SHALL track AI provider usage for cost awareness.

#### Scenario: Track token consumption
- **WHEN** AI API call is made
- **THEN** system records: provider, model, input_tokens, output_tokens, timestamp
- **AND** system aggregates to daily totals by provider

#### Scenario: Track embedding operations
- **WHEN** embeddings are generated
- **THEN** system records: provider (bundled/ollama/openai), document_count, chunk_count, tokens_used

#### Scenario: Display AI cost estimates
- **WHEN** user views AI usage in Analytics
- **THEN** system displays: tokens by provider, estimated cost (based on public pricing), trend over time

### Requirement: Storage metrics tracking
The system SHALL track storage usage breakdown.

#### Scenario: Calculate storage breakdown
- **WHEN** usage aggregation job runs (daily)
- **THEN** system computes: database size, documents folder size, qdrant folder size, total

#### Scenario: Track storage by project
- **WHEN** computing storage metrics
- **THEN** system estimates per-project storage: tasks, meetings, documents, embeddings

#### Scenario: Storage warning threshold
- **WHEN** total storage exceeds 80% of configured limit (default 5GB)
- **THEN** system generates warning in Analytics
- **AND** system suggests: archive old projects, remove unused documents

### Requirement: Productivity insights
The system SHALL compute and display productivity insights.

#### Scenario: Calculate completion rate
- **WHEN** user views Productivity in Analytics
- **THEN** system displays: tasks completed / tasks created ratio, trend over time

#### Scenario: Calculate average completion time
- **WHEN** tasks have lifecycle events (created → completed)
- **THEN** system computes: average time to completion, by priority, by project

#### Scenario: Identify overdue trends
- **WHEN** computing productivity insights
- **THEN** system shows: overdue task rate, average days overdue, improving or worsening trend

#### Scenario: Follow-through rate
- **WHEN** meetings have extracted tasks
- **THEN** system computes: percentage of meeting tasks completed, average days to complete meeting tasks

### Requirement: Skill and suggestion metrics
The system SHALL track skill execution and suggestion effectiveness.

#### Scenario: Track skill executions
- **WHEN** skills run
- **THEN** system records: skill_id, status (success/failed), execution_time, output_type

#### Scenario: Track suggestion outcomes
- **WHEN** suggestions are accepted, dismissed, or expired
- **THEN** system records: suggestion_type, outcome, time_to_action

#### Scenario: Display automation ROI
- **WHEN** user views Skills in Analytics
- **THEN** system shows: skills by usage count, success rate, time saved estimate

### Requirement: Analytics dashboard UI
The system SHALL provide a comprehensive analytics dashboard.

#### Scenario: View analytics dashboard
- **WHEN** user clicks Analytics in sidebar
- **THEN** system displays dashboard with sections: Activity, AI Usage, Storage, Productivity, Automation

#### Scenario: Time range selection
- **WHEN** viewing analytics
- **THEN** user can select: Today, This Week, This Month, Last 30 Days, Custom Range
- **AND** all metrics update for selected range

#### Scenario: Project filter
- **WHEN** viewing analytics
- **THEN** user can filter by: All Projects, or specific project
- **AND** metrics show filtered data

#### Scenario: Comparison view
- **WHEN** user selects comparison mode
- **THEN** system shows: current period vs previous period, percentage change, trend indicators

### Requirement: Analytics export
The system SHALL allow exporting analytics data.

#### Scenario: Export to CSV
- **WHEN** user clicks "Export" in Analytics
- **THEN** system generates CSV with: date, metric_type, metric_key, value
- **AND** user can select which sections to include

#### Scenario: Export to JSON
- **WHEN** user selects JSON export format
- **THEN** system generates structured JSON with nested metrics by category

#### Scenario: Scheduled export
- **WHEN** user enables "Weekly Analytics Email" (if email integration exists)
- **OR** user creates a skill with analytics export action
- **THEN** system exports analytics on schedule

### Requirement: Metrics data retention
The system SHALL manage metrics data retention.

#### Scenario: Auto-prune old metrics
- **WHEN** usage_metrics data is older than 2 years
- **THEN** system auto-prunes during daily maintenance job
- **AND** system logs pruning in audit log

#### Scenario: Export before prune warning
- **WHEN** metrics are approaching 2-year retention limit
- **THEN** system shows warning in Analytics: "Data older than [date] will be pruned. Export now?"
