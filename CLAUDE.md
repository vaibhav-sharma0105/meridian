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
│   ├── ai/                       # litellm.rs, extractor.rs, embeddings.rs, chunking.rs, search.rs
│   └── integrations/             # External integrations (GitHub, Jira, Slack)
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

> **For agents:** These have schema/UI scaffolding but need implementation. See `openspec/changes/archive/2026-07-15-phase-4-skills-automation/tasks.md` Section 30 for full details.

| Item | Status | What's Missing |
|------|--------|----------------|
| **Skill Sharing** | PARKED | No multi-user sync (local-first app) |
| **Owner Tracking** | PARKED | `owner_id` column never set |
| **Clone Source** | PARKED | `cloned_from_id` not set in `clone_skill()` |
| **Skills → Suggestions** | NOT IMPL | Skills don't create suggestions |
| **Suggestion → Skill Trigger** | NOT IMPL | Accepting suggestion doesn't trigger skill |
| **Retry Logic** | NOT IMPL | Failed skills stay failed |
| **Timeout Handling** | NOT IMPL | No timeout for long-running skills |

### 16. External Integrations

Phase 5 adds a unified integration framework for connecting external services (GitHub, Jira, Slack).

**Key files (Backend):**
- `src-tauri/src/integrations/mod.rs` — `IntegrationProvider` trait, provider registry
- `src-tauri/src/integrations/models.rs` — `Integration`, `IntegrationConfig`, `IntegrationLink` structs
- `src-tauri/src/integrations/repository.rs` — CRUD for `integrations`, `integration_cache`, `integration_links` tables
- `src-tauri/src/integrations/github.rs` — GitHub OAuth + sync (issues, PRs)
- `src-tauri/src/integrations/jira.rs` — Jira OAuth + sync (issues)
- `src-tauri/src/integrations/slack.rs` — Slack OAuth + channel sync
- `src-tauri/src/integrations/slack_socket.rs` — Slack Socket Mode WebSocket client with auto-reconnect and action item detection
- `src-tauri/src/integrations/google.rs` — Google Workspace OAuth + Directory API member fetch (used for team roster sync only — no generic issues/PRs fetch_data content). Needs Workspace domain admin approval for the directory scope; see §18 Known Gaps and CREDENTIALS_SETUP.md Part 5b.
- `src-tauri/src/integrations/webhook.rs` — Local HTTP server for OAuth callbacks
- `src-tauri/src/commands/integrations.rs` — 18 Tauri commands for integration management
- `src-tauri/src/daemon/jobs.rs` — `sync_integration` and `poll_integration_syncs` job handlers

**Key files (Frontend):**
- `src/hooks/useIntegrations.ts` — React Query hooks for integrations
- `src/hooks/useIntegrationLinks.ts` — Hooks for task ↔ external item links
- `src/stores/integrationStore.ts` — Zustand store for OAuth state
- `src/components/integrations/IntegrationsPage.tsx` — Unified settings page (Native vs MCP sections)
- `src/components/integrations/SetupWizard.tsx` — Step-by-step OAuth setup with toggleable step completion
- `src/components/integrations/GitHubSettings.tsx` — GitHub repo selection and sync config
- `src/components/integrations/JiraSettings.tsx` — Jira project selection
- `src/components/integrations/SlackSettings.tsx` — Channel autonomy config
- `src/components/integrations/SlackDraftsPanel.tsx` — Pending Slack drafts with delayed send queue
- `src/components/integrations/MCPSettings.tsx` — MCP Server permission config
- `src/components/integrations/NotificationSettings.tsx` — Desktop notification preferences
- `src/components/integrations/BackgroundJobsPanel.tsx` — Shows active/recent daemon jobs (syncs, embeddings, skills)
- `src/components/integrations/LinkPicker.tsx` — Search and link external items to tasks
- `src/components/tasks/IntegrationLinkBadge.tsx` — Badge showing linked GitHub/Jira items

