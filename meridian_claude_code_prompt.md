# MERIDIAN — Complete One-Shot Build Prompt for Claude Code

## PRIME DIRECTIVE

You are building **Meridian** — a local-first, AI-powered meeting intelligence desktop application for top 1% operators. Your job is to build the **entire application end-to-end**, including every file, every screen, every database migration, every AI integration, every edge case handler, and every documentation file listed in this prompt. Do not ask the user any questions. Do not skip any section. Do not leave placeholder TODOs unless explicitly marked as Phase 2. Every feature described here must be fully implemented and working.

When you finish, a user must be able to:
1. Install and run Meridian on Windows (also cross-platform: macOS, Linux)
2. Configure their AI provider from the UI
3. Paste a meeting transcript and receive structured, tagged, assigned tasks in under 60 seconds
4. Manage those tasks across projects with full filtering, views, and inline editing
5. Upload documents to project folders and query them with AI
6. Generate 5 types of structured AI outputs from a scoped chat panel
7. Export their data in JSON, CSV, or Markdown
8. Update the app without losing any data

---

## 1. APP IDENTITY

- **Name:** Meridian
- **Tagline:** "Every minute before and after a meeting should be worth more than the meeting itself."
- **Window title:** Meridian
- **Config/data folder:** `~/.meridian/` (Windows: `%APPDATA%\meridian\`)
- **DB file:** `~/.meridian/meridian.db`
- **Backup folder:** `~/.meridian/backups/`
- **Logs folder:** `~/.meridian/logs/`
- **Bundle identifier:** `com.meridian.app`
- **Version:** `0.1.0`

---

## 2. TECH STACK (EXACT)

### Desktop Shell
- **Tauri v2** (Rust backend + React frontend)
- Target: Windows (primary), macOS, Linux
- Single binary output per platform

### Frontend
- **React 18** + **TypeScript 5**
- **Vite** as bundler
- **Tailwind CSS v3** for styling
- **Zustand** for global state management
- **React Router v6** for in-app navigation
- **@tanstack/react-query** for async data fetching and caching
- **@dnd-kit** for drag-and-drop
- **cmdk** for ⌘K / Ctrl+K command palette
- **react-i18next** + **i18next** for multi-language support
- **react-hot-toast** for notifications/toasts
- **lucide-react** for icons (no emoji as icons)
- **date-fns** for date handling
- **zod** for runtime validation
- **recharts** for analytics charts

### Backend (Rust / Tauri commands)
- **rusqlite** with FTS5 for SQLite
- **keyring** crate for OS-level keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- **reqwest** for HTTP calls (LiteLLM, platform APIs, URL fetching)
- **serde / serde_json** for serialization
- **tokio** for async runtime
- **tauri-plugin-notification** for desktop push notifications
- **tauri-plugin-updater** for in-app update checking
- **tauri-plugin-fs** for file system access
- **tauri-plugin-dialog** for file picker dialogs
- **uuid** crate for UUID generation
- **chrono** for timestamps
- **pdf-extract** for PDF text extraction
- **docx-rs** for DOCX parsing
- **calamine** for CSV/Excel parsing
- **scraper** for URL/HTML parsing

### AI
- **LiteLLM** as unified AI gateway (called via HTTP from Rust backend)
- **Ollama** as local sidecar for document embeddings (optional, graceful fallback)
- SQLite FTS5 for keyword search fallback when Ollama is unavailable

### Database
- **SQLite** with FTS5 extension (Phase 1)
- Repository pattern in Rust — all DB calls go through typed repository structs
- Schema versioned with integer migrations — `schema_version` table tracks applied migrations
- Designed for zero-friction PostgreSQL migration in Phase 2 (avoid SQLite-specific syntax where possible)

---

## 3. PROJECT STRUCTURE

```
meridian/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs       # DB pool, migrations runner
│   │   │   ├── migrations/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── v001_initial.sql
│   │   │   │   ├── v002_fts.sql
│   │   │   │   └── v003_embeddings.sql
│   │   │   └── repositories/
│   │   │       ├── mod.rs
│   │   │       ├── projects.rs
│   │   │       ├── meetings.rs
│   │   │       ├── tasks.rs
│   │   │       ├── documents.rs
│   │   │       ├── ai_settings.rs
│   │   │       ├── prompt_templates.rs
│   │   │       └── notifications.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── projects.rs
│   │   │   ├── meetings.rs
│   │   │   ├── tasks.rs
│   │   │   ├── documents.rs
│   │   │   ├── ai.rs              # LiteLLM calls, task extraction, chat
│   │   │   ├── settings.rs        # AI config, keychain read/write
│   │   │   ├── export.rs          # JSON/CSV/Markdown export
│   │   │   ├── import.rs
│   │   │   ├── notifications.rs
│   │   │   └── updater.rs
│   │   ├── ai/
│   │   │   ├── mod.rs
│   │   │   ├── litellm.rs         # LiteLLM HTTP client
│   │   │   ├── ollama.rs          # Ollama embedding client
│   │   │   ├── extractor.rs       # Task extraction logic + prompts
│   │   │   ├── embeddings.rs      # Embedding + similarity search
│   │   │   └── prompts.rs         # All system prompts as constants
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── project.rs
│   │   │   ├── meeting.rs
│   │   │   ├── task.rs
│   │   │   ├── document.rs
│   │   │   ├── ai_settings.rs
│   │   │   └── analytics.rs
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── backup.rs          # Auto-backup logic
│   │       ├── file_parser.rs     # PDF/DOCX/PPTX/CSV/TXT/URL parser
│   │       └── health_score.rs    # Meeting health score calculator
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── i18n/
│   │   ├── index.ts
│   │   └── locales/
│   │       ├── en.json
│   │       ├── hi.json
│   │       └── gu.json
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.tsx        # 3-column layout wrapper
│   │   │   ├── Sidebar.tsx         # Left: projects nav
│   │   │   ├── MainCanvas.tsx      # Center: task views
│   │   │   └── ContextPanel.tsx    # Right: meeting / AI chat
│   │   ├── onboarding/
│   │   │   ├── OnboardingWizard.tsx
│   │   │   ├── steps/
│   │   │   │   ├── WelcomeStep.tsx
│   │   │   │   ├── AISetupStep.tsx
│   │   │   │   ├── FirstProjectStep.tsx
│   │   │   │   └── FirstTranscriptStep.tsx
│   │   ├── projects/
│   │   │   ├── ProjectList.tsx
│   │   │   ├── ProjectCard.tsx
│   │   │   ├── ProjectCreate.tsx
│   │   │   └── ProjectSettings.tsx
│   │   ├── tasks/
│   │   │   ├── TaskListView.tsx
│   │   │   ├── TaskKanbanView.tsx
│   │   │   ├── TaskTableView.tsx
│   │   │   ├── TaskCard.tsx
│   │   │   ├── TaskInlineEditor.tsx
│   │   │   ├── TaskConfidenceBadge.tsx
│   │   │   ├── TaskBulkActions.tsx
│   │   │   └── TaskFilters.tsx
│   │   ├── meetings/
│   │   │   ├── MeetingIngest.tsx   # Paste / upload transcript
│   │   │   ├── MeetingCard.tsx
│   │   │   ├── MeetingHealthBadge.tsx
│   │   │   └── IntegrationGuide.tsx # Step-by-step platform guides
│   │   ├── documents/
│   │   │   ├── DocFolder.tsx
│   │   │   ├── DocUpload.tsx
│   │   │   ├── DocSearch.tsx
│   │   │   └── DocCard.tsx
│   │   ├── ai/
│   │   │   ├── AIChatPanel.tsx     # Scoped project chat
│   │   │   ├── OutputTemplates.tsx # 5 default templates
│   │   │   ├── AISettings.tsx      # Provider config screen
│   │   │   └── ModelPicker.tsx     # Fetch + select model
│   │   ├── analytics/
│   │   │   ├── ProjectDashboard.tsx
│   │   │   ├── HealthScoreCard.tsx
│   │   │   ├── VelocityChart.tsx
│   │   │   ├── WorkloadHeatmap.tsx
│   │   │   └── FollowThroughRate.tsx
│   │   ├── notifications/
│   │   │   └── NotificationCenter.tsx
│   │   ├── command-palette/
│   │   │   └── CommandPalette.tsx
│   │   └── shared/
│   │       ├── ThemeToggle.tsx
│   │       ├── ExportDialog.tsx
│   │       ├── ImportDialog.tsx
│   │       ├── ConfirmDialog.tsx
│   │       ├── EmptyState.tsx
│   │       └── UpdateBanner.tsx
│   ├── stores/
│   │   ├── projectStore.ts
│   │   ├── taskStore.ts
│   │   ├── meetingStore.ts
│   │   ├── documentStore.ts
│   │   ├── aiStore.ts
│   │   ├── notificationStore.ts
│   │   └── uiStore.ts             # theme, view mode, panel state
│   ├── hooks/
│   │   ├── useProjects.ts
│   │   ├── useTasks.ts
│   │   ├── useMeetings.ts
│   │   ├── useDocuments.ts
│   │   ├── useAI.ts
│   │   ├── useKeyboardShortcuts.ts
│   │   └── useAutoSave.ts
│   ├── lib/
│   │   ├── tauri.ts               # typed invoke wrappers
│   │   ├── validators.ts          # zod schemas
│   │   └── constants.ts
│   └── styles/
│       └── globals.css
├── docs/
│   ├── INSTALL.md
│   ├── CHANGELOG.md
│   ├── MIGRATIONS.md
│   └── integrations/
│       ├── zoom.md
│       ├── google-meet.md
│       └── microsoft-teams.md
├── agents/
│   ├── agent.md
│   ├── skills/
│   │   ├── ingest_meeting.json
│   │   ├── extract_tasks.json
│   │   ├── generate_output.json
│   │   └── search_project_docs.json
├── scripts/
│   ├── update.sh                  # One-command update (Unix)
│   ├── update.ps1                 # One-command update (Windows)
│   └── backup.sh
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
└── README.md
```

---

## 4. DATABASE SCHEMA (COMPLETE — implement exactly as specified)

Run all migrations on startup via the migrations runner. Never drop or truncate tables. All deletes are soft (via `archived_at` or `status` fields).

### Migration v001 — Core tables

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS projects (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  description  TEXT,
  color        TEXT NOT NULL DEFAULT '#6366f1',
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
  archived_at  TEXT
);

CREATE TABLE IF NOT EXISTS meetings (
  id               TEXT PRIMARY KEY,
  project_id       TEXT NOT NULL REFERENCES projects(id),
  title            TEXT NOT NULL,
  platform         TEXT NOT NULL DEFAULT 'manual',
  raw_transcript   TEXT,
  ai_summary       TEXT,
  health_score     INTEGER,
  health_breakdown TEXT,  -- JSON: {decisions, tasks_per_attendee, had_agenda, follow_through}
  attendees        TEXT,  -- JSON array of names
  meeting_at       TEXT,
  ingested_at      TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tasks (
  id                   TEXT PRIMARY KEY,
  project_id           TEXT NOT NULL REFERENCES projects(id),
  meeting_id           TEXT REFERENCES meetings(id),
  title                TEXT NOT NULL,
  description          TEXT,
  assignee             TEXT,
  assignee_confidence  TEXT NOT NULL DEFAULT 'unassigned',
  -- VALUES: 'committed' | 'inferred' | 'unassigned'
  assignee_source_quote TEXT,  -- exact quote from transcript
  due_date             TEXT,
  due_confidence       TEXT NOT NULL DEFAULT 'none',
  -- VALUES: 'committed' | 'inferred' | 'none'
  due_source_quote     TEXT,
  status               TEXT NOT NULL DEFAULT 'open',
  -- VALUES: 'open' | 'in_progress' | 'done' | 'cancelled'
  tags                 TEXT NOT NULL DEFAULT '[]',  -- JSON array
  kanban_column        TEXT NOT NULL DEFAULT 'open',
  kanban_order         INTEGER NOT NULL DEFAULT 0,
  is_duplicate         INTEGER NOT NULL DEFAULT 0,  -- boolean
  duplicate_of_id      TEXT REFERENCES tasks(id),
  notes                TEXT,
  created_at           TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at           TEXT NOT NULL DEFAULT (datetime('now')),
  completed_at         TEXT
);

CREATE TABLE IF NOT EXISTS documents (
  id               TEXT PRIMARY KEY,
  project_id       TEXT NOT NULL REFERENCES projects(id),
  filename         TEXT NOT NULL,
  file_path        TEXT NOT NULL,
  file_type        TEXT NOT NULL,
  -- VALUES: 'pdf' | 'docx' | 'txt' | 'md' | 'pptx' | 'csv' | 'xlsx' | 'url'
  source_url       TEXT,
  content_text     TEXT,
  chunks           TEXT,  -- JSON array of {text, index}
  embeddings_ready INTEGER NOT NULL DEFAULT 0,
  embedding_model  TEXT,
  file_size_bytes  INTEGER,
  uploaded_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ai_settings (
  id               TEXT PRIMARY KEY,
  label            TEXT NOT NULL,
  provider         TEXT NOT NULL,
  -- VALUES: 'openai' | 'anthropic' | 'gemini' | 'groq' | 'litellm' | 'ollama' | 'custom'
  base_url         TEXT,
  model_id         TEXT,
  ollama_base_url  TEXT NOT NULL DEFAULT 'http://localhost:11434',
  ollama_model     TEXT NOT NULL DEFAULT 'nomic-embed-text',
  embedding_provider TEXT NOT NULL DEFAULT 'ollama',
  -- VALUES: 'ollama' | 'openai' | 'anthropic' | 'none'
  is_active        INTEGER NOT NULL DEFAULT 0,
  created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS prompt_templates (
  id                   TEXT PRIMARY KEY,
  name                 TEXT NOT NULL,
  description          TEXT,
  system_prompt        TEXT NOT NULL,
  user_prompt_template TEXT NOT NULL,
  output_format        TEXT NOT NULL DEFAULT 'markdown',
  -- VALUES: 'markdown' | 'json' | 'jira' | 'plain'
  is_default           INTEGER NOT NULL DEFAULT 0,
  is_builtin           INTEGER NOT NULL DEFAULT 1,
  created_at           TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notifications (
  id          TEXT PRIMARY KEY,
  type        TEXT NOT NULL,
  -- VALUES: 'task_due' | 'task_overdue' | 'update_available' | 'system'
  title       TEXT NOT NULL,
  body        TEXT NOT NULL,
  task_id     TEXT REFERENCES tasks(id),
  project_id  TEXT REFERENCES projects(id),
  is_read     INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS chat_history (
  id           TEXT PRIMARY KEY,
  project_id   TEXT REFERENCES projects(id),
  meeting_id   TEXT REFERENCES meetings(id),
  role         TEXT NOT NULL,  -- 'user' | 'assistant'
  content      TEXT NOT NULL,
  template_id  TEXT REFERENCES prompt_templates(id),
  created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
  key    TEXT PRIMARY KEY,
  value  TEXT NOT NULL
);

-- Seed default app settings
INSERT OR IGNORE INTO app_settings (key, value) VALUES
  ('theme', 'system'),
  ('language', 'en'),
  ('onboarding_complete', 'false'),
  ('notification_email_digest', 'true'),
  ('notification_desktop', 'true'),
  ('email_digest_address', ''),
  ('task_due_warning_days', '2'),
  ('default_task_view', 'list');
```

### Migration v002 — FTS5 virtual tables

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS meetings_fts USING fts5(
  title, ai_summary, raw_transcript,
  content='meetings', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
  title, description, notes, assignee,
  content='tasks', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
  filename, content_text,
  content='documents', content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS meetings_ai AFTER INSERT ON meetings BEGIN
  INSERT INTO meetings_fts(rowid, title, ai_summary, raw_transcript)
  VALUES (new.rowid, new.title, new.ai_summary, new.raw_transcript);
END;

CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
  INSERT INTO tasks_fts(rowid, title, description, notes, assignee)
  VALUES (new.rowid, new.title, new.description, new.notes, new.assignee);
END;

CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
  INSERT INTO documents_fts(rowid, filename, content_text)
  VALUES (new.rowid, new.filename, new.content_text);
END;
```

### Migration v003 — Embeddings store

```sql
CREATE TABLE IF NOT EXISTS document_embeddings (
  id          TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id),
  chunk_index INTEGER NOT NULL,
  chunk_text  TEXT NOT NULL,
  embedding   BLOB NOT NULL,  -- float32 array serialized as bytes
  model       TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_embeddings_document
  ON document_embeddings(document_id);
```

### Seed default prompt templates (insert on first run)

```sql
INSERT OR IGNORE INTO prompt_templates (id, name, description, system_prompt, user_prompt_template, output_format, is_default, is_builtin) VALUES

('tpl_2x2', '2×2 Leadership Update',
'Slide-ready executive status update in 4 quadrants',
'You are a chief of staff preparing a concise executive update. Format your response as a 2×2 grid with exactly these four quadrants: "Accomplishments" (what was completed), "In Progress" (active work), "Blockers" (what is stuck and needs leadership attention), "Next Steps" (committed actions with owners and dates). Use bullet points. Be specific. Maximum 3 bullets per quadrant. Use the project context provided.',
'Project: {{project_name}}
Open tasks: {{open_tasks}}
Completed tasks (recent): {{completed_tasks}}
Recent meetings: {{recent_meetings}}

Generate a 2×2 leadership update for this project.',
'markdown', 1, 1),

('tpl_jira', 'Jira Feature Request',
'Structured Jira ticket from meeting discussion',
'You are a senior product manager writing Jira tickets. Output a structured ticket with these exact fields: Summary (one line), Type (Bug/Feature/Task/Story), Priority (Critical/High/Medium/Low), Description (problem statement), Acceptance Criteria (numbered list of testable criteria), Labels (comma separated), Story Points (estimate: 1/2/3/5/8/13). Use the context provided. Be precise and actionable.',
'Project: {{project_name}}
Context from meeting: {{meeting_context}}
Relevant tasks: {{related_tasks}}
Project documents: {{doc_context}}

Generate a Jira ticket for the main feature or issue discussed.',
'jira', 1, 1),

('tpl_agenda', 'Next Meeting Agenda',
'Auto-generated agenda from open tasks and project status',
'You are an executive assistant preparing a meeting agenda. Generate a structured agenda with: (1) Quick wins to celebrate from completed tasks, (2) Blockers requiring group decision, (3) Tasks overdue or at risk, (4) Open items needing assignment or date commitment, (5) AOB. Each item should have a suggested time allocation. Total meeting should not exceed 45 minutes unless context demands it.',
'Project: {{project_name}}
Open tasks: {{open_tasks}}
Overdue tasks: {{overdue_tasks}}
Blockers: {{blockers}}
Last meeting summary: {{last_meeting_summary}}

Generate the agenda for the next meeting on this project.',
'markdown', 1, 1),

('tpl_status', 'Project Status Report',
'Comprehensive open vs done status for stakeholders',
'You are a project manager writing a stakeholder status report. Include: Executive Summary (2 sentences), Overall Status (Green/Amber/Red with reason), Completed This Period, In Progress, Upcoming, Risks and Mitigations, Team Workload summary. Be honest about delays. Flag anything Red clearly.',
'Project: {{project_name}}
All tasks: {{all_tasks}}
Analytics: {{analytics}}
Follow-through rate: {{follow_through_rate}}
Velocity: {{velocity}}

Generate a complete project status report.',
'markdown', 1, 1),

('tpl_freeform', 'Free-form Prompt',
'Ask anything about this project',
'You are a knowledgeable project intelligence assistant with full context of this project''s meetings, tasks, documents, and history. Answer the user''s question accurately and concisely using the provided context. If the answer requires information not in the context, say so clearly.',
'Project: {{project_name}}
Project context: {{full_context}}

User question: {{user_question}}',
'markdown', 1, 1);
```

---

## 5. RUST BACKEND — TAURI COMMANDS

Implement all the following Tauri commands. Each command must be registered in `main.rs`. All commands return `Result<T, String>` where the error string is a human-readable message shown to the user.

### AI Commands (`commands/ai.rs`)

#### `verify_ai_connection`
- Accepts: `{ provider, base_url, api_key, model_id }`
- Retrieves API key from keychain using the label
- Makes a minimal chat completion call: `[{"role":"user","content":"Say OK"}]`, max_tokens=5
- Returns: `{ success: bool, error: Option<String>, latency_ms: u64 }`
- On failure: parse the HTTP error and return specific message ("Invalid API key", "Model not found", "Rate limited", "Network unreachable")

#### `fetch_available_models`
- Accepts: `{ provider, base_url, api_key_label }`
- Calls the provider's models endpoint
- For LiteLLM: `GET {base_url}/models`
- For OpenAI: `GET https://api.openai.com/v1/models`
- For Anthropic: return hardcoded list of current Claude models
- For Ollama: `GET http://localhost:11434/api/tags`
- Returns: `Vec<{ id: String, name: String, context_window: Option<u32> }>`

#### `save_ai_settings`
- Accepts: full `AISetting` struct
- Stores API key in OS keychain via `keyring` crate using label as service name
- Saves all other fields to `ai_settings` table (no API key in DB ever)
- Sets `is_active = 1` for this record, `is_active = 0` for all others
- Returns: `AISetting` (without key)

#### `extract_tasks_from_transcript`
- Accepts: `{ meeting_id, transcript, project_id }`
- Retrieves active AI settings + API key from keychain
- Calls LiteLLM with the task extraction system prompt (see Section 7)
- Parses structured JSON response
- For each extracted task: check for semantic duplicates against existing open tasks in same project using FTS5 similarity
- Inserts tasks with confidence fields populated
- Updates meeting `ai_summary` with generated summary
- Calculates and stores `health_score`
- Returns: `Vec<Task>`
- Edge cases:
  - Transcript under 50 words: return error "Transcript too short to extract tasks"
  - AI returns malformed JSON: retry once with explicit JSON repair instruction
  - Network timeout: return partial results if any parsed, else error

#### `chat_with_project`
- Accepts: `{ project_id, meeting_id?, message, template_id?, conversation_history }`
- Builds context: fetches project tasks, last 3 meetings, relevant doc chunks via FTS5/embeddings
- If `template_id` provided: uses template's system prompt and formats user_prompt_template with context variables
- Calls LiteLLM with full conversation history
- Saves both user message and assistant response to `chat_history`
- Streams response back via Tauri event: `emit("chat_chunk", { content, done })`
- Returns: `ChatMessage`

#### `check_ollama_status`
- Pings `http://localhost:11434/api/tags`
- Returns: `{ running: bool, models: Vec<String> }`

#### `embed_document_chunks`
- Accepts: `document_id`
- Fetches document chunks from DB
- For each chunk: calls configured embedding provider (Ollama → OpenAI → Anthropic, in priority order based on settings)
- Stores embeddings in `document_embeddings` table
- Updates `documents.embeddings_ready = 1`
- Returns: `{ chunks_embedded: u32 }`

#### `search_documents`
- Accepts: `{ project_id, query, use_semantic: bool }`
- If `use_semantic` and embeddings available: embed query → cosine similarity search
- Always also run FTS5 keyword search
- Merge and deduplicate results, rank by combined score
- Returns: `Vec<{ document_id, chunk_text, score, filename }>`

### Meeting Commands (`commands/meetings.rs`)

#### `ingest_meeting`
- Accepts: `{ project_id, title, platform, raw_transcript, meeting_at? }`
- Validates transcript is not empty
- Inserts into `meetings` table
- Triggers `extract_tasks_from_transcript` automatically
- Returns: `{ meeting: Meeting, tasks: Vec<Task> }`

#### `get_meetings_for_project`
- Returns all meetings for a project, ordered by `meeting_at` desc

#### `delete_meeting` (soft: set status field)
- Does NOT delete tasks — they remain linked

### Task Commands (`commands/tasks.rs`)

#### `get_tasks_for_project`
- Accepts: `{ project_id, filters: TaskFilters }`
- Filters: `{ assignee?, status?, tags?, date_range?, search_query? }`
- Returns tasks ordered by: overdue first, then by `kanban_order`

#### `update_task`
- Accepts: full task fields
- Auto-sets `updated_at`
- If `status = 'done'`: sets `completed_at`
- Triggers FTS update

#### `bulk_update_tasks`
- Accepts: `{ task_ids: Vec<String>, updates: PartialTask }`
- Updates all specified fields for all listed tasks atomically

#### `reorder_tasks`
- Accepts: `{ task_id, new_column, new_order }`
- Updates `kanban_column` and `kanban_order` for affected tasks

### Document Commands (`commands/documents.rs`)

#### `upload_document`
- Accepts: `{ project_id, file_path }` OR `{ project_id, url }`
- Copies file to `~/.meridian/documents/{project_id}/`
- Parses content based on file type:
  - PDF: use `pdf-extract`
  - DOCX: use `docx-rs`
  - PPTX: extract text from slide XML
  - CSV/XLSX: use `calamine`, convert to text table
  - TXT/MD: read as-is
  - URL: fetch with `scraper`, extract main content (readability algorithm)
- Chunks content: 512 tokens per chunk, 50 token overlap
- Inserts document + chunks
- Triggers background embedding (non-blocking)
- Returns: `Document`
- Edge cases:
  - File over 50MB: reject with clear error
  - URL returns 404/403: return specific error
  - Password-protected PDF: return "Password-protected files are not supported"
  - Duplicate URL: check `source_url` field and warn user

### Export/Import Commands (`commands/export.rs`, `commands/import.rs`)

#### `export_project`
- Accepts: `{ project_id, format: 'json' | 'csv' | 'markdown', include_docs: bool }`
- JSON: full nested export `{ project, meetings[], tasks[], documents[] }`
- CSV: flat tasks export with all columns
- Markdown: human-readable report with sections
- Saves to user-chosen location via dialog
- Returns: `{ file_path, size_bytes }`

#### `export_all`
- Same as above but for all projects
- Always JSON format for backup purposes

#### `import_project`
- Accepts: `{ file_path }`
- Validates JSON schema version
- Checks for duplicate project names — appends "(imported)" if conflict
- Inserts all records with new UUIDs (avoids ID collision)
- Returns: `{ projects_imported, meetings_imported, tasks_imported }`

### Settings / Update Commands

#### `check_for_updates`
- Uses `tauri-plugin-updater` to check GitHub releases endpoint
- Returns: `{ update_available: bool, version?: String, release_notes?: String }`

#### `backup_database`
- Copies `meridian.db` to `~/.meridian/backups/meridian_{timestamp}.db`
- Keeps last 10 backups, deletes older ones
- Called automatically before any migration and before app updates

---

## 6. AI PROMPTS (EXACT — implement these verbatim in `ai/prompts.rs`)

### Task Extraction Prompt

**System:**
```
You are Meridian's task extraction engine. Your job is to analyze meeting transcripts and extract every actionable task, decision, and commitment discussed.

RULES:
1. Extract ONLY real commitments — things someone said they would do, must do, or agreed to do
2. For each task, identify the assignee from context clues (name mentioned, "I will", "you should", role titles)
3. For deadlines: only mark as 'committed' if an explicit date/timeframe was stated. Mark 'inferred' if context suggests urgency
4. Generate a short auto-tag for each task from: blocker, decision, deliverable, follow-up, dependency, research, review, approval
5. If two attendees' responsibilities seem related, note the dependency
6. Look for decisions made (mark as tag: decision) — these are as important as tasks
7. Always include the exact quote from the transcript that led to each extraction

OUTPUT: Respond with ONLY valid JSON, no markdown, no explanation. Schema:
{
  "summary": "2-3 sentence meeting summary",
  "decisions": ["list of decisions made"],
  "tasks": [
    {
      "title": "concise action item title",
      "description": "fuller context if needed",
      "assignee": "name or null",
      "assignee_confidence": "committed | inferred | unassigned",
      "assignee_source_quote": "exact quote or null",
      "due_date": "YYYY-MM-DD or null",
      "due_confidence": "committed | inferred | none",
      "due_source_quote": "exact quote or null",
      "tags": ["blocker", "deliverable"],
      "notes": "any additional context"
    }
  ],
  "attendees": ["names detected"],
  "health": {
    "had_agenda": true | false,
    "decisions_count": 0,
    "tasks_count": 0,
    "attendees_count": 0
  }
}
```

**User template:**
```
Project: {{project_name}}
Existing open tasks (for duplicate detection): {{existing_tasks}}

TRANSCRIPT:
{{transcript}}

Extract all tasks from this transcript.
```

### Meeting Health Score Calculation (in `utils/health_score.rs`)

```
Score = 0-100 calculated as:
- Had agenda (detected from transcript keywords "agenda", "today we'll cover"): +20 points
- Decisions per topic ratio (decisions_count / max(topics_count, 1)): up to +25 points
- Tasks per attendee (healthy = 1-3 per person): up to +20 points (penalty for 0 or >5)
- Follow-through from last meeting (% of last meeting's tasks now done): up to +25 points
- Meeting duration efficiency (estimated from transcript length): up to +10 points

Color coding: 0-40 = Red, 41-70 = Amber, 71-100 = Green
```

### Context Builder for Chat (in `ai/extractor.rs`)

When building context for the chat panel, always include:
1. Project name and description
2. Open tasks (title, assignee, due_date, tags) — max 50 tasks
3. Recently completed tasks (last 30 days) — max 20
4. Last 3 meeting summaries
5. Relevant document chunks (top 5 by similarity to user's message)
6. Analytics snapshot (velocity, follow-through rate)

Format context as structured text, not JSON, to save tokens.

---

## 7. FRONTEND — DETAILED SCREEN SPECIFICATIONS

### 7.1 App Shell Layout (always visible after onboarding)

**Three-column layout, fixed heights:**
- Left sidebar: 240px wide, fixed, project navigation
- Main canvas: flexible width, scrollable
- Right context panel: 360px wide, collapsible via keyboard shortcut `]`

**Left Sidebar contains:**
- Meridian logo + app name at top
- "New Project" button
- Project list (draggable to reorder)
  - Each project shows: color dot, name, open task count badge
  - Active project highlighted
- Divider
- "All Tasks" view (across projects)
- "Meetings" view
- Divider
- Notification bell with unread count
- Settings gear icon
- Theme toggle (sun/moon/system)
- User/version info at bottom

**Main Canvas tabs (per project):**
- Tasks (default) — with view switcher: List | Kanban | Table
- Meetings
- Documents
- Analytics
- Chat

**Right Context Panel:**
- When a task is selected: show task detail + inline editor
- When a meeting is selected: show transcript + extracted tasks + health score
- Always shows: AI Chat at bottom (collapsible)

### 7.2 Onboarding Wizard

Show on first launch (`onboarding_complete = false` in app_settings).

**Step 1 — Welcome:**
- Full-screen centered layout
- Meridian logo (SVG, geometric/minimal)
- Headline: "Your meetings, finally working for you."
- Subline: "Meridian turns any transcript into structured tasks, project context, and AI-powered outputs — in seconds."
- CTA: "Get started" button + "Skip setup" link (both must work)
- Skip sets `onboarding_complete = true` and goes to empty state

**Step 2 — AI Setup:**
- Title: "Connect your AI"
- Provider dropdown: OpenAI, Anthropic, Google Gemini, Groq, LiteLLM (self-hosted), Ollama (local only)
- Base URL field (shown for LiteLLM and Ollama)
- API Key field (masked input, stored in keychain on save — never in DB)
- "Verify connection" button → spinner → green checkmark or specific error message
- On success: "Fetch available models" button appears
- Model picker dropdown populated from API
- "Use Ollama for document search" toggle (separate, optional)
  - If toggled: shows Ollama URL field + "Check Ollama status" button
  - If Ollama not running: yellow warning "Ollama not detected. Document semantic search will use keyword search until Ollama is running. Install guide →"
- Progress: Step 2 of 4

**Step 3 — First Project:**
- Title: "Create your first project"
- Name input (required)
- Description input (optional)
- Color picker (8 preset colors)
- "Create project" button
- Progress: Step 3 of 4

**Step 4 — First Transcript:**
- Title: "Paste your first meeting"
- Large textarea: "Paste a transcript, AI summary, or meeting notes here..."
- Platform selector below textarea: Manual | Zoom | Google Meet | Teams | Slack | Webex
- "Or connect a platform →" link (opens integration guide modal)
- "Extract tasks" button — must be disabled if textarea is empty
- On success: show extracted tasks preview in right panel
- "Finish setup" button
- Progress: Step 4 of 4

### 7.3 Integration Guide Modal

Triggered from onboarding Step 4 and from Settings → Integrations tab.

Show a step-by-step guide for each platform. Each guide is a vertical stepper with numbered steps. Steps include screenshots placeholder areas with instructions.

**Zoom Integration Guide:**
```
Step 1: Enable Cloud Recording
  → Log in to zoom.us → Settings → Recording → Enable "Cloud recording"
  → Also enable: "Audio transcript" under Cloud recording settings

Step 2: Get your transcript
  → After your meeting ends, go to zoom.us → Recordings
  → Click your recording → Click "Audio Transcript" tab
  → Copy all text → Paste into Meridian

Step 3 (Optional — Phase 2): Connect via API
  → zoom.us → App Marketplace → Build App → OAuth
  → Required scopes: recording:read, recording:write
  → [Coming in Meridian v0.2]
```

**Google Meet Integration Guide:**
```
Step 1: Enable transcription
  → In Google Meet during a call: Activities → Transcripts → Start Transcription
  → Transcription requires Google Workspace (Business Standard or higher)

Step 2: Get your transcript
  → After the meeting: check your Google Drive → Meet Recordings folder
  → Open the transcript doc → Select All → Copy
  → Paste into Meridian

Step 3: Enable AI notes (alternative)
  → Google Meet → Settings → Transcripts & Notes → Enable Gemini AI notes
  → After meeting: AI summary appears in Google Docs → Copy → Paste into Meridian

Step 3 (Optional — Phase 2): Connect via API
  → Google Cloud Console → Enable "Google Meet API"
  → OAuth 2.0 credentials → Required scope: meet.recordings.readonly
  → [Coming in Meridian v0.2]
```

**Microsoft Teams Integration Guide:**
```
Step 1: Enable transcription in meeting
  → During a Teams meeting: More options (···) → Record and transcribe → Start transcription
  → Note: Requires Teams Premium or Microsoft 365 E3/E5

Step 2: Get your transcript after meeting
  → Go to Teams → Chat → find the meeting
  → Click "..." → Open in Microsoft Stream
  → Download transcript (VTT format) or copy text
  → Paste into Meridian

Step 3: Alternative — use Teams AI recap
  → If you have Copilot in Teams: after the meeting, Copilot generates a recap
  → Copy the recap text → Paste into Meridian

Step 3 (Optional — Phase 2): Connect via Microsoft Graph API
  → portal.azure.com → App Registrations → New registration
  → Required permissions: OnlineMeetings.Read, CallRecords.Read.All
  → [Coming in Meridian v0.2]
```

Each guide must have:
- A "Copy guide link" button (copies a deep link to that guide)
- A "Mark as configured" toggle (stored in app_settings)
- A "Test connection" button (Phase 2 — show as disabled with "Coming soon" tooltip)

### 7.4 Task Views

**Task Card (shared across all views):**
Every task card must show:
- Title (inline editable on click)
- Assignee chip (with color avatar initial)
- Due date (red if overdue, amber if due within 2 days)
- Tags (colored pills: blocker=red, decision=purple, deliverable=blue, follow-up=amber)
- Confidence indicators:
  - If `assignee_confidence = 'inferred'`: show `~` prefix on assignee name + tooltip "Assignee inferred from context — click to confirm"
  - If `due_confidence = 'inferred'`: show `~` prefix on date + tooltip "Date inferred — click to confirm"
  - If `assignee_confidence = 'unassigned'`: show "Unassigned" in red
  - If `is_duplicate = true`: show yellow "Possible duplicate" badge + link to original task
- Source meeting name (small, muted)
- Three-dot menu: Edit, Duplicate, Move to project, Mark done, Delete

**List View:**
- Flat sorted list, grouped by: (toggle) None | Assignee | Tag | Status
- Bulk select via checkbox column on left
- Sort controls: Due date | Created | Assignee | Status

**Kanban View:**
- Columns: Open | In Progress | Done
- Cards draggable between columns via @dnd-kit
- Column header shows task count
- "Add task" button at bottom of each column

**Table View:**
- Spreadsheet-like grid
- Columns: Title | Assignee | Due Date | Status | Tags | Meeting | Created
- Inline cell editing
- Column resize and reorder
- Sticky header

**Task Filters Bar (above all views):**
- Assignee multi-select dropdown (populated from tasks in project)
- Status filter: All | Open | In Progress | Done
- Tag filter: multi-select
- Date range picker
- Search input (FTS across title + description + notes)
- "Clear all filters" button (shows only when filters active)

**Bulk Actions Bar (appears when tasks selected):**
- Shows: "N tasks selected"
- Actions: Assign to, Set due date, Add tag, Move to project, Mark done, Delete
- Escape to deselect all

### 7.5 AI Chat Panel

Located in the right context panel, always accessible.

**Scope selector at top:**
- "Project: [Project Name]" chip — clicking shows scope picker
- Scope options: This project | Specific meeting (dropdown) | All projects

**Template quick-launch buttons (5 buttons, horizontal scroll):**
- 2×2 Update | Jira Ticket | Next Agenda | Status Report | Free-form
- Clicking a template pre-fills the input with the template's user prompt
- User can edit before sending

**Chat thread:**
- User messages: right-aligned, background tinted
- Assistant messages: left-aligned, full markdown rendered (use `react-markdown`)
- Each assistant message has:
  - Copy button
  - "Save as note" button (saves to project notes)
  - Thumbs up/down (stored locally for template improvement)

**Input area:**
- Multiline textarea (Shift+Enter for newline, Enter to send)
- Send button
- Character count (warn at 2000, max 4000)
- "Context used" indicator: chips showing what context was included (tasks count, docs count, meetings count)

**Streaming:**
- Show typing indicator while waiting for first chunk
- Stream tokens as they arrive via Tauri events
- "Stop generating" button while streaming

### 7.6 Analytics Dashboard (per project)

Show 4 metric cards at top:
1. **Meeting Health** — average health score this month, with trend arrow
2. **Project Velocity** — tasks closed this week vs last week
3. **Follow-through Rate** — % of tasks from last meeting now completed
4. **Active Assignees** — count of people with open tasks

Below cards:

**Velocity Chart** — `recharts` LineChart:
- X-axis: last 8 weeks
- Y-axis: tasks closed per week
- Line with dots, hover tooltip

**Assignee Workload Heatmap** — grid:
- Rows: assignees
- Columns: weeks
- Cell color: green (1-2 tasks) → amber (3-4) → red (5+)
- Hover shows task list

**Meeting Health History** — `recharts` BarChart:
- One bar per meeting
- Color coded by score range
- Click bar to open that meeting

**Follow-through Timeline** — shows % completion per meeting cycle

### 7.7 Command Palette (`⌘K` / `Ctrl+K`)

Built with `cmdk`. Opens as a floating modal centered on screen.

**Groups and actions:**
```
Navigation:
  → Go to project: [project name]
  → Go to All Tasks
  → Open Settings
  → Open Notifications

Actions:
  → New Project
  → Paste new transcript
  → New task in [current project]
  → Export current project

AI:
  → Generate 2×2 update
  → Generate next agenda
  → Ask AI about [current project]

Recent:
  → [last 5 viewed meetings/projects]
```

Search is live across all groups. Keyboard navigation with arrow keys. Enter to execute. Escape to close.

### 7.8 Settings Screen (gear icon in sidebar)

**Tabs:**
1. **AI & Models** — full AI settings UI (provider, key, model, Ollama config)
2. **Integrations** — platform guides for Zoom, Google Meet, Teams, with status indicators
3. **Notifications** — email digest toggle + address, desktop notification toggle, warning days ahead
4. **Appearance** — theme toggle (Light/Dark/System), language selector
5. **Data** — export all data, import data, backup now, view backup history
6. **About** — version, check for updates, changelog link

### 7.9 Notification Center

Bell icon in sidebar with unread count badge.

**Notification types:**
- Task due in N days: "Review API integration — due in 2 days (assigned to Rohan)"
- Task overdue: "Write test cases — overdue by 3 days (Priya)"
- New update available: "Meridian v0.2.0 is available — see what's new"

**Email digest:**
- Sent daily at 8am local time (use system scheduler or app-level timer on launch)
- Contains: overdue tasks, tasks due today, tasks due tomorrow
- Plain text email via SMTP (user provides SMTP settings or use simple mailto fallback)

---

## 8. KEYBOARD SHORTCUTS

Implement all via `useKeyboardShortcuts` hook registered globally:

| Shortcut | Action |
|---|---|
| `⌘K` / `Ctrl+K` | Open command palette |
| `⌘N` / `Ctrl+N` | New task in current project |
| `⌘⇧N` / `Ctrl+Shift+N` | New project |
| `⌘M` / `Ctrl+M` | Open new meeting ingest |
| `⌘/` / `Ctrl+/` | Focus AI chat input |
| `⌘E` / `Ctrl+E` | Export current project |
| `⌘1/2/3` | Switch task view (List/Kanban/Table) |
| `]` | Toggle right panel |
| `[` | Toggle sidebar |
| `Escape` | Close modal / deselect tasks |
| `⌘Z` / `Ctrl+Z` | Undo last task change |
| `Delete` (when task selected) | Prompt to delete task |
| `Space` (when task focused in list) | Toggle task done/open |

---

## 9. THEMING

Tailwind config must include:
- CSS variables for all semantic colors (--meridian-primary, --meridian-surface, etc.)
- Dark mode via `class` strategy (toggle `dark` class on `<html>`)
- System preference detection via `window.matchMedia('(prefers-color-scheme: dark)')`
- Theme persisted in `app_settings` table

**Color palette:**
```
Primary: Indigo (#6366f1 light, #818cf8 dark)
Surface: White (#ffffff) / Zinc-950 (#09090b)
Background: Zinc-50 (#fafafa) / Zinc-900 (#18181b)
Border: Zinc-200 (#e4e4e7) / Zinc-800 (#27272a)
Text primary: Zinc-900 (#18181b) / Zinc-50 (#fafafa)
Text muted: Zinc-500 (#71717a)
Success: Emerald-500
Warning: Amber-500
Danger: Red-500
Blocker tag: Red-100 / Red-800
Decision tag: Purple-100 / Purple-800
Deliverable tag: Blue-100 / Blue-800
Follow-up tag: Amber-100 / Amber-800
```

---

## 10. INTERNATIONALISATION

- Use `react-i18next` throughout — zero hardcoded UI strings in components
- Default language: English (`en`)
- Also provide: Hindi (`hi`), Gujarati (`gu`)
- All translation keys in `src/i18n/locales/en.json` (and equivalent files)
- Language stored in `app_settings.language`
- Language selector in Settings → Appearance
- Date/time formatting via `date-fns/locale` matching selected language
- Number formatting via `Intl.NumberFormat`

---

## 11. AUTO-SAVE

- Every task field change triggers a debounced save (300ms debounce)
- Show a subtle "Saved" indicator in bottom-left for 1.5 seconds after each save
- On network/DB error during save: show persistent "Save failed — click to retry" banner
- Never show a "Save" button anywhere in the UI
- Implement optimistic updates: update UI immediately, roll back on DB error

---

## 12. UPDATE & MIGRATION SYSTEM

### In-app update checker:
- On launch: check GitHub releases API (configure endpoint in `tauri.conf.json`)
- If update available: show non-intrusive banner at top of app with version + "See what's new" link + "Update now" button
- "Update now": triggers auto-backup first, then runs updater

### Auto-backup before update:
```rust
// In backup.rs
pub fn backup_database() -> Result<String, String> {
    let db_path = get_db_path();
    let backup_dir = get_backup_dir();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("meridian_{}.db", timestamp));
    std::fs::copy(&db_path, &backup_path)?;
    // Keep only last 10 backups
    cleanup_old_backups(&backup_dir, 10)?;
    Ok(backup_path.to_string_lossy().to_string())
}
```

### Migration runner:
```rust
// On every app launch
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_VERSION_TABLE)?;
    let current = get_schema_version(conn)?;
    let migrations = get_all_migrations(); // sorted by version number
    for migration in migrations.iter().filter(|m| m.version > current) {
        backup_database()?; // backup before each migration
        conn.execute_batch(&migration.sql)?;
        set_schema_version(conn, migration.version)?;
    }
    Ok(())
}
```

---

## 13. EXPORT / IMPORT FORMATS

### JSON Export Schema (version-stamped):
```json
{
  "meridian_export_version": "1",
  "exported_at": "ISO timestamp",
  "app_version": "0.1.0",
  "project": { ...full project object },
  "meetings": [ ...array ],
  "tasks": [ ...array ],
  "documents": [ ...array with content_text, excluding embeddings ],
  "prompt_templates": [ ...custom templates only ]
}
```

### CSV Export (tasks only):
Headers: `id, title, description, assignee, assignee_confidence, due_date, due_confidence, status, tags, meeting_title, meeting_date, created_at, completed_at, notes`

### Markdown Export:
```markdown
# Project: [Name]
Exported: [date]

## Summary
[analytics snapshot]

## Open Tasks
### By Assignee
**[Name]**
- [ ] Task title (due: date) [tags]

## Completed Tasks (last 30 days)
- [x] Task title (completed: date)

## Meetings
### [Meeting title] — [date] — Health: [score]/100
**Summary:** ...
**Tasks extracted:** N
```

---

## 14. DOCUMENTATION FILES TO CREATE

### `docs/INSTALL.md`

Must contain complete step-by-step installation for Windows, macOS, Linux.

```markdown
# Installing Meridian

## Windows

### Prerequisites
1. Install Rust: https://rustup.rs/
   - Run the installer, choose default options
   - Restart terminal after installation
   - Verify: `rustc --version`

2. Install Node.js (v18 or higher): https://nodejs.org/
   - Choose LTS version
   - Verify: `node --version`

3. Install Visual Studio C++ Build Tools (required by Tauri on Windows):
   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Select: "Desktop development with C++"

4. Install WebView2 (usually pre-installed on Windows 11):
   - If missing: https://developer.microsoft.com/microsoft-edge/webview2/

### Install Meridian
```bash
# Clone the repository
git clone https://github.com/yourusername/meridian.git
cd meridian

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
# Installer will be at: src-tauri/target/release/bundle/
```

### Optional: Install Ollama (for local document search)
1. Download from https://ollama.ai/download
2. Run the installer
3. Open terminal and pull the embedding model:
   `ollama pull nomic-embed-text`
4. Verify Ollama is running: `ollama list`
5. In Meridian Settings → AI & Models → enable Ollama and click "Check status"

## macOS
[Same structure with macOS-specific steps]

## Linux
[Same structure with Linux-specific steps — include webkit2gtk dependency]

## First Launch
1. Open Meridian
2. The setup wizard will guide you through connecting your AI provider
3. Create your first project
4. Paste your first meeting transcript
5. Review the extracted tasks
```

### `docs/CHANGELOG.md`

```markdown
# Changelog

## [0.1.0] — Initial Release
### Added
- Manual transcript ingestion with AI task extraction
- Project and task management (list, kanban, table views)
- Document folders with FTS5 search
- AI chat panel with 5 output templates
- Meeting health score
- Analytics dashboard
- Import / export (JSON, CSV, Markdown)
- Integration guides: Zoom, Google Meet, Microsoft Teams
- Desktop notifications and email digest
- OS keychain for secure API key storage
- Light / dark / system theme
- Keyboard shortcuts and command palette
- Multi-language support (en, hi, gu)

### Migration
- No migration required for fresh install
- v001: Initial schema
- v002: FTS5 virtual tables
- v003: Document embeddings store
```

### `docs/MIGRATIONS.md`

```markdown
# Migration Guide

## Upgrading Meridian

Meridian automatically backs up your database before applying any migration.
Backups are stored in: `~/.meridian/backups/`

### Manual backup (recommended before major updates)
**Windows (PowerShell):**
```powershell
.\scripts\update.ps1 --backup-only
```

**macOS/Linux:**
```bash
./scripts/backup.sh
```

### Update to latest version
**Windows:**
```powershell
git pull origin main
npm install
.\scripts\update.ps1
```

**macOS/Linux:**
```bash
git pull origin main
npm install
./scripts/update.sh
```

### Restore from backup
```bash
# Stop Meridian first, then:
cp ~/.meridian/backups/meridian_YYYYMMDD_HHMMSS.db ~/.meridian/meridian.db
# Restart Meridian
```

## Migration SQL Reference
Each version's migration SQL is in: `src-tauri/src/db/migrations/`
```

### `scripts/update.sh`

```bash
#!/bin/bash
set -e
echo "🔄 Updating Meridian..."
echo "📦 Backing up database..."
cp ~/.meridian/meridian.db ~/.meridian/backups/meridian_$(date +%Y%m%d_%H%M%S).db
echo "📥 Pulling latest changes..."
git pull origin main
echo "📦 Installing dependencies..."
npm install
echo "🔨 Building..."
npm run tauri build
echo "✅ Meridian updated successfully!"
echo "📂 Installer at: src-tauri/target/release/bundle/"
```

### `scripts/update.ps1`

```powershell
param([switch]$BackupOnly)
Write-Host "Updating Meridian..." -ForegroundColor Cyan
$backupDir = "$env:APPDATA\meridian\backups"
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
Copy-Item "$env:APPDATA\meridian\meridian.db" "$backupDir\meridian_$timestamp.db"
Write-Host "Database backed up" -ForegroundColor Green
if ($BackupOnly) { exit 0 }
git pull origin main
npm install
npm run tauri build
Write-Host "Meridian updated successfully!" -ForegroundColor Green
```

---

## 15. AGENT FILES

### `agents/agent.md`

```markdown
# Meridian — Agent Onboarding Guide

## What is Meridian?
Meridian is a local-first AI-powered meeting intelligence desktop app built with Tauri (Rust) + React + TypeScript.

## Architecture Overview
- **Desktop:** Tauri v2 (Rust backend, React frontend)
- **Database:** SQLite with FTS5, migration-versioned
- **AI:** LiteLLM as unified gateway, Ollama for local embeddings
- **State:** Zustand stores, @tanstack/react-query for async

## Key Directories
- `src-tauri/src/commands/` — all Tauri IPC commands
- `src-tauri/src/ai/` — LiteLLM client, prompts, extractors
- `src-tauri/src/db/` — repositories and migrations
- `src/components/` — all React UI components
- `src/stores/` — Zustand global state
- `agents/skills/` — agent skill definitions

## Core Data Flow
Transcript paste → `ingest_meeting` command → `extract_tasks_from_transcript` →
LiteLLM task extraction prompt → JSON parse → duplicate check → insert tasks →
health score calculation → return to UI

## AI Prompt Location
All system prompts are constants in: `src-tauri/src/ai/prompts.rs`
Do not hardcode prompts in commands — always import from prompts.rs

## Adding a New Feature
1. Add DB migration in `src-tauri/src/db/migrations/`
2. Add model in `src-tauri/src/models/`
3. Add repository methods in `src-tauri/src/db/repositories/`
4. Add Tauri command in `src-tauri/src/commands/`
5. Register command in `main.rs`
6. Add typed invoke wrapper in `src/lib/tauri.ts`
7. Add Zustand store or react-query hook
8. Build UI component

## Testing
- Rust unit tests: `cargo test` in `src-tauri/`
- Frontend: `npm test`
- Integration: `npm run tauri dev` and test manually

## Environment
- No `.env` files — all secrets in OS keychain via `keyring` crate
- No hardcoded API keys anywhere in codebase
```

### `agents/skills/ingest_meeting.json`

```json
{
  "name": "ingest_meeting",
  "version": "1.0.0",
  "description": "Ingest a meeting transcript or summary into a Meridian project",
  "input_schema": {
    "type": "object",
    "properties": {
      "project_id": { "type": "string", "description": "Target project UUID" },
      "title": { "type": "string", "description": "Meeting title" },
      "platform": { "type": "string", "enum": ["zoom", "google_meet", "teams", "slack", "webex", "manual"] },
      "raw_transcript": { "type": "string", "description": "Full transcript text" },
      "meeting_at": { "type": "string", "description": "ISO 8601 datetime of meeting" }
    },
    "required": ["project_id", "raw_transcript"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "meeting_id": { "type": "string" },
      "tasks_extracted": { "type": "integer" },
      "health_score": { "type": "integer" }
    }
  },
  "tauri_command": "ingest_meeting"
}
```

### `agents/skills/extract_tasks.json`

```json
{
  "name": "extract_tasks",
  "version": "1.0.0",
  "description": "Extract structured tasks from a meeting transcript using AI",
  "input_schema": {
    "type": "object",
    "properties": {
      "meeting_id": { "type": "string" },
      "transcript": { "type": "string" },
      "project_id": { "type": "string" }
    },
    "required": ["meeting_id", "transcript", "project_id"]
  },
  "tauri_command": "extract_tasks_from_transcript"
}
```

### `agents/skills/generate_output.json`

```json
{
  "name": "generate_output",
  "version": "1.0.0",
  "description": "Generate a structured output (2x2, Jira, agenda, status, freeform) for a project",
  "input_schema": {
    "type": "object",
    "properties": {
      "project_id": { "type": "string" },
      "template_id": { "type": "string", "enum": ["tpl_2x2", "tpl_jira", "tpl_agenda", "tpl_status", "tpl_freeform"] },
      "message": { "type": "string", "description": "User message or question (for freeform)" }
    },
    "required": ["project_id", "template_id"]
  },
  "tauri_command": "chat_with_project"
}
```

### `agents/skills/search_project_docs.json`

```json
{
  "name": "search_project_docs",
  "version": "1.0.0",
  "description": "Search documents in a project using keyword or semantic search",
  "input_schema": {
    "type": "object",
    "properties": {
      "project_id": { "type": "string" },
      "query": { "type": "string" },
      "use_semantic": { "type": "boolean", "default": true }
    },
    "required": ["project_id", "query"]
  },
  "tauri_command": "search_documents"
}
```

---

## 16. EDGE CASES — IMPLEMENT ALL OF THESE

### AI / Extraction
- Transcript under 50 words → return error "Transcript too short — paste the full meeting text"
- AI returns invalid JSON → retry once with explicit repair instruction, if still invalid show raw response with "Could not parse tasks — copy the AI response below"
- API key expired mid-session → intercept 401 response → show "Your AI key has expired — update it in Settings" banner
- LiteLLM server unreachable → show specific "Cannot reach LiteLLM at [url] — check your server" message
- Same task extracted from two meetings → flag as duplicate, show side-by-side comparison, let user merge or keep both
- No tasks found in transcript → show "No tasks detected — the transcript may be a status update with no action items" (not an error)
- Context window exceeded → chunk transcript and extract in parts, merge results

### Documents
- File over 50MB → reject immediately with size shown: "This file is 67MB — maximum is 50MB"
- Password-protected PDF → show "This PDF is password-protected — please unlock it first"
- URL returns non-200 → show specific HTTP error: "Could not fetch URL (404 Not Found)"
- Duplicate URL already in project → warn: "This URL is already in your documents — add anyway?"
- Ollama stops mid-embedding → mark document as `embeddings_ready = false`, show retry button
- PPTX with only images → extract alt text and slide titles, warn "This presentation has limited text content"
- Very large DOCX (200+ pages) → chunk during parsing, show progress indicator

### Data / DB
- DB file corrupted → detect on startup, offer to restore from latest backup or start fresh
- Disk full during write → catch OS error, show "Disk full — free up space and try again"
- Concurrent writes (multiple windows) → SQLite WAL mode enabled, handle SQLITE_BUSY with retry
- Missing project_id reference → show "Project not found — it may have been deleted" instead of crashing
- Task drag-drop to wrong column → validate on drop, revert if invalid transition

### UI / UX
- Long transcript paste (>100KB) → show progress spinner during processing
- Command palette with no results → show "No results — try a different keyword"
- AI chat with empty project (no tasks, no meetings) → show contextual empty state: "Add a meeting transcript first — I'll have more to work with"
- Export with 0 tasks → export succeeds but file has empty tasks array (not an error)
- Theme change during AI streaming → don't interrupt stream
- App opened offline → show "Offline — AI features unavailable. Local data still accessible."
- Import file from newer app version → show "This export was created with Meridian v0.3 — some fields may not be available in your version"
- Keychain unavailable (Linux without libsecret) → fall back to AES-encrypted local file with warning

---

## 17. PERFORMANCE REQUIREMENTS

- App launch to interactive: under 2 seconds on standard hardware
- Task extraction (average transcript): under 10 seconds (show streaming progress)
- Document upload + parse (10MB PDF): under 5 seconds (async, non-blocking UI)
- FTS5 search response: under 100ms
- Embedding search: under 500ms
- DB writes: use WAL mode (`PRAGMA journal_mode=WAL`)
- All Tauri commands: non-blocking — use async Rust with Tokio
- Frontend: no component renders over 16ms — use `React.memo` and `useMemo` appropriately

---

## 18. BUILD COMMANDS (add to `package.json`)

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "build:debug": "tauri build --debug",
    "test": "vitest",
    "test:rust": "cd src-tauri && cargo test",
    "lint": "eslint src --ext .ts,.tsx",
    "type-check": "tsc --noEmit",
    "i18n:extract": "i18next-scanner"
  }
}
```

---

## 19. FINAL CHECKLIST — Before considering this complete

Claude Code must verify every item below is implemented:

- [ ] Tauri app launches on Windows without errors
- [ ] SQLite DB created with all 3 migrations applied on first launch
- [ ] Onboarding wizard shows on first launch, skippable at every step
- [ ] AI settings screen: enter key → verify → fetch models → select → save to keychain
- [ ] Manual transcript paste → tasks extracted with confidence badges
- [ ] Tasks show in all 3 views (List, Kanban, Table)
- [ ] Inline task editing works (click to edit, auto-save)
- [ ] Drag-and-drop tasks between Kanban columns
- [ ] Bulk task selection and actions work
- [ ] ⌘K / Ctrl+K command palette opens and is functional
- [ ] All 5 keyboard shortcuts from Section 8 work
- [ ] Integration guides for Zoom, Meet, Teams are complete and readable
- [ ] Document upload works for all 6 types (PDF, DOCX, TXT, PPTX, CSV, URL)
- [ ] FTS5 keyword search returns results
- [ ] Ollama status check works (with graceful fallback message)
- [ ] AI chat panel streams responses
- [ ] All 5 output templates work end-to-end
- [ ] Analytics dashboard shows all 4 metrics with charts
- [ ] Export works in JSON, CSV, Markdown
- [ ] Import works from JSON export
- [ ] Light / dark / system theme all work correctly
- [ ] Language selector changes UI language (en, hi, gu)
- [ ] Desktop notifications fire for overdue tasks
- [ ] In-app notification center shows and marks read
- [ ] Auto-backup runs before migrations
- [ ] Update checker polls and shows banner
- [ ] `docs/INSTALL.md` is complete and accurate
- [ ] `docs/CHANGELOG.md` exists
- [ ] `docs/MIGRATIONS.md` exists
- [ ] `scripts/update.sh` and `scripts/update.ps1` are executable
- [ ] `agents/agent.md` exists with full onboarding guide
- [ ] All 4 agent skill JSON files exist
- [ ] Zero hardcoded strings in components (all via i18n)
- [ ] Zero API keys in source code or DB (all via keychain)
- [ ] All edge cases from Section 16 are handled
- [ ] `README.md` at root with project overview and quick start

---

**Start by reading this entire prompt once. Then begin with the project scaffold: `npm create tauri-app@latest meridian -- --template react-ts`, then implement in the order: DB migrations → Rust commands → AI integration → Frontend shell → Onboarding → Core features → Analytics → Polish → Documentation → Agent files.**

Do not stop until every item on the checklist above is complete.
