# Meridian MCP Server Setup

The Meridian MCP server exposes your tasks, meetings, and projects to AI agents like Claude Code.

## Quick Setup

### 1. Build the MCP Server

```bash
cd src-tauri
cargo build --release -p meridian-mcp
```

The binary will be at `src-tauri/target/release/meridian-mcp`.

### 2. Configure Claude Code

The project includes a `.mcp.json` file that configures the MCP server.

When you open this project in Claude Code, it will prompt you to approve the MCP server.

Alternatively, add to your global MCP config or use `claude mcp add`:

```bash
claude mcp add meridian /path/to/meridian/src-tauri/target/release/meridian-mcp
```

### 3. Approve & Use

After approval, Claude Code can access your Meridian data.

---

## Available Tools

### `list_projects`
List all Meridian projects with task counts.

### `list_tasks`
List tasks with optional filters:
- `project_id` — Filter by project
- `status` — `open`, `in_progress`, `done`, `cancelled`
- `priority` — `low`, `medium`, `high`, `critical`
- `assignee` — Filter by name (partial match)
- `due_before` / `due_after` — Date filters (YYYY-MM-DD)
- `text_search` — Full-text search
- `limit` — Max results (default 100)

### `get_task`
Get detailed task info including project name and meeting title.

### `list_meetings`
List meetings with optional filters:
- `project_id` — Filter by project
- `include_archived` — Include archived meetings

### `get_meeting`
Get full meeting details including transcript and extracted tasks.

### `get_task_context`
Get rich context for a task:
- Full task details
- Source meeting with relevant transcript excerpt
- Project information

---

## Example Usage

In Claude Code:

```
"What are my overdue tasks?"
→ Claude calls list_tasks with due_before filter

"Help me with task abc-123"
→ Claude calls get_task_context to get full context

"Draft a standup update from my recent tasks"
→ Claude calls list_tasks filtered by updated_at

"Summarize the sprint planning meeting"
→ Claude calls list_meetings then get_meeting
```

---

## Troubleshooting

### Check server is working
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./meridian-mcp
```

Should return a JSON response with `protocolVersion`.

### Check database path
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_projects","arguments":{}}}' | ./meridian-mcp 2>&1
```

The stderr will show the database path being used.

### Logs
The server logs to stderr. In Claude Code, check the MCP server logs for errors.
