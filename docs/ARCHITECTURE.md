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

### Data Flow Summary

Reads:
```
User action (click/type)
  → Zustand store update (taskStore.setFilters / uiStore.setSelectedTask)
  → React Query hook (useTasks/useMeetings) re-runs query
  → useTasks strips client-only filter fields
  → invoke("get_tasks_for_project", { projectId, filters })
  → Rust: commands/tasks.rs → db/repositories/tasks.rs (SQL)
  → SQLite → Vec<Task>
  → React Query cache updated
  → Component re-renders
```

Writes (create/update/delete):
```
Component calls api.updateTask(input)
  → invoke("update_task", { input })
  → Rust: optimistic mutation in onMutate (React Query)
  → commands/tasks.rs → repositories/tasks.rs
  → qc.setQueryData (immediate) OR qc.invalidateQueries (eventual)
```

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

**Providers** (three available):
- **Bundled (default)**: MiniLM-L6-v2 via ONNX Runtime (~86MB model). Works offline, 384-dimensional vectors.
- **Ollama**: Local Ollama server with nomic-embed-text or other models. Requires Ollama running.
- **OpenAI**: `text-embedding-3-small` API. Requires API key and internet.

**Hybrid Search (RRF)** combines semantic (Qdrant vectors) and keyword (FTS5) search using Reciprocal Rank Fusion with k=60. Results tagged with match type: `semantic`, `keyword`, or `both`.

**Embedding Worker:**
- In-process background worker polls the `daemon_jobs` table for `embed_document` jobs
- Started via `start_embedding_worker` command; runs in a separate thread with its own tokio runtime
- Jobs queued automatically on document upload with priority (10=high, 5=normal, 1=migration)
- `IndexingBanner` component shows progress and allows starting the worker manually

**Document Parsing:**
- `src-tauri/src/documents/parsers/xlsx.rs` — XLSX via calamine
- `src-tauri/src/documents/parsers/pdf.rs` — PDF via pdf-extract

**Key Files:**
- `src-tauri/src/ai/embeddings.rs` — `BundledEmbedder`, `EmbeddingProvider` trait
- `src-tauri/src/ai/chunking.rs` — Text chunking (500 tokens, 50 overlap)
- `src-tauri/src/ai/search.rs` — Hybrid search with RRF fusion
- `src-tauri/src/daemon/` — Background worker for embedding jobs
- `src-tauri/resources/models/all-MiniLM-L6-v2/` — Bundled ONNX model

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

**Observation Types:**
- `task_completion` — Recorded when task status changes to "done"
- `priority_set` — Recorded when task priority is changed
- `assignee_set` — Recorded when task assignee is changed
- `draft_edit` — Recorded when user edits an AI-generated draft
- `suggestion_dismissed` — Recorded when user dismisses a workflow suggestion

**Aggregation** (daemon job `aggregate_patterns`, every 15 minutes):
- Processes unprocessed observations from `pattern_observations`
- Updates `pattern_models` with confidence scores
- Applies 10% decay to patterns inactive for 30+ days
- Prunes processed observations older than 90 days

**Key Files:**
- `src-tauri/src/patterns/models.rs` — Pattern observation and model structs
- `src-tauri/src/patterns/repository.rs` — Pattern CRUD operations
- `src-tauri/src/commands/patterns.rs` — Tauri commands for pattern queries
- `src-tauri/src/daemon/jobs.rs` — `aggregate_patterns` job handler
- `src/components/patterns/` — Frontend components for suggestions and settings

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

**Key Files:**
- `src-tauri/src/suggestions/` — Suggestion models and repository
- `src-tauri/src/drafts/` — Draft message models and repository
- `src-tauri/src/sensitive/mod.rs` — Sensitive content detection (PII, credentials, financial)
- `src-tauri/src/daemon/jobs.rs` — `generate_suggestions` job handler
- `src/components/suggestions/` — SuggestionCard, SuggestionsList UI

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

**Known Gaps:** `shared`, `owner_id`, `cloned_from_id` columns exist but are non-functional (parked). No retry logic or timeout handling. See "Skills: Known Gaps / Future Work" at the end of this section.

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
- `src-tauri/src/skills/models.rs` — Skill, SkillRun, TriggerConfig, ActionConfig structs
- `src-tauri/src/skills/repository.rs` — CRUD operations for skills and runs
- `src-tauri/src/skills/cron.rs` — Cron parsing with timezone support
- `src-tauri/src/skills/events.rs` — Event types and filter matching
- `src-tauri/src/skills/executor.rs` — Context building and action execution
- `src-tauri/src/skills/approval.rs` — Approval workflow (approve/reject/expire)
- `src-tauri/src/skills/dispatcher.rs` — Trigger dispatch
- `src-tauri/src/skills/builtin.rs` — Built-in template loading (`include_str!` from templates.json)
- `src-tauri/src/skills/folders.rs` — Filesystem operations (list, install, validate, delete, read, execute)
- `src-tauri/src/skills/sandbox.rs`, `sync.rs`, `chat_extract.rs` — sandboxed execution, created-files dir, chat-to-skill extraction
- `src-tauri/src/commands/skills.rs` — 39 Tauri commands (incl. folder ops, initialize/reset builtin, folder picker/export)
- `src-tauri/resources/builtin-skills/templates.json` — 5 embedded skill templates
- `src/components/skills/` — SkillsPage, SkillCard, SkillEditorModal, SkillHistoryPanel, SkillApprovalModal, ChatToSkillPreview, SkillFoldersPanel
- `src/components/skills/SkillFoldersPanel.tsx` — File tree UI, upload, execute dialog
- `src/components/skills/SkillsList.tsx` — "Upload Skill" button (folder picker replaces old Import)
- `src/components/skills/SkillsPage.tsx` — Auto-shows folders panel when packages exist
- `src/components/ai/SkillPicker.tsx`, `AIChatPanel.tsx` — `/skill` command picker and invocation
- `src/hooks/useSkills.ts` — React Query hooks (includes useResetBuiltinSkills)
- `src/hooks/useAI.ts` — `UnifiedSkill`, `executeSkill`, `loadSkillContent`
- `src-tauri/src/commands/ai.rs` — `skillContext` param
- `tests/e2e/skills.spec.ts` — 29 Playwright E2E tests

**Trigger types:** `schedule` (cron, e.g. "every Monday at 9am") · `event` (task_created, task_completed, meeting_imported, …) · `manual` (user-initiated via UI button).

**Approval Modes:**
- `auto`: Execute immediately, no notification
- `notify`: Execute and notify user of results
- `approve_first`: Require approval for actions with side effects
- `approve_always`: Always require approval before execution

**Action Types:**
- `summarize`: Generate summary of tasks/meetings
- `draft_message`: Create email/Slack draft
- `create_tasks`: Suggest tasks to create (requires approval)
- `analyze`: Provide insights on project data
- `custom`: User-defined prompt

**Context Configuration:**
- `scope`: "project" (current project only) or "global" (all projects)
- `include_documents`: Include project documents in skill context (checkbox in Basic mode)
- `document_filter`: Regex pattern to filter documents by filename (shown when include_documents is on)
- `max_documents`: Maximum documents to include, 1-50 (shown when include_documents is on, default: 10)
- `max_tokens`: Token budget for context (default: 8000, truncates by priority)
- `priority_order`: Truncation priority (default: tasks > meetings > documents)

