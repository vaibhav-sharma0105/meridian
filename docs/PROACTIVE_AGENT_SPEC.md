# Proactive Agent — Specification

## Overview

A background reasoning agent that observes Meridian's data (tasks, meetings, patterns) and surfaces actionable insights without being prompted. It runs on a configurable schedule and writes to a dedicated `agent_suggestions` table; suggestions appear in the notification panel and optionally as inline cards in the task/meeting views.

---

## Goals

- Reduce cognitive load: the user shouldn't need to search for at-risk tasks or stale meetings.
- Be specific, not verbose: each suggestion must name an exact task, meeting, or person.
- Respect local-first: all inference runs on the local machine using the user's already-configured AI provider (OpenAI/Anthropic/Ollama/LiteLLM). No new network dependency.
- Stay out of the way: suggestions are dismissible and never block primary workflows.

---

## Trigger Conditions

The agent runs in three modes:

| Mode | Trigger | Typical cadence |
|---|---|---|
| **On sync** | After `sync_connections` completes and new meetings are imported | ~per sync cycle |
| **On open** | When the app launches and the DB was last read > 8 hours ago | Once per work session |
| **Scheduled** | A background timer if the app stays open | Every 4 hours |

---

## Suggestion Types

### 1. Stale Task Warning
**Trigger:** Open task with no `updated_at` in > 7 days and priority ≥ medium.  
**Output:** "Task _X_ hasn't moved in 10 days. Still relevant?"  
**Action options:** Snooze 7 days · Mark done · Archive

### 2. Unassigned High-Priority Task
**Trigger:** Task with `priority = critical` or `high` and `assignee IS NULL`, created > 24 hours ago.  
**Output:** "Critical task _X_ has no owner."  
**Action options:** Assign now (opens inline editor) · Dismiss

### 3. Meeting Without Tasks
**Trigger:** Meeting ingested > 1 hour ago with no linked tasks extracted (`task_count = 0`).  
**Output:** "Meeting _Y_ was ingested but no tasks were extracted. Run extraction?"  
**Action options:** Extract tasks now · Dismiss

### 4. Duplicate or Near-Duplicate Tasks (AI-powered)
**Trigger:** Two open tasks in the same project with cosine similarity > 0.92 (via embedding vectors already stored in `task_embeddings`).  
**Output:** "Tasks _A_ and _B_ look very similar. Merge?"  
**Action options:** Merge (keeps higher-priority, appends description) · Keep both · Dismiss

### 5. Deadline Proximity
**Trigger:** Task with `due_date` within 2 calendar days and status not `done` or `cancelled`.  
**Output:** "Task _X_ is due in 1 day."  
**Action options:** Open task · Snooze until tomorrow · Mark done

### 6. Workload Imbalance
**Trigger:** One assignee has ≥ 3× the open task count of the next-busiest assignee, within the same project.  
**Output:** "Alice has 14 open tasks; Bob has 3. Consider rebalancing."  
**Action options:** View tasks by assignee (opens kanban filtered) · Dismiss

---

## Data Model

```sql
CREATE TABLE IF NOT EXISTS agent_suggestions (
    id          TEXT PRIMARY KEY,
    type        TEXT NOT NULL,           -- stale_task | unassigned | no_tasks_meeting | ...
    project_id  TEXT,
    entity_id   TEXT,                    -- task.id or meeting.id the suggestion is about
    entity_type TEXT,                    -- "task" | "meeting"
    message     TEXT NOT NULL,
    actions     TEXT,                    -- JSON array of {label, action_key}
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    dismissed_at TEXT,
    snoozed_until TEXT
);
```

Migration: `v007_agent_suggestions.rs`

---

## Rust Implementation Sketch

```
src-tauri/src/agent/
    mod.rs             -- AgentRunner: orchestrates trigger → evaluate → persist
    rules.rs           -- Rule functions (pure SQL): stale_tasks(), unassigned_critical(), etc.
    similarity.rs      -- Cosine similarity scan over task_embeddings
    scheduler.rs       -- Background thread with tokio::time::interval; fires AgentRunner
```

New Tauri commands:
- `get_agent_suggestions(project_id)` — returns undismissed suggestions
- `dismiss_suggestion(id)`
- `snooze_suggestion(id, until_date)`
- `run_agent_now()` — manual trigger from UI

---

## Frontend Integration

- **NotificationCenter**: A new section "Suggestions" above pending imports. Each suggestion renders as a compact card with action buttons.
- **Task/Meeting cards**: An optional `SuggestionBadge` overlay (⚠ dot) when the card's entity has an active suggestion. Clicking opens the suggestion inline.
- **Settings panel**: Toggle "Proactive suggestions on/off" + per-type toggles + snooze duration config. Persisted to `app_settings`.

---

## Privacy and Scope

- All inference is local. The only outbound call is to the user's configured AI provider (for duplicate detection via embeddings, which already runs for search). No telemetry.
- The agent has read-only access to tasks, meetings, and embeddings. It never modifies data — only writes to `agent_suggestions`.
- Suggestions are project-scoped and only visible to the local user.

---

## Out of Scope (v1)

- Predictive deadline estimation (needs historical velocity data — feasible in v2 after analytics are richer).
- Cross-project suggestions (too noisy without user context).
- Natural-language commands triggered by the agent ("Schedule a review meeting for…") — belongs in the AI chat panel, not the background agent.
- Push notifications / OS-level alerts — Meridian is local-first; showing in-app is sufficient.
