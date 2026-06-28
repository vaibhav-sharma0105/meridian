# Phase 1 Implementation — Specific Changes

## Design Decisions

| Question | Decision |
|----------|----------|
| Tauri config location | Nest in `meridian-app/` (cleaner, adjust npm scripts) |
| Binary name | Standalone `meridian-mcp` |
| Search implementation | Both FTS5 + smart NL parsing (filters from natural language) |
| Transcript in context | Extract relevant excerpt around task mention (requires schema addition) |

---

## Summary

Convert `src-tauri/` to a Cargo workspace with three crates:
1. **meridian-core** — shared DB and models (extracted from current code)
2. **meridian-app** — Tauri desktop app (current app, updated imports)
3. **meridian-mcp** — MCP server binary (new)

---

## File Changes Overview

### New Files to Create

```
src-tauri/
├── Cargo.toml                    # UPDATE: workspace definition
├── meridian-core/
│   ├── Cargo.toml                # NEW
│   └── src/
│       ├── lib.rs                # NEW: re-exports db + models
│       ├── db/                   # MOVE from src/db/
│       │   ├── mod.rs
│       │   ├── connection.rs
│       │   ├── migrations/
│       │   └── repositories/
│       └── models/               # MOVE from src/models/
│           ├── mod.rs
│           ├── task.rs
│           ├── meeting.rs
│           ├── project.rs
│           └── ...
│
├── meridian-app/
│   ├── Cargo.toml                # NEW (based on current, adds meridian-core dep)
│   ├── build.rs                  # MOVE from src-tauri/build.rs
│   ├── tauri.conf.json           # MOVE from src-tauri/tauri.conf.json
│   ├── capabilities/             # MOVE from src-tauri/capabilities/
│   ├── icons/                    # MOVE from src-tauri/icons/
│   └── src/
│       ├── lib.rs                # MOVE + UPDATE imports
│       ├── main.rs               # MOVE
│       ├── commands/             # MOVE + UPDATE imports
│       ├── ai/                   # MOVE
│       ├── connectors/           # MOVE
│       └── utils/                # MOVE
│
└── meridian-mcp/
    ├── Cargo.toml                # NEW
    └── src/
        ├── main.rs               # NEW
        ├── server.rs             # NEW
        ├── resources.rs          # NEW
        └── tools.rs              # NEW
```

### Files to Delete (after moving)

```
src-tauri/src/db/           # Moved to meridian-core
src-tauri/src/models/       # Moved to meridian-core
src-tauri/src/lib.rs        # Moved to meridian-app
src-tauri/src/main.rs       # Moved to meridian-app
src-tauri/src/commands/     # Moved to meridian-app
src-tauri/src/ai/           # Moved to meridian-app
src-tauri/src/connectors/   # Moved to meridian-app
src-tauri/src/utils/        # Moved to meridian-app
src-tauri/build.rs          # Moved to meridian-app
src-tauri/tauri.conf.json   # Moved to meridian-app
src-tauri/capabilities/     # Moved to meridian-app
src-tauri/icons/            # Moved to meridian-app
```

---

## Detailed Changes

### 1. src-tauri/Cargo.toml (Workspace Root)

**Before:**
```toml
[package]
name = "meridian"
version = "0.1.0"
...

[dependencies]
tauri = ...
rusqlite = ...
...
```

**After:**
```toml
[workspace]
members = ["meridian-core", "meridian-app", "meridian-mcp"]
resolver = "2"

# No [package] section — this is just the workspace root
```

### 2. meridian-core/Cargo.toml (NEW)

```toml
[package]
name = "meridian-core"
version = "0.1.0"
edition = "2021"
description = "Shared database and models for Meridian"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
dirs-next = "2"
thiserror = "1"
```

### 3. meridian-core/src/lib.rs (NEW)

```rust
//! Meridian Core — shared database and model code
//!
//! Used by both the Tauri app and the MCP server.

pub mod db;
pub mod models;

// Re-export commonly used types
pub use db::connection::{get_db_path, init_db};
pub use models::task::{Task, TaskFilters, CreateTaskInput, UpdateTaskInput};
pub use models::meeting::Meeting;
pub use models::project::Project;
```

### 4. meridian-app/Cargo.toml (NEW)

```toml
[package]
name = "meridian-app"
version = "0.1.0"
description = "Meridian - AI-powered meeting intelligence"
authors = ["Meridian"]
edition = "2021"

[lib]
name = "meridian_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
meridian-core = { path = "../meridian-core" }

tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
tauri-plugin-updater = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"

serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
keyring = "2"
anyhow = "1"
thiserror = "1"
zip = "0.6"
sha2 = "0.10"
rand = "0.8"
base64 = "0.22"
urlencoding = "2"

[features]
custom-protocol = ["tauri/custom-protocol"]
```

