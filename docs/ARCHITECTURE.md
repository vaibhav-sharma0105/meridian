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
│  SQLite + SQLCipher (~/.meridian/meridian.db)            │
│  FTS5 full-text search · Schema v007 · Encrypted at rest │
└─────────────────────────────────────────────────────────┘
                    │  HTTP (outbound only)
           AI Provider (OpenAI / Anthropic / LiteLLM / Ollama)
                    │
           Qdrant (localhost:6334) — semantic vector search
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

Current version: **v007**. Migrations run automatically on startup.

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
audit_log         — id, timestamp, action_type, entity_type, entity_id, user_id,
                    session_id, summary, details (JSON), before_state, after_state,
                    risk_level, external_effects, agent_initiated, autonomy_mode
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

### Embedding Architecture

```
Document Upload
  → parse_document() [documents/parsers/]
    ├── XLSX → calamine → markdown tables
    ├── PDF → pdf-extract → cleaned text
    └── TXT/MD → direct read
  → chunk_text() [ai/chunking.rs]
    └── 500 tokens, 50 overlap, natural breaks
  → embed() [ai/embeddings.rs]
    ├── BundledEmbedder (MiniLM-L6-v2 via ONNX)
    ├── OllamaEmbedder (nomic-embed-text default)
    └── OpenAI (text-embedding-3-small)
  → Qdrant upsert (project collection)
  → Set embeddings_ready = true
```

**Provider Selection**: Settings → `embedding_provider` field. Default is "bundled" (works offline). Fallback chain: configured → Ollama → bundled.

**Hybrid Search (RRF)**:
```
hybrid_search(query, project_id)
  → Embed query (if provider available)
  → Qdrant search (semantic, score >= 0.3)
  → FTS5 search (keyword)
  → RRF fusion: score = Σ 1/(60 + rank)
  → Deduplicate by document_id
  → Tag match_type: "semantic" | "keyword" | "both"
```

**ONNX Model**: MiniLM-L6-v2 bundled in `resources/models/`. Loaded lazily via `OnceLock` on first embed call. Mean pooling + L2 normalization → 384-dim vectors.

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

### Secret Storage

| Secret | Storage |
|---|---|
| Zoom access/refresh tokens | OS keychain (keyring crate) |
| AI provider API keys | OS keychain |
| Sheets relay secret key | `app_settings` table (not keychain — avoids unsigned app prompts) |
| OAuth client IDs/secrets | Build-time env vars (`ZOOM_CLIENT_ID`, `ZOOM_CLIENT_SECRET`) |

No secrets ever touch disk directly or appear in logs.

### Database Encryption

The SQLite database uses SQLCipher for encryption at rest (AES-256-CBC).

**Key derivation modes:**
- **Device mode** (default): Key derived from machine fingerprint (hostname + username + salt) via PBKDF2 (100k iterations). Transparent to users but not portable across machines.
- **Password mode**: User-provided password with PBKDF2 (100k iterations). Portable across machines.

**Key configuration** stored in `~/.meridian/key.json`:
```json
{ "mode": "device", "salt": "hex...", "pbkdf2_iterations": 100000 }
```

**Backward compatibility**: Existing unencrypted databases continue to work. New installs auto-initialize device-mode encryption. Migration to encrypted DB is optional.

### Audit Logging

All CRUD operations on tasks, meetings, and projects are logged to `audit_log` table:
- `action_type`: create, update, delete, archive, bulk_update, move, ingest, sync
- `entity_type`: task, meeting, project, connection, pending_import
- `risk_level`: low, medium, high, critical (classified by action type + external effects)
- `agent_initiated`: boolean flag to distinguish agent vs user actions
- `autonomy_mode`: supervised, semi_autonomous, autonomous

Retention: 2 years, with automatic pruning via background job.

### Pattern Learning

Pattern learning observes user behavior to provide smart suggestions. All data stays local.

**Data Flow:**
```
User action (task complete, priority change, draft edit)
  │
  ▼
Tauri command records observation ──→ pattern_observations table
  │                                      (observation_type, context_data JSON)
  │
  ▼
Daemon aggregation job (every 15 min)
  │
  ├── Group by observation_type & project_id
  ├── Calculate patterns (sequences, keyword mappings, style metrics)
  ├── Compute confidence (count × recency × consistency)
  ├── Apply decay (10% monthly for inactive patterns)
  └── Prune old observations (>90 days)
  │
  ▼
pattern_models table
  │ (pattern_type, model_data JSON, confidence, observation_count)
  │
  ▼
Frontend queries patterns via get_workflow_suggestions, get_smart_defaults, etc.
  │
  ▼
UI displays suggestions: WorkflowSuggestion, SmartDefaultIndicator, StyleAppliedBadge
```

