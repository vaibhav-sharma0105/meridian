# CLAUDE.md — Meridian Agent Context

> **Mandate for every agent:** After completing any change — feature, fix, refactor, or test — update this file and `docs/ARCHITECTURE.md` to reflect what changed. Stale documentation is worse than no documentation.

---

## What Is Meridian

Meridian is a **local-first, AI-powered meeting intelligence desktop app** built with Tauri v2. It ingests meeting transcripts (pasted text, Zoom, or Google Sheets Relay from Gmail automation), extracts structured tasks using AI, and lets users manage those tasks across projects with List/Kanban/Table views, inline editing, and an AI chat panel.

**Data lives entirely on the user's machine** — `~/.meridian/meridian.db` (SQLite). No backend server. The only outbound network calls are to the user's configured AI provider (OpenAI/Anthropic/Ollama/LiteLLM).

---

## Tech Stack — Exact Versions

| Layer | Technology | Version |
|---|---|---|
| Desktop shell | Tauri | v2.x |
| Frontend | React + TypeScript | 18.x / 5.x |
| Build | Vite | 5.x |
| Styling | Tailwind CSS | v3 |
| State | Zustand | 4.x |
| Async data | @tanstack/react-query | v5 |
| Drag & drop | @dnd-kit | 6.x |
| Backend | Rust (stable) | 1.77+ |
| Database | SQLite via rusqlite + SQLCipher | Encrypted at rest |
| Vector storage | Qdrant (client) | For semantic search |
| Encryption | ring crate (PBKDF2) | Key derivation |
| Secrets | keyring crate (OS keychain) | — |
| HTTP client | reqwest (async) | — |
| Testing | Vitest (unit) + Playwright (E2E) | — |

---

## Repository Structure

```
meridian/
├── src/                          # React + TypeScript frontend
│   ├── App.tsx                   # Root: onboarding gate → AppShell
│   ├── components/
│   │   ├── layout/               # AppShell, Sidebar, MainCanvas, ContextPanel
│   │   ├── tasks/                # TaskCard, TaskListView, TaskKanbanView, TaskTableView, TaskFilters
│   │   ├── meetings/             # MeetingCard, MeetingIngest, MeetingHealthBadge
│   │   ├── ai/                   # AIChatPanel, AISettings, OutputTemplates
│   │   ├── connections/          # ConnectionsSettings (Zoom + Sheets Relay UI)
│   │   ├── documents/            # DocFolder (file upload + AI query)
│   │   ├── analytics/            # ProjectDashboard (velocity, workload charts)
│   │   ├── onboarding/           # OnboardingWizard + steps
│   │   ├── projects/             # ProjectCreate, ProjectSettings
│   │   ├── notifications/        # NotificationCenter
│   │   └── shared/               # EmptyState, UpdateBanner
│   ├── hooks/                    # useTasks, useMeetings, useSync, useAI, ...
│   ├── stores/                   # Zustand: uiStore, taskStore, projectStore, ...
│   ├── lib/
│   │   └── tauri.ts              # ★ THE ENTIRE FRONTEND API CONTRACT ★
│   └── styles/globals.css        # Design tokens, CSS vars, scrollbar, animations
│
├── src-tauri/src/                # Rust backend
│   ├── lib.rs                    # ★ ALL TAURI COMMANDS MUST BE REGISTERED HERE ★
│   ├── commands/                 # One file per domain (tasks, meetings, ai, audit, ...)
│   ├── db/
│   │   ├── repositories/         # All SQL lives here (never in commands/)
│   │   └── migrations/           # Versioned schema files (v001–v007+)
│   ├── models/                   # Rust structs with serde (match TS interfaces)
│   ├── connectors/               # zoom.rs, sheets_relay.rs, sync.rs
│   ├── crypto/                   # Encryption key derivation (PBKDF2, device-key modes)
│   ├── audit/                    # Audit logging (action tracking, risk classification)
│   ├── vectors/                  # Qdrant vector storage client
│   ├── documents/                # Document parsers (XLSX, PDF, etc.)
│   └── ai/                       # litellm.rs, extractor.rs, embeddings.rs, chunking.rs, search.rs
│
├── tests/e2e/                    # Playwright tests
│   ├── fixtures.ts               # mockedPage fixture (injects Tauri mock)
│   └── setup/tauri-mock.ts       # window.__TAURI_INTERNALS__ mock + fixture data
│
├── CLAUDE.md                     # ← You are here
├── AGENTS.md                     # Model-agnostic pointer for any AI agent
├── docs/ARCHITECTURE.md          # Deep architecture: data flow, decisions
├── README.md                     # Full setup + user guide
└── CREDENTIALS_SETUP.md          # Zoom + Gmail OAuth credential creation
```

