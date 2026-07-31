# Phase 8: Integration Visibility — Design Spec

**Date:** 2026-07-31  
**Status:** Draft  
**Depends on:** Phase 5 (External Integrations Framework), Phase 6-7 (Governance & Team)

## Overview

Phase 8 makes integration data (GitHub, Jira, Slack) accessible to both users and the AI assistant. The core problem: Meridian fetches external data but keeps it invisible — users can't browse it, AI chat can't reference it, and there's no unified view of "what needs my attention."

### Goals

1. **My Activity Dashboard** — Unified view of items needing attention across Meridian tasks, pending approvals, and integration data
2. **Integration Browser** — Project-scoped UI to browse and search cached GitHub/Jira/Slack items
3. **AI Chat Context** — Integration data automatically included in AI conversations when relevant
4. **Commit Feed + Skill Filtering** — Fetch recent commits, filter via LLM using user-defined skills
5. **Cache Management** — User control over retention, clearing, and archiving

## Data Model

### New Tables

#### `integration_project_mapping`

Maps external repos/projects to Meridian projects. Required because integrations are account-level (your GitHub account) but users need project-scoped views.

```sql
CREATE TABLE integration_project_mapping (
    id TEXT PRIMARY KEY,
    integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    external_key TEXT NOT NULL,           -- 'owner/repo' for GitHub, 'PROJECT' for Jira
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    UNIQUE(integration_id, external_key)
);
```

#### `attention_items`

Pre-computed cache of items needing attention. Stores references only — actual content fetched via JOIN at display time.

```sql
CREATE TABLE attention_items (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,            -- 'task', 'approval', 'integration_cache'
    source_id TEXT NOT NULL,              -- FK to source table
    severity TEXT NOT NULL DEFAULT 'info',-- 'critical', 'warning', 'info'
    category TEXT NOT NULL,               -- 'overdue', 'review_requested', 'commit_match', etc.
    reason_text TEXT,                     -- Human-readable reason for UI (e.g., "Overdue by 3 days")
    matched_skill_id TEXT,                -- Which filter skill matched (null for rule-based items)
    computed_at TEXT NOT NULL,
    dismissed_at TEXT,
    UNIQUE(source_type, source_id, category)
);

CREATE INDEX idx_attention_active ON attention_items(dismissed_at) WHERE dismissed_at IS NULL;
CREATE INDEX idx_attention_severity ON attention_items(severity, computed_at DESC);
```

### Schema Changes

#### `skills` table

Add filter configuration for `action: filter` skills:

```sql
ALTER TABLE skills ADD COLUMN filter_config JSON;
```

`filter_config` schema:
```json
{
  "integration_types": ["github"],
  "item_types": ["commit", "pr"],
  "prompt": "..."  // Optional override, otherwise uses system_prompt
}
```

#### `integration_cache` table

Add columns for filter results and lifecycle:

```sql
ALTER TABLE integration_cache ADD COLUMN attention_score REAL;
ALTER TABLE integration_cache ADD COLUMN attention_reason TEXT;
ALTER TABLE integration_cache ADD COLUMN evaluated_at TEXT;
ALTER TABLE integration_cache ADD COLUMN archived_at TEXT;
ALTER TABLE integration_cache ADD COLUMN expires_at TEXT;

CREATE INDEX idx_cache_attention ON integration_cache(attention_score DESC) WHERE archived_at IS NULL;
CREATE INDEX idx_cache_type_sync ON integration_cache(integration_id, external_type, synced_at DESC);
```

#### `app_settings` keys

| Key | Default | Description |
|-----|---------|-------------|
| `cache_retention_days` | 30 | Auto-delete cache items older than this |
| `attention_refresh_minutes` | 5 | How often daemon recomputes attention_items |
| `ai_integration_context_tokens` | 4000 | Token budget for integration data in AI chat |

## Component Design

### 1. My Activity Dashboard

#### Sidebar Entry

