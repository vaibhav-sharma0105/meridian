## Why

Meridian can now learn patterns, make suggestions, and run automated skills (Phases 1-4), but it operates in isolation from where work actually happens. Users still manually copy information between Meridian and their work tools (GitHub, Jira, Slack). Phase 5 connects Meridian to external systems, enabling bidirectional sync, AI-assisted communication, and true workflow automation across the tools knowledge workers use daily.

## What Changes

- **Integration Framework**: Unified infrastructure for OAuth flows, credential storage, sync scheduling, and per-integration autonomy settings
- **GitHub Integration**: OAuth App auth, read issues/PRs assigned to user, create issues, comment on PRs, bidirectional task linking
- **Jira Integration**: Atlassian OAuth or API token, read assigned issues and sprint context, create/update issues, bidirectional linking
- **Slack Integration**: Socket Mode (no public endpoint), bot mode + optional user token mode, channel-level autonomy, draft-before-send for high-risk channels
- **Desktop Notifications**: Wire up existing Tauri notification plugin with severity levels (info/warning/critical), sound for critical
- **MCP Server Write Operations**: Add create_task, update_task, create_meeting_note, run_skill commands with permission system
- **Webhook Receiver**: Local HTTP server with security tokens for real-time updates from external systems

## Capabilities

### New Capabilities

- `integration-framework`: Base infrastructure for all integrations — OAuth flows, encrypted credential storage, sync scheduling, autonomy settings, webhook receiver
- `integration-github`: GitHub OAuth App integration — issues, PRs, comments, bidirectional task linking with Meridian
- `integration-jira`: Atlassian integration — issues, sprints, JQL sync, bidirectional task linking
- `integration-slack`: Slack Socket Mode integration — channel monitoring, message sending, channel-level autonomy (draft/notify/auto-send)
- `desktop-notifications`: Native OS notifications via Tauri plugin — severity levels, sounds, user preferences

### Modified Capabilities

- `integration-mcp-server`: Add write operations (create_task, update_task, create_meeting_note, run_skill) with configurable permissions
- `notification-system`: Extend to support desktop notifications alongside in-app notifications

## Impact

- **Database**: New `integrations`, `integration_cache`, `integration_links` tables; extend `notifications` for desktop flag
- **Backend**: New `src-tauri/src/integrations/` module with per-integration connectors; webhook HTTP server
- **Frontend**: Integration Hub UI in settings, connection wizards, sync status indicators, linked item badges on tasks
- **Security**: OAuth tokens encrypted in DB, webhook security tokens, audit logging for all external actions
- **Daemon**: Integration sync jobs, webhook listener process
- **MCP**: Extended tool set with write operations, permission checks