**Tables:**
- `pattern_observations`: Raw observations with `observation_type`, `entity_type`, `entity_id`, `project_id`, `context_data` (JSON), `processed_at`
- `pattern_models`: Aggregated patterns with `pattern_type`, `project_id`, `model_data` (JSON), `confidence` (0.0-1.0), `observation_count`

**Pattern Types:**
1. `workflow_sequence` — Detects task completion sequences (A → B) within time windows
2. `smart_defaults` — Learns keyword → priority and keyword → assignee mappings
3. `communication_style` — Analyzes draft edits for length preference, formality, common phrases

**Confidence Thresholds:**
- Workflow suggestions: >= 0.5 (easy to dismiss)
- Smart defaults: >= 0.5 (pre-filled but editable)
- Communication style: >= 0.6 (directly affects AI output)

**Negative Learning:**
Dismissed suggestions are recorded. After 3 dismissals, patterns move to `negative_sequences` and stop being suggested.

### Proactive Agent

The proactive agent surfaces actionable suggestions based on task/meeting state and learned patterns. Suggestions are generated by a background daemon job (`generate_suggestions`) that runs every 30 minutes.

**Suggestion Types:**
1. `overdue_task` — Tasks past due date by 24+ hours
2. `stale_task` — In-progress tasks with no updates for 7+ days
3. `meeting_followup` — Meetings 24+ hours old with no linked tasks
4. `workflow_sequence` — Next-step suggestions based on learned workflow patterns

**Data Flow:**
```
daemon/jobs.rs: generate_suggestions
  ├── detect_overdue_tasks()  → suggestions table
  ├── detect_stale_tasks()    → suggestions table
  ├── detect_meeting_followups() → suggestions table
  └── detect_workflow_suggestions() → (checks pattern_models)
                                    → suggestions table
```

**Daily Limits:**
- Default: 10 suggestions per day (configurable via `suggestions_max_per_day` app setting)
- Ordered by severity: critical > warning > info

**Draft Generation:**
Tasks with action keywords (send, email, message, follow up) can auto-generate draft messages:
- Uses LiteLLM/OpenAI for draft text
- Adapts to learned communication style (length, formality, phrases)
- Includes "Drafted by Meridian" signature (toggleable)

**Sensitive Content Detection:**
Before sending/copying drafts, content is scanned for:
- PII: SSN (`\d{3}-\d{2}-\d{4}`), phone numbers, email addresses
- Credentials: API keys (`sk-...`), passwords, tokens
- Financial: Credit card numbers (Luhn-validated), bank accounts

Non-blocking warnings appear above the draft editor. Detections are logged to audit_log.

**Task Plans:**
New tasks can be analyzed for complexity:
- `simple` — Single action, auto-generates draft suggestion
- `medium` — 2-5 steps, suggests subtasks
- `complex` — Flags for manual breakdown

### Skills & Automation

Skills are user-defined automations that execute on schedule, event, or manual trigger. All skill data is local (SQLite).

**Data Flow:**
```
Trigger fires (cron schedule / event / manual button)
  │
  ▼
skills/cron.rs (schedule) │ skills/events.rs (event) │ commands/skills.rs (manual)
  │
  ▼
Create skill_run record (status=pending)
  │
  ▼
skills/executor.rs: build_context()
  ├── Fetch tasks (filtered by scope/project)
  ├── Fetch meetings
  └── Fetch documents (if context_config.include_documents)
  │
  ▼
Execute action (summarize / draft_message / create_tasks / analyze / custom)
  │
  ├── If approval_mode == auto|notify → commit result, set status=completed
  └── If approval_mode == approve_first|approve_always → set status=approval_pending
      └── Create notification → User approves/rejects → commit or cancel
```

**Tables:**
- `skills`: Definition (trigger_type, trigger_config JSON, context_config JSON, action_config JSON, approval_mode, enabled, shared, owner_id, cloned_from_id)
- `skill_runs`: Execution history (status, output, error, duration_ms, approval_decision)

**Known Gaps:** `shared`, `owner_id`, `cloned_from_id` columns exist but are non-functional (parked). No retry logic or timeout handling. See CLAUDE.md Section 15.

**Trigger Types:**
1. `schedule` — Cron expression with timezone. Daemon polls `get_due_scheduled_skills()` every 60s.
2. `event` — Fires on task_created, task_completed, meeting_imported, etc. via `EventDispatcher`.
3. `manual` — User clicks "Run" in UI.

**Event Integration:**
`EventDispatcher::fire_task_created()` / `fire_task_completed()` / `fire_meeting_imported()` are called from `commands/tasks.rs` and `commands/meetings.rs`. The dispatcher queries enabled event-skills matching the event type and filter, then queues execution jobs.

**Daemon Jobs:**
- `execute_skill` — Runs a single skill (builds context, executes action, records output)
- `poll_scheduled_skills` — Finds due scheduled skills and queues `execute_skill` jobs
- `check_skill_approvals` — Expires approval_pending runs older than 24h

