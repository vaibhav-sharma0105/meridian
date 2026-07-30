## Why

Meridian users often need to take action on tasks that require coding, debugging, or technical investigation. Currently they must context-switch between Meridian (for task context) and Claude Code (for execution). This phase embeds Claude Code terminals directly in Meridian, passing rich task context and enabling work without context loss.

## What Changes

- **Embedded Terminals**: Claude Code sessions run inside Meridian UI, not separate windows
- **Task-Scoped Sessions**: Launch terminal from any task; session receives task context automatically
- **Max Concurrent Sessions**: Up to 8 terminal sessions can run simultaneously
- **Rich Initial Context**: Task details, project info, linked integration items, relevant documents passed at launch
- **User-Configurable Context**: User can select what context to include before launching
- **MCP Refresh**: Claude Code can pull fresh context from Meridian via MCP tools during session
- **Change Indicator**: Terminal tab shows badge when Meridian data has changed since launch
- **Session Lifecycle**: Manual close + configurable idle timeout; sessions persist across task state changes
- **Repo Context**: Terminal sessions invoked from Meridian's current repo context; Claude Code operates on the same codebase

## Capabilities

### New Capabilities
- `embedded-terminal`: Claude Code sessions running within Meridian UI from current repo context
- `terminal-context`: Rich, configurable context passing from Meridian to Claude Code
- `terminal-sync`: Change indicator and MCP-based context refresh
- `session-management`: Multi-session orchestration with lifecycle controls (manual close + idle timeout)

### Modified Capabilities
- `mcp-server`: Add introspection tools for Claude Code context needs

## Technical Specifications

### Architecture Decision: Required Spike

**Problem**: Embedding terminal in Tauri is non-trivial; multiple approaches with different tradeoffs.

**Options to Evaluate in Spike:**

| Approach | Pros | Cons | Spike Tasks |
|----------|------|------|-------------|
| **WebView + xterm.js** | Web-standard, rich features | Memory overhead, IPC complexity | Build POC with Tauri webview |
| **Native PTY + custom renderer** | Lightweight, native feel | Complex to build, platform-specific | Evaluate portable-pty crate |
| **Spawn external window** | Simplest, Claude Code handles UI | Context switch, less integrated | Test with `--context` flag |
| **WebSocket to Claude Code server** | Decoupled, existing infra | Requires Claude Code server mode | Check if API exists |

**Spike Deliverable**: Working POC of preferred approach; architecture decision document; performance benchmarks

### Resource Management

**Memory Limits:**
- Monitor per-session memory usage via Tauri's resource APIs
- Warning at 200MB per session; hard limit at 500MB
- Total terminal memory budget: 2GB across all sessions

**Idle Suspension:**
- After 10 minutes of no input: suspend session (freeze process, keep state)
- Visual indicator: "Session paused — click to resume"
- On resume: restore in <2 seconds
- After 30 minutes suspended: prompt "Keep or close?"

**Session Caps:**
- Hard limit: 8 concurrent active sessions
- Soft limit: 4 sessions before warning "You have 4 terminals open. Close unused?"
- New session when at limit: "Close a session to open a new one" with session list

### Session Persistence

**Across App Restart:**
- Store session state in `terminal_sessions` table: task_id, repo_path, working_directory, scroll_position
- On app launch: "Restore 3 terminal sessions?" prompt
- If restored: session resumes with note "Restored from previous session. Context may be stale — refresh if needed."

**State Stored:**
- Working directory
- Environment variables (excluding secrets)
- Last 1000 lines of scrollback
- Associated task/project IDs

**Not Stored:**
- Running processes (killed on app close)
- Credentials (re-authenticated on restore)

### Context Format

**Serialization**: CLAUDE.md-style markdown document

```markdown
# Task Context from Meridian

## Task: Fix login API bug
- Status: In Progress
- Priority: High
- Assignee: You
- Due: 2026-08-05

## Project: Acme Backend
- Path: /Users/you/projects/acme-backend
- Recent activity: 5 tasks completed this week

## Linked Items
- GitHub Issue #142: "Login fails on mobile" (open)
- GitHub PR #156: "Fix session handling" (draft, yours)

## Relevant Documents
- API Authentication Guide (internal doc, 2 pages)
- Login Flow Diagram (embedded)

## Recent Context
- Yesterday: Completed "Add rate limiting to auth endpoint"
- Meeting: "Sprint Planning" discussed auth issues
```

**Context Size**: Max 8000 tokens; truncate by priority (task > linked > docs > history)

### Tool Namespace

**Separation Strategy:**
- Meridian MCP tools prefixed: `meridian_*` (e.g., `meridian_get_task`)
- Claude Code tools: unprefixed (existing behavior)
- On conflict: Meridian tools take precedence in embedded context

**Available Meridian Tools in Terminal:**
- `meridian_get_task` — Refresh current task
- `meridian_list_related_tasks` — Tasks in same project
- `meridian_get_integration_data` — Linked GitHub/Jira items
- `meridian_update_task_status` — Mark task done without leaving terminal
- `meridian_add_task_note` — Add update to task

### Learning Curve Mitigation

**First-Time Experience:**
- On first terminal launch: "Welcome to Claude Code in Meridian" overlay
- 3-step quick tour: "This is your task context", "Claude can help with code", "Use these commands"
- "Don't show again" checkbox

**Help Access:**
- `?` key: Show command palette with common actions
- "Help" button in terminal header
- Contextual hints: "Tip: Ask Claude to explain this error"

**Action Templates:**
- "Fix this bug" — Pre-filled prompt template
- "Write tests for task" — Generates test scaffolding prompt
- "Explain codebase" — Asks Claude to summarize relevant files
- Templates shown in context selector dialog

### Change Indicator Details

**Badge Content:**
- Shows count: "2 updates"
- Tooltip on hover: "Task status changed; 1 new comment"
- Click: Opens diff summary panel

**Diff Preview:**
- Side panel showing what changed since session start
- "Refresh context" button to pull latest into session
- Changes highlighted: additions in green, removals in red

### Task-Terminal Relationship

**Cardinality:**
- One task → Multiple terminals (allowed; different investigations)
- Terminal without task (allowed; general coding session with project context only)
- Same task in multiple terminals: Each independent; no sync between them

**UI Organization:**
- Tabs grouped by task
- Task-less terminals grouped under "General"
- Tab label: Task title (truncated) or "General — Project Name"

## Impact

- **Architecture**: Spike required for embedding decision; session state management; context serialization
- **Database**: New `terminal_sessions` table tracking active sessions, associated tasks, launch context, scroll state, suspended flag
- **Backend**: Terminal spawner and manager; context builder in CLAUDE.md format; MCP extensions for introspection; resource monitor; session persistence/restore
- **Frontend**: TerminalPanel with xterm.js or native renderer; TaskTerminalButton; SessionTabs with grouping; ContextSelector dialog with templates; OnboardingOverlay; ChangeIndicator with diff preview
- **MCP**: Add `get_capabilities`, `get_project_summary`, `get_relevant_context`, `search_semantic`, plus `meridian_*` variants for embedded use
- **Security**: Terminal sessions run in Meridian's repo context; no additional filesystem access beyond user's normal permissions; secrets excluded from persistence