**Skill Run Lifecycle:**
1. Trigger fires (cron/event/manual)
2. Create skill_run record with status=pending
3. Build context (tasks, meetings, documents)
4. Execute action
5. If needs_approval: set status=approval_pending, create notification
6. On approve: apply pending changes, set status=completed
7. On reject: set status=cancelled with reason

**Editor Enhancements:**
- System prompt textarea with `{{variable}}` insertion helper (6 variables: tasks, meetings, project_name, date, overdue_count, completed_today)
- Test Run button (visible when editing existing skill) — previews context without executing
- History panel with status filter dropdown and paginated run list (10 per page)

**Sharing [PARKED]:**
- `shared`: Boolean flag stored but non-functional (local-first app has no sharing mechanism)
- Shared skills show "Shared" badge on card (cosmetic only)
- Clone works but `cloned_from_id` not tracked
- `owner_id` column exists but never set

**Progressive Skill Loading:**
- Phase 1: LLM gets lightweight context (name + description only)
- Phase 2: When skill invoked, `loadSkillContent()` fetches full skill.md content
- Phase 3: For folder packages, find and execute main script (main.py, run.sh, index.js)
- Content cached in `loadedSkillContent` ref per conversation; cleared on `clearMessages()`

**Dynamic Skill Invocation (LLM-driven):**
- Both DB skills AND folder packages are merged into `UnifiedSkill` with `originalSkill`/`originalFolder` references
- Compact context format sent to LLM: `📦 **name** - description` (not verbose YAML)
- LLM receives clear instructions: when to invoke (explicit request, direct match) vs when not to (simple questions, conversation)
- When a skill matches intent, LLM outputs `**[SKILL_INVOKE: skill_name]**` at response start
- Frontend parses the marker and executes via `executeSkill()`, which handles both types
- Subtle UI: only a small green checkmark + skill name after completion (no running/failed states shown)
- Both manual (`/skill` picker) and automatic (LLM-detected) invocation supported

**Skill Selection in AI Chat (`/skill` command):**
- Type `/skill` in the chat input to open the SkillPicker popup
- Popup shows enabled skills in a scrollable list (5 visible at a time)
- Search bar at bottom filters skills by name/description
- Clicking a skill adds it as a badge above the input
- The skill's context (name + description) is prepended to the AI message
- User can send with just the skill selected (no additional text required)

**Skill Execution Deduplication:**
- `invokedSkills` Set tracks invoked skills per conversation
- `processedMsgIndices` ref prevents race-condition re-executions
- Only the last assistant message is processed (not full array iteration)
- One skill per response is enforced in the LLM prompt

### Skill Format (YAML+MD)

Skills use the Anthropic standard format: YAML frontmatter + Markdown body with `# Section` headings. JSON is kept internally in SQLite; YAML+MD is the user-facing authoring/exchange format.

**User-facing format (skill.md):**
```yaml
---
name: Weekly Progress Report
description: Generate weekly summary every Monday
trigger:
  type: schedule
  cron: "0 9 * * 1"
action:
  type: summarize
settings:
  approval_mode: notify
  category: reporting
---

# Instructions

Summarize the week's progress using {{tasks}} and {{meetings}}.
Group completed tasks by assignee. List overdue items.

# Context

{{tasks}} {{meetings}} {{project_name}} {{date}}

# Output Format

## Weekly Report — {{project_name}}
### Completed | In Progress | Overdue
```

**Sections in body:**

| Section | Required | Description |
|---------|----------|-------------|
| `Instructions` | Yes | Step-by-step what to do |
| `Context` | No | Data/variables to inject |
| `Output Format` | No | Expected output structure |
| `Examples` | No | Input/output examples |

**Key files:**
- `src/lib/skill-format.ts` — Parse/serialize YAML+MD, convert DB↔skill.md, variable list
- `src/lib/skill-prompt.ts` — Legacy XML parser (still used by v2 JSON export/import)
- `src/components/skills/SkillEditorModal.tsx` — Single textarea showing YAML+MD content
- `src/components/skills/PromptSectionEditor.tsx` — Legacy component (unused by editor, kept for reference)

**Editor:** Single monospace textarea showing the raw YAML+MD content. Users edit frontmatter (name, trigger, action, settings) and markdown body (`# Instructions`, etc.) directly. Variable insertion helper available via toolbar button.

**Import/Export:**
- **Export**: Saves as `.md` file (YAML+MD format) via Tauri native save dialog
- **Import**: Accepts `.md`, `.yaml`, `.yml` (YAML+MD), `.json` (v1), `.skill.json` (v2)
- Detection: `isSkillMdFormat()` checks for `---` prefix; falls back to JSON parsing

**Backward compatibility:**
- Old XML-tagged `system_prompt` strings are parsed into markdown sections on edit
- Old freeform strings land in the `# Instructions` section
- Legacy v1/v2 JSON imports still work

### Skill Types & Permissions

Three distinct skill types with different permission models:

| Type | Editable | Deletable | Source |
|------|----------|-----------|--------|
| Built-in | Yes | No (reset only) | `resources/builtin-skills/templates.json`, loaded on first launch |
| User-created | Yes | Yes | Created via editor or imported |
| Folder packages | No (read-only) | Yes | Uploaded from `~/.meridian/skills/` |

**Built-in flag (`is_builtin`):**
- Migration v013 adds `is_builtin INTEGER NOT NULL DEFAULT 0` to skills table
- `load_builtin_skills()` sets `is_builtin: true` when creating templates
- `delete_skill()` rejects deletion if `is_builtin = true`
- `reset_builtin_skills()` deletes only `WHERE is_builtin = 1` then re-creates
- UI: "Built-in" badge on card, no Delete option in menu

**Folder packages (`~/.meridian/skills/`):**
- Each subfolder is a skill package with scripts, configs, README
- File tree viewer with progressive disclosure (expand/collapse)
- Read-only in UI — view file contents but no inline editing
- Deletable (removes entire folder from disk)
- Executable scripts require human-in-the-loop confirmation dialog

**Folder upload & validation:**
- "Upload Skill" button uses the `pick_folder_dialog` command for the native folder picker
- macOS: `osascript -e "choose folder"` (AppleScript) — NSOpenPanel via Tauri/rfd has sheet attachment issues
- Windows/Linux: `rfd::FileDialog::new().pick_folder()`
- Validation: `skill.md` must exist with YAML frontmatter containing `name:` and `description:`

**Skill export (directory-based):**
- `export_skill_to_directory` command: folder picker → creates `{slug}/skill.md`
- Serializes skill to YAML+MD format via `skillToSkillFile()` in `src/lib/skill-format.ts`
- Exported packages can be directly re-uploaded as folder packages

**Script execution:**
- Supported: `.py`, `.js`, `.ts`, `.sh`, `.bash`, `.zsh`, `.rb`, `.pl` (cross-platform)
- Platform-specific: `.ps1`, `.bat`, `.cmd` (Windows only)
- Runs with user permissions in the skill folder as working directory
- Path traversal protection: validates path stays within `~/.meridian/skills/<folder>/`

### Skills: Known Gaps / Future Work

> These have schema/UI scaffolding but need implementation. See `openspec/changes/archive/2026-07-15-phase-4-skills-automation/tasks.md` Section 30 for full details.

| Item | Status | What's Missing |
|------|--------|----------------|
| **Skill Sharing** | PARKED | No multi-user sync (local-first app) |
| **Owner Tracking** | PARKED | `owner_id` column never set |
| **Clone Source** | PARKED | `cloned_from_id` not set in `clone_skill()` |
| **Skills → Suggestions** | NOT IMPL | Skills don't create suggestions |
| **Suggestion → Skill Trigger** | NOT IMPL | Accepting suggestion doesn't trigger skill |
| **Retry Logic** | NOT IMPL | Failed skills stay failed |
| **Timeout Handling** | NOT IMPL | No timeout for long-running skills |

