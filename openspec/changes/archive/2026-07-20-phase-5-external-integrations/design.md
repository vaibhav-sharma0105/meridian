## Context

Meridian is a local-first desktop app with existing integrations (Zoom, Sheets Relay) that follow a polling model. Phase 5 adds more sophisticated integrations (GitHub, Jira, Slack) requiring OAuth flows, bidirectional sync, real-time events (Slack), and write operations.

**Current state:**
- Zoom integration: OAuth + PKCE, polls for recordings/transcripts
- Sheets Relay: Polls Google Sheets for Gmail automation output
- MCP Server: Read-only access to Meridian data
- Daemon: Background job queue for embeddings, suggestions, skills
- Notifications: In-app only, desktop plugin loaded but not wired

**Constraints:**
- Local-first: No cloud backend for routing webhooks
- Security: OAuth tokens must be encrypted, audit logging required
- Slack Socket Mode requires persistent connection (not polling)
- Must work offline (cached data, graceful degradation)

## Goals / Non-Goals

**Goals:**
- Unified integration framework that works for OAuth and API key auth
- GitHub and Jira with bidirectional task linking and incremental sync
- Slack with channel-level autonomy and draft-before-send safety
- Desktop notifications with severity levels and sound
- MCP write operations with permission system

**Non-Goals:**
- Multiple Slack workspaces (single workspace per install)
- GitHub/Jira webhooks for real-time updates (polling-only for MVP)
- Full message history sync for Slack (privacy concern)
- Email sending (draft-only, copy to clipboard)

## Decisions

### Decision 1: Integration Framework Architecture

**Choice:** Trait-based integration abstraction with per-integration modules.

**Rationale:**
- Each integration has different auth, sync, and action patterns
- Common framework handles: credential storage, sync scheduling, audit logging
- Per-integration module implements: auth flow, sync logic, actions

**Structure:**
```
src-tauri/src/integrations/
├── mod.rs           # Integration trait, registry, common types
├── framework.rs     # OAuth helper, sync scheduler, cache manager
├── github.rs        # GitHub-specific implementation
├── jira.rs          # Jira-specific implementation
├── slack.rs         # Slack-specific implementation
└── webhook.rs       # Local webhook HTTP server
```

**Alternative considered:** Single generic integration handler — rejected because auth flows and APIs differ too much.

### Decision 2: OAuth Token Storage

**Choice:** Store encrypted OAuth tokens in `integrations.config` JSON column (SQLCipher encrypted).

**Rationale:**
- Already using SQLCipher for database encryption
- Consistent with existing credential storage (AI API keys)
- No additional encryption layer needed

