# Connect Meridian to Claude Code

Let Claude Code access your Meridian tasks, meetings, and projects — ask questions like "what's overdue?" or "help me with this task" and get context-aware assistance.

---

## Prerequisites

- **Meridian app** installed and set up with at least one project
- **Claude Code** installed ([download here](https://claude.ai/code))
- **Rust toolchain** installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

---

## Setup (5 minutes)

### Step 1: Build the MCP Server

Open Terminal and run:

```bash
cd /path/to/meridian
npm run build:mcp
```

This creates the server binary. Note the path shown — you'll need it next.

### Step 2: Add to Claude Code

In Terminal, run:

```bash
claude mcp add meridian /path/to/meridian/src-tauri/target/release/meridian-mcp
```

Replace `/path/to/meridian` with your actual Meridian folder path.

### Step 3: Approve the Server

Next time you start Claude Code, it will ask to approve the "meridian" MCP server. Click **Approve**.

That's it! Claude Code can now access your Meridian data.

---

## What You Can Ask

Once connected, try these in Claude Code:

| You say | What happens |
|---------|--------------|
| "What tasks are overdue?" | Shows tasks past their due date |
| "List my high priority tasks" | Filters by priority |
| "What's assigned to me?" | Filters by your name |
| "Help me with task X" | Gets full context including meeting notes |
| "Summarize yesterday's standup meeting" | Retrieves meeting transcript and summary |
| "Draft a status update from my recent tasks" | Composes update from your task activity |
| "What did we decide about the auth feature?" | Searches meetings for relevant discussions |

---

## Verify It's Working

In Claude Code, type:

```
List my Meridian projects
```

You should see your projects with task counts. If not, see Troubleshooting below.

---

## Troubleshooting

### "meridian" server not showing up

1. Check the binary exists:
   ```bash
   ls /path/to/meridian/src-tauri/target/release/meridian-mcp
   ```

2. Re-add it:
   ```bash
   claude mcp add meridian /path/to/meridian/src-tauri/target/release/meridian-mcp
   ```

### "No projects found" or empty results

The MCP server reads from `~/.meridian/meridian.db`. Make sure:
- You've opened the Meridian app at least once
- You have at least one project created

### Server errors

Test the server directly:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | /path/to/meridian-mcp
```

Should return JSON with `protocolVersion`. If it errors, rebuild with `npm run build:mcp`.

---

## Updating

After pulling new Meridian updates, rebuild the server:

```bash
cd /path/to/meridian
git pull
npm run build:mcp
```

No need to re-add to Claude Code — it uses the same binary path.