---

### External Integrations

Phase 5 adds a unified framework for connecting GitHub, Jira, Slack and other external services.

**Data Flow (OAuth Connection):**
```
User clicks "Connect" on integration card
  │
  ▼
SetupWizard: shows prerequisites, step-by-step instructions
  │
  ▼
start_oauth_flow command: generate auth URL, store state in memory HashMap
  │
  ▼
Open browser → User authorizes → Redirect to localhost:8765/oauth/callback
  │
  ▼
handle_oauth_callback: exchange code for tokens, create integration record
  │
  ▼
Integration appears in "Connected" section
```

**Data Flow (Sync):**
```
User clicks "Sync" or sync_interval triggers
  │
  ▼
sync_integration command: get IntegrationProvider for type
  │
  ▼
provider.fetch_data(&integration): API calls to GitHub/Jira/Slack
  │
  ▼
Returns FetchResult { items: Vec<FetchedItem>, errors: Vec<String> }
  │
  ▼
Upsert items into integration_cache table
  │
  ▼
Update integration.last_sync timestamp
```

**Data Flow (Task Linking):**
```
User opens LinkPicker for a task
  │
  ▼
Fetch cached items from connected integrations
  │
  ▼
User searches and selects external item (issue/PR/ticket)
  │
  ▼
create_integration_link: creates bidirectional link record
  │
  ▼
IntegrationLinkBadge shows on task card
```

**Tables:**
- `integrations`: Connected services (type, name, config JSON with encrypted tokens, autonomy_mode, status, last_sync, error_message)
- `integration_cache`: Fetched external data (integration_id, external_type, external_id, external_url, data JSON, synced_at)
- `integration_links`: Task↔External mappings (local_type, local_id, external_type, external_id, sync_enabled)

**IntegrationProvider Trait:**
```rust
#[async_trait]
pub trait IntegrationProvider: Send + Sync {
    fn integration_type(&self) -> &'static str;
    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String>;
    async fn exchange_token(&self, code: &str, redirect_uri: &str, code_verifier: Option<&str>) -> Result<OAuthTokenResponse, String>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse, String>;
    async fn fetch_data(&self, integration: &Integration) -> Result<FetchResult, String>;
    fn get_scopes(&self) -> Vec<&'static str>;
    fn validate_config(&self, config: &IntegrationConfig) -> Result<(), String>;
}
```

**MCP Server Permissions:**
```rust
struct McpPermissions {
    read_tasks: bool,        // default true
    read_meetings: bool,     // default true  
    read_projects: bool,     // default true
    create_task: bool,       // default false
    update_task: bool,       // default false
    delete_task: bool,       // default false
    create_meeting_note: bool, // default false
    run_skill: bool,         // default false
    rate_limit_per_minute: u32, // default 100
}
```

**Key Files:**
- Backend: `src-tauri/src/integrations/` (mod, models, repository, github, jira, slack, slack_socket, webhook)
- Daemon: `src-tauri/src/daemon/jobs.rs` (sync_integration, poll_integration_syncs job handlers)
- Commands: `src-tauri/src/commands/integrations.rs` (26 commands), `daemon.rs` (get_background_jobs), `settings.rs` (MCP permissions)
- MCP: `src-tauri/meridian-mcp/src/handlers.rs` (write tools with permission checks, rate limiting)
- Frontend: `src/components/integrations/` (IntegrationsPage, SetupWizard, BackgroundJobsPanel, SlackDraftsPanel, *Settings, LinkPicker)
- Hooks: `src/hooks/useIntegrations.ts`, `src/hooks/useIntegrationLinks.ts`
- Store: `src/stores/integrationStore.ts`

**Daemon Integration Jobs:**
```
poll_integration_syncs (every 5 min)
  └─> For each connected integration where last_sync + sync_interval < now
        └─> Queue sync_integration job

sync_integration job
  └─> Get IntegrationProvider for type
  └─> provider.fetch_data(&integration)
  └─> Upsert items to integration_cache
  └─> Update last_sync timestamp
  └─> Create notification (info for success, critical for errors)
```

**Slack Socket Mode:**
- WebSocket connection via tokio-tungstenite
- Auto-reconnect with exponential backoff (1s → 300s max)
- Action item detection: mentions (@user), questions (?), requests (please/could you), deadlines (by/due/EOD), follow-ups
- Per-channel autonomy modes: auto, notify, approve_first, approve_always

**Integration UI Flow:**
1. User clicks Integrations (Link2 icon) in sidebar → Opens IntegrationsPage modal
2. Page shows two collapsible sections: Native Integrations and MCP Servers
3. Click "Connect" on any integration → Opens SetupWizard
4. SetupWizard shows: Prerequisites, step-by-step instructions, OAuth flow (steps are toggleable for accidental clicks)
5. After OAuth: Integration appears in "Connected" section with Settings button
6. Settings modals: Configure sync interval, repo/project/channel selection, disconnect

**OAuth Flow:**
1. `start_oauth_flow` generates auth URL and stores state in memory
2. User authorizes in browser, redirected to callback URL
3. `handle_oauth_callback` exchanges code for token, creates integration record
4. Token refresh via `refresh_integration_token` when expired

**MCP Write Operations:**
MCP server (`meridian-mcp`) includes write tools with permission checks:
- `create_task`, `update_task`, `create_meeting_note`, `run_skill`
- Permissions stored in `app_settings.mcp_permissions` JSON
- Rate limited to 100 ops/minute with sliding window
- All operations logged to audit log with `agent_initiated: true`

**Desktop Notifications:**
- `tauri-plugin-notification` wired for OS-level notifications
- Severity levels: `info` (badge only), `warning` (toast), `critical` (toast + sound)
- User can disable via `desktop_notifications_enabled` setting
- NotificationCenter shows severity badges and integration icons

**Database (v014 migration):**
- `integrations` — Stores connected services with encrypted OAuth tokens in `config` JSON column
- `integration_cache` — Stores fetched external data (issues, PRs, channels)
- `integration_links` — Bidirectional links between Meridian tasks and external items
- `notifications` extended with `severity`, `desktop`, `integration_id` columns

**Additional key files:**
- `src-tauri/src/integrations/google.rs` — Google Workspace OAuth + Directory API member fetch (used for team roster sync only — no generic issues/PRs `fetch_data` content). Needs Workspace domain admin approval for the directory scope; see Team & Sync Known Gaps and CREDENTIALS_SETUP.md Part 5b.
- `src-tauri/src/integrations/webhook.rs` — Local HTTP server for OAuth callbacks
- `src/components/integrations/GitHubSettings.tsx` — GitHub repo selection and sync config
- `src/components/integrations/JiraSettings.tsx` — Jira project selection
- `src/components/integrations/SlackSettings.tsx` — Channel autonomy config
- `src/components/integrations/SlackDraftsPanel.tsx` — Pending Slack drafts with delayed send queue
- `src/components/integrations/MCPSettings.tsx` — MCP Server permission config
- `src/components/integrations/NotificationSettings.tsx` — Desktop notification preferences
- `src/components/integrations/BackgroundJobsPanel.tsx` — Active/recent daemon jobs (syncs, embeddings, skills), auto-refresh every 5s
- `src/components/integrations/LinkPicker.tsx` — Search and link external items to tasks
- `src/components/tasks/IntegrationLinkBadge.tsx` — Badge showing linked GitHub/Jira items
- `init_integration_jobs()` — Called on daemon startup to schedule the initial poll job