**Built-in Skills:**
- 5 templates embedded via `include_str!("../../resources/builtin-skills/templates.json")` in `skills/builtin.rs`
- Auto-loaded on first launch (gated by `app_settings.builtin_skills_initialized`)
- Templates: Weekly Summary, Meeting Follow-up, Overdue Alert, Sprint Prep, End of Day Digest
- "Reset defaults" button deletes `WHERE is_builtin = 1` then re-seeds via `reset_builtin_skills` command
- `is_builtin` column (migration v013) prevents deletion of built-in skills; UI hides Delete option

**Folder-based Skill Packages:**
- Installed to `~/.meridian/skills/<folder_name>/`
- `skills/folders.rs` handles filesystem: list, install (copy dir), validate, delete, read file, execute script
- Validation: `skill.md` must exist with YAML frontmatter containing `name:` and `description:`
- File tree built recursively with executable detection (by extension: `.py`, `.js`, `.sh`, etc.)
- Scripts execute via `std::process::Command` with path-traversal protection
- Frontend: `SkillFoldersPanel.tsx` shows tree view, file viewer modal, execute confirmation dialog
- Human-in-the-loop: execution requires explicit user confirmation in modal
- Upload via "Upload Skill" button (replaces old file-based Import)

**Platform-specific Folder Picker (`pick_folder_dialog`):**
- macOS: `osascript -e "choose folder"` (AppleScript) — works reliably unlike NSOpenPanel sheet attachment
- Windows/Linux: `rfd::FileDialog::new().pick_folder()` (rfd crate v0.16)
- Both wrapped in `tokio::task::spawn_blocking` (blocking I/O)
- Used for both skill upload and export-to-directory

**Skill Export (directory-based):**
- `export_skill_to_directory` command creates a package directory with `skill.md` inside
- Uses same platform folder picker to choose export location
- Directory named with kebab-case slug of skill name
- Format: YAML+MD via `skillToSkillFile()` — exported packages can be re-uploaded directly

**Chat-to-Skill Extraction:**
- `extract_skill_from_chat` command in `commands/skills.rs` uses LiteLLM to extract skill definition from natural language
- AIChatPanel shows Wand2 icon on assistant messages → `ChatToSkillPreview` component
- On confirm: sets `uiStore.skillEditorData` → navigates to Skills view → auto-opens editor pre-filled

**AI Chat Skill Integration:**
```
User message
  │
  ▼
useAI hook: merge DB skills + folder packages → UnifiedSkill[]
  │
  ▼
Format compact context: "📦 **name** - description" (one line per skill)
  │
  ▼
chatWithProject API: skillContext param → system prompt injection
  │
  ▼
LLM decides: invoke skill? → outputs **[SKILL_INVOKE: skill_name]**
  │
  ▼
AIChatPanel: parseSkillInvocation() extracts skill name
  │
  ├── Check: already invoked this conversation? → skip
  ├── Check: processedMsgIndices ref → prevent race condition
  │
  ▼
loadSkillContent(): fetch full skill.md (cached per conversation)
  │
  ▼
executeSkill(): DB skill → runSkillManually | Folder → executeSkillScript
  │
  ▼
UI: subtle green checkmark + skill name (only on completion)
```

**UnifiedSkill Interface:**
- `type: "db" | "folder"` — discriminator for execution routing
- `originalSkill?: Skill` — reference for DB skill execution
- `originalFolder?: SkillFolder` — reference for folder package execution
- `folderName?: string` — for folder script execution

**Skill Picker:**
- `/skill` command in chat input triggers popup
- Shows both DB skills (⚡) and folder packages (📦) in unified list
- Search filters by name/description
- Selected skill shown as badge above input

**Deduplication:**
- `invokedSkills: Set<string>` tracks skills invoked per conversation
- `processedMsgIndices: useRef<Set<number>>` prevents race condition re-execution
- Only last assistant message processed (not full array)
- `clearMessages()` resets both tracking mechanisms

**Pattern Observations:**
- `record_skill_output_edit` — records when user edits skill output (feeds communication style learning)
- Skill enable/disable and manual triggers record observations for workflow pattern analysis

**Key Files:**
- `src-tauri/src/skills/` — models, repository, cron, events, executor, approval, dispatcher, builtin, folders
- `src-tauri/src/commands/skills.rs` — 24 Tauri commands (including folder operations + initialize/reset builtin)
- `src-tauri/resources/builtin-skills/templates.json` — embedded skill templates
- `src/components/skills/` — SkillsPage, SkillCard, SkillEditorModal, SkillHistoryPanel, SkillApprovalModal, ChatToSkillPreview, SkillFoldersPanel
- `src/hooks/useSkills.ts` — React Query hooks (includes useResetBuiltinSkills)
- `tests/e2e/skills.spec.ts` — 18 Playwright E2E tests (creation flow + history view)

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