---

## Critical Conventions — Read Before Every Change

### 1. The Tauri Command Pipeline (most common source of bugs)

Every new backend feature follows this exact chain — missing any step silently breaks things:

```
1. Write Rust function in src-tauri/src/commands/<domain>.rs
   └── Must be: pub async fn, #[tauri::command], return Result<T, String>

2. Register in src-tauri/src/lib.rs inside .invoke_handler(tauri::generate_handler![...])
   └── FORGETTING THIS = "command not found" error at runtime, no compile warning

3. Add TypeScript wrapper in src/lib/tauri.ts
   └── Pattern: export const myCommand = (arg: Type) => invoke<ReturnType>("my_command", { arg });

4. Use from a hook or component via the tauri.ts export
   └── Never call invoke() directly in components — always go through tauri.ts
```

### 2. Client-Side Filter Fields

`TaskFilters` has fields that are **stripped before hitting the backend** in `useTasks.ts`:

```typescript
// src/hooks/useTasks.ts
const backendFilters = {
  ...effectiveFilters,
  project_id: undefined,   // client-side: applied after fetch
  meeting_ids: undefined,  // client-side: applied after fetch
};
```

When adding a new filter field: if it cannot be handled by the existing Rust SQL, add it to this strip list AND apply it in the `queryFn` after the fetch. If the backend CAN handle it, just add it to `TaskFilters` in `tauri.ts` and add SQL in `tasks.rs`.

### 3. React Query Cache Keys

All queries use this key pattern:
```typescript
["tasks", projectId, effectiveFilters]   // task lists
["meetings", projectId]                  // meeting lists
["projects"]                             // project list
["notifications"]                        // notification list
```

When mutating data, always invalidate or `setQueryData` the correct key:
```typescript
// Instant UI update (no refetch):
qc.setQueryData<Type[]>(["meetings", projectId], old => old?.map(...));

// Eventual consistency (schedules refetch):
qc.invalidateQueries({ queryKey: ["meetings", projectId] });
```

Use `setQueryData` for mutations where the new value is known immediately (rename, status change). Use `invalidateQueries` for complex mutations where the server may return derived data.

### 4. Onboarding Gate

`App.tsx` calls `getAppSettings()` on mount. If `settings["onboarding_complete"] !== "true"`, it shows `OnboardingWizard` instead of `AppShell`. **In Playwright tests**, the Tauri mock must return:
```javascript
get_app_settings: { onboarding_complete: "true", theme: "light", language: "en" }
```
Without this, tests time out waiting for the sidebar that never renders.

### 5. Tauri v2 Mock for Tests

`window.__TAURI_INTERNALS__` in Playwright tests requires **both** `invoke` and `transformCallback`:
```javascript
window.__TAURI_INTERNALS__ = {
  invoke: async (cmd, args) => { ... },
  transformCallback: (callback, once) => { /* returns numeric ID */ ++callbackId },
  convertFileSrc: (path) => path,
  metadata: { currentWindow: { label: 'main' } },
};
```
Missing `transformCallback` → `@tauri-apps/api` event listeners crash → React never mounts → all tests time out.

### 6. Database Migrations

New schema changes go in a new migration file `src-tauri/src/db/migrations/v00N_description.rs`. The migration runner in `db/connection.rs` applies them in order. Never modify existing migration files — always add a new one.

### 7. Database Encryption

SQLCipher encrypts the database at rest. Key derivation modes:
- **Device mode** (default for new installs): Key derived from machine fingerprint (hostname + username + salt). Transparent to users but not portable.
- **Password mode**: User-provided password with PBKDF2 (100k iterations). Portable across machines.

Key config stored in `~/.meridian/key.json`:
```json
{ "mode": "device", "salt": "hex...", "pbkdf2_iterations": 100000 }
```