---

## Governance & Autonomy

Phase 6 introduces unified autonomy control, risk classification, approval workflows, and undo capabilities for agent-initiated actions.

### Autonomy Modes

Three global autonomy modes control how much agent assistance runs automatically:

| Mode | Low Risk | Medium Risk | High Risk | Critical Risk |
|------|----------|-------------|-----------|---------------|
| Manual | Requires approval | Requires approval | Requires approval | Requires approval |
| Supervised (default) | Auto-execute | Auto-execute | Requires approval | Requires approval |
| Autonomous | Auto-execute | Auto-execute | Auto-execute | Requires approval |

### Autonomy Inheritance

```
Global Autonomy (app_settings.autonomy_mode)
    ↓ inherited by (if NULL)
Integration Autonomy (integrations.autonomy_mode)
    ↓ inherited by (if NULL)
Skill Autonomy (skills.autonomy_mode)
```

When evaluating an action, the system resolves the effective autonomy by walking up the chain.

### Risk Classification

Risk is calculated from three weighted scores:

```
RiskScore = (action_type_weight × destination_score × content_score) / max_possible

action_type_weight:
  read=1, create=2, update=3, external_send=4, delete=5

destination_score:
  internal=1, team=2, external=3, executive=4

content_score:
  normal=1, sensitive=2, pii=3, financial=4
```

**Critical Override:** If ANY individual score is at maximum (delete=5, executive=4, financial=4), the action is classified as Critical regardless of the composite score.

### Approval Flow

```
Action Initiated (skill, MCP, suggestion)
  └─> evaluate_action(action_type, destination, content)
        └─> calculate_risk() → RiskLevel
        └─> resolve_effective_autonomy() → AutonomyMode
        └─> should_require_approval(risk, mode)?
              ├─ No  → Execute immediately
              └─ Yes → queue_for_approval()
                         └─> Create pending_approval record
                         └─> Set timeout (default 24h)
                         └─> Notify user
                         └─> Wait for approve/reject/timeout
```

### Undo System

Agent-initiated mutations can be undone if they're internal and not deletes:

```
capture_action_state(action_type, entity_type, entity_id, before_state, after_state)
  └─> is_undoable = !is_external(entity_type) && before_state.is_some() && action_type != "delete"
  └─> Create action_history record

undo_action(action_id)
  └─> Check undoable=true AND undo_action_id=NULL
  └─> Determine reversal_type (create→delete, update→restore)
  └─> Execute reversal SQL
  └─> Mark original action as undone
```

External entity types (slack_message, github_issue, jira_issue) are always non-undoable.

### Governance Data Flow

```
Frontend (GovernancePage)
  ├─ Approvals tab: usePendingApprovals() → get_pending_approvals
  │    └─ User clicks Approve/Reject → approve_pending_action / reject_pending_action
  ├─ History tab: useActionHistory() → get_action_history
  │    └─ User clicks Undo → undo_action
  ├─ Dashboard tab: useGovernanceMetrics() → get_governance_metrics
  │    └─ Aggregated daily by daemon job: aggregate_governance_metrics
  └─ Settings tab: useAutonomySetting() → get/set_autonomy_setting
```

### Daemon Jobs

| Job | Frequency | Purpose |
|-----|-----------|---------|
| `check_approval_timeouts` | Every minute | Archive expired pending approvals |
| `aggregate_governance_metrics` | Daily at midnight | Compute risk/approval aggregates |
| `detect_anomalies` | Hourly | Flag activity spikes and high rejection rates |

### Key Files

**Backend:**
- `src-tauri/src/governance/mod.rs` — Module root
- `src-tauri/src/governance/models.rs` — RiskLevel, AutonomyMode, PendingApproval, ActionHistory
- `src-tauri/src/governance/risk.rs` — Risk classification engine
- `src-tauri/src/governance/autonomy.rs` — Autonomy controller with inheritance
- `src-tauri/src/governance/approval.rs` — Approval queue operations
- `src-tauri/src/governance/undo.rs` — Action history and undo system
- `src-tauri/src/governance/repository.rs` — CRUD for governance tables
- `src-tauri/src/commands/governance.rs` — 20 Tauri commands

**Frontend:**
- `src/hooks/useGovernance.ts` — React Query hooks
- `src/components/governance/GovernancePage.tsx` — Main view with tabs
- `src/components/governance/AutonomySettings.tsx` — Mode selector
- `src/components/governance/ApprovalQueue.tsx` — Pending approvals list
- `src/components/governance/UndoBar.tsx` — Toast-style undo notification
- `src/components/governance/ActionHistoryPanel.tsx` — Filterable history
- `src/components/governance/GovernanceDashboard.tsx` — Metrics and charts

**Database (v015 migration):**
- `pending_approvals` — Approval queue with timeouts
- `action_history` — Before/after state for undo
- `governance_metrics` — Daily aggregates
- `risk_adjustments` — User-defined risk overrides

---

## Team & Sync (Phase 7)

Phase 7 adds team roster management, intelligent assignee suggestions, and data export/import.

### Team Roster

- `team_members` table stores members from multiple sources: `manual`, `slack`, `google` (Google sync is UI/command scaffolding only — no Google integration exists in this codebase yet, see Known Gaps below)
- Each member has `workload_score` (0-1) computed from open task count via `compute_all_workload_scores()` (`team/repository.rs`). This now actually runs: daily as part of `process_poll_team_syncs_job` (`daemon/jobs.rs`), and on-demand via the "Recompute Workloads" button in `TeamSettings.tsx` (`compute_team_workloads` command). Previously the scoring function existed and was unit-tested but nothing ever called it, so `workload_score` stayed NULL forever.
- Sync from Slack via `sync_team_from_slack` command
- Daily sync job via `poll_team_syncs` daemon job

### Assignee Intelligence

Multi-factor scoring for task assignment suggestions:
- `pattern_score` — Based on historical assignments in `pattern_models`
- `workload_score` — Inverse of current workload (available = high score)
- `expertise_score` — Keyword matching between task and member expertise
- `recency_score` — Recent task completions by the member
- **Empty-roster fallback**: `get_assignee_suggestions()` (`team/assignee.rs`) suggests directly from `smart_defaults` keyword→assignee patterns (`pattern_models`) when `team_members` is empty, instead of returning no suggestions. These fallback suggestions carry `member.source = "pattern"` (synthetic, not a real `TeamMember` row) so the frontend can distinguish them once AssigneePicker is wired up.
- `record_assignee_selection()` is wired to the `record_assignee_selection` Tauri command (`commands/team.rs`) and exposed as `api.recordAssigneeSelection()`. Not yet called from any UI — see Known Gaps.

**Key files (Backend):**
- `src-tauri/src/team/mod.rs` — Module root
- `src-tauri/src/team/models.rs` — TeamMember, AssigneeSuggestion structs
- `src-tauri/src/team/repository.rs` — CRUD, workload computation, `record_expertise_observation()` (expertise auto-learning)
- `src-tauri/src/team/assignee.rs` — Multi-factor scoring algorithm
- `src-tauri/src/commands/team.rs` — 10 Tauri commands (incl. `sync_team_from_google`, `record_assignee_selection`)
- `src-tauri/src/integrations/google.rs` — Google Workspace OAuth provider + Directory API member fetch. Env-gated on `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET` (see CREDENTIALS_SETUP.md Part 5b) — requires a Workspace domain and admin-approved `admin.directory.user.readonly` scope; personal Google accounts can't use this.

