# Meridian MCP Server — Implementation Plan

## Overview

This document describes the technical implementation of `meridian-mcp`, an MCP server that exposes Meridian's data to external AI agents.

---

## Phase 1 Scope

**Goal:** Ship a read-only MCP server that Claude Code (and other MCP clients) can use to query Meridian data.

**Deliverables:**
1. `meridian-mcp` binary in the workspace
2. MCP resources: projects, tasks, meetings with filtering
3. MCP tools: `search`, `get_task_context`, `get_project_summary`
4. Setup documentation

**Not in scope:**
- Write operations (update/create tasks)
- Document content access
- Semantic search
- Internal proactive agent

---

## Repository Structure

### Current Structure
```
src-tauri/
├── Cargo.toml           # Single package
├── src/
│   ├── lib.rs           # Tauri app lib
│   ├── main.rs          # Tauri entry point
│   ├── commands/        # Tauri IPC commands
│   ├── db/
│   │   ├── connection.rs
│   │   ├── migrations/
│   │   └── repositories/   # SQL query functions
│   └── models/          # Rust structs (Task, Meeting, etc.)
```

### New Structure (Workspace)
```
src-tauri/
├── Cargo.toml           # Workspace root
├── meridian-core/       # Shared library (NEW)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── db/          # Moved from src/db
│       │   ├── connection.rs
│       │   ├── migrations/
│       │   └── repositories/
│       └── models/      # Moved from src/models
│
├── meridian-app/        # Tauri app (RENAMED from src/)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── commands/    # Tauri-specific, uses meridian-core
│       ├── ai/
│       ├── connectors/
│       └── utils/
│
└── meridian-mcp/        # MCP server (NEW)
    ├── Cargo.toml
    └── src/
        ├── main.rs      # MCP server entry point
        ├── server.rs    # MCP protocol handling
        ├── resources.rs # Resource URI handlers
        └── tools.rs     # Tool implementations
```

### Why This Structure?

1. **Code reuse**: Both `meridian-app` and `meridian-mcp` import `meridian-core` for DB access and models
2. **Independent binaries**: MCP server can run without Tauri
3. **Shared schema**: Same model structs ensure consistency
4. **Minimal changes**: Tauri app code moves but doesn't change much

---

## Dependencies

### meridian-core/Cargo.toml
```toml
[package]
name = "meridian-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
dirs-next = "2"
thiserror = "1"
```

### meridian-mcp/Cargo.toml
```toml
[package]
name = "meridian-mcp"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "meridian-mcp"
path = "src/main.rs"

[dependencies]
meridian-core = { path = "../meridian-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Note:** We'll implement MCP protocol manually (it's simple JSON-RPC over stdio) rather than adding a heavy dependency.

---

## MCP Protocol Implementation

### Protocol Basics

MCP uses JSON-RPC 2.0 over stdio:
- Server reads JSON lines from stdin
- Server writes JSON lines to stdout
- Stderr is for logging (not protocol)

### Message Types

```rust
// Incoming requests
enum McpRequest {
    Initialize { ... },
    ListResources,
    ReadResource { uri: String },
    ListTools,
    CallTool { name: String, arguments: Value },
}

// Outgoing responses
struct McpResponse {
    jsonrpc: "2.0",
    id: RequestId,
    result: Option<Value>,
    error: Option<McpError>,
}
```

### Server Lifecycle

```
Client                          Server (meridian-mcp)
   |                                   |
   |-- initialize ------------------>  |
   |<-- initialize result ------------ |
   |                                   |
   |-- initialized ----------------->  |  (notification, no response)
   |                                   |
   |-- resources/list -------------->  |
   |<-- resources list --------------- |
   |                                   |
   |-- resources/read { uri } ------>  |
   |<-- resource contents ------------ |
   |                                   |
   |-- tools/call { name, args } --->  |
   |<-- tool result ------------------ |
   |                                   |
```

---

## Implementation Files

### src/main.rs
```rust
//! Meridian MCP Server
//! 
//! Exposes Meridian's data (tasks, meetings, projects) via the
//! Model Context Protocol for use by external AI agents.

use anyhow::Result;
use tracing_subscriber;

mod server;
mod resources;
mod tools;

fn main() -> Result<()> {
    // Log to stderr (stdout is for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Meridian MCP server");
    
    // Run the server (blocks, reads stdin, writes stdout)
    server::run()
}
```

### src/server.rs
```rust
//! MCP protocol handler — reads JSON-RPC from stdin, dispatches to handlers