### 5. Import Changes in meridian-app/src/

**Before (in commands/tasks.rs):**
```rust
use crate::db::repositories::tasks;
use crate::models::task::{Task, TaskFilters};
```

**After:**
```rust
use meridian_core::db::repositories::tasks;
use meridian_core::models::task::{Task, TaskFilters};
// OR use re-exports:
use meridian_core::{Task, TaskFilters};
```

### 6. meridian-mcp/Cargo.toml (NEW)

```toml
[package]
name = "meridian-mcp"
version = "0.1.0"
edition = "2021"
description = "MCP server for Meridian — exposes data to AI agents"

[[bin]]
name = "meridian-mcp"
path = "src/main.rs"

[dependencies]
meridian-core = { path = "../meridian-core" }

serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 7. meridian-mcp/src/main.rs (NEW)

```rust
//! Meridian MCP Server
//!
//! Exposes Meridian's data via the Model Context Protocol.
//! Run this binary and configure it in Claude Code's MCP settings.

use anyhow::Result;

mod server;
mod resources;
mod tools;

fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("Meridian MCP server starting");
    tracing::info!("DB path: {:?}", meridian_core::get_db_path());

    server::run()
}
```

---

## Tauri Configuration Update

The `tauri.conf.json` needs to be updated for the new directory structure.

**Move to:** `meridian-app/tauri.conf.json`

**Update paths:**
```json
{
  "build": {
    "beforeDevCommand": "npm run vite:dev",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../../dist"  // Relative to meridian-app/
  },
  ...
}
```

---

## npm Scripts Update

**package.json changes:**

```json
{
  "scripts": {
    "dev": "cd src-tauri/meridian-app && tauri dev",
    "build": "cd src-tauri/meridian-app && tauri build",
    "build:mcp": "cd src-tauri && cargo build --release -p meridian-mcp"
  }
}
```

---

## New Repository Functions Needed

### meridian-core/src/db/repositories/tasks.rs

**Add these functions (or verify they exist):**

```rust
/// Get a single task by ID
pub fn get_task(conn: &Connection, task_id: &str) -> Result<Option<Task>, String>;

/// Search tasks using FTS5
pub fn search_tasks(conn: &Connection, query: &str) -> Result<Vec<Task>, String>;

/// Get recently updated tasks for a project
pub fn get_recent_tasks(conn: &Connection, project_id: &str, limit: usize) -> Result<Vec<Task>, String>;
```

### meridian-core/src/db/repositories/meetings.rs

**Add:**
```rust
/// Get a single meeting by ID
pub fn get_meeting(conn: &Connection, meeting_id: &str) -> Result<Option<Meeting>, String>;

/// Search meetings using FTS5
pub fn search_meetings(conn: &Connection, query: &str) -> Result<Vec<Meeting>, String>;
```

### meridian-core/src/db/repositories/projects.rs

**Add:**
```rust
/// Get a single project by ID  
pub fn get_project(conn: &Connection, project_id: &str) -> Result<Option<Project>, String>;
```

---

## Testing Plan

### 1. Verify Tauri App Still Works
```bash
npm run dev
# App should launch and work exactly as before
```

### 2. Build MCP Server
```bash
npm run build:mcp
# Should produce: src-tauri/target/release/meridian-mcp
```

### 3. Test MCP Server Manually
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./meridian-mcp
# Should return initialize response
```

### 4. Test with Claude Code
1. Add to `~/.claude/claude_desktop_config.json`
2. Ask Claude: "List my Meridian projects"
3. Verify it returns actual project data

---

## Rollback Plan

If something breaks:
1. The original `src/` directory structure is preserved in git
2. Revert Cargo.toml to non-workspace version
3. Remove `meridian-core/` and `meridian-mcp/` directories
4. Move files back from `meridian-app/` to `src/`

---

## Estimated Effort

| Task | Estimate |
|------|----------|
| Create workspace structure | 30 min |
| Move/split files | 1 hour |
| Update imports in Tauri app | 30 min |
| Schema migration (source positions) | 30 min |
| Update AI extractor for positions | 1 hour |
| Implement MCP server (basic) | 2 hours |
| Implement NL query parser | 2 hours |
| Add missing repository functions | 1 hour |
| Transcript excerpt extraction | 30 min |
| Testing & debugging | 2 hours |
| Documentation | 30 min |
| **Total** | **11-12 hours** |

---

## Schema Addition: Task Source Position

To extract relevant transcript excerpts, we need to track where in the transcript each task was mentioned.

### Migration: v007_task_source_position.rs