Backward compatibility: Existing unencrypted databases continue to work. Migration to encrypted DB is optional and requires explicit user action.

### 8. Audit Logging

All CRUD operations on tasks, meetings, and projects are logged to `audit_log` table with:
- `action_type`: create, update, delete, archive, bulk_update, etc.
- `entity_type`: task, meeting, project
- `risk_level`: low, medium, high, critical
- `agent_initiated`: boolean flag for agent vs user actions
- `autonomy_mode`: supervised, semi_autonomous, autonomous

Query via `get_audit_log` command with filters. 2-year retention with automatic pruning.

### 9. Embeddings & Semantic Search

Document embeddings enable semantic search across project documents. Three providers are available:

- **Bundled (default)**: MiniLM-L6-v2 via ONNX Runtime (~86MB model). Works offline, 384-dimensional vectors.
- **Ollama**: Uses local Ollama server with nomic-embed-text or other models. Requires Ollama running.
- **OpenAI**: Uses text-embedding-3-small API. Requires API key and internet.

**Key files:**
- `src-tauri/src/ai/embeddings.rs` — `BundledEmbedder`, `EmbeddingProvider` trait
- `src-tauri/src/ai/chunking.rs` — Text chunking (500 tokens, 50 overlap)
- `src-tauri/src/ai/search.rs` — Hybrid search with RRF fusion
- `src-tauri/src/daemon/` — Background worker for embedding jobs
- `src-tauri/resources/models/all-MiniLM-L6-v2/` — Bundled ONNX model

**Hybrid Search (RRF):**
Combines semantic (Qdrant vectors) and keyword (FTS5) search using Reciprocal Rank Fusion with k=60. Results tagged with match type: "semantic", "keyword", or "both".

**Embedding Worker:**
- In-process background worker polls `daemon_jobs` table for `embed_document` jobs
- Started via `start_embedding_worker` command, runs in separate thread with its own tokio runtime
- Jobs queued automatically on document upload with priority (10=high, 5=normal, 1=migration)
- `IndexingBanner` component shows progress and allows starting worker manually

**Document Parsing:**
- `src-tauri/src/documents/parsers/xlsx.rs` — XLSX via calamine
- `src-tauri/src/documents/parsers/pdf.rs` — PDF via pdf-extract

### 10. Pattern Learning

The system learns from user behavior to provide smarter suggestions. Patterns are stored locally and never leave the device.

**Observation Types:**
- `task_completion` — Recorded when task status changes to "done"
- `priority_set` — Recorded when task priority is changed
- `assignee_set` — Recorded when task assignee is changed
- `draft_edit` — Recorded when user edits an AI-generated draft
- `suggestion_dismissed` — Recorded when user dismisses a workflow suggestion

**Pattern Types:**
- `workflow_sequence` — Learns "after task A, user usually does B" sequences
- `smart_defaults` — Learns keyword → priority and keyword → assignee mappings
- `communication_style` — Learns length preference, formality, common phrases

**Key files:**
- `src-tauri/src/patterns/models.rs` — Pattern observation and model structs
- `src-tauri/src/patterns/repository.rs` — Pattern CRUD operations
- `src-tauri/src/commands/patterns.rs` — Tauri commands for pattern queries
- `src-tauri/src/daemon/jobs.rs` — `aggregate_patterns` job handler
- `src/components/patterns/` — Frontend components for suggestions and settings

**Aggregation:**
- Runs every 15 minutes as daemon job (`aggregate_patterns`)
- Processes unprocessed observations from `pattern_observations` table
- Updates `pattern_models` with confidence scores
- Applies 10% decay to patterns inactive for 30+ days
- Prunes processed observations older than 90 days

**Confidence Thresholds:**
- Workflow suggestions: >= 0.5
- Smart defaults: >= 0.5
- Communication style: >= 0.6

### 11. Proactive Agent

The proactive agent surfaces actionable suggestions based on task/meeting state and user patterns.

**Suggestion Types:**
- `overdue_task` — Tasks past due date by 24+ hours
- `stale_task` — In-progress tasks with no updates for 7+ days
- `meeting_followup` — Meetings 24+ hours old with no linked tasks
- `workflow_sequence` — Next-step suggestions based on learned patterns

