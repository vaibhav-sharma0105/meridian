## Context

Meridian has evolved from a meeting transcript manager to an AI-powered assistant that learns patterns (Phase 2) and makes proactive suggestions (Phase 3). The next step is enabling users to define **skills** — reusable automation recipes that execute based on triggers. This follows Anthropic's agent skills standard emphasizing progressive disclosure, clear triggers, and graceful degradation.

**Current state:**
- Daemon job queue (`daemon_jobs`) handles background processing (embeddings, suggestions)
- AI module (`litellm.rs`) supports chat completion for various providers
- Pattern learning infrastructure records observations and aggregates models
- Suggestion engine detects opportunities but requires manual user action

**Constraints:**
- Local-first: All skill definitions and execution stay on device
- No external skill registry or cloud execution
- Must work offline (skills can still fire; AI actions wait for connectivity)

## Goals / Non-Goals

**Goals:**
- Enable users to define automation workflows via form-based UI or natural language
- Support three trigger types: schedule (cron), event (task_created, etc.), manual
- Ship 5 built-in skills as templates users can enable and customize
- Integrate skills with existing suggestion and notification systems
- Audit all skill executions for transparency

**Non-Goals:**
- No visual workflow builder (node-based editor) — form-based is sufficient
- No real-time collaboration on shared skills — clone-and-customize model only
- No external integrations beyond existing Zoom/Sheets Relay — skills operate on local data
- No skill marketplace or cloud sync of skills

## Decisions

### Decision 1: Skill Storage Schema

**Choice:** Single `skills` table with JSON columns for trigger/context/action configuration.

**Rationale:** 
- Flexible schema allows evolution without migrations
- JSON columns work well with SQLite + rusqlite
- Alternative considered: Normalized tables per trigger/action type — rejected because it adds complexity for marginal query benefits

**Schema:**
```sql
CREATE TABLE skills (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  trigger_type TEXT NOT NULL,  -- 'schedule' | 'event' | 'manual'
  trigger_config TEXT,         -- JSON: cron, event_type, filter, etc.
  context_config TEXT,         -- JSON: scope, documents, instructions
  action_config TEXT,          -- JSON: action type, output format
  approval_mode TEXT DEFAULT 'notify',
  enabled INTEGER DEFAULT 1,
  shared INTEGER DEFAULT 0,
  owner_id TEXT,
  category TEXT,
  icon TEXT,
  tags TEXT,                   -- JSON array
  created_at TEXT,
  updated_at TEXT
);

CREATE TABLE skill_runs (
  id TEXT PRIMARY KEY,
  skill_id TEXT REFERENCES skills(id) ON DELETE CASCADE,
  status TEXT NOT NULL,        -- pending, running, completed, failed, etc.
  trigger_type TEXT,
  trigger_context TEXT,        -- JSON: event payload, schedule time
  output TEXT,
  error TEXT,
  started_at TEXT,
  completed_at TEXT,
  duration_ms INTEGER,
  approval_decision TEXT,
  created_at TEXT
);
```

### Decision 2: Trigger Execution Architecture

**Choice:** Extend existing daemon job queue with new job type `execute_skill`.

**Rationale:**
- Reuses battle-tested job infrastructure (retries, priority, error handling)
- Schedule triggers: Daemon polls every 60s, computes due skills, queues jobs
- Event triggers: Entity mutations fire events; event dispatcher matches skills and queues jobs
- Alternative considered: Separate scheduler process — rejected to avoid complexity

**Event Dispatch Flow:**
```
Task/Meeting mutation
  → Repository logs change
  → Event emitted to dispatcher (in-process channel)
  → Dispatcher matches event against registered skill filters
  → Matching skills queued as daemon_jobs
```

### Decision 3: Approval Mode Implementation

**Choice:** Four-tier approval system with progressive trust.

**Rationale:** Follows Anthropic's progressive disclosure principle — start conservative, allow users to relax controls as trust builds.

| Mode | Behavior |
|------|----------|
| `auto` | Execute immediately, no user interaction |
| `notify` | Execute and create notification with results |
| `approve_first` | Preview side effects, wait for approval before commit |
| `approve_always` | Require explicit approval for every run |

Default for new skills: `notify` for read-only actions, `approve_first` for write actions.

### Decision 4: Chat-to-Skill Extraction

**Choice:** Use existing LiteLLM client with structured output prompts.

**Rationale:**
- No new dependencies
- Prompt template extracts trigger, context, action into JSON
- UI shows preview with editable fields before confirmation
- Alternative considered: Grammar-based parsing without AI — rejected because natural language is too varied