**Database (v014 migration):**
- `integrations` — Stores connected services with encrypted OAuth tokens in `config` JSON column
- `integration_cache` — Stores fetched external data (issues, PRs, channels)
- `integration_links` — Bidirectional links between Meridian tasks and external items
- `notifications` extended with `severity`, `desktop`, `integration_id` columns

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

**Integration UI Flow:**
1. User clicks Integrations (Link2 icon) in sidebar → Opens IntegrationsPage modal
2. Page shows two collapsible sections: Native Integrations and MCP Servers
3. Click "Connect" on any integration → Opens SetupWizard
4. SetupWizard shows: Prerequisites, step-by-step instructions, OAuth flow (steps are toggleable for accidental clicks)
5. After OAuth: Integration appears in "Connected" section with Settings button
6. Settings modals: Configure sync interval, repo/project/channel selection, disconnect

**Daemon Sync Jobs:**
- `sync_integration` — Fetches data from integration provider, updates cache, creates notifications
- `poll_integration_syncs` — Runs every 5 minutes, checks all connected integrations for sync due
- `init_integration_jobs()` — Called on daemon startup to schedule initial poll job
- BackgroundJobsPanel shows active/recent jobs with auto-refresh (5-second interval)

**Slack Socket Mode:**
- `slack_socket.rs` — WebSocket client for Slack real-time events
- Auto-reconnect with exponential backoff (1s initial, 300s max)
- Action item detection: mentions, questions, requests, deadlines, follow-ups
- Per-channel autonomy: `auto`, `notify`, `approve_first`, `approve_always`

**MCP Permissions (expanded):**
```typescript
interface McpPermissions {
  read_tasks: boolean;      // default: true
  read_meetings: boolean;   // default: true
  read_projects: boolean;   // default: true
  create_task: boolean;     // default: false
  update_task: boolean;     // default: false
  delete_task: boolean;     // default: false
  create_meeting_note: boolean; // default: false
  run_skill: boolean;       // default: false
  rate_limit_per_minute: number; // default: 100
}
```

### 17. Governance & Autonomy

The governance layer provides unified control over agent actions with risk classification, approval workflows, and undo capabilities.

**Autonomy Modes:**
- `Manual` — All agent actions require explicit user approval
- `Supervised` (default) — Low/medium-risk actions auto-execute; high/critical-risk require approval
- `Autonomous` — Most actions auto-execute; only critical-risk requires approval

**Autonomy Inheritance:**
```
Global Autonomy Mode (app_settings.autonomy_mode)
    ↓ inherited by
Integration Autonomy (integrations.autonomy_mode, nullable)
    ↓ inherited by
Skill Autonomy (skills.autonomy_mode, nullable)
```

**Risk Classification:**
Risk level is calculated from three weighted scores:
- `action_type_weight`: read=1, create=2, update=3, external_send=4, delete=5
- `destination_score`: internal=1, team=2, external=3, executive=4
- `content_score`: normal=1, sensitive=2, pii=3, financial=4

Critical override: Any maximum individual score (delete, executive, financial) → Critical risk

**Key files (Backend):**
- `src-tauri/src/governance/mod.rs` — Module root
- `src-tauri/src/governance/models.rs` — RiskLevel, AutonomyMode, PendingApproval, ActionHistory
- `src-tauri/src/governance/risk.rs` — Risk classification engine with learned adjustments
- `src-tauri/src/governance/autonomy.rs` — AutonomyController with inheritance resolution
- `src-tauri/src/governance/approval.rs` — Approval queue operations (create, approve, reject, timeout)
- `src-tauri/src/governance/undo.rs` — Action history and undo system
- `src-tauri/src/governance/repository.rs` — CRUD for governance tables
- `src-tauri/src/commands/governance.rs` — 21 Tauri commands

**Key files (Frontend):**
- `src/hooks/useGovernance.ts` — React Query hooks for all governance operations
- `src/components/governance/AutonomySettings.tsx` — Global mode selector with inheritance info
- `src/components/governance/ApprovalQueue.tsx` — Pending approvals list with bulk actions
- `src/components/governance/UndoBar.tsx` — Toast-style undo with countdown timer
- `src/components/governance/ActionHistoryPanel.tsx` — Filterable action history with diff view
- `src/components/governance/GovernanceDashboard.tsx` — Metrics, charts, anomaly alerts