**Key files:**
- `src-tauri/src/suggestions/` — Suggestion models and repository
- `src-tauri/src/drafts/` — Draft message models and repository
- `src-tauri/src/sensitive/mod.rs` — Sensitive content detection (PII, credentials, financial)
- `src-tauri/src/daemon/jobs.rs` — `generate_suggestions` job handler
- `src/components/suggestions/` — SuggestionCard, SuggestionsList UI

**Suggestion Limits:**
- Default: 10 suggestions per day
- Job runs every 30 minutes
- Suggestions ordered by severity (critical > warning > info)

**Draft Generation:**
- Uses LiteLLM/OpenAI for draft text
- Adapts to learned communication style (length, formality, phrases)
- Includes "Drafted by Meridian" signature (toggleable)

**Sensitive Content Detection:**
- PII: SSN, phone numbers, email addresses
- Credentials: API keys, passwords, tokens
- Financial: Credit card numbers, bank accounts
- Non-blocking warnings displayed above draft editor

### 12. Skills & Automation

Skills are user-defined automations that run on schedule, event, or manual trigger.

**Skill Types:**
- `schedule`: Cron-based execution (e.g., "every Monday at 9am")
- `event`: Triggered by task_created, task_completed, meeting_imported, etc.
- `manual`: User-initiated via UI button

**Key Files:**
- `src-tauri/src/skills/models.rs` — Skill, SkillRun, TriggerConfig, ActionConfig structs
- `src-tauri/src/skills/repository.rs` — CRUD operations for skills and runs
- `src-tauri/src/skills/cron.rs` — Cron parsing with timezone support
- `src-tauri/src/skills/events.rs` — Event types and filter matching
- `src-tauri/src/skills/executor.rs` — Context building and action execution
- `src-tauri/src/skills/approval.rs` — Approval workflow (approve/reject/expire)
- `src-tauri/src/skills/builtin.rs` — Built-in template loading (include_str! from templates.json)
- `src-tauri/resources/builtin-skills/templates.json` — 5 skill templates (Weekly Summary, Meeting Follow-up, Overdue Alert, Sprint Prep, End of Day Digest)
- `src-tauri/src/commands/skills.rs` — Tauri commands (29 endpoints incl. initialize/reset builtin + folder picker/export)
- `src/components/skills/` — SkillsPage, SkillCard, SkillEditorModal, SkillHistoryPanel, ChatToSkillPreview, SkillFoldersPanel
- `src/hooks/useSkills.ts` — React Query hooks for skills
- `tests/e2e/skills.spec.ts` — 24 Playwright E2E tests

**Built-in Skills:**
Templates are embedded at compile time via `include_str!()` and auto-loaded on first app launch (gated by `app_settings.builtin_skills_initialized = "true"`). The "Reset defaults" button in SkillsPage calls `reset_builtin_skills` which clears the flag and re-seeds.

**Chat-to-Skill (AI Extraction):**
The `extract_skill_from_chat` command uses LiteLLM to parse natural language into a skill definition. In AIChatPanel, the Wand2 icon on assistant messages opens `ChatToSkillPreview`, which shows an editable preview. On confirm, it sets `uiStore.skillEditorData` and navigates to the Skills view where the editor auto-opens pre-filled.

**Skill Selection in AI Chat (`/skill` command):**
- Type `/skill` in the chat input to open the SkillPicker popup
- Popup shows enabled skills in a scrollable list (5 visible at a time)
- Search bar at bottom filters skills by name/description
- Clicking a skill adds it as a badge above the input
- The skill's context (name + description) is prepended to the AI message
- User can send with just the skill selected (no additional text required)
- Key files: `src/components/ai/SkillPicker.tsx`, `AIChatPanel.tsx`

**Dynamic Skill Invocation (LLM-driven):**
- Both DB skills AND folder packages are merged into `UnifiedSkill` with `originalSkill`/`originalFolder` references
- Compact context format sent to LLM: `📦 **name** - description` (not verbose YAML)
- LLM receives clear instructions: when to invoke (explicit request, direct match) vs when not to (simple questions, conversation)
- When skill matches intent, LLM outputs `**[SKILL_INVOKE: skill_name]**` at response start
- Frontend parses marker and executes via `executeSkill()` which handles both types
- Subtle UI: only shows small green checkmark + skill name after completion (no running/failed states shown)
- Both manual (`/skill` picker) and automatic (LLM-detected) invocation supported
- SkillPicker shows both DB skills (⚡) and folder packages (📦) in unified list

