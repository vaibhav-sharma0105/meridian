# Meridian — Architecture Reference

> **Maintenance mandate:** Update this document whenever the data flow, schema, component structure, or any architectural decision changes. This is the authoritative reference for how the system works and *why*.

---

## System Overview

Meridian is a three-layer desktop application:

```
┌─────────────────────────────────────────────────────────┐
│  React + TypeScript (Vite)          localhost:1420       │
│  Zustand stores · React Query · Tailwind CSS            │
└───────────────────┬─────────────────────────────────────┘
                    │  Tauri IPC (invoke / events)
┌───────────────────▼─────────────────────────────────────┐
│  Rust (Tokio async)                                      │
│  Command handlers · Repository pattern · HTTP clients   │
└───────────────────┬─────────────────────────────────────┘
                    │  rusqlite
┌───────────────────▼─────────────────────────────────────┐
│  SQLite (~/.meridian/meridian.db)                        │
│  FTS5 full-text search · Schema v005                     │
└─────────────────────────────────────────────────────────┘
                    │  HTTP (outbound only)
           AI Provider (OpenAI / Anthropic / LiteLLM / Ollama)
```

All data is local. The only outbound calls are to the user-configured AI endpoint.

---

## Frontend Architecture

### Entry Point & Routing

```
main.tsx
  └── App.tsx
        ├── getAppSettings() ──→ if onboarding_complete=true → AppShell
        └──────────────────────→ else → OnboardingWizard
```

`App.tsx` is the only routing decision point. There is no React Router page tree — the entire app is a single-page shell where view switching happens via Zustand store state (`uiStore.activeView`).

### Layout — Three Columns

```
AppShell
  ├── Sidebar (w-56, left)          — brand, nav, project list, utility strip
  ├── MainCanvas (flex-1, center)   — tab bar + task/meeting/document/chat views
  └── ContextPanel (340px, right)   — task detail editor + AI chat (50/50 split)
```

The right panel splits height exactly 50/50 when a task is selected:
```tsx
// ContextPanel.tsx
<div className="h-1/2 flex-shrink-0 overflow-y-auto">  {/* task editor */}
<div className="h-1/2 flex-shrink-0 overflow-hidden">  {/* AI chat    */}
```
**Never use `style={{ height: "50%" }}`** inside a flex container — use Tailwind's `h-1/2` with `flex-shrink-0`. The percentage style is unreliable in flex context.

### State Management

Two layers of state:

| Layer | Tool | What it stores |
|---|---|---|
| Server state | React Query | Tasks, meetings, projects, notifications |
| UI state | Zustand | Active view, selected task, filters, theme, sidebar open |

**React Query cache keys:**
```
["tasks", projectId, effectiveFilters]
["meetings", projectId]
["projects"]
["notifications"]
["pending-imports"]
["pending-imports-count"]
```

**Zustand stores:**
- `uiStore` — activeView, viewMode, selectedTaskId, sidebarOpen, rightPanelOpen, theme, settings modals
- `taskStore` — filters (TaskFilters), selectedTaskIds (bulk select), tasksByProject cache
- `projectStore` — projects list, activeProjectId
- `notificationStore` — notifications list, unreadCount
- `meetingStore` — local meeting state (supplementary to React Query)

### The API Contract (`src/lib/tauri.ts`)

`tauri.ts` is the **only place** `invoke()` is called from the frontend. Every Tauri IPC command has a typed TypeScript wrapper here. This file serves as:
1. The living contract between frontend and backend
2. The source of truth for all TypeScript models (interfaces for Task, Meeting, Project, etc.)
3. The documentation layer for what each command does

**Adding a command = adding an entry here.** Components and hooks always import from `@/lib/tauri`, never call `invoke` directly.

### Filter Architecture

Task filters flow through this pipeline:

```
uiStore / TaskFilters.tsx
  → taskStore.setFilters({ ... })
  → useTasks(projectId) reads storeFilters
  → strips client-only fields (meeting_ids, project_id)
  → invoke("get_tasks_for_project", { projectId, filters: backendFilters })
  → Rust SQL applies: assignee, status, priority, date_from/date_to (created_at), search_query
  → client-side post-filter: meeting_ids, project_id (All Tasks view)
  → React Query cache updated
```

**Client-only filter fields** (must be stripped before backend call):
- `project_id` — All Tasks view only; backend already scopes to a single project
- `meeting_ids` — multi-select; Rust SQL doesn't support array IN clauses via serde

**Date filter semantics**: `date_from` and `date_to` filter by `created_at` (task creation date), NOT `due_date`. This matches industry-standard "Created date" filter behavior.

---

## Backend Architecture

### Command Layer (`commands/`)

Command files are thin wrappers that:
1. Lock the DB mutex (`state.db.lock()`)
2. Call into a repository function
3. Return `Result<T, String>`

**Rules:**
- No business logic in commands — only argument extraction and DB access
- All SQL lives in `db/repositories/`
- Return human-readable error strings (they surface directly to the user)

### Repository Layer (`db/repositories/`)

One file per domain. All SQL is here. Pattern:

```rust
pub fn get_tasks_for_project(conn: &Connection, project_id: &str, filters: &TaskFilters) -> Result<Vec<Task>, String> {
    let mut conditions = vec!["project_id = ?1".to_string()];
    let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = vec![...];
    // Build dynamic WHERE clause
    // ...
    let sql = format!("SELECT {} FROM tasks WHERE {} ORDER BY ...", TASK_COLUMNS, where_clause);
    // ...
}
```

Dynamic SQL is built by appending to `conditions` and `bind_values`. This avoids SQL injection while allowing flexible filtering.

