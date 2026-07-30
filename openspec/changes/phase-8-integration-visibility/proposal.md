## Why

Meridian has external integrations (GitHub, Jira, Slack) but the data fetched from them is invisible to both users and the AI assistant. Users cannot browse integration data, the embedded AI chat cannot answer questions about GitHub issues or Jira tickets, and there's no visibility into what background jobs are running. This phase makes integration data accessible throughout the application.

## What Changes

- **AI Chat Integration Access**: Embedded AI chat can answer questions about and use integration data as context when helping users
- **Task Metadata in AI Context**: Dates, assignees, priority, status automatically included in AI conversations
- **Project-Scoped Integration Browser**: UI to view fetched integration data (GitHub issues/PRs, Jira items, Slack threads) per project
- **My Activity Dashboard**: Global view of items needing attention across all integrations; designed for morning standup use case ("What do I need to deal with today?"); sidebar badge shows count of attention-needed items
- **Clickable External Links**: All integration items link directly to their source (GitHub, Jira, Slack)
- **Cache Management**: User-controlled cache clearing with Delete/Archive options and automated cleanup cron (configurable N days retention)
- **Integration Linking Workflow**: AI-suggested, manual, and lazy linking methods; user picks preferred workflow during integration setup
- **Background Jobs Visibility**: Active, recent, and scheduled jobs visible in UI with expandable details; shown in both Integrations page (scoped to integration jobs) and Settings → Advanced (all jobs), cross-linked between views

## Capabilities

### New Capabilities
- `integration-context`: Feed integration_cache data into AI chat context for answering questions and providing relevant information
- `integration-browser`: Project-scoped UI for viewing, searching, and managing cached integration data
- `my-activity-dashboard`: Global dashboard showing attention-needed items across projects and integrations; sidebar badge with count; morning standup as primary use case
- `cache-management`: User-controlled cache clearing (immediate and scheduled cron) with Delete/Archive options; configurable retention period
- `background-jobs-visibility`: Unified view of active, recent, and scheduled daemon jobs; dual location (Integrations + Settings) with cross-linking

### Modified Capabilities
- `ai-chat`: Extended to include task metadata and integration data in context
- `integrations`: Add linking workflow selection (AI-suggested, manual, lazy) during setup
- `notifications`: Link from notification to full context in integration browser

## Technical Specifications

### AI Context Management
- **Token Budget**: Maximum 4000 tokens for integration context per conversation turn
- **Relevance Scoring**: Embedding-based retrieval to select most relevant integration items for current conversation topic
- **Truncation Strategy**: Priority order: directly mentioned items > recently updated > user-assigned > other; truncate oldest/lowest-relevance first
- **Source Attribution**: Each integration item in context tagged with source (GitHub/Jira/Slack) and timestamp to prevent AI confusion and enable freshness caveats

### Cache Management
- **Staleness Indicators**: Show "Updated 2 hours ago" on cached items; items older than sync interval shown with warning indicator
- **Manual Refresh**: Per-integration "Refresh now" button in browser; loading state during sync
- **Archive Retention**: Archived cache data retained for 90 days, then auto-purged
- **Database Indexes**: Composite index on `(project_id, source, updated_at)` for efficient filtering; partial index on `is_archived = false` for active queries

### My Activity Dashboard
- **Empty State**: When no integrations connected, show onboarding prompt with benefits and "Connect Integration" CTA
- **Attention Prioritization**: Items sorted by severity (critical > warning > info); "Top 5" shown by default with "See all (N)" expansion
- **Workflow Selection Default**: Default to "lazy" linking (ask when needed); show one-line explanation during setup with "Change anytime in settings"

## Impact

- **Database**: Extend `app_settings` with cache retention preferences; add composite indexes on integration_cache for efficient querying; add `archived_at` column for archive retention
- **Backend**: New AI context builder with token budgeting and relevance scoring; cache cleanup cron job with archive purge; background jobs status API
- **Frontend**: IntegrationBrowser component per project with staleness indicators; MyActivity dashboard in sidebar with priority filtering; BackgroundJobsPanel enhanced with active/recent/scheduled tabs; empty state onboarding
- **MCP**: Add `query_integrations`, `get_linked_items`, `get_my_activity` tools
- **Performance**: Integration context for AI chat uses embedding retrieval for relevance; pre-computed attention counts updated on sync