**Progressive Skill Loading:**
- Phase 1: LLM gets lightweight context (name + description only)
- Phase 2: When skill invoked, `loadSkillContent()` fetches full skill.md content
- Phase 3: For folder packages, find and execute main script (main.py, run.sh, index.js)
- Content cached in `loadedSkillContent` ref per conversation
- Clear cache on `clearMessages()` for new conversations

**Skill Execution Deduplication:**
- `invokedSkills` Set tracks invoked skills per conversation
- `processedMsgIndices` ref prevents race condition re-executions
- Only last assistant message processed (not full array iteration)
- One skill per response enforced in LLM prompt
- Key files: `src/hooks/useAI.ts` (UnifiedSkill, executeSkill, loadSkillContent), `src-tauri/src/commands/ai.rs` (skillContext), `AIChatPanel.tsx` (execution with dedup), `SkillPicker.tsx` (unified picker)

**Editor Enhancements:**
- System prompt textarea with `{{variable}}` insertion helper (6 variables: tasks, meetings, project_name, date, overdue_count, completed_today)
- Test Run button (visible when editing existing skill) — previews context without executing
- History panel with status filter dropdown and paginated run list (10 per page)

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

**Sharing [PARKED]:**
- `shared`: Boolean flag stored but non-functional (local-first app has no sharing mechanism)
- Shared skills show "Shared" badge on card (cosmetic only)
- Clone works but `cloned_from_id` not tracked
- `owner_id` column exists but never set

**Skill Run Lifecycle:**
1. Trigger fires (cron/event/manual)
2. Create skill_run record with status=pending
3. Build context (tasks, meetings, documents)
4. Execute action
5. If needs_approval: set status=approval_pending, create notification
6. On approve: apply pending changes, set status=completed
7. On reject: set status=cancelled with reason

### 13. Skill Format (YAML+MD)

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