### Database Schema

Current version: **v005**. Migrations run automatically on startup.

**Core tables:**

```sql
projects          — id, name, color, archived_at
meetings          — id, project_id, title, platform, raw_transcript, ai_summary,
                    decisions, health_score, health_breakdown, attendees,
                    duration_minutes, meeting_at, ingested_at, updated_at
tasks             — id, project_id, meeting_id, title, description, assignee,
                    assignee_confidence, due_date, due_confidence, status, priority,
                    confidence_score, tags (JSON), kanban_column, kanban_order,
                    notes, is_duplicate, duplicate_of_id, created_at, updated_at
documents         — id, project_id, title, file_type, content, embedding (JSON)
connections       — id, provider, account_email, scopes, token_expires_at, last_sync_at
pending_imports   — id, provider, title, summary_full, source_email_id, status, ...
app_settings      — key, value (key-value store for app config)
notifications     — id, type, title, body, read_at, created_at
```

**Unique indexes for deduplication:**
```sql
CREATE UNIQUE INDEX idx_pending_ext_meeting ON pending_imports(external_meeting_id) WHERE external_meeting_id IS NOT NULL;
CREATE UNIQUE INDEX idx_pending_email ON pending_imports(source_email_id) WHERE source_email_id IS NOT NULL;
```

**Adding a migration:**
1. Create `src-tauri/src/db/migrations/v00N_description.rs`
2. Add the SQL constant and export
3. Add to the migration runner in `db/migrations/mod.rs`
4. Never modify existing migration files

### AI Pipeline

```
User pastes transcript (or Zoom/Sheets sync)
  → ingest_meeting_core() [commands/meetings.rs]
  → ai/extractor.rs: extract_tasks_from_transcript()
    → LiteLLM HTTP POST /chat/completions
    → System prompt: ai/prompts.rs (TASK_EXTRACTION_PROMPT)
    → Response parsed into Vec<CreateTaskInput>
  → health_score.rs: score_meeting()
  → Tasks inserted into DB
  → Meeting inserted into DB
  → React Query invalidates ["tasks", projectId] + ["meetings", projectId]
```

**LiteLLM HTTP format:**
```rust
POST {litellm_base_url}/chat/completions
Authorization: Bearer {api_key}
Body: { model, messages, temperature, max_tokens }
```

The `ai_settings` table stores: `provider`, `model`, `api_key` (encrypted in OS keychain), `base_url` (for LiteLLM self-hosted or custom OpenAI-compatible endpoints), `temperature`.

### Sync Architecture

```
useSync() [hooks/useSync.ts]
  → syncConnections() → invoke("sync_connections")
  → sync.rs::sync_all_connections()
    ├── sync_zoom() → zoom.rs HTTP calls → Vec<PendingImport>
    └── sync_sheets_relay() → sheets_relay.rs HTTP call → Vec<SheetRow>
                              → row_to_pending_import() (JSON blob extraction)
                              → upsert_pending_import() [INSERT OR IGNORE]
  → Returns SyncResult { new_imports, skipped_duplicates, errors }
```

**Deduplication**: `INSERT OR IGNORE` against the `source_email_id` UNIQUE INDEX. Same meeting arriving twice is silently skipped. `skipped_duplicates` count is returned so the UI can show a toast.

**Sheets Relay JSON blob handling**: The Google Workspace automation puts the full meeting JSON into multiple sheet columns. `extract_embedded_json()` scans `import_id`, `title`, `summary`, `action_items` fields in order — the first valid JSON object with known keys wins. `source_subject` always wins as title (strips "Meeting assets for ... are ready!").

---

## Security Model

| Secret | Storage |
|---|---|
| Zoom access/refresh tokens | OS keychain (keyring crate) |
| AI provider API keys | OS keychain |
| Sheets relay secret key | `app_settings` table (not keychain — avoids unsigned app prompts) |
| OAuth client IDs/secrets | Build-time env vars (`ZOOM_CLIENT_ID`, `ZOOM_CLIENT_SECRET`) |

No secrets ever touch disk directly or appear in logs. The SQLite database is unencrypted but contains no API keys.

---

## Component Patterns

### TaskCard

The card uses a custom checkbox (sr-only native input + styled div) so design is not constrained by OS browser styling. Metadata items are dot-separated inline (not flex gap) for a tighter layout. The `cancelingRef` pattern in MeetingCard prevents `onBlur` from committing when `onKeyDown` Escape triggered unmount (stale closure race condition).

### Filter Popovers (MeetingFilter, DateFilter)

Both use the same pattern:
1. `useRef` for outside-click detection
2. `useEffect` with `mousedown` listener on `document` when open
3. `animate-fade-in` CSS class on the dropdown
4. Absolute positioning `top-full left-0 mt-1`
5. `z-50` to escape overflow containers

### Kanban Board

Uses `@dnd-kit`. Each column is a `useDroppable` + `useDraggable` per card. Column status identity: each column has a `COLUMN_CHROME` entry with dot color, drop-zone accent, and label color. When `isOver`, the drop zone changes to a color-matched highlight.

---

## Known Technical Debt

| Item | Notes |
|---|---|
| `row_to_meeting` / `row_to_task` unused functions | Kept for potential future use; safe to delete |
| `ok: bool` field on `SheetResponse` never read | Harmless dead code |
| Deprecated `tauri_plugin_shell::Shell::open` | Should migrate to `tauri-plugin-opener` |
| `param_idx` unused assignment in `tasks.rs:114` | Cosmetic; harmless |
| `uuid::Uuid` unused import in `ai_settings.rs` | Cosmetic |
