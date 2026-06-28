# Meridian MCP Server — Product Requirements Document

## Executive Summary

Meridian is a local-first meeting intelligence app that extracts tasks from transcripts. This PRD describes **Meridian MCP Server** — a Model Context Protocol interface that exposes Meridian's data to external AI agents (Claude Code, Cursor, or any MCP-compatible client).

Instead of building intelligence inside Meridian, we expose structured data and let powerful external agents help users execute their work: drafting communications, breaking down tasks, suggesting priorities, and providing context-aware assistance.

---

## Problem Statement

Users extract tasks from meetings but then must:
1. **Manually triage** — figure out what's urgent, what's stale, what needs attention
2. **Context-switch** — leave Meridian to draft follow-ups, write code, or communicate with teammates
3. **Lose context** — when working in other tools, they don't have meeting transcript, related tasks, or project background readily available

### Current Pain Points

| Pain | Example |
|------|---------|
| "What should I work on next?" | User scans task list manually, no prioritization help |
| "I need to send a standup update" | User copies task titles into Slack/email manually |
| "This task is vague, what did we actually discuss?" | User hunts for the meeting transcript |
| "Are there duplicate tasks?" | No automated detection |
| "Who's overloaded?" | User counts tasks per assignee manually |

---

## Solution: MCP-First Architecture

### Design Principle

> **Meridian stores and structures. External agents reason and act.**

Rather than building a mediocre internal agent, we expose Meridian's rich data via MCP and let best-in-class agents (Claude, GPT, etc.) provide intelligence. This gives users:

- **Better reasoning** — frontier models vs. constrained local inference
- **Flexibility** — use any MCP-compatible agent
- **Composability** — agents can combine Meridian data with filesystem, web, and other MCP servers
- **Simpler codebase** — Meridian focuses on data, not AI orchestration

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        User's Machine                           │
│                                                                 │
│  ┌─────────────┐         MCP (stdio)         ┌───────────────┐ │
│  │ Claude Code │◄───────────────────────────►│ meridian-mcp  │ │
│  │   (Agent)   │                             │   (Server)    │ │
│  └─────────────┘                             └───────┬───────┘ │
│         │                                            │         │
│         │ Also accesses:                             ▼         │
│         │ • Filesystem                      ~/.meridian/       │
│         │ • Web (WebSearch)                  meridian.db       │
│         │ • Other MCP servers                                  │
│                                                                 │
│  ┌─────────────┐                             ┌───────────────┐ │
│  │  Meridian   │         (optional)          │   Same DB     │ │
│  │  Tauri App  │◄───────────────────────────►│   (SQLite)    │ │
│  └─────────────┘                             └───────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Key insight:** `meridian-mcp` reads the same SQLite database as the Tauri app. No sync needed. The agent always sees current data.

---

## User Stories

### Story 1: Context-Rich Task Assistance
> "As a developer, I want to ask Claude Code for help with a specific task and have it automatically know the meeting context, project background, and related information."

**Flow:**
1. User in Claude Code: "Help me with task abc-123 from Meridian"
2. Agent calls `get_task_context("abc-123")`
3. Agent receives: task details + source meeting transcript + project info
4. Agent provides contextual help (code suggestions, breakdown, draft PR description)

### Story 2: Daily Standup Draft
> "As a team lead, I want to generate a standup update based on my recent task activity without manual copy-paste."

**Flow:**
1. User: "Draft my standup update from Meridian tasks"
2. Agent calls `search("my tasks updated in last 24 hours")`
3. Agent drafts: "Yesterday: Completed X, Y. Today: Working on Z. Blockers: None."
4. User edits and sends

### Story 3: Workload Analysis
> "As a project manager, I want to understand team workload distribution across a project."

**Flow:**
1. User: "Show me workload distribution for Project Alpha"
2. Agent calls `meridian://tasks?project_id=alpha&status=open`
3. Agent analyzes assignee distribution, due dates, priorities
4. Agent provides summary: "Alice: 12 tasks (3 critical), Bob: 4 tasks, Unassigned: 7 tasks"

### Story 4: Meeting Follow-up Draft
> "As a meeting organizer, I want to draft a follow-up email summarizing decisions and action items from a meeting."

**Flow:**
1. User: "Draft follow-up email for yesterday's sprint planning meeting"
2. Agent calls `search("sprint planning meeting")` → gets meeting ID
3. Agent calls `meridian://meetings/{id}` → gets transcript, summary, extracted tasks
4. Agent drafts email with decisions and assigned action items