**Extraction Prompt Pattern:**
```
Extract automation skill from: "{user_description}"

Output JSON:
{
  "name": "...",
  "trigger": { "type": "...", "cron": "..." | "event_type": "..." },
  "context": { "scope": "..." },
  "action": { "type": "...", "format": "..." }
}
```

### Decision 5: Built-in Skill Implementation

**Choice:** Ship as YAML definitions bundled in app resources; instantiate on first use.

**Rationale:**
- Definitions stored in `src-tauri/resources/builtin-skills/`
- On app startup, check if built-in skills exist in DB; if not, insert from templates
- User modifications create a user-owned copy; original template preserved
- Alternative considered: Hard-coded in Rust — rejected because YAML is easier to update

### Decision 6: Skill Context Limits

**Choice:** Token-based truncation with priority ordering.

**Rationale:** Graceful degradation when context exceeds AI limits.

- Default max tokens: 8000
- Priority order: tasks (highest) → meetings → documents (lowest)
- Truncation removes lowest-priority content first
- Log truncation in run details for transparency

### Decision 6a: Document Context Loading

**Choice:** Opt-in document inclusion via `include_documents` context config flag.

**Rationale:**
- Documents can be large; including them by default would bloat context
- Users explicitly enable via toggle in skill editor (Basic mode checkbox)
- `document_filter` regex allows targeting specific files (e.g., `.*\.md$` for markdown only)
- `max_documents` caps at 10 by default to prevent token overflow
- Documents loaded via `get_documents_for_project()` from documents repository
- Each document contributes: id, name, file_type, and content snippet (first 500 chars)

**Implementation:**
```rust
// In executor.rs build_context()
if config.include_documents.unwrap_or(false) {
    let docs = docs_repo::get_documents_for_project(conn, project_id)?;
    // Apply filter and limit, push to context.documents
}
```

### Decision 6b: Shared Skills UI [PARKED]

**Choice:** Checkbox toggle in skill editor for `shared` field.

**Status:** PARKED — UI implemented but non-functional. Local-first app has no sharing mechanism.

**What exists:**
- `shared` checkbox in editor, badge on card
- `owner_id` column (never set)
- `cloned_from_id` column (not set on clone)

**What's missing for real sharing:**
- Multi-user auth
- Cloud sync / server backend
- Actual visibility filtering by owner

## Risks / Trade-offs

**[Risk] Cron library choice**
- Need reliable cron parsing and next-run computation in Rust
- **Mitigation:** Use `cron` or `croner` crate; both are well-maintained

**[Risk] Event dispatch performance**
- High-frequency task updates could flood event queue
- **Mitigation:** Debounce within 1s window per entity; rate-limit skill triggers per minute

**[Risk] Skill execution blocking AI**
- Long-running skill could block other daemon jobs
- **Mitigation:** ~~60s default timeout~~ [NOT IMPLEMENTED]; skills run in separate thread with their own tokio runtime (same pattern as embedding worker)

**[Risk] User creates infinite loop**
- Skill A triggers on task_created → creates tasks → triggers Skill A
- **Mitigation:** Track source_skill_id on created entities; skills don't trigger on entities they created

**[Trade-off] No inter-skill dependencies**
- Skills cannot chain (skill A output → skill B input)
- **Reasoning:** Adds significant complexity; multi-step actions within single skill sufficient for MVP

## Migration Plan

1. **Database migration (v011):** Add `skills`, `skill_runs` tables; add `skill_run_id` to suggestions and notifications
2. **Backend modules:** Create `src-tauri/src/skills/` with model, repository, executor
3. **Commands:** Add skill CRUD, execution, history commands
4. **Daemon integration:** Register `execute_skill` job handler, add schedule poller
5. **Event dispatcher:** Add entity mutation hooks, skill filter matching
6. **Frontend:** Skills section in sidebar, editor modal, history panel
7. **Built-in skills:** Add YAML templates, first-run initialization
8. **Chat-to-skill:** Extraction prompt, preview UI, refinement flow

**Rollback:** 
- Disable skills feature flag if critical bugs
- Migration is additive (new tables only) — no data loss on rollback

### Decision 7: Skill Types & Permissions

**Choice:** Three-tier type system (built-in, user-created, folder packages) with `is_builtin` DB flag.

**Rationale:**
- Built-in skills need delete protection — users accidentally deleting templates then losing access to them
- Folder packages are a different artifact entirely (directory on disk vs DB record) — read-only in UI
- `is_builtin` flag added via migration v013 as `INTEGER NOT NULL DEFAULT 0`
- `delete_skill()` rejects if `is_builtin = true`; "Reset defaults" mechanism for restoring templates

| Type | Storage | Editable | Deletable | Source |
|------|---------|----------|-----------|--------|
| Built-in | SQLite (`is_builtin=1`) | Yes | No (reset only) | `templates.json` |
| User-created | SQLite (`is_builtin=0`) | Yes | Yes | Editor/import/clone |
| Folder packages | Filesystem (`~/.meridian/skills/`) | View only | Yes | Upload |