**Database (v015 migration):**
- `pending_approvals` — Queue for actions awaiting approval with timeout
- `action_history` — Tracks before/after state for undoable actions
- `governance_metrics` — Daily aggregates for dashboard (risk distribution, approval rates)
- `risk_adjustments` — User-defined risk overrides for contacts/channels
- `audit_log` extended with `risk_level`, `autonomy_mode`, `autonomy_source`, `approval_id`, `undo_action_id`
- `integrations.autonomy_mode` and `skills.autonomy_mode` columns added

**Approval Flow:**
1. Action evaluated by `evaluate_action` → returns `ApprovalDecision`
2. If `requires_approval: true`, action queued in `pending_approvals` with timeout
3. User approves/rejects via UI or action times out (default: 24h, configurable)
4. Approved actions execute and create `action_history` entry
5. Rejected/timed-out actions archived with reason

**Undo System:**
- `capture_action_state()` records before/after JSON snapshots
- `undo_action()` creates reversal action (create → delete, update → restore)
- External actions (Slack, GitHub, Jira) marked `undoable: false`
- UndoBar shows for 10 seconds after reversible agent actions

**Daemon Jobs:**
- `check_approval_timeouts` — Runs every minute, archives expired approvals
- `aggregate_governance_metrics` — Runs daily, computes risk/approval aggregates
- `detect_anomalies` — Runs hourly, flags activity spikes and high rejection rates

### 18. Team & Sync

Phase 7 adds team roster management, intelligent assignee suggestions, and data export/import.

**Team Roster:**
- `team_members` table stores members from multiple sources: `manual`, `slack`, `google` (Google sync is UI/command scaffolding only — no Google integration exists in this codebase yet, see Known Gaps below)
- Each member has `workload_score` (0-1) computed from open task count via `compute_all_workload_scores()` (`team/repository.rs`). This now actually runs: daily as part of `process_poll_team_syncs_job` (`daemon/jobs.rs`), and on-demand via the "Recompute Workloads" button in `TeamSettings.tsx` (`compute_team_workloads` command). Previously the scoring function existed and was unit-tested but nothing ever called it, so `workload_score` stayed NULL forever.
- Sync from Slack via `sync_team_from_slack` command
- Daily sync job via `poll_team_syncs` daemon job

**Assignee Intelligence:**
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

**Expertise auto-learning:**
- `team::repository::record_expertise_observation()` bumps a per-keyword pending count (`team_members.expertise_pending`, v017 migration) each time an assignee completes a task whose title/description keywords don't already match an existing expertise tag.
- A keyword is promoted into the member's visible `expertise` array only after `EXPERTISE_PROMOTION_THRESHOLD` (3) separate completions — a single task never mutates expertise on its own, matching the spec's "confidence increases with repetition."
- `expertise_pending` is a separate column from `metadata` specifically so a Slack/Google roster resync (which overwrites `metadata` wholesale) doesn't wipe learning progress.
- Hooked into the existing task-completion handler in `commands/tasks.rs` (same place that already recorded `task_completion` pattern observations), splitting `task.assignee` on commas to handle multi-assignee tasks.

**Export/Import:**
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
- Export/import progress is real: backend emits `export_progress`/`import_progress` events (`tauri::Emitter`) after each stage, frontend listens via `onExportProgress`/`onImportProgress` (`lib/tauri.ts`) and renders an actual percentage bar, not just a spinner.
- Test coverage: `sync/import.rs` has round-trip tests (encrypted, unencrypted, wrong password, Replace vs Merge, conflict detection, checksum-tamper detection, Qdrant-unavailable graceful degradation, shared-patterns round-trip); `sync/crypto.rs` has its own encrypt/decrypt/tamper tests.
- Conflict rows in `ImportDialog.tsx` show local-vs-import "updated at" timestamps (`ImportConflict.local_updated`/`import_updated`), not just names, matching the data-import spec's diff-preview requirement.
- `ConflictResolution::Ask` reaching the data layer (`import_project`/`import_task`/`import_meeting`/`import_team_member` in `sync/import.rs`) now returns an explicit error instead of silently behaving like `Skip` — it's a latent-bug guard, not a real code path: the UI always resolves every conflict to `skip`/`overwrite` before calling `import_all_data`, so this should never actually trigger.