New nav item between "All Tasks" and "Skills":

```
📋 All Tasks
⚡ My Activity  [5]    ← Badge shows critical + warning count
⚡ Skills
🛡️ Governance
```

#### Layout

Three collapsible sections by severity:

- **Critical** — Overdue tasks >3 days, overdue Jira items
- **Needs Attention** — Pending approvals, PR reviews requested, overdue 1-3 days, stale tasks, Slack mentions
- **Info** — Commits matching filters, approaching deadlines

Each item shows:
- Icon by source type (task/GitHub/Jira/Slack/approval)
- Title (linked to source)
- One-line reason/context
- Timestamp
- Actions: [View/Open] [Dismiss]

#### Attention Item Sources

| Source | Category | Severity | Rule |
|--------|----------|----------|------|
| Task | overdue_critical | critical | Due date > 3 days ago |
| Task | overdue | warning | Due date 1-3 days ago |
| Task | stale | warning | Status = in_progress, no update 7+ days |
| Approval | pending | warning | Status = pending |
| GitHub | review_requested | warning | User is requested reviewer |
| GitHub | changes_requested | warning | User is PR author, changes requested |
| GitHub | commit_match | info | Matches a filter skill |
| Jira | assigned_due_soon | warning | Assigned to user, due within 2 days |
| Jira | assigned_overdue | critical | Assigned to user, past due |
| Slack | mention | info | User mentioned in last 24h |
| Slack | action_item | warning | Action item detected mentioning user |

#### Filters

Dropdown with quick filters:
- All (default)
- Tasks only
- Integrations only
- By project (picker)
- By integration (GitHub/Jira/Slack)

#### Empty State

When no integrations connected, show onboarding prompt with benefits and "Connect Integrations" CTA.

### 2. Integration Browser

#### Entry Point

New tab in project view: `[Tasks] [Meetings] [Documents] [Integrations]`

Also reachable from My Activity — clicking an integration item navigates to the browser with that item expanded.

#### Layout

- Integration type tabs: [GitHub] [Jira] [Slack]
- Search bar with full-text search
- List of items grouped by repo/project
- Expandable rows showing full detail

#### Expandable Content by Type

**GitHub PR/Issue:**
- Description (markdown rendered)
- Labels, milestone, CI status
- Comments (recent 5, "Load more" link)
- [Open on GitHub] button

**GitHub Commit:**
- Full commit message
- Files changed summary
- [View Diff] link

**Jira Issue:**
- Description, status, priority
- Assignee, due date
- Recent comments
- Linked issues

**Slack Thread:**
- Full thread (up to 20 messages)
- Detected action items highlighted

#### Staleness Indicators

- Fresh (< sync interval): No indicator
- Stale (> sync interval): Yellow dot + "Updated 2h ago"
- Error: Red dot + "Sync failed" + retry button

#### Cache Management Section

Per-integration footer:
```
Cache: 142 items · 2.3 MB · Oldest: 28 days ago
Retention: [30 days ▾]   [Clear Old] [Clear All]
☐ Archive instead of delete
```

### 3. AI Chat Integration Access

#### Context Injection

Integration data injected into `chat_with_project` system prompt after task/meeting context, before conversation history.

#### Relevance Scoring

For each cached item, compute:
```
score = (embedding_similarity * 0.5) + (recency_bonus * 0.3) + (assigned_bonus * 0.2)
```

Where:
- `embedding_similarity`: Cosine similarity between user message and item title+description
- `recency_bonus`: 1.0 if < 24h, 0.5 if < 7d, 0.0 otherwise
- `assigned_bonus`: 1.0 if assigned to user, 0.0 otherwise

Select top items until 4000 token budget exhausted.

#### Context Format