**Editor:**
Single monospace textarea showing the raw YAML+MD content. Users edit frontmatter (name, trigger, action, settings) and markdown body (# Instructions, etc.) directly. Variable insertion helper available via toolbar button.

**Import/Export:**
- **Export**: Saves as `.md` file (YAML+MD format) via Tauri native save dialog
- **Import**: Accepts `.md`, `.yaml`, `.yml` (YAML+MD), `.json` (v1), `.skill.json` (v2)
- Detection: `isSkillMdFormat()` checks for `---` prefix; falls back to JSON parsing

**Backward compatibility:**
- Old XML-tagged `system_prompt` strings are parsed into markdown sections on edit
- Old freeform strings land in the `# Instructions` section
- Legacy v1/v2 JSON imports still work

### 14. Skill Types & Permissions

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

**Key files:**
- `src-tauri/src/skills/folders.rs` — Filesystem operations (list, install, validate, delete, read, execute)
- `src-tauri/src/commands/skills.rs` — Tauri commands (20 endpoints incl. folder ops, picker, export-to-dir)
- `src/components/skills/SkillFoldersPanel.tsx` — File tree UI, upload, execute dialog
- `src/components/skills/SkillsList.tsx` — "Upload Skill" button (folder picker replaces old Import)
- `src/components/skills/SkillsPage.tsx` — Auto-shows folders panel when packages exist

**Folder upload & validation:**
- "Upload Skill" button uses `pick_folder_dialog` command for native folder picker
- macOS: Uses `osascript -e "choose folder"` (AppleScript) — NSOpenPanel via Tauri/rfd has sheet attachment issues
- Windows/Linux: Uses `rfd::FileDialog::new().pick_folder()`
- Validation: `skill.md` must exist with YAML frontmatter containing `name:` and `description:`

**Skill export (directory-based):**
- `export_skill_to_directory` command: folder picker → creates `{slug}/skill.md`
- Serializes skill to YAML+MD format via `skillToSkillFile()` in `src/lib/skill-format.ts`
- Exported packages can be directly re-uploaded as folder packages

**Script execution:**
- Supported: `.py`, `.js`, `.ts`, `.sh`, `.bash`, `.zsh`, `.rb`, `.pl` (cross-platform)
- Platform-specific: `.ps1`, `.bat`, `.cmd` (Windows only)
- Runs with user permissions in skill folder as working directory
- Path traversal protection: validates path stays within `~/.meridian/skills/<folder>/`

### 15. Skills: Known Gaps / Future Work

> **For agents:** These have schema/UI scaffolding but need implementation. See `openspec/changes/phase-4-skills-automation/tasks.md` Section 30 for full details.

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

## Design System

### Colors
- **Primary accent**: `indigo-500` (#6366f1) — use ONLY for truly interactive/important elements (active state, CTA buttons, selected rings)
- **Background**: `white` / `zinc-900` (canvas), `#111113` (sidebar dark)
- **Borders**: `zinc-100` / `zinc-800` (subtle), `zinc-200` / `zinc-700` (hover)
- **Text hierarchy**: `zinc-900` (titles), `zinc-500` (body/description), `zinc-400` (metadata/labels)
- **Priority borders**: `red-500` critical, `orange-400` high, `yellow-400` medium, `zinc-300` low

### Typography
- Font: `Inter` at `13–13.5px` base, `letter-spacing: -0.01em`
- Title weight: `font-semibold` (600)
- Description: `text-[12px] text-zinc-500 line-clamp-2`
- Metadata: `text-[11px] text-zinc-400` with `·` dot separators

### Component Patterns
- **Cards**: `border-l-[3px]` priority color, subtle border (`zinc-100/zinc-800`), hover → `zinc-50/zinc-800` (NOT transparent — avoid opacity tricks that look disabled)
- **Active filter state**: Replace select with `ActiveChip` component (colored pill with inline `×`)
- **Tabs**: Underline style (`border-b-2 border-indigo-500` on active, `border-transparent` inactive)
- **Popovers/dropdowns**: `absolute top-full mt-1`, `shadow-xl`, `animate-fade-in`, close on outside click via `useEffect` + `mousedown`
- **Custom checkboxes**: `sr-only` native input + styled div, `Check` icon from lucide-react

### Spacing
- Card padding: `px-3 py-2.5`
- Section headers: `px-4`
- Filter bar: `px-4 py-2`
- Gap between metadata items: dot-separated (not gap-based)

---

## Data Flow Summary

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

For writes (create/update/delete):
```
Component calls api.updateTask(input)
  → invoke("update_task", { input })
  → Rust: optimistic mutation in onMutate (React Query)
  → commands/tasks.rs → repositories/tasks.rs
  → qc.setQueryData (immediate) OR qc.invalidateQueries (eventual)
```

---

## Sync Architecture (Zoom + Sheets Relay)

```
useSync() → syncConnections() → invoke("sync_connections")
  → sync.rs: sync_zoom() + sync_sheets_relay()
  → For each meeting/row: upsert_pending_import() [INSERT OR IGNORE]
  → Dedup by: external_meeting_id (Zoom) / source_email_id (Sheets)
  → SyncResult { new_imports, skipped_duplicates, errors }
  → useSync.ts: toast for new imports + duplicates skipped
```

Sheets Relay special handling: JSON blobs in cells are detected by `extract_embedded_json()` in `sheets_relay.rs`. The `source_subject` column always wins as meeting title (strips "Meeting assets for " prefix and " are ready!" suffix).

---

## Testing

### Unit Tests
```bash
npm run test           # Vitest — runs src/**/*.test.ts files
npm run test:rust       # Cargo test — runs src-tauri/src/**/*_test.rs
```

### E2E Tests (Playwright)
```bash
# Terminal 1 — must be running first:
npm run vite:dev       # Vite dev server on localhost:1420

# Terminal 2:
npm run test:e2e       # 39 tests, ~4 seconds
npm run test:e2e:ui    # Interactive Playwright UI
```

E2E tests run in Playwright's Chromium (not the Tauri app) — **zero data pollution to SQLite**. All Tauri calls are mocked. Mock data lives in `tests/e2e/setup/tauri-mock.ts`.

---

## Running the App

```bash
npm run dev            # Full Tauri app (Rust + React, hot reload)
npm run vite:dev       # React only (no Rust, port 1420)
npm run build          # Production binary
```

Credentials for Zoom OAuth must be set as env vars before `npm run dev`:
```bash
export ZOOM_CLIENT_ID=your_id
export ZOOM_CLIENT_SECRET=your_secret
npm run dev
```

---

## Observed Development Preferences

These preferences were captured from actual development sessions and should guide agent behavior:

1. **Ask before acting on ambiguous tasks** — ask 2 questions at a time, wait for answers before proceeding. Never assume.
2. **No speculative abstractions** — don't add helpers, utilities, or patterns "for future use". Solve exactly the problem at hand.
3. **No cosmetic additions** — don't add comments, docstrings, type annotations, or error handling to code you didn't change.
4. **Minimal scope** — a bug fix doesn't need surrounding cleanup. A feature doesn't need extra configurability.
5. **Verify before recommending** — if you reference a function, file, or flag, confirm it exists. Don't recommend stale patterns.
6. **Fix root causes, not symptoms** — identify the actual bug before writing a fix. Don't retry the same failing approach.
7. **Confirm destructive actions** — always ask before deleting files, force-pushing, or modifying shared infrastructure.
8. **UI changes require browser verification** — after any frontend change, check the result in context. Don't claim "done" based on code review alone.
9. **Progressive disclosure in UI** — less critical information should be hidden or de-emphasized. Important information (title, status, priority) must always be visible.
10. **Human attention psychology** — design decisions should direct user attention toward what matters. Accent color (indigo) reserved for truly important/actionable elements only.
11. **indigo accent sparingly** — one clear primary action per screen. Supporting actions use zinc/muted tones.
12. **Hover states must look interactive**, not disabled — avoid transparent overlays; use solid `zinc-50 / zinc-800` backgrounds.
13. **`setQueryData` for instant updates** — after a successful mutation, patch the cache immediately. Don't rely on `invalidateQueries` alone for user-facing updates.

---

## When You Finish a Change

Update the following before marking work complete:

1. **This file (`CLAUDE.md`)** — if you added a new pattern, convention, or gotcha
2. **`docs/ARCHITECTURE.md`** — if data flow, schema, or component structure changed
3. **`tests/e2e/setup/tauri-mock.ts`** — if you added new Tauri commands, add mock responses
4. **`src/lib/tauri.ts`** — the living API contract; keep it the authoritative source
5. **Playwright tests** — add/update tests for new UI flows

---

## Known Gotchas

| Gotcha | Details |
|---|---|
| Missing command registration | New `#[tauri::command]` must be added to `lib.rs` invoke_handler. No compile error — only a runtime "command not found". |
| `height: "50%"` in flex | Don't use inline percentage height in flex children — use `h-1/2 flex-shrink-0` Tailwind classes instead. |
| Onboarding gate in tests | Mock must return `onboarding_complete: "true"` in `get_app_settings` response. |
| Tauri v2 `transformCallback` | The mock for `window.__TAURI_INTERNALS__` MUST include `transformCallback`. Without it, React never mounts. |
| Stale closure in onBlur | Input `onBlur` captures stale state when `onKeyDown` (Escape) triggers unmount. Use a `cancelingRef` guard. |
| `getByText` strict mode | Playwright's `.or()` locator fails if both branches match. Use `.first()` or target one specific element. |
| Client filter fields | `meeting_ids` and `project_id` in `TaskFilters` are client-only — strip them in `useTasks.ts` before the `invoke` call. |
| `INSERT OR IGNORE` dedup | `upsert_pending_import` silently skips duplicates (returns `false`). Track in `SyncResult.skipped_duplicates`. |
| Encrypted DB auto-init | New installs auto-initialize device-mode encryption. Existing unencrypted DBs continue working (backward compatible). |
| Qdrant not embedded | Qdrant runs as external service (localhost:6334). Check `is_available()` before operations. |
| Audit log performance | Always query with filters and pagination. Unfiltered queries on large logs are slow. |
| macOS folder picker | `@tauri-apps/plugin-dialog` `open({ directory: true })` and `rfd::FileDialog::pick_folder()` don't work reliably on macOS (NSOpenPanel sheet issue). Use `osascript -e "choose folder"` via `pick_folder_dialog` command instead. |