### Story 5: Find Stale or At-Risk Items
> "As a user, I want to identify tasks that need attention (overdue, stale, unassigned critical)."

**Flow:**
1. User: "What needs attention in Meridian?"
2. Agent calls `search("overdue tasks")`, `search("stale tasks")`, `search("unassigned critical")`
3. Agent synthesizes: "3 overdue tasks, 2 critical tasks without owners, 5 tasks untouched for 2 weeks"

---

## MCP Server Specification

### Resources (Read-Only Data)

Resources provide direct access to Meridian entities via URI patterns.

| Resource | URI Pattern | Description |
|----------|-------------|-------------|
| **Projects list** | `meridian://projects` | All projects with metadata |
| **Project detail** | `meridian://projects/{id}` | Single project + stats (task counts by status) |
| **Tasks list** | `meridian://tasks` | All tasks (supports query params) |
| **Tasks filtered** | `meridian://tasks?project_id=X&status=open&priority=high&assignee=Y` | Filtered task list |
| **Task detail** | `meridian://tasks/{id}` | Single task with full metadata |
| **Meetings list** | `meridian://meetings` | All meetings (supports query params) |
| **Meetings filtered** | `meridian://meetings?project_id=X` | Filtered meeting list |
| **Meeting detail** | `meridian://meetings/{id}` | Meeting with transcript + summary |
| **Documents list** | `meridian://documents?project_id=X` | Documents in project's doc folder |
| **Document content** | `meridian://documents/{id}` | Document text content |

#### Query Parameters for Tasks

| Param | Type | Description |
|-------|------|-------------|
| `project_id` | string | Filter by project |
| `status` | string | `open`, `in_progress`, `done`, `cancelled` |
| `priority` | string | `low`, `medium`, `high`, `critical` |
| `assignee` | string | Filter by assignee name (partial match) |
| `due_before` | date | Tasks due before this date |
| `due_after` | date | Tasks due after this date |
| `updated_after` | datetime | Tasks updated since this time |
| `search` | string | Full-text search in title/description |

### Tools (Actions)

Tools allow the agent to perform operations beyond reading.

| Tool | Parameters | Description | Phase |
|------|------------|-------------|-------|
| `search` | `query: string` | Natural language search across tasks + meetings using FTS5 | 1 |
| `get_task_context` | `task_id: string` | Rich context bundle: task + source meeting + project + linked docs | 1 |
| `get_project_summary` | `project_id: string` | Project overview: description, task stats, recent activity | 1 |
| `update_task` | `task_id, updates: {status?, notes?, assignee?}` | Update task fields | 2 |
| `create_task` | `project_id, task: {title, description?, ...}` | Create new task | 2 |
| `add_task_note` | `task_id, note: string` | Append to task notes | 2 |

### Tool: `search`

**Purpose:** Natural language search that maps to appropriate filters and FTS queries.

**Input:**
```json
{
  "query": "overdue tasks assigned to Alice in Project Alpha"
}
```

**Behavior:**
1. Parse intent: tasks, filtered by due_date < today, assignee contains "Alice", project = "Alpha"
2. Execute SQL with FTS5 for any free-text portions
3. Return matching entities

**Output:**
```json
{
  "results": [
    { "type": "task", "id": "...", "title": "...", "relevance": 0.95 },
    ...
  ],
  "query_interpretation": "Tasks where due_date < 2026-06-28 AND assignee LIKE '%Alice%' AND project_id = 'alpha'"
}
```

### Tool: `get_task_context`

**Purpose:** Bundle all relevant context for a task into a single response.

**Input:**
```json
{
  "task_id": "abc-123"
}
```

**Output:**
```json
{
  "task": {
    "id": "abc-123",
    "title": "Implement user authentication",
    "description": "Set up OAuth2 flow with Google...",
    "status": "in_progress",
    "priority": "high",
    "assignee": "Vaibhav",
    "due_date": "2026-07-01",
    "tags": ["backend", "security"],
    "notes": "Started looking at passport.js...",
    "created_at": "2026-06-25T10:00:00Z",
    "updated_at": "2026-06-27T15:30:00Z"
  },
  "source_meeting": {
    "id": "mtg-456",
    "title": "Sprint Planning - Week 26",
    "meeting_at": "2026-06-25T09:00:00Z",
    "transcript_excerpt": "...Vaibhav will handle the auth implementation. We discussed using OAuth2 with Google as the primary provider, with fallback to email/password. Timeline is end of next week...",
    "decisions": ["Use OAuth2 with Google", "Fallback to email/password", "Due by July 1st"],
    "attendees": ["Vaibhav", "Alice", "Bob"]
  },
  "project": {
    "id": "proj-789",
    "name": "Meridian",
    "description": "AI-powered meeting intelligence app"
  },
  "related_documents": [
    { "id": "doc-1", "name": "AUTH_SPEC.md", "path": "docs/AUTH_SPEC.md" }
  ]
}
```