**Key files (Frontend):**
- `src/hooks/useTeam.ts` — React Query hooks
- `src/components/team/TeamSettings.tsx` — Roster management UI (Sync Slack, Sync Google, Recompute Workloads)
- `src/components/team/TeamMemberCard.tsx` — Member display with workload
- `src/components/tasks/AssigneePicker.tsx` — Multi-assignee picker with AI suggestions, mounted in `TaskEditModal` only (chips + dropdown, same `value`/`onChange` contract as `AssigneeChipInput` — comma-separated string). Selecting or overriding a suggestion calls `record_assignee_selection` internally for pattern learning. `wasOverride` is only true if the AI's #1 suggestion hasn't already been added and you picked someone else instead — adding a 4th/5th person after honoring the top pick doesn't count as an override.
- `src/components/integrations/GoogleSettings.tsx` — Google Workspace connection settings + manual "Sync Team Roster Now"

**Reachability:** `TeamSettings`, `ExportDialog`, and `ImportDialog` are mounted via a "Team & Data" section inside `IntegrationsPage.tsx` (Settings → Integrations → Team & Data), not as separate sidebar entries. This was a real bug, not just missing polish — all three components existed and were fully implemented but had zero JSX usage anywhere in the app until this was wired up; `TeamSettings` now takes an optional `onClose` prop for its modal wrapper.

**Bulk task updates also feed learning:** `bulk_update_tasks` (commands/tasks.rs) now snapshots pre-update task state and fires the same `record_completion_observation`/`record_assignee_observation` calls as the single-task `update_task` path — previously, completing multiple tasks via a bulk action taught the system nothing (patterns *and* expertise), only completing them one at a time did.

### Expertise Auto-Learning

- `team::repository::record_expertise_observation()` bumps a per-keyword pending count (`team_members.expertise_pending`, v017 migration) each time an assignee completes a task whose title/description keywords don't already match an existing expertise tag.
- A keyword is promoted into the member's visible `expertise` array only after `EXPERTISE_PROMOTION_THRESHOLD` (3) separate completions — a single task never mutates expertise on its own, matching the spec's "confidence increases with repetition."
- `expertise_pending` is a separate column from `metadata` specifically so a Slack/Google roster resync (which overwrites `metadata` wholesale) doesn't wipe learning progress.
- Hooked into the existing task-completion handler in `commands/tasks.rs` (same place that already recorded `task_completion` pattern observations), splitting `task.assignee` on commas to handle multi-assignee tasks.

### Export / Import