use std::io::{BufRead, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::resources;
use crate::tools;

#[derive(Deserialize)]
struct Request {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct Response {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

pub fn run() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() { continue; }
        
        let request: Request = serde_json::from_str(&line)?;
        let response = handle_request(request);
        
        if let Some(resp) = response {
            serde_json::to_writer(&mut stdout, &resp)?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }
    }
    
    Ok(())
}

fn handle_request(req: Request) -> Option<Response> {
    let result = match req.method.as_str() {
        "initialize" => handle_initialize(),
        "initialized" => return None, // notification, no response
        "resources/list" => resources::list(),
        "resources/read" => resources::read(req.params),
        "tools/list" => tools::list(),
        "tools/call" => tools::call(req.params),
        _ => Err(method_not_found()),
    };
    
    Some(Response {
        jsonrpc: "2.0",
        id: req.id,
        result: result.ok(),
        error: result.err(),
    })
}
```

### src/resources.rs
```rust
//! MCP resource handlers — expose Meridian data via URIs

use meridian_core::db::{connection, repositories};
use serde_json::{json, Value};

/// List available resources
pub fn list() -> Result<Value, RpcError> {
    Ok(json!({
        "resources": [
            {
                "uri": "meridian://projects",
                "name": "Projects",
                "description": "List all Meridian projects",
                "mimeType": "application/json"
            },
            {
                "uri": "meridian://tasks",
                "name": "Tasks", 
                "description": "List tasks (supports filtering via query params)",
                "mimeType": "application/json"
            },
            {
                "uri": "meridian://meetings",
                "name": "Meetings",
                "description": "List meetings (supports filtering via query params)",
                "mimeType": "application/json"
            }
        ]
    }))
}

/// Read a specific resource by URI
pub fn read(params: Option<Value>) -> Result<Value, RpcError> {
    let uri = params
        .and_then(|p| p.get("uri").and_then(|u| u.as_str()))
        .ok_or_else(|| invalid_params("missing uri"))?;
    
    let conn = connection::init_db()
        .map_err(|e| internal_error(&e))?;
    
    // Parse URI: meridian://resource/id?params
    let parsed = parse_uri(uri)?;
    
    match parsed.resource.as_str() {
        "projects" => read_projects(&conn, parsed.id, parsed.params),
        "tasks" => read_tasks(&conn, parsed.id, parsed.params),
        "meetings" => read_meetings(&conn, parsed.id, parsed.params),
        _ => Err(invalid_params("unknown resource")),
    }
}

fn read_projects(conn: &Connection, id: Option<&str>, _params: &QueryParams) -> Result<Value, RpcError> {
    if let Some(project_id) = id {
        // Single project with stats
        let project = repositories::projects::get_project(conn, project_id)
            .map_err(|e| internal_error(&e))?;
        let stats = repositories::projects::get_project_stats(conn, project_id)
            .map_err(|e| internal_error(&e))?;
        Ok(json!({ "project": project, "stats": stats }))
    } else {
        // All projects
        let projects = repositories::projects::get_all_projects(conn)
            .map_err(|e| internal_error(&e))?;
        Ok(json!({ "projects": projects }))
    }
}

fn read_tasks(conn: &Connection, id: Option<&str>, params: &QueryParams) -> Result<Value, RpcError> {
    if let Some(task_id) = id {
        // Single task
        let task = repositories::tasks::get_task(conn, task_id)
            .map_err(|e| internal_error(&e))?;
        Ok(json!({ "task": task }))
    } else {
        // Filtered task list
        let filters = TaskFilters {
            project_id: params.get("project_id").cloned(),
            status: params.get("status").cloned(),
            priority: params.get("priority").cloned(),
            assignee: params.get("assignee").cloned(),
            // ... more filters
        };
        let tasks = repositories::tasks::get_all_tasks(conn, &filters)
            .map_err(|e| internal_error(&e))?;
        Ok(json!({ "tasks": tasks }))
    }
}

// Similar for meetings...
```

### src/tools.rs
```rust
//! MCP tool handlers — actions the agent can perform

use meridian_core::db::{connection, repositories};
use serde_json::{json, Value};

/// List available tools
pub fn list() -> Result<Value, RpcError> {
    Ok(json!({
        "tools": [
            {
                "name": "search",
                "description": "Search across tasks and meetings using natural language",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language search query"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_task_context",
                "description": "Get full context for a task: task details, source meeting, project info",
                "inputSchema": {
                    "type": "object", 
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "The task ID"
                        }
                    },
                    "required": ["task_id"]
                }
            },
            {
                "name": "get_project_summary",
                "description": "Get project overview with task statistics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The project ID"
                        }
                    },
                    "required": ["project_id"]
                }
            }
        ]
    }))
}