```sql
ALTER TABLE tasks ADD COLUMN source_start_char INTEGER;
ALTER TABLE tasks ADD COLUMN source_end_char INTEGER;
```

### AI Extractor Update

When extracting tasks from transcripts, the AI should return character positions:

```json
{
  "title": "Set up CI pipeline",
  "source_quote": "John mentioned we need CI...",
  "source_start_char": 2450,
  "source_end_char": 2580
}
```

### Excerpt Extraction Logic

```rust
fn extract_transcript_excerpt(
    transcript: &str, 
    start: usize, 
    end: usize,
    context_chars: usize  // e.g., 500 chars before/after
) -> String {
    let excerpt_start = start.saturating_sub(context_chars);
    let excerpt_end = (end + context_chars).min(transcript.len());
    
    let mut excerpt = String::new();
    if excerpt_start > 0 {
        excerpt.push_str("...");
    }
    excerpt.push_str(&transcript[excerpt_start..excerpt_end]);
    if excerpt_end < transcript.len() {
        excerpt.push_str("...");
    }
    excerpt
}
```

### Fallback for Existing Tasks

Tasks without `source_start_char` will fall back to:
1. Using `assignee_source_quote` or `due_source_quote` if available
2. First 1000 chars of transcript as generic context

---

## Smart Search: Natural Language Query Parsing

The `search` tool will parse natural language queries into structured filters + FTS.

### Query Parser Examples

| Query | Parsed Filters |
|-------|----------------|
| "overdue tasks" | `due_date < today, status != done` |
| "tasks assigned to Alice" | `assignee LIKE '%Alice%'` |
| "high priority in Project X" | `priority IN (high, critical), project_name = 'Project X'` |
| "stale tasks" | `updated_at < 7 days ago, status = open` |
| "unassigned critical" | `assignee IS NULL, priority = critical` |
| "tasks from last week's meetings" | `meeting.meeting_at > 7 days ago` |
| "what did we decide about auth" | FTS: "decide auth" (no filters, pure search) |

### Implementation Approach

```rust
struct ParsedQuery {
    filters: TaskFilters,
    fts_query: Option<String>,  // Remaining text for full-text search
    date_context: Option<DateRange>,
}

fn parse_nl_query(query: &str) -> ParsedQuery {
    let mut filters = TaskFilters::default();
    let mut remaining = query.to_lowercase();
    
    // Pattern matching for common phrases
    if remaining.contains("overdue") {
        filters.due_before = Some(today());
        filters.status_not = Some("done".into());
        remaining = remaining.replace("overdue", "");
    }
    
    if remaining.contains("unassigned") {
        filters.assignee_null = Some(true);
        remaining = remaining.replace("unassigned", "");
    }
    
    // Regex for "assigned to X"
    if let Some(caps) = ASSIGNED_TO_RE.captures(&remaining) {
        filters.assignee = Some(caps[1].to_string());
        remaining = ASSIGNED_TO_RE.replace(&remaining, "").to_string();
    }
    
    // Priority keywords
    for p in ["critical", "high", "medium", "low"] {
        if remaining.contains(p) {
            filters.priority = Some(p.into());
            remaining = remaining.replace(p, "");
            break;
        }
    }
    
    // Time phrases
    if remaining.contains("last week") { ... }
    if remaining.contains("this month") { ... }
    if remaining.contains("stale") { 
        filters.updated_before = Some(days_ago(7));
    }
    
    // What's left goes to FTS
    let fts = remaining.trim();
    let fts_query = if fts.is_empty() { None } else { Some(fts.into()) };
    
    ParsedQuery { filters, fts_query, date_context: None }
}
```

### Combined Execution

```rust
fn tool_search(args: Value) -> Result<Value, RpcError> {
    let query = args.get("query").and_then(|q| q.as_str()).unwrap();
    let conn = init_db()?;
    
    let parsed = parse_nl_query(query);
    
    // Get tasks matching filters
    let mut tasks = get_all_tasks(&conn, &parsed.filters)?;
    
    // If there's FTS query, filter further
    if let Some(fts) = &parsed.fts_query {
        let fts_ids = search_tasks_fts(&conn, fts)?;
        tasks.retain(|t| fts_ids.contains(&t.id));
    }
    
    // Also search meetings if query seems meeting-related
    let meetings = if query.contains("meeting") || query.contains("discuss") {
        search_meetings(&conn, &parsed.fts_query.unwrap_or_default())?
    } else {
        vec![]
    };
    
    Ok(json!({
        "tasks": tasks,
        "meetings": meetings,
        "interpretation": format!("Filters: {:?}, FTS: {:?}", parsed.filters, parsed.fts_query)
    }))
}
```