---

## Security & Privacy

### Authentication Model

**Local-only:** If you can read `~/.meridian/meridian.db`, you're authorized. The MCP server runs locally with the same permissions as the user.

**No network exposure:** The MCP server communicates via stdio (standard input/output) — no TCP ports opened.

### Data Handling

- All data stays local — no cloud sync, no telemetry
- External agents (Claude, etc.) may send data to their APIs for inference, but this is under user control via their agent configuration
- MCP server itself makes no outbound network calls

### Write Operations (Phase 2)

When we add write tools (`update_task`, `create_task`):
- Operations are logged to `~/.meridian/logs/mcp_audit.log`
- No confirmation prompts (agent already has user trust via MCP config)
- Conflicts with GUI are handled via SQLite's built-in locking

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Adoption** | 50% of active Meridian users configure MCP | MCP config file exists |
| **Usage** | 10+ MCP queries per user per week | MCP server logs |
| **Context quality** | Users report task context is "helpful" | Survey |
| **Time saved** | Reduce standup prep time by 5 min/day | User interviews |

---

## Phases

### Phase 1: Read-Only MCP Server (This Document)

**Scope:**
- `meridian-mcp` binary in workspace
- Resources: projects, tasks, meetings (with filtering)
- Tools: `search`, `get_task_context`, `get_project_summary`
- Documentation for Claude Code setup

**Not in Phase 1:**
- Write operations
- Document content access
- Semantic/embedding search

### Phase 2: Write Operations

**Scope:**
- Tools: `update_task`, `create_task`, `add_task_note`
- Audit logging
- Conflict handling with GUI

### Phase 3: Rich Context

**Scope:**
- Document content via `meridian://documents/{id}`
- Embedding-powered semantic search
- "Related tasks" via similarity
- Meeting transcript excerpts (relevant portions only)

### Future: Internal Agent Integration

The proactive agent described in `PROACTIVE_AGENT_SPEC.md` could use MCP internally:
- Agent detects issues (stale tasks, etc.)
- Agent generates suggestions with `get_task_context`
- Suggestions include rich context for handoff to external agents

---

## Open Questions

1. **Should we support multiple concurrent agents?** SQLite handles concurrent reads fine; writes would need coordination.

2. **How do we handle large transcripts?** Full meeting transcripts can be 50KB+. Should `get_task_context` include full transcript or excerpts?

3. **Should we expose notifications/suggestions?** The internal agent writes to `agent_suggestions` — should MCP expose these too?

4. **Rate limiting?** Probably not needed for local use, but worth considering if agents get chatty.

---

## Appendix: Example Claude Code Session

```
User: What's overdue in my Meridian tasks?

Claude: I'll check your Meridian tasks.
[Calls meridian-mcp search tool with "overdue tasks"]

Based on your Meridian data, you have 3 overdue tasks:

1. **Update API documentation** (Project: Backend)
   - Due: June 25 (3 days overdue)
   - Assigned to: You
   - From meeting: "API Review" on June 20

2. **Review PR #142** (Project: Frontend) 
   - Due: June 26 (2 days overdue)
   - Assigned to: You
   - No linked meeting

3. **Send Q2 report to stakeholders** (Project: Admin)
   - Due: June 27 (1 day overdue)
   - Assigned to: You
   - From meeting: "Quarterly Planning"

Would you like me to help draft the Q2 report, or break down any of these tasks?
```

```
User: Help me with the API documentation task

Claude: [Calls get_task_context for that task]

I found the context for this task. It came from your "API Review" meeting on June 20 where the team discussed:

- The `/users` endpoint needs examples for pagination
- Authentication section is missing OAuth2 flow diagrams  
- Error responses need standardization (currently inconsistent)

The relevant doc is at `docs/API.md`. Would you like me to:
1. Draft the pagination examples?
2. Create a text description for the OAuth2 flow?
3. Propose a standard error response format?
```