**Schema:**
```sql
CREATE TABLE integrations (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,  -- 'github', 'jira', 'slack'
  name TEXT NOT NULL,
  config TEXT NOT NULL,  -- JSON: { access_token, refresh_token, expires_at, ... }
  permissions TEXT,      -- JSON: { read: true, write: false, ... }
  autonomy_mode TEXT DEFAULT 'manual',
  status TEXT DEFAULT 'disconnected',
  last_sync TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

**Alternative considered:** OS keychain via keyring crate — rejected because it causes permission prompts on unsigned macOS builds.

### Decision 3: Slack Socket Mode vs Webhooks

**Choice:** Socket Mode (WebSocket) for Slack events.

**Rationale:**
- No public endpoint required (local-first friendly)
- Real-time event delivery without polling
- Works behind firewalls/NAT

**Trade-off:** Requires persistent connection; if daemon stops, events are missed until reconnect.

**Alternative considered:** Slack webhooks — rejected because requires public URL, complicates local-first architecture.

### Decision 4: Channel-Level Autonomy for Slack

**Choice:** Store per-channel autonomy settings in integration config.

**Rationale:**
- Different channels have different risk profiles (#general vs #bot-updates)
- User needs fine-grained control over where bot can auto-send
- Default to "draft only" for safety

**Autonomy levels:**
1. `draft` — Create draft, never auto-send
2. `notify` — Create draft, notify user to send
3. `delayed` — Queue with 10-min delay, user can cancel
4. `auto` — Send immediately (low-risk channels only)

### Decision 5: Bidirectional Link Storage

**Choice:** Separate `integration_links` table for task ↔ external item links.

**Rationale:**
- Clean separation from task/meeting tables
- Supports multiple links per task (task linked to both GitHub issue and Jira)
- Easy to query links by integration

**Schema:**
```sql
CREATE TABLE integration_links (
  id TEXT PRIMARY KEY,
  integration_id TEXT NOT NULL REFERENCES integrations(id),
  local_type TEXT NOT NULL,    -- 'task', 'meeting'
  local_id TEXT NOT NULL,
  external_type TEXT NOT NULL, -- 'issue', 'pr', 'jira_issue'
  external_id TEXT NOT NULL,
  external_url TEXT,
  sync_enabled BOOLEAN DEFAULT TRUE,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(integration_id, local_id, external_id)
);
```

### Decision 6: Desktop Notifications via Tauri Plugin

**Choice:** Use existing `tauri-plugin-notification` with severity-based presentation.

**Rationale:**
- Plugin already loaded in Cargo.toml
- Cross-platform (macOS, Windows, Linux)
- Supports sounds, actions, grouping

**Implementation:**
- Add `severity` and `desktop` columns to notifications table
- On notification insert with `desktop: true`, call Tauri notification API
- Respect user preferences (can disable desktop, disable sound)

### Decision 7: MCP Write Permissions

**Choice:** User-configurable permission flags per write operation.

**Rationale:**
- Users may want read-only MCP access for safety
- Different operations have different risk levels
- Explicit opt-in for write capabilities

**Permissions stored in app_settings:**
```json
{
  "mcp_permissions": {
    "create_task": true,
    "update_task": true,
    "create_meeting_note": false,
    "run_skill": false
  }
}
```

### Decision 8: Webhook Security

**Choice:** Random security token per integration, validated on every request.

**Rationale:**
- Simple but effective security model
- Token stored with integration config
- Webhook URL includes token: `http://localhost:PORT/webhook/{token}`

**Alternative considered:** HMAC signature validation — overkill for local-only server.

## Risks / Trade-offs

**[Risk] Slack connection drops**
- Socket Mode WebSocket can disconnect
- **Mitigation:** Auto-reconnect with exponential backoff; queue events during disconnect

**[Risk] OAuth token expiry during offline**
- Refresh tokens may expire if user is offline for extended period
- **Mitigation:** Prompt re-authentication on 401 errors; clear status to "disconnected"

**[Risk] Rate limiting by external APIs**
- GitHub/Jira have rate limits that could be hit with large syncs
- **Mitigation:** Respect rate limit headers; implement backoff; cache aggressively

**[Risk] Stale bidirectional links**
- External item may be deleted, link becomes stale
- **Mitigation:** On sync, validate links exist; mark broken links for user review

**[Trade-off] No real-time GitHub/Jira updates**
- Polling means updates lag by sync interval
- **Reasoning:** Webhooks require public endpoint; MVP uses polling, can add webhooks later if users run a tunnel

**[Trade-off] Single Slack workspace**
- Cannot connect multiple workspaces
- **Reasoning:** Simplifies MVP; multi-workspace adds complexity with channel selection

## Migration Plan

1. **Database migration (v014):** Add `integrations`, `integration_cache`, `integration_links` tables; add `severity`, `desktop`, `integration_id` to notifications
2. **Backend modules:** Create `src-tauri/src/integrations/` with framework and per-integration modules
3. **Commands:** Add integration CRUD, sync triggers, link management
4. **Daemon jobs:** Add `sync_integration` job type, Slack Socket Mode listener
5. **Desktop notifications:** Wire up Tauri plugin to notification creation
6. **MCP enhancement:** Add write tools with permission checks
7. **Frontend:** Integration Hub UI, connection wizards, link badges on tasks
8. **Settings:** Per-integration autonomy, MCP permissions, notification preferences

**Rollback:** Migrations are additive (new tables/columns). Disable integrations via feature flag if critical bugs.

## Open Questions

1. **Webhook port:** Use fixed port or dynamic? Need to avoid conflicts with other local services.
2. **Slack channel discovery:** Fetch all channels on connect, or let user paste channel IDs?
3. **GitHub App vs OAuth App:** OAuth App is simpler, but GitHub App allows finer permissions. Start with OAuth App?
4. **Jira Server support:** Focus on Jira Cloud only, or support Server/Data Center?