/// Execute a tool
pub fn call(params: Option<Value>) -> Result<Value, RpcError> {
    let params = params.ok_or_else(|| invalid_params("missing params"))?;
    let name = params.get("name").and_then(|n| n.as_str())
        .ok_or_else(|| invalid_params("missing tool name"))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    
    match name {
        "search" => tool_search(args),
        "get_task_context" => tool_get_task_context(args),
        "get_project_summary" => tool_get_project_summary(args),
        _ => Err(invalid_params("unknown tool")),
    }
}

fn tool_search(args: Value) -> Result<Value, RpcError> {
    let query = args.get("query").and_then(|q| q.as_str())
        .ok_or_else(|| invalid_params("missing query"))?;
    
    let conn = connection::init_db().map_err(|e| internal_error(&e))?;
    
    // Use FTS5 search
    let task_results = repositories::tasks::search_tasks(&conn, query)
        .map_err(|e| internal_error(&e))?;
    let meeting_results = repositories::meetings::search_meetings(&conn, query)
        .map_err(|e| internal_error(&e))?;
    
    Ok(json!({
        "tasks": task_results,
        "meetings": meeting_results,
        "query": query
    }))
}

fn tool_get_task_context(args: Value) -> Result<Value, RpcError> {
    let task_id = args.get("task_id").and_then(|t| t.as_str())
        .ok_or_else(|| invalid_params("missing task_id"))?;
    
    let conn = connection::init_db().map_err(|e| internal_error(&e))?;
    
    // Get task
    let task = repositories::tasks::get_task(&conn, task_id)
        .map_err(|e| internal_error(&e))?
        .ok_or_else(|| invalid_params("task not found"))?;
    
    // Get source meeting if linked
    let meeting = task.meeting_id.as_ref().and_then(|mid| {
        repositories::meetings::get_meeting(&conn, mid).ok().flatten()
    });
    
    // Get project
    let project = repositories::projects::get_project(&conn, &task.project_id)
        .map_err(|e| internal_error(&e))?;
    
    Ok(json!({
        "task": task,
        "source_meeting": meeting,
        "project": project
    }))
}

fn tool_get_project_summary(args: Value) -> Result<Value, RpcError> {
    let project_id = args.get("project_id").and_then(|p| p.as_str())
        .ok_or_else(|| invalid_params("missing project_id"))?;
    
    let conn = connection::init_db().map_err(|e| internal_error(&e))?;
    
    let project = repositories::projects::get_project(&conn, project_id)
        .map_err(|e| internal_error(&e))?
        .ok_or_else(|| invalid_params("project not found"))?;
    
    // Get task counts by status
    let stats = repositories::analytics::get_project_task_stats(&conn, project_id)
        .map_err(|e| internal_error(&e))?;
    
    // Get recent tasks
    let recent_tasks = repositories::tasks::get_recent_tasks(&conn, project_id, 10)
        .map_err(|e| internal_error(&e))?;
    
    Ok(json!({
        "project": project,
        "stats": stats,
        "recent_tasks": recent_tasks
    }))
}
```

---

## Setup Documentation

### For Users: Claude Code Integration

Create/edit `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "meridian": {
      "command": "/path/to/meridian-mcp",
      "args": []
    }
  }
}
```

After restart, Claude Code can access Meridian data:
- "What tasks are overdue in Meridian?"
- "Help me with task abc-123"
- "Show me the sprint planning meeting summary"

### Building the Binary

```bash
cd src-tauri
cargo build --release -p meridian-mcp
# Binary at: target/release/meridian-mcp
```

---

## Migration Path

### Step 1: Create Workspace Structure

1. Create `meridian-core/` with shared code
2. Move `db/` and `models/` to core
3. Update imports in Tauri app

### Step 2: Build MCP Server

1. Create `meridian-mcp/` with MCP protocol handler
2. Implement resources and tools
3. Test with manual JSON-RPC

### Step 3: Integration Testing

1. Test with Claude Code
2. Verify all resource URIs work
3. Verify tools return expected data

### Step 4: Documentation & Release

1. Add setup docs to README
2. Include pre-built binary in releases
3. Update CLAUDE.md with MCP info

---

## Testing Strategy

### Unit Tests
- URI parsing
- Filter construction
- JSON serialization

### Integration Tests
- Full MCP request/response cycle
- Database queries return expected data
- Error handling for missing entities

### Manual Testing with Claude Code
- Configure MCP server
- Try various queries
- Verify context quality

---

## Open Implementation Questions

1. **Should we use async?** The current plan uses sync I/O for simplicity. Async would be needed if we add long-running operations.

2. **Connection pooling?** Currently opens new connection per request. Fine for low-volume MCP use; could pool if needed.

3. **Transcript excerpts:** Full transcripts can be large. Should `get_task_context` include full transcript or just relevant excerpts?

4. **Search implementation:** Phase 1 uses FTS5. Should we also parse query for filters ("overdue" → due_date < today)?