### Decision 8: Skill Folder Packages

**Choice:** Filesystem-based skill packages in `~/.meridian/skills/` with `skill.md` validation.

**Rationale:**
- Skill packages can contain scripts, configs, and supporting files — not suitable for single-column DB storage
- Follows Anthropic standard format: directory with `skill.md` (YAML frontmatter + markdown body)
- Validation requires only `name:` and `description:` in frontmatter (minimal gate for flexibility)
- File tree viewer with progressive disclosure; script execution with human-in-the-loop confirmation

**Folder structure:**
```
~/.meridian/skills/
├── weekly-summary/
│   ├── skill.md          # Required: YAML+MD skill definition
│   ├── generate.py       # Optional: executable script
│   └── README.md         # Optional: documentation
└── meeting-prep/
    ├── skill.md
    └── templates/
        └── agenda.md
```

### Decision 9: Platform-Specific Folder Picker

**Choice:** AppleScript on macOS, `rfd` crate on Windows/Linux.

**Rationale:**
- `@tauri-apps/plugin-dialog` `open({ directory: true })` and `rfd::FileDialog::pick_folder()` both have macOS issues where NSOpenPanel attached as a sheet to the Tauri window doesn't properly enable the "Open" button for folder selection
- AppleScript's `choose folder` invokes a standalone system dialog that reliably works
- `osascript -e "choose folder"` returns POSIX path or exits non-zero on cancel
- Windows/Linux: `rfd::FileDialog` works correctly without the sheet-attachment issue
- Both wrapped in `tokio::task::spawn_blocking` since they block the thread

### Decision 10: Skill Export as Directory

**Choice:** Export creates a directory package with `skill.md` inside (not single-file JSON).

**Rationale:**
- Aligns with Anthropic skill standard (directory-based packages)
- Exported skills can be directly re-uploaded as folder packages
- Directory named with kebab-case slug of skill name
- Format round-trips: export → upload works seamlessly
- Platform-native folder picker selects destination (same picker infrastructure as upload)

### Decision 11: AI Chat Skill Integration

**Choice:** Progressive disclosure with compact context and unified execution.

**Rationale:**
- LLM context is expensive — send minimal info (name + description) for decision-making
- Full skill content loaded only when skill is invoked (not upfront)
- Both DB skills and folder packages unified via `UnifiedSkill` interface
- Single execution path handles both types, routing to `runSkillManually` or `executeSkillScript`

**Context Format (compact):**
```
📦 **Weekly Summary** - Summarizes tasks every Monday
⚡ **Meeting Follow-up** - Creates tasks from meeting notes
```

**LLM Instructions:**
- Clear "when to invoke" vs "when NOT to invoke" guidance
- Single skill per response rule
- Marker format: `**[SKILL_INVOKE: skill_name]**`

**Execution Flow:**
```
User message
  → LLM receives compact skill list
  → LLM decides to invoke (or not)
  → Response with [SKILL_INVOKE: name] marker (if invoking)
  → Frontend parses marker
  → Check: already invoked this conversation? → skip
  → Load full skill content (progressive)
  → Execute: DB skill → runSkillManually | Folder → executeSkillScript
  → Subtle UI feedback (checkmark only on completion)
```

**Deduplication:**
- `invokedSkills` Set tracks skills invoked per conversation
- `processedMsgIndices` ref prevents race condition re-execution
- Only process last assistant message (not full array)
- State cleared on `clearMessages()`

### Decision 12: Unified Skill Picker

**Choice:** Single picker showing both DB skills and folder packages.

**Rationale:**
- Users shouldn't care where a skill is stored — it's an implementation detail
- Unified `UnifiedSkill` interface with `type: "db" | "folder"` discriminator
- Visual differentiation: ⚡ for DB skills, 📦 for folder packages
- Picker receives merged list from `useAI` hook

### Decision 13: Folder Package Enabled State

**Choice:** Store enabled state in `app_settings` table (not filesystem).

**Rationale:**
- Filesystem-based skills don't have a natural place for mutable state
- `app_settings` already exists and handles key-value storage
- Key format: `skill_folder_enabled_{folder_name}`
- Default: enabled (true)
- `list_skill_folders_with_state()` merges filesystem data with DB state

## Open Questions

1. **Skill versioning:** Should we track versions for breaking config changes? → Defer; handle via migration if needed
2. **Skill limits:** Maximum skills per user? → Start unlimited; add limit if performance degrades
3. **Team sharing scope:** Current design is all-or-nothing shared. Need project-level sharing? → Defer to post-MVP
