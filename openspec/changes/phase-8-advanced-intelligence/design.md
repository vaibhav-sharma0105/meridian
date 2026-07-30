## Context

Meridian has completed Phases 0-7, establishing:
- Pattern learning engine (workflow sequences, smart defaults, communication style)
- Proactive suggestion system with draft communications
- Skills automation with scheduled/event/manual triggers
- External integrations (GitHub, Jira, Slack) with governance
- Autonomy controller with risk classification and approval flow
- Team roster with assignee intelligence (Phase 7)

Phase 8 elevates Meridian from a project-scoped assistant to a cross-project intelligence layer that understands relationships between work items, predicts needs, learns estimation patterns, and provides visibility into how the system is being used.

**Current limitations:**
- Suggestions are project-scoped; no awareness of cross-project dependencies
- Document pre-fetch is reactive (user searches) not predictive
- No learning from task estimation accuracy
- Onboarding covers basics but not agentic features
- No visibility into AI token usage, storage, or productivity trends

## Goals / Non-Goals

**Goals:**
- Detect and surface cross-project blockers and dependencies
- Predict user needs (documents, agendas, blockers) before they ask
- Learn from estimation patterns to provide accurate time predictions
- Provide comprehensive usage analytics for cost management and productivity insights
- Guide users through agentic features with interactive onboarding

**Non-Goals:**
- Multi-user collaboration (remains local-first single-user)
- Cloud sync of analytics data
- Real-time cross-project notifications (batch analysis is sufficient)
- Automated calendar management (draft/suggest only)
- Full project management features (Gantt charts, critical path)

## Decisions

### 1. Cross-Project Link Storage

**Decision:** Create `cross_project_links` table with typed relationships rather than embedding in existing tables.

**Alternatives considered:**
- Add `blocks_project_id` to tasks table → Too limited, only handles one relationship type
- JSON field on tasks → Query performance issues, hard to index
- Graph database → Overkill for local-first app, adds dependency

**Rationale:** Separate link table allows multiple relationship types (blocks, related_to, duplicates), efficient querying with indexes, and clean separation of concerns.