**Vector embeddings in export/import:**
- `vectors/qdrant.rs::export_snapshot()`/`import_snapshot()` use Qdrant's native snapshot API — `create_snapshot()` via the gRPC client, then download/upload via raw `reqwest` calls to Qdrant's REST port (gRPC port − 1, e.g. 6334 → 6333; the gRPC client's own `download_snapshot()` needs an extra crate feature we don't otherwise need, and has no upload/recover equivalent at all).
- Export snapshots every existing Qdrant collection (not scoped by `project_ids`) into `vectors/qdrant_snapshot/{collection}.snapshot`. If Qdrant isn't running, export proceeds without vectors rather than failing — `contents.vectors` reflects what actually got included.
- Snapshot files are binary blobs and are **not** part of the sha256 checksum — the zip format's own per-entry CRC32 covers them.

**Shared Patterns (`pattern_contributions` table, previously schema-only):**
- Opt-in via "Contribute to team patterns" toggle in `LearningSettings.tsx` (`app_settings.pattern_contribution_enabled`). When on, every `patterns::repository::insert_observation()` call also anonymizes and contributes: only `task_keywords`/`new_priority`/`old_priority`/`new_status` are ever kept (`SAFE_CONTEXT_KEYS`) — task titles, names, project IDs, and entity IDs are dropped entirely, never hashed-but-kept. Content that trips `sensitive::scan_content()` is never contributed. The anonymized JSON is SHA-256 hashed and stored (`pattern_contributions.observation_hash`) — dedup via `UNIQUE(pattern_type, observation_hash)`.
- **The schema only stores hashes, not content** — by original design (see `design.md` Decision 5). This means team-scope `pattern_models` rows can't be reconstructed with real keyword/assignee content; `upsert_team_pattern_model()` stores only a `{"contribution_count": N}` summary. Team patterns are a validation/count signal ("N teammates share evidence of a pattern in this category"), not a content-transfer mechanism — don't build logic that assumes otherwise without changing the export schema first.
- Export includes `pattern_contributions` when `include_patterns` is checked (`sync/export.rs`); import merges via `merge_team_contributions()`, incrementing `contributor_count` only for hashes not already known locally (re-importing the same export, or two teammates independently observing the same anonymized pattern, doesn't inflate the count).
- **Regression risk if touching `pattern_models`:** the table's `UNIQUE(pattern_type, project_id)` constraint predates the `scope` column and doesn't include it. A team-scope row with `project_id = NULL` would collide with a personal global row of the same `pattern_type`. Team rows use the sentinel `project_id = "__team__"` (`TEAM_SCOPE_PROJECT_ID` in `patterns/repository.rs`) to stay distinct without a migration. Covered by `test_personal_and_team_scope_dont_collide_on_same_pattern_type`.
- "Use team patterns" toggle (`app_settings.use_team_patterns`) currently only controls whether the Team Patterns section renders in `LearningSettings.tsx` — it does **not** filter team-scope rows out of any suggestion query, because (per the point above) team rows carry no exploitable content for today's consumers anyway. Wiring it into query-time filtering would be a no-op until team `model_data` carries more than a count.
- Key files: `patterns/repository.rs` (`maybe_contribute`, `anonymize_context_data`, `merge_team_contributions`), `commands/patterns.rs` (5 new commands), `components/patterns/LearningSettings.tsx`.

**Database:**
- `team_members` (v016) — id, name, email, avatar_url, source, source_id, role, expertise (JSON), workload_score
- `team_members.expertise_pending` (v017) — JSON keyword→count map for expertise auto-learning, kept separate from `metadata` so roster resyncs don't erase it
- `pattern_contributions` (v016) — hash-only contribution log, see Shared Patterns above
- `pattern_models.scope`/`contributor_count` (v016) — now actually read/written (previously dead columns; `PatternModel` didn't even expose them as Rust fields)

**Spec location:** Phase 7's specs (`team-roster`, `assignee-intelligence`, `data-export`, `data-import`, `shared-patterns`) have been archived and merged into canonical `openspec/specs/` — the change proposal itself now lives at `openspec/changes/archive/2026-07-30-phase-7-team-sync/`. Check the canonical spec files for current requirements, not the archived change folder.

**Team & Sync: Known Gaps / Future Work**

> **For agents:** these need a product decision or external dependency before they can be implemented — don't guess at them.

| Item | Status | What's Missing |
|------|--------|-----------------|
| **Google Workspace domain admin approval** | EXTERNAL DEPENDENCY | The `google.rs` provider, OAuth wizard step, `sync_team_from_google` command, and `GoogleSettings.tsx` UI are all implemented — but `admin.directory.user.readonly` is a restricted scope only a Workspace domain admin can approve. Without real `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET` credentials and that approval (see CREDENTIALS_SETUP.md Part 5b), connecting will fail at Google's consent step — that's expected, not a bug. Personal Gmail accounts can never use this (no Workspace domain to query). |
| **Skill sharing team features** | SUPERSEDED | `proposal.md` for this phase lists `skill-sharing` as a modified capability (team visibility, clone tracking via `cloned_from_id`), but the current authoritative spec at `openspec/specs/skill-sharing/spec.md` has since explicitly REMOVED those requirements ("Meridian is a local-first single-user app... no multi-user sharing functionality exists"). Nothing to fix here — the proposal is stale, not the code. |
| **Skills/document-metadata/audit-log export** | NOT IMPL | `ExportOptions.include_skills`/`include_documents` exist but export.rs never reads them (there's no `include_audit` field at all — audit log export isn't in `ExportOptions` in any form). All three are disabled in `ExportDialog.tsx` with a "coming soon" label rather than silently doing nothing. |
| **AssigneePicker in inline/filter editors** | NOT IMPL (by choice) | AssigneePicker is mounted in `TaskEditModal` only, per product decision — `TaskInlineEditor` and `TaskFilters` still use the plain `AssigneeChipInput`. Expand deliberately if those surfaces need suggestions too, not by default. |
| **Expertise auto-learning threshold policy** | Implemented with a fixed default | `EXPERTISE_PROMOTION_THRESHOLD = 3` (`team/repository.rs`) — a keyword is promoted after 3 completions. If this needs to be configurable per-user, that's new scope, not a bug. |

### 19. Integration Visibility (Phase 8)

Phase 8 makes integration data (GitHub, Jira, Slack) accessible to users via My Activity dashboard and to AI chat.

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
- `src/components/activity/MyActivityDashboard.tsx` — main dashboard view
- `src/components/activity/AttentionItem.tsx` — single attention item row
- `src/components/activity/AttentionFilters.tsx` — filter dropdown
- `src/stores/uiStore.ts` — `activeView: "activity"` added

**App Settings:**
- `cache_retention_days`: 30 (auto-archive cache items older than this)
- `attention_refresh_minutes`: 5 (daemon refresh interval)
- `ai_integration_context_tokens`: 4000 (token budget for AI chat, future use)

**Phase 8: Known Gaps / Future Work**

| Item | Status | What's Missing |
|------|--------|-----------------|
| **Integration Browser UI** | NOT IMPL | Project-scoped UI to browse cached GitHub/Jira/Slack items with expandable details |
| **AI Chat Integration Context** | NOT IMPL | Relevance-scored integration data injection into AI chat system prompt |
| **Filter Skills** | NOT IMPL | Skills with `action: filter` to match commits against user-defined criteria |
| **MCP Integration Tools** | NOT IMPL | `query_integrations`, `get_my_activity`, `get_linked_items` for meridian-mcp |
| **E2E Tests** | NOT IMPL | Playwright tests for My Activity dashboard |

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