```markdown
## Linked Integration Data

### GitHub (acme/backend) — updated 5 min ago
- **PR #156** "Session handling fix" — draft, you're author, 3 comments
  Last comment: "@alice: LGTM after adding the test"
- **Issue #89** "Auth token refresh fails" — open, assigned to you

### Jira (ACME) — updated 15 min ago  
- **ACME-142** "Implement SSO" — In Progress, due Aug 10

### Recent Commits (matched by "auth changes" filter)
- abc123 (2h ago): "Refactor auth middleware" — +50 -20
```

#### Fallback Behavior

- No integrations connected: Skip section
- Qdrant unavailable: Fall back to keyword matching
- All items filtered by relevance: Skip section

### 4. Commit Sync + Filter Skills

#### Filter Skill Definition

```yaml
---
name: Auth Changes Monitor
description: Surface commits touching authentication code
trigger:
  type: event
  events: [integration_sync]
action:
  type: filter
settings:
  approval_mode: auto
filter_config:
  integration_types: [github]
  item_types: [commit]
---

# Filter Criteria

Surface commits that:
- Touch files in `src/auth/` or `**/authentication/**`
- Modify files containing "token", "session", or "JWT"
- Are from external contributors

Ignore:
- Documentation-only changes
- Dependency updates alone
```

#### Sync + Filter Flow

1. **GitHub sync daemon** fetches commits from last 48h across all branches
2. **Store in cache** with `external_type: 'commit'`
3. **Queue filter evaluation job**
4. **Filter job** iterates unevaluated commits:
   - Build prompt: skill criteria + commit data
   - LLM returns: `{match: bool, confidence: float, reason: string}`
   - Update cache: `attention_score`, `attention_reason`, `evaluated_at`
   - If match: create `attention_item`

#### LLM Evaluation Prompt

```
You are evaluating whether a commit matches filter criteria.

## Filter Criteria
{skill.system_prompt}

## Commit
SHA: {sha}
Author: {author}
Message: {message}
Files: {files}
Stats: +{additions} -{deletions}

## Response
Return JSON only: {"match": true/false, "confidence": 0.0-1.0, "reason": "one line"}
```

#### Re-evaluation Triggers

- New items synced → evaluate against all active filter skills
- Filter skill created/updated → batch re-evaluate matching cached items
- Filter skill deleted → clear `matched_skill_id` references

#### Cost Control

- Batch: 50 items per job run
- Skip unchanged: Track `evaluated_at`, skip if skill unchanged
- Cache results: Don't re-call LLM for same item+skill pair

#### Commit Data Stored

```json
{
  "sha": "abc123",
  "message": "Refactor auth middleware",
  "author": "alice",
  "author_email": "alice@example.com", 
  "timestamp": "2026-07-31T10:00:00Z",
  "branch": "main",
  "files": ["src/auth/middleware.rs"],
  "additions": 50,
  "deletions": 20
}
```

### 5. Background Jobs Visibility

#### Enhanced BackgroundJobsPanel

New "Scheduled" tab showing upcoming jobs:
- Next sync per integration
- Next attention refresh
- Scheduled skill runs

Job row expansion shows:
- Full payload
- Duration (completed)
- Error + Retry button (failed)

#### Cross-linking

- IntegrationsPage: "View sync jobs" → filtered BackgroundJobsPanel
- Settings > Advanced: Full BackgroundJobsPanel

### 6. Cache Management

#### Auto-cleanup Job

New daemon job `cleanup_integration_cache`:
- Runs daily at 3 AM
- Deletes: `synced_at < (now - retention_days)` AND `archived_at IS NULL`
- Deletes archived: `archived_at < (now - 90 days)`

#### Manual Actions

- **Clear Old**: Delete items older than retention
- **Clear All**: Delete all for integration (with confirmation)
- **Archive mode**: Set `archived_at` instead of delete

### 7. MCP Tools

New tools for `meridian-mcp`:

#### `query_integrations`

Search cached integration data.

```json
{
  "integration_type": "github",
  "item_type": "pr",
  "project_id": "...",
  "text_search": "auth",
  "limit": 20
}
```

#### `get_my_activity`

Get attention items.

```json
{
  "severity": "warning",
  "source_type": "github",
  "limit": 20
}
```

#### `get_linked_items`

Get external items linked to a task.

```json
{
  "task_id": "..."
}
```

## File Structure

### Backend (src-tauri/src/)

```
commands/
  attention.rs          # get_attention_items, dismiss_attention_item, refresh_attention
  
integrations/
  repository.rs         # Add: get_cached_items_for_project, search_cache, archive_cache
  context.rs            # NEW: build_integration_context for AI chat
  
ai/
  context.rs            # NEW: build_integration_context, relevance_scoring
  filter.rs             # NEW: evaluate_filter_skill, batch_evaluate
  
daemon/
  jobs.rs               # Add: compute_attention_items, cleanup_integration_cache, evaluate_filters
  
db/migrations/
  v018_integration_visibility.rs   # Schema changes
```

### Frontend (src/)

```
components/
  activity/
    MyActivityDashboard.tsx    # Main dashboard view
    AttentionItem.tsx          # Single attention item row
    AttentionFilters.tsx       # Filter dropdown
    
  integrations/
    IntegrationBrowser.tsx     # Project-scoped browser
    IntegrationItemRow.tsx     # Expandable row
    IntegrationItemDetail.tsx  # Expanded content by type
    CacheManagement.tsx        # Retention settings, clear buttons
    BackgroundJobsPanel.tsx    # Enhanced with Scheduled tab
    
hooks/
  useAttention.ts              # Query attention_items
  useIntegrationBrowser.ts     # Query/search integration_cache
  
stores/
  uiStore.ts                   # Add: activeView = 'activity'
```

### MCP (meridian-mcp/src/)

```
handlers.rs                    # Add: query_integrations, get_my_activity, get_linked_items
```

## Migration

### v018_integration_visibility.rs

```sql
-- New tables
CREATE TABLE integration_project_mapping (...);
CREATE TABLE attention_items (...);

-- Schema changes
ALTER TABLE skills ADD COLUMN filter_config JSON;
ALTER TABLE integration_cache ADD COLUMN attention_score REAL;
ALTER TABLE integration_cache ADD COLUMN attention_reason TEXT;
ALTER TABLE integration_cache ADD COLUMN evaluated_at TEXT;
ALTER TABLE integration_cache ADD COLUMN archived_at TEXT;
ALTER TABLE integration_cache ADD COLUMN expires_at TEXT;

-- Indexes
CREATE INDEX idx_attention_active ON attention_items(dismissed_at) WHERE dismissed_at IS NULL;
CREATE INDEX idx_attention_severity ON attention_items(severity, computed_at DESC);
CREATE INDEX idx_cache_attention ON integration_cache(attention_score DESC) WHERE archived_at IS NULL;
CREATE INDEX idx_cache_type_sync ON integration_cache(integration_id, external_type, synced_at DESC);

-- Default settings
INSERT INTO app_settings (key, value) VALUES ('cache_retention_days', '30');
INSERT INTO app_settings (key, value) VALUES ('attention_refresh_minutes', '5');
INSERT INTO app_settings (key, value) VALUES ('ai_integration_context_tokens', '4000');
```

## Testing

### E2E Tests (Playwright)

- My Activity renders with mocked attention items
- Dismiss attention item removes from list
- Integration Browser shows cached items
- Expandable rows show detail
- Filter dropdown works
- Cache management actions work

### Unit Tests (Rust)

- `compute_attention_items` generates correct items from sources
- Relevance scoring math is correct
- Filter skill evaluation parses LLM response
- Cache cleanup respects retention settings
- MCP tools return expected data

## Open Questions

None — all clarified during design discussion.

## Dependencies

- Phase 5 integrations must be connected for data to appear
- Qdrant recommended for embedding-based relevance scoring (falls back to keyword matching)
- LLM provider configured for filter skill evaluation
