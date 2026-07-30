## Why

Meridian skills are currently created manually within the app. Teams often have reusable automation defined in their repositories (`.claude/skills/` or `.agents/skills/`) that could be imported. Additionally, skills that generate files (PDFs, documents, code) need secure sandboxed execution with proper file management. This phase enables skill sharing via Git repositories and adds secure code execution with file output.

## What Changes

- **GitHub Skill Sync**: Import skills from connected repos' `.claude/skills/` or `.agents/skills/` directories
- **Selective Import**: User picks which skills to import; supports nested directories with scripts, templates, assets
- **Fork-on-Import**: Skills copied locally to `~/.meridian/skills/{name}/`; sync pulls updates with binary conflict resolution (keep yours or replace with repo)
- **Naming Conflicts**: Block import if skill name exists; allow rename during import via UI
- **Sandboxed Execution**: Containerized script execution with network allowlist or explicit full access opt-in
- **File Output**: Generated files stored in `~/.meridian/created_files/{date}/` with timestamp suffix
- **File Preview**: AI chat shows file preview and download option for generated files
- **Trust Model**: Trust persists until skill content changes (auto-revokes) plus manual revoke in settings
- **Multi-Skill Selection**: AI autonomously selects relevant skills from frontmatter during conversation
- **Chat-to-Skill**: Convert multi-turn conversations into skills with AI asking clarifying questions; AI proactively suggests skill creation when detecting reusable pattern
- **Default Approval Mode**: New skills default to manual approval per global autonomy settings
- **Repo Watch Skill**: Built-in template skill for monitoring repository changes

## Capabilities

### New Capabilities
- `skill-sync`: Import skills from GitHub repos with selective, fork-on-import model; block + rename on naming conflict
- `skill-sandbox`: Containerized execution environment with configurable network access (allowlist or explicit full)
- `file-output`: Structured file storage with date-based organization and preview support
- `chat-to-skill`: Multi-turn conversation extraction into Anthropic-standard skill format; proactive AI suggestion when reusable pattern detected
- `multi-skill-execution`: AI autonomously chains multiple skills based on frontmatter matching
- `skill-defaults`: New skills default to manual approval per global autonomy settings

### Modified Capabilities
- `skills`: Add sync status, trust state, network permissions to skill metadata
- `ai-chat`: Display file previews inline; trigger skill creation from conversation
- `skill-execution`: Route through sandbox when scripts present; capture file outputs

## Technical Specifications

### Execution Environment
- **Primary**: Docker container when available (detected at runtime via `docker info`)
- **Fallback for non-Docker environments**:
  - macOS: Use `sandbox-exec` with deny-network, deny-file-write-except-output profile
  - Linux: Use `firejail` or `bubblewrap` if installed; otherwise process isolation with restricted PATH
  - Windows: Process isolation with restricted token; warn user about limited sandboxing
- **Resource Limits**: 60 second timeout, 512MB memory limit, single CPU core, no fork allowed
- **Network Modes**:
  - `none` (default): No network access
  - `allowlist`: Skill declares domains in frontmatter; only those allowed
  - `full`: Explicit user opt-in with warning; requires re-approval on skill update

### Git Authentication
- **Private Repos**: Reuse OAuth token from existing GitHub integration; if integration not connected, prompt to connect first
- **Sync Strategy**: Manual "Sync now" button (primary); optional polling interval (15min/1hr/daily) configurable per-repo
- **Change Detection**: Compare local `.origin` commit SHA with remote HEAD; badge shows "Update available" when different

### Skill Discovery & Import
- **Discovery**: In connected repo settings, show "Skills found: N" with "Browse & Import" button; badge on repos with importable skills
- **Size Limits**: Warn if skill package > 10MB; block if > 50MB; show size before import
- **Diff View**: Before sync update, show files changed since last sync with add/modify/delete indicators

### Multi-Skill Execution
- **Selection**: AI reads `name` and `description` from frontmatter; selects skills where description matches user intent (embedding similarity > 0.7)
- **Ordering**: Skills executed in order of relevance score; if skill A declares `depends_on: B` in frontmatter, B runs first
- **Fallback**: If no skill matches confidently (all < 0.7), AI asks user "Did you mean one of these?" with top 3 options

### Chat-to-Skill Quality
- **Minimum Requirements**: At least 3 conversation turns before suggesting skill creation; user must have expressed a repeatable pattern
- **Clarification Questions**: AI asks about trigger, scope, required inputs, output format before generating
- **Generated Skill Review**: Always show full skill.md preview before saving; user can edit before confirming

### First-Run Experience
- **Starter Templates**: Besides "Repo Watch", include 3 more templates: "Weekly Status Report", "Meeting Prep", "Task Cleanup"
- **Import Prompt**: If connected repos have skills, show "Found N skills in your repos. Import them?" on first visit to Skills page

## Impact

- **Database**: Extend `skills` with `sync_source`, `sync_commit`, `trust_state`, `network_mode`, `last_sync_check` columns
- **Backend**: New `src-tauri/src/skills/sync.rs` for Git operations; `src-tauri/src/skills/sandbox.rs` for containerized execution with platform fallbacks; file output management
- **Frontend**: SkillImportWizard for repo browsing and selection with size display; SkillTrustSettings; FilePreviewCard in AI chat; ChatToSkillDialog with clarification flow; SyncDiffView before updates; starter template cards
- **MCP**: Add `queue_skill`, `get_skill_result`, `bulk_create_tasks` tools
- **Security**: Sandbox isolation critical; platform-appropriate fallbacks; network allowlist enforcement; no filesystem escape; size limits on imports