- `src-tauri/src/sync/export.rs` — ZIP archive built in memory (projects, tasks, meetings, team members, pattern contributions, Qdrant vector snapshots), hashed for integrity, optionally encrypted as a whole
- `src-tauri/src/sync/import.rs` — Archive parsing with conflict resolution, transactional apply
- `src-tauri/src/sync/manifest.rs` — Version tracking, content inventory, `compute_checksum()` helper
- `src-tauri/src/sync/crypto.rs` — Real AES-256-GCM encryption (PBKDF2-SHA256 key derivation, 100k iterations, same pattern as `crypto/key.rs`'s SQLCipher key derivation). Encrypts the *finished* zip as a byte stream (`MRX1` magic + salt + nonce + ciphertext) rather than per-entry, since the `zip` crate (0.6) has no write-side AES support. If no password is given the file is a plain, unencrypted zip — `crypto::is_encrypted()` detects which on import.
- `src-tauri/src/commands/sync.rs` — 5 Tauri commands (incl. `pick_export_save_path`/`pick_import_file_path` native dialogs). `export_single_skill`/`import_single_skill` were removed (previously 7) — they had zero call sites anywhere in the UI and the import side never actually inserted the parsed skill into the DB; the skills UI already has a separate, working `export_skill_to_directory`/`import_skill` path (`commands/skills.rs`), so the dead pair was deleted rather than fixed.
- `src/components/sync/ExportDialog.tsx` — Content selection (honest: skills/document-metadata checkboxes are disabled with "coming soon", since export.rs never touches those regardless of what's checked), native save dialog, real progress bar
- `src/components/sync/ImportDialog.tsx` — Conflict resolution UI, native open dialog, real progress bar, "Restore Backup" tab

**`conn`/`Send` split (important if touching export.rs or import.rs):** `export_data`/`import_data` need to do async work (Qdrant snapshot calls) partway through, but `rusqlite::Connection` isn't `Sync` — a `#[tauri::command]`'s future must be `Send`, and passing `conn`/a `MutexGuard<Connection>` into *anything* that returns a future spanning an `.await`, even one that only touches it before its own first await, makes the **caller's** future non-`Send` too (the borrow must stay valid for the callee future's whole lifetime as far as the borrow checker is concerned). Fix: `build_local_entries`/`apply_local_import` are plain sync functions that do all `conn` work and return owned data; `finish_export`/`finish_import` are async and take no `conn`. `commands/sync.rs` calls the sync half inside the `state.db.lock()` block, drops the lock, then awaits the async half. `export_data`/`import_data` remain as convenience wrappers combining both for tests, which aren't subject to this constraint. Progress callbacks (`ProgressFn`) also need an explicit `+ Sync` bound for the same reason.

**Export/Import integrity & safety (previously non-functional, fixed):**
- `checksum.sha256` is written into the archive (hash over the fixed-order data files + manifest) and verified on import when present; mismatch aborts the import with an error.
- Meetings are now imported, not just exported — `import_meeting()` mirrors `import_project`/`import_task`. `import_task`'s existence check was also fixed (it previously called a `get_task()` that errors on "not found" instead of returning `Option`, which made importing brand-new tasks fail).
- `ImportMode::Replace` is now actually applied: it wipes locally-present content types (in FK-safe order: tasks → meetings → projects, plus team_members) before inserting, instead of silently behaving like Merge.
- The whole SQL portion of import runs inside `conn.unchecked_transaction()` — commits only if every item succeeds, otherwise rolls back automatically. Previously each row was written with no atomicity.
- Pre-import backup reuses the existing `utils::backup::backup_database()` (the same mechanism used before schema migrations) rather than a new system — path returned in `ImportResult.backup_path` and shown in the Import dialog's completion screen. Restoring any backup is available directly from the Import dialog's "Restore Backup" tab (`commands::migration::{list_backups, restore_from_backup}`).
- `pick_export_save_path`/`pick_import_file_path` (commands/sync.rs) are real native dialogs (osascript on macOS, `rfd` elsewhere) — the export path picker used to just hardcode `~/Downloads/...` and the import file picker did nothing at all.
- Export/import progress is real: backend emits `export_progress`/`import_progress` events (`tauri::Emitter`), frontend listens via `onExportProgress`/`onImportProgress` (`lib/tauri.ts`) and renders an actual percentage bar, not just a spinner.
- Test coverage: `sync/import.rs` has round-trip tests (encrypted, unencrypted, wrong password, Replace vs Merge, conflict detection, checksum-tamper detection, Qdrant-unavailable graceful degradation, shared-patterns round-trip); `sync/crypto.rs` has its own encrypt/decrypt/tamper tests.
- Conflict rows in `ImportDialog.tsx` show local-vs-import "updated at" timestamps (`ImportConflict.local_updated`/`import_updated`), not just names, matching the data-import spec's diff-preview requirement.
- `ConflictResolution::Ask` reaching the data layer (`import_project`/`import_task`/`import_meeting`/`import_team_member` in `sync/import.rs`) now returns an explicit error instead of silently behaving like `Skip` — it's a latent-bug guard, not a real code path: the UI always resolves every conflict to `skip`/`overwrite` before calling `import_all_data`, so this should never actually trigger.

**Vector embeddings in export/import:**
- `vectors/qdrant.rs::export_snapshot()`/`import_snapshot()` use Qdrant's native snapshot API — `create_snapshot()` via the gRPC client, then download/upload via raw `reqwest` calls to Qdrant's REST port (gRPC port − 1, e.g. 6334 → 6333; the gRPC client's own `download_snapshot()` needs an extra crate feature we don't otherwise need, and has no upload/recover equivalent at all).
- Export snapshots every existing Qdrant collection (not scoped by `project_ids`) into `vectors/qdrant_snapshot/{collection}.snapshot`. If Qdrant isn't running, export proceeds without vectors rather than failing — `contents.vectors` reflects what actually got included.
- Snapshot files are binary blobs and are **not** part of the sha256 checksum — the zip format's own per-entry CRC32 covers them.

### Shared Patterns (`pattern_contributions` table, previously schema-only)

- Opt-in via "Contribute to team patterns" toggle in `LearningSettings.tsx` (`app_settings.pattern_contribution_enabled`). When on, every `patterns::repository::insert_observation()` call also anonymizes and contributes: only `task_keywords`/`new_priority`/`old_priority`/`new_status` are ever kept (`SAFE_CONTEXT_KEYS`) — task titles, names, project IDs, and entity IDs are dropped entirely, never hashed-but-kept. Content that trips `sensitive::scan_content()` is never contributed. The anonymized JSON is SHA-256 hashed and stored (`pattern_contributions.observation_hash`) — dedup via `UNIQUE(pattern_type, observation_hash)`.
- **The schema only stores hashes, not content** — by original design (see `design.md` Decision 5). This means team-scope `pattern_models` rows can't be reconstructed with real keyword/assignee content; `upsert_team_pattern_model()` stores only a `{"contribution_count": N}` summary. Team patterns are a validation/count signal ("N teammates share evidence of a pattern in this category"), not a content-transfer mechanism — don't build logic that assumes otherwise without changing the export schema first.
- Export includes `pattern_contributions` when `include_patterns` is checked (`sync/export.rs`); import merges via `merge_team_contributions()`, incrementing `contributor_count` only for hashes not already known locally (re-importing the same export, or two teammates independently observing the same anonymized pattern, doesn't inflate the count).
- **Regression risk if touching `pattern_models`:** the table's `UNIQUE(pattern_type, project_id)` constraint predates the `scope` column and doesn't include it. A team-scope row with `project_id = NULL` would collide with a personal global row of the same `pattern_type`. Team rows use the sentinel `project_id = "__team__"` (`TEAM_SCOPE_PROJECT_ID` in `patterns/repository.rs`) to stay distinct without a migration. Covered by `test_personal_and_team_scope_dont_collide_on_same_pattern_type`.
- "Use team patterns" toggle (`app_settings.use_team_patterns`) currently only controls whether the Team Patterns section renders in `LearningSettings.tsx` — it does **not** filter team-scope rows out of any suggestion query, because (per the point above) team rows carry no exploitable content for today's consumers anyway. Wiring it into query-time filtering would be a no-op until team `model_data` carries more than a count.
- Key files: `patterns/repository.rs` (`maybe_contribute`, `anonymize_context_data`, `merge_team_contributions`), `commands/patterns.rs` (5 new commands), `components/patterns/LearningSettings.tsx`.

### Database (Phase 7)

- `team_members` (v016) — id, name, email, avatar_url, source, source_id, role, expertise (JSON), workload_score
- `team_members.expertise_pending` (v017) — JSON keyword→count map for expertise auto-learning, kept separate from `metadata` so roster resyncs don't erase it
- `pattern_contributions` (v016) — hash-only contribution log, see Shared Patterns above
- `pattern_models.scope`/`contributor_count` (v016) — now actually read/written (previously dead columns; `PatternModel` didn't even expose them as Rust fields)

**Spec location:** Phase 7's specs (`team-roster`, `assignee-intelligence`, `data-export`, `data-import`, `shared-patterns`) have been archived and merged into canonical `openspec/specs/` — the change proposal itself now lives at `openspec/changes/archive/2026-07-30-phase-7-team-sync/`. Check the canonical spec files for current requirements, not the archived change folder.

### Team & Sync: Known Gaps / Future Work

> These need a product decision or external dependency before they can be implemented — don't guess at them.

| Item | Status | What's Missing |
|------|--------|-----------------|
| **Google Workspace domain admin approval** | EXTERNAL DEPENDENCY | The `google.rs` provider, OAuth wizard step, `sync_team_from_google` command, and `GoogleSettings.tsx` UI are all implemented — but `admin.directory.user.readonly` is a restricted scope only a Workspace domain admin can approve. Without real `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET` credentials and that approval (see CREDENTIALS_SETUP.md Part 5b), connecting will fail at Google's consent step — that's expected, not a bug. Personal Gmail accounts can never use this (no Workspace domain to query). |
| **Skill sharing team features** | SUPERSEDED | `proposal.md` for this phase lists `skill-sharing` as a modified capability (team visibility, clone tracking via `cloned_from_id`), but the current authoritative spec at `openspec/specs/skill-sharing/spec.md` has since explicitly REMOVED those requirements ("Meridian is a local-first single-user app... no multi-user sharing functionality exists"). Nothing to fix here — the proposal is stale, not the code. |
| **Skills/document-metadata/audit-log export** | NOT IMPL | `ExportOptions.include_skills`/`include_documents` exist but export.rs never reads them (there's no `include_audit` field at all — audit log export isn't in `ExportOptions` in any form). All three are disabled in `ExportDialog.tsx` with a "coming soon" label rather than silently doing nothing. |
| **AssigneePicker in inline/filter editors** | NOT IMPL (by choice) | AssigneePicker is mounted in `TaskEditModal` only, per product decision — `TaskInlineEditor` and `TaskFilters` still use the plain `AssigneeChipInput`. Expand deliberately if those surfaces need suggestions too, not by default. |
| **Expertise auto-learning threshold policy** | Implemented with a fixed default | `EXPERTISE_PROMOTION_THRESHOLD = 3` (`team/repository.rs`) — a keyword is promoted after 3 completions. If this needs to be configurable per-user, that's new scope, not a bug. |

---

## Integration Visibility (Phase 8)

Phase 8 makes integration data (GitHub, Jira, Slack) accessible to users via the My Activity dashboard and to AI chat.

**My Activity Dashboard:**
- Sidebar entry "My Activity" with badge showing critical+warning count
- Shows pre-computed attention items grouped by severity (Critical, Needs Attention, Info)
- Items come from: overdue tasks, stale tasks, pending approvals, integration cache matches
- Filters by source type (task/approval/github/jira/slack) and severity

**Attention Items:**
- Pre-computed in `attention_items` table, refreshed every 5 minutes by daemon
- Upsert pattern: `UNIQUE(source_type, source_id, category)` prevents duplicates
- Dismissable per-item with `dismissed_at` timestamp

**Integration Project Mapping:**
- `integration_project_mapping` table maps external repos/projects to Meridian projects
- Required because integrations are account-level but users need project-scoped views
- Key: `(integration_id, external_key)` → `project_id`

**Cache Management:**
- `integration_cache` extended with: `attention_score`, `attention_reason`, `evaluated_at`, `archived_at`, `expires_at`
- Retention: 30 days default (`cache_retention_days` setting), archived items 90 days
- `cleanup_integration_cache` daemon job runs daily at 3 AM UTC

**Key files (Backend):**
- `src-tauri/src/attention/` — models, repository for attention items
- `src-tauri/src/commands/attention.rs` — get_attention_items, get_attention_count, dismiss_attention_item
- `src-tauri/src/integrations/mapping.rs` — project mapping CRUD
- `src-tauri/src/daemon/jobs.rs` — `compute_attention_items`, `cleanup_integration_cache` jobs
- `src-tauri/src/db/migrations/v018_integration_visibility.rs` — schema

**Key files (Frontend):**
- `src/hooks/useAttention.ts` — React Query hooks for attention items
- `src/hooks/useIntegrationBrowser.ts` — React Query hooks for integration cache browsing
- `src/components/activity/MyActivityDashboard.tsx` — main dashboard view
- `src/components/activity/AttentionItem.tsx` — single attention item row
- `src/components/activity/AttentionFilters.tsx` — filter dropdown
- `src/components/integrations/IntegrationBrowser.tsx` — project-scoped integration item browser
- `src/components/integrations/IntegrationItemRow.tsx` — expandable integration item row
- `src/components/integrations/IntegrationItemDetail.tsx` — expanded item detail view
- `src/stores/uiStore.ts` — `activeView: "activity" | "integrations"` added

**Integration Browser:**
- Project-level tab showing cached GitHub issues/PRs, Jira issues, Slack threads
- Filter by source (github/jira/slack) and item type (issue/pr/commit/thread/message)
- Text search across cached items
- Expandable rows showing title, description, labels, state, files changed

**AI Chat Integration Context:**
- `chat_with_project` includes integration cache data in AI context
- `build_project_context_with_integrations()` in `extractor.rs` appends up to 20 integration items
- Items formatted with type label, title, URL, and description preview

**MCP Tools (Phase 8):**
- `list_integration_items` — List cached items with optional project/type/limit filters
- `search_integration_items` — Text search across integration cache
- `get_attention_items` — Get items needing attention by severity/source

**App Settings:**
- `cache_retention_days`: 30 (auto-archive cache items older than this)
- `attention_refresh_minutes`: 5 (daemon refresh interval)
- `ai_integration_context_tokens`: 4000 (token budget for AI chat, future use)

### Phase 8: Known Gaps / Future Work

| Item | Status | What's Missing |
|------|--------|-----------------|
| **Filter Skills** | NOT IMPL | Skills with `action: filter` to match commits against user-defined criteria |
| ~~E2E Tests~~ | DONE | `tests/e2e/integration-visibility.spec.ts` covers My Activity Dashboard (5 tests), Integration Browser (6), and the sidebar attention badge (1). This row was stale. |

---

## Message Center, Role & Productivity (Phase 10)

### Dual Retention Model

Two independent windows govern the same rows in `message_center`:

```
                    message_center row created
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
     AI CONTEXT WINDOW                 UI PERSISTENCE
  user_profile.ai_context_days    user_profile.message_retention
       (7 / 30 / 90, def 30)         (90d / 1y / forever, def forever)
              │                               │
   filters what the LLM sees       filters what the user browses
              │                               │
   get_messages_for_ai_context()       get_messages() + cleanup job
```

A message can be invisible to the AI while still fully browsable and searchable in the UI. This is the point of the design: agent context stays bounded and cheap, while the user's history is not silently discarded.

### AI Context Data Flow

```
chat_with_project (commands/ai.rs)
  ├─ SELECT ai_context_days FROM user_profile
  ├─ messages::get_messages_for_ai_context(conn, project_id, days)
  │     WHERE deleted_at IS NULL
  │       AND created_at > now - days
  │       AND (ai_visible_until IS NULL OR ai_visible_until > now)
  │     ORDER BY created_at DESC LIMIT 50
  └─ extractor::build_project_context_full(..., &center_messages)
        └─ "=== MESSAGE CENTER ===" (max 20 entries, content truncated to 400 bytes
            on a UTF-8 char boundary)
```

`build_project_context` / `build_project_context_with_integrations` remain as delegating wrappers passing an empty message slice, so pre-Phase-10 callers are unchanged.

### Message Producers

Only these create `message_center` rows:

| Producer | Type | Notes |
|---|---|---|
| `skills::executor::complete_skill_run` | `skill_result` | Text output; `file_refs: None` |
| `commands::skills::execute_skill_sandboxed` | `skill_result` | Auto-pinned, carries `file_refs` |
| `daemon::jobs` integration sync | `integration_sync` | Only when `items_synced > 0` |
| `commands::messages::pin_message` | `pinned_chat` | User-initiated from AI chat |
| `daemon::jobs::generate_digest_job` | `digest` | Daily at 6 AM UTC; skipped when the window is empty |

### Notifications Are the Other Half of Routing

`RoutingDecision::MessageCenterWithNotification` requires **both** a `message_center` row and a notification pointing at it through `notifications.message_id` (v022). `create_notification_for_message()` writes the link; `create_notification_full()` does not. `should_create_message()` covers only the message half, so creating a message without also creating the linked notification leaves the result undiscoverable.

Producers that emit both halves: `complete_skill_run`, the integration-sync job, and `generate_digest_job`. The frontend renders a "View full result" button on any notification carrying a `message_id`; it calls `uiStore.openMessageCenter(messageId)`, which switches to the Message Center view and sets `focusedMessageId` so `MessageCenterView` scrolls to and ring-highlights the target for 3 seconds.

**File references:** `file_refs` holds paths **relative to `~/.meridian/created_files/`** so they survive restore onto a machine with a different home dir. `skills::sync::get_created_files_dir()` is the single owner of that directory layout; `messages::storage` resolves and measures paths but never creates them.

### Role Inference

```
task create/update (commands/tasks.rs)
  → pattern_observations.role_signal   ('creates_tasks_for_others' | 'receives_assignments' | ...)
        │
        ▼
infer_role daemon job (daily)
  ├─ compute_role_scores()   weighted 8×4 signal→role matrix, normalized to 1.0
  ├─ classify_role()         primary + secondary (secondary shown when > 0.3)
  └─ check_role_drift()
        ├─ recent  = observations in [14 days ago, now)
        ├─ prior   = observations in [42 days ago, 14 days ago)
        └─ alert when max score delta > 0.2 AND primary role changed
              ├─ notifications row (type = 'role_drift')
              └─ frontend polls get_role_drift_alert
```

Drift compares two *behavioural windows* rather than successive stored snapshots — inference runs daily, so snapshot-to-snapshot deltas could never satisfy the 14-day requirement.

### Role-Based My Activity Ordering

`get_attention_items` (commands/attention.rs) reorders results through `role::ordering::order_activity_items` when the profile has a **confirmed** role.

The Phase 10 design named four sort flags — `is_team_item`, `is_assigned_to_me`, `is_review_request`, `is_blocker` — but `attention_items` stores none of them, and the phase never defined a current-user identity. v022 adds `user_profile.display_name` / `user_email` / `user_aliases`, and the flags are derived:

| Flag | Derivation |
|---|---|
| `is_assigned_to_me` | `task` item whose `tasks.assignee` (comma-split, normalized) matches an identity token |
| `is_team_item` | `task` item assigned to someone else, plus every `approval` item |
| `is_review_request` | `approval` item — governance approvals are Meridian's only review queue; no GitHub PR-review attention items are produced |
| `is_blocker` | `critical` severity |

Sort key: **severity rank → role rule → `computed_at` DESC**. Severity stays primary because the dashboard groups by severity and a critical item must never sort below a warning.

Ordering is a no-op (falls back to the repository's severity + recency order) when identity is unset, the role is unconfirmed, or the role is `pm`/unrecognised. Confirming or changing a role invalidates `["attention-items"]` in `useRole.ts` so the dashboard reorders immediately, per the spec's "immediate benefit" requirement.

### Suggestion Weighting by Role

`get_pending_suggestions` reorders through `role::weighting::weight_suggestions` when the profile has a confirmed role. Same shape as the ordering rule: **severity first**, role weight second, recency third — weighting decides what matters most among equally urgent suggestions and never promotes an info item above a critical one.

Only two of the spec's four Suggestion Weighting rows map to types this codebase produces:

| Spec row | Producer | Tech Lead / IC / PM / Manager |
|---|---|---|
| Task overdue | `overdue_task` | 1.0 / 1.5 / 1.2 / 1.0 |
| Meeting follow-up | `meeting_followup` | 1.2 / 1.0 / 1.3 / 1.5 |
| PR review needed | none | — |
| Team velocity drop | none | — |

`stale_task` and `workflow_sequence` are produced but absent from the table. Every unmapped type weighs **1.0**, so it keeps its relative order rather than being silently demoted.

### File Archival

`archive_old_files` (v023, **off by default**) compresses `created_files/YYYY-MM-DD/` directories older than `archive_after_days` into `created_files/archive/{date}.zip`. It runs inside the daily `cleanup_messages` job.

This is not a deletion path — files stay readable inside the archive, which is what the spec's "still accessible" requires. Originals are removed only after the zip is finalized, and an existing archive for the same day is never overwritten (the day is skipped and the originals survive). Directories whose names don't parse as `YYYY-MM-DD` — including `archive/` itself — are left untouched.

### Digests

`messages/digest.rs` collects four counts from local tables only — completed, created, overdue, meetings — over a 1-day (`daily`) or 7-day (`weekly`) window, renders markdown, and never calls an AI provider.

Two scheduled jobs share one implementation, with the period passed at dispatch:

| Job | Schedule | Why |
|---|---|---|
| `generate_digest` | Daily, 06:00 UTC | After the 04:00 `cleanup_messages` job, so a fresh digest is never swept by retention on creation |
| `generate_weekly_digest` | Mondays, 07:00 UTC | An hour after the daily job so the two never collide; start-of-week is when a look-back is most useful |

Empty windows are skipped rather than posting a blank digest.

### Productivity Patterns

Completion hour/day/category land in `pattern_observations`; `aggregate_productivity` (every 6h) rolls them into `user_profile.productivity_patterns` JSON. Below `MINIMUM_COMPLETIONS` (50) the system returns research-based defaults rather than learned peaks, so suggestions are never derived from noise.

### Database (v021 migration)

- `message_center` — messages with `ai_visible_until`, `deleted_at`, `file_refs` (JSON, relative to `created_files/`)
- `user_profile` — single `'default'` row: role, scores, retention prefs, productivity JSON
- `pattern_observations` extended with `role_signal`, `completion_hour`, `completion_day_of_week`, `task_category`

### Database (v022 migration)

- `user_profile.display_name` / `user_email` / `user_aliases` (JSON array) — current-user identity for role ordering
- `notifications.message_id` → `message_center(id) ON DELETE SET NULL`, plus a partial index — the deep-link target

### Database (v023 migration)

- `user_profile.archive_old_files` (default 0) / `archive_after_days` (default 90) — opt-in file archival

### Database (v024 migration)

- `user_profile.show_suggestions` (default 1) / `data_retention_days` (default 365) — the two `ProductivitySettings` fields the command accepted but previously dropped for lack of columns. `get_productivity_settings` reads them back.

### MCP Tools (Phase 10)

| Tool | Permission | Behaviour |
|---|---|---|
| `create_report` | `create_report` | Writes a `digest` message + linked notification |
| `get_reports` | read (none) | Returns `source_type = 'mcp'` messages only, so an agent reading back its own reports doesn't get skill/sync traffic |
| `draft_message` | `draft_message` | Parks a `pinned_chat` draft for review; **never sends**, returns `sent: false` |

All three go through `insert_message_with_notification`, mirroring the in-app `MessageCenterWithNotification` rule so MCP-authored content is as discoverable as skill output.

> **Note:** `meridian-mcp` did not compile before this work — `bulk_create_tasks` was built against an older `CreateTaskInput` (an `i32` priority plus `source`/`source_id` fields that no longer exist). Fixed to match `tool_create_task`. The crate is not covered by `cargo test` at the workspace root, which is how it stayed broken.

### Two Settings Commands (easy to confuse)

- `update_productivity_settings` — takes a `ProductivitySettings` **struct** (`{tracking_enabled, show_suggestions, data_retention_days}`)
- `update_retention_settings` — takes **flat optional** `ai_context_days` / `message_retention` / `productivity_tracking_enabled`

Role inference signals are recorded into `pattern_observations.role_signal` from task create/update (`commands/tasks.rs`). The daily `infer_role` job recomputes scores and checks drift. Because the daemon worker has no `AppHandle` and cannot emit Tauri events, the frontend **polls** `get_role_drift_alert`; the daemon additionally raises a `role_drift` notification.

**Key files:** `src-tauri/src/messages/`, `src-tauri/src/role/`, `src-tauri/src/productivity/`, `src/components/messages/`, `src/components/role/`, `src/components/productivity/`, `src/hooks/{useMessages,useRole,useProductivity}.ts`, migration `v021_message_center_role.rs`.

### Phase 10: Known Gaps / Future Work

| Item | Status | What's Missing |
|------|--------|-----------------|
| **Spec rows with no producer** | BLOCKED | Two Suggestion Weighting rows (`PR review needed`, `Team velocity drop`) and the Tech Lead ordering rule reference item types **no job creates**. Both weigh/sort as neutral. Unblocking needs Phase 8's Filter Skills, not a Phase 10 change. |
| **`data_retention_days` enforcement** | STORED, NOT ENFORCED | v024 persists the value and the UI round-trips it, but no job prunes `pattern_observations` by it. The column is honest storage; the pruning job is unwritten. |

### Frontend/Backend Contract Enforcement

Two independent test suites guard the Tauri boundary, because neither compiler sees across it:

| Test | Guards | Catches |
|---|---|---|
| `tests/command_contract.rs` | Command **names** | `invoke("name")` with no matching `generate_handler!` entry — runtime "command not found", across all 256 call sites |
| `tests/serialization_contract.rs` | Wire **shapes** | Enum tagging drift and field renames on types crossing the boundary |

**Still unguarded (verify by hand):** command *argument* names and struct field alignment. When adding a command, read the Rust signature directly — `state` is not a JS argument, `Option<T>` params may be omitted, and a struct param must be nested (`{ settings: {...} }`) rather than flattened.

**Enum tagging is the sharpest edge.** Serde's default external tagging emits `{"Learning":{...}}` and bare `"Ready"`; TypeScript consumers here model discriminated unions (`{"type":"Learning"}`). Any enum returned from a command needs `#[serde(tag = "type")]`. Build E2E mocks from the Rust type, never from the TS interface — a mock derived from the TS side makes tests pass against a shape the backend never emits.

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