```sql
CREATE TABLE cross_project_links (
  id TEXT PRIMARY KEY,
  link_type TEXT NOT NULL,  -- 'blocks', 'related_to', 'duplicate_meeting'
  source_type TEXT NOT NULL,  -- 'task', 'meeting'
  source_id TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id TEXT NOT NULL,
  confidence REAL,
  detected_by TEXT,  -- 'ai', 'user', 'pattern'
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### 2. Predictive Pre-fetch Strategy

**Decision:** Background job runs 30 minutes before scheduled meetings to pre-fetch relevant documents into a warm cache.

**Alternatives considered:**
- Real-time fetch on meeting open → Too slow, documents may be large
- Continuous background embedding → Wastes resources when user is idle
- User-triggered pre-fetch → Defeats the "predictive" goal

**Rationale:** 30-minute window balances freshness with preparation time. Job checks:
1. Meetings in next 30 min with attendees
2. Open tasks linked to meeting or assigned to attendees
3. Documents matching task keywords or meeting title
4. Pre-loads document chunks into memory cache (not new embeddings)

### 3. Estimation Learning Approach

**Decision:** Track task lifecycle events and compute estimation accuracy as a derived metric, not a separate model.

**Alternatives considered:**
- Dedicated ML model for estimation → Too complex for local-first, needs training data
- Simple average of past durations → Ignores task complexity factors
- User-provided estimates only → Users often skip estimation

**Rationale:** Store lifecycle events in `task_estimation_log`:
```sql
CREATE TABLE task_estimation_log (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  event_type TEXT NOT NULL,  -- 'created', 'started', 'completed', 'estimate_changed'
  event_value TEXT,  -- estimate value or status
  event_at TEXT NOT NULL
);
```

Compute estimation accuracy on-demand by:
1. Grouping tasks by keywords/assignee/priority
2. Comparing estimated vs actual duration
3. Applying recency weighting (recent tasks matter more)
4. Suggesting estimate based on similar completed tasks

### 4. Usage Analytics Architecture

**Decision:** Daily aggregation job writes to `usage_metrics` table; dashboard reads aggregates, not raw data.

**Alternatives considered:**
- Real-time counters → Performance overhead on every operation
- Log file analysis → Complex parsing, not queryable
- External analytics service → Violates local-first principle

**Rationale:** Daily aggregation balances granularity with performance:
```sql
CREATE TABLE usage_metrics (
  date TEXT NOT NULL,
  metric_type TEXT NOT NULL,  -- 'tasks_created', 'ai_tokens', 'storage_bytes', etc.
  metric_key TEXT,  -- optional breakdown key (project_id, provider, etc.)
  value INTEGER NOT NULL,
  PRIMARY KEY (date, metric_type, metric_key)
);
```

Tracked metrics:
- Task CRUD counts (by project)
- Meeting imports (by source)
- AI tokens consumed (by provider)
- Embedding operations (count, tokens)
- Storage size (DB, documents, vectors)
- Skill executions (by skill)
- Suggestion accept/dismiss rates

### 5. Onboarding Expansion Strategy

**Decision:** Add optional "Agentic Tour" after basic onboarding, with demo mode using synthetic data.

**Alternatives considered:**
- Force all users through extended onboarding → Frustrating for power users
- Tooltips only → Easy to dismiss, not comprehensive
- Video tutorials → Requires external hosting, not local-first

**Rationale:** 
- Basic onboarding remains unchanged (AI provider, first project)
- New "Take the Agentic Tour" button after completion
- Tour walks through: autonomy settings, skills, suggestions, governance
- Demo mode creates temporary synthetic project with pre-populated data
- Tooltips appear on first encounter of advanced features (dismissible)

### 6. Cross-Project Analysis Frequency

**Decision:** Run cross-project analysis every 6 hours as daemon job, not on-demand.

**Alternatives considered:**
- Real-time on every task change → Performance impact, unnecessary frequency
- Daily only → Too slow to catch urgent blockers
- User-triggered → Relies on user remembering to run it

**Rationale:** 6-hour interval catches blockers within reasonable time while keeping resource usage low. Job:
1. Scans all projects for open tasks
2. Extracts keywords and relationships from task titles/descriptions
3. Uses embedding similarity to find related tasks across projects
4. Checks for explicit references (task IDs, project names mentioned)
5. Creates/updates `cross_project_links` entries
6. Generates suggestions for confirmed blockers

## Risks / Trade-offs

### Performance at Scale
**Risk:** Cross-project analysis with many projects/tasks could be slow.
**Mitigation:** 
- Index on task keywords and project_id
- Limit similarity search to recent/open tasks (not archived)
- Use Qdrant's efficient vector search with filters
- Cap at 10 projects per analysis run, round-robin

### AI Token Costs for Predictions
**Risk:** Predictive features increase AI calls, raising costs.
**Mitigation:**
- Pre-fetch uses local embedding similarity, not AI
- Agenda drafting reuses existing meeting summarization prompt
- Blocker detection is primarily pattern-based, AI confirms
- Usage dashboard shows token consumption for awareness

### Estimation Accuracy Cold Start
**Risk:** New users have no history for estimation learning.
**Mitigation:**
- Show "Not enough data" until 10+ completed tasks
- Optionally use broad category defaults (bug: 2h, feature: 8h)
- Learn quickly from first completions with high recency weight

### Onboarding Fatigue
**Risk:** Extended onboarding might overwhelm users.
**Mitigation:**
- Agentic tour is explicitly optional
- Can be accessed later from Settings
- Demo mode is clearly marked, easy to exit
- Tour progress is saved, can resume later

### Storage Growth from Metrics
**Risk:** Usage metrics table could grow large over time.
**Mitigation:**
- Daily aggregation keeps one row per metric per day
- Auto-prune metrics older than 2 years (matches audit log retention)
- Export before prune for long-term analysis

## Migration Plan

1. **Database migration v016**: Create new tables (cross_project_links, task_estimation_log, usage_metrics, onboarding_progress)
2. **Backfill estimation logs**: Generate 'created' and 'completed' events from existing task timestamps
3. **Initial metrics aggregation**: Run one-time job to compute metrics from existing audit_log
4. **Daemon jobs**: Register new jobs (cross_project_analysis, predictive_prefetch, usage_aggregation)
5. **Frontend deployment**: Analytics view, onboarding tour, prediction indicators ship together
6. **Rollback**: Migration is additive (new tables), no breaking changes to existing features

## Open Questions

1. **Cross-project link visibility**: Should users see AI-detected links before confirming them, or only after confidence threshold?
2. **Estimation display**: Show estimate range (2-4 hours) or single value (3 hours)?
3. **Demo mode data**: Generate synthetic data on-the-fly or ship pre-built dataset?
4. **Metrics export format**: CSV only, or also JSON/Parquet for data analysis tools?
