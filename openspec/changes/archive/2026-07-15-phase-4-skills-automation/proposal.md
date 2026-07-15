## Why

Meridian can now learn patterns and make suggestions (Phase 2-3), but users still manually execute repetitive workflows. Skills transform Meridian from a reactive assistant into a proactive automation engine — users define "what should happen when X occurs" and Meridian handles the rest with appropriate oversight. This follows Anthropic's agent skills standard for progressive disclosure, clear triggers, and graceful degradation.

## What Changes

- **Skill Engine Core**: Database schema for skills with trigger/context/action/approval configuration. Runtime execution with audit logging.
- **Trigger System**: Three trigger types — schedule (cron), event (task_created, meeting_imported, etc.), manual (user-initiated).
- **Built-in Skills**: Ship 5 production-ready skills — Weekly Summary, Meeting Follow-up, Overdue Alert, Sprint Prep, End of Day.
- **Skill Editor UI**: Form-based editor with progressive disclosure (basic → advanced), cron builder, test run capability.
- **Chat-to-Skill**: Natural language → structured skill definition via AI, with preview and confirmation.
- **Skill Execution**: Background daemon runs scheduled/event skills, maintains execution history, respects autonomy settings.
- **Skill Sharing**: Mark skills as "shared" for team visibility, clone and customize pattern.
- **Skill Export/Import**: Directory-based packages with `skill.md` (YAML+MD format) for portability.
- **Folder Packages**: Upload skill directories with scripts, configs, supporting files to `~/.meridian/skills/`.
- **AI Chat Integration**: LLM dynamically invokes skills based on user intent; progressive disclosure of skill content.

## Capabilities

### New Capabilities

- `skill-definition`: Skill data model following Anthropic skills standard — trigger, context, action, approval modes, metadata
- `skill-triggers`: Schedule (cron), event-based, and manual trigger types with validation and scheduling
- `skill-context`: Scope configuration (project/global), document inclusion, custom instructions
- `skill-actions`: Action types (summarize, draft_message, create_tasks, analyze, custom) with output formats
- `skill-execution`: Background execution, job queue, status tracking ✓ (retry logic and timeout handling not implemented)
- `skill-history`: Execution logs with output, errors, timing, approval decisions
- `skill-editor-ui`: Form-based editor with Basic/YAML modes, trigger type cards, cron presets, dry-run testing
- `chat-to-skill`: AI-powered natural language to skill definition conversion
- `skill-sharing`: ~~Team visibility toggle, cloning, ownership tracking~~ **[PARKED]** — UI exists but no actual sharing mechanism (local-first app)
- `builtin-skills`: Pre-configured skills shipped with app (Weekly Summary, Meeting Follow-up, etc.)
- `folder-packages`: Upload skill directories containing scripts, configs, and supporting files
- `ai-chat-skills`: LLM analyzes user intent and invokes matching skills; progressive content loading
- `unified-skill-picker`: Single picker showing both DB skills (⚡) and folder packages (📦)
- `skill-deduplication`: Prevent duplicate invocations within same conversation

### Modified Capabilities

- `suggestion-engine`: ~~Skills can generate suggestions; suggestion acceptance can trigger skill runs~~ **[NOT IMPLEMENTED]** — schema ready (`skill_run_id` column) but no code
- `notification-system`: Skill outputs can create notifications with appropriate severity ✓
- `pattern-observation`: Skill corrections recorded as observations for learning ✓

## Impact

- **Database**: New `skills`, `skill_runs` tables; FK from skill_runs to skills; `is_builtin` flag for protection
- **Daemon**: Extended job types for skill execution; event listener for trigger matching
- **Frontend**: Skills section with unified view (DB + folders), Skill Editor (Basic/YAML modes), history panel, AI Chat integration
- **AI Module**: Chat-to-skill extraction; skill context injection; dynamic invocation markers
- **AI Chat**: Compact skill context sent to LLM; progressive content loading; unified execution path
- **Filesystem**: `~/.meridian/skills/` for folder packages; enabled state in `app_settings`
- **Audit**: All skill executions logged with context and approval decisions
- **Settings**: Skill-specific autonomy overrides; per-skill enable/disable; folder package management
