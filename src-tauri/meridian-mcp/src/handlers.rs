//! MCP request handlers
//!
//! Dispatches incoming requests to appropriate handlers and returns responses.

use crate::protocol::{
    InitializeResult, Request, ResourceContent, ResourceDefinition, Response, RpcError,
    ServerCapabilities, ServerInfo, ToolDefinition, ToolsCapability, ResourcesCapability,
};
use meridian_lib::db::{connection, repositories};
use meridian_lib::models::task::{CreateTaskInput, TaskFilters, UpdateTaskInput};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

lazy_static::lazy_static! {
    static ref RATE_LIMITER: Mutex<RateLimiter> = Mutex::new(RateLimiter::new(100, Duration::from_secs(60)));
}

struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: Vec<Instant>,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Vec::new(),
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        self.requests.retain(|t| now.duration_since(*t) < self.window);
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Handle an incoming MCP request
pub fn handle_request(req: Request) -> Option<Response> {
    debug!("Handling method: {}", req.method);

    // Notifications (no id) don't get responses
    if req.id.is_none() && req.method == "notifications/initialized" {
        info!("Client initialized");
        return None;
    }

    let result = match req.method.as_str() {
        "initialize" => handle_initialize(),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(req.params),
        "resources/list" => handle_resources_list(),
        "resources/read" => handle_resources_read(req.params),
        _ => {
            warn!("Unknown method: {}", req.method);
            Err(RpcError::method_not_found(&req.method))
        }
    };

    Some(match result {
        Ok(value) => Response::success(req.id, value),
        Err(error) => Response::error(req.id, error),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Initialize
// ─────────────────────────────────────────────────────────────────────────────

fn handle_initialize() -> Result<Value, RpcError> {
    info!("Initializing MCP connection");

    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: ToolsCapability { list_changed: false },
            resources: ResourcesCapability {
                subscribe: false,
                list_changed: false,
            },
        },
        server_info: ServerInfo {
            name: "meridian-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    Ok(serde_json::to_value(result).unwrap())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tools
// ─────────────────────────────────────────────────────────────────────────────

fn handle_tools_list() -> Result<Value, RpcError> {
    let tools = vec![
        ToolDefinition {
            name: "list_projects".to_string(),
            description: "List all Meridian projects with task counts".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "list_tasks".to_string(),
            description: "List tasks with optional filters. Returns id, title, status, priority, assignee, due_date, project_id, meeting_id.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Filter by project ID"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["open", "in_progress", "done", "cancelled"],
                        "description": "Filter by status"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Filter by priority"
                    },
                    "assignee": {
                        "type": "string",
                        "description": "Filter by assignee name (partial match)"
                    },
                    "due_before": {
                        "type": "string",
                        "format": "date",
                        "description": "Tasks due before this date (YYYY-MM-DD)"
                    },
                    "due_after": {
                        "type": "string",
                        "format": "date",
                        "description": "Tasks due after this date (YYYY-MM-DD)"
                    },
                    "text_search": {
                        "type": "string",
                        "description": "Full-text search in title and description"
                    },
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived tasks (default: false)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 100)"
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_task".to_string(),
            description: "Get detailed information about a specific task, including linked meeting title and project name.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The task ID"
                    }
                },
                "required": ["task_id"]
            }),
        },
        ToolDefinition {
            name: "list_meetings".to_string(),
            description: "List meetings with optional filters. Returns id, title, project_id, meeting_at, ai_summary.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Filter by project ID"
                    },
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived meetings (default: false)"
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_meeting".to_string(),
            description: "Get detailed information about a specific meeting, including full transcript and extracted tasks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meeting_id": {
                        "type": "string",
                        "description": "The meeting ID"
                    }
                },
                "required": ["meeting_id"]
            }),
        },
        ToolDefinition {
            name: "get_task_context".to_string(),
            description: "Get rich context for a task: task details, source meeting info, and project details. Use this when you need full context to help with a task.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The task ID"
                    }
                },
                "required": ["task_id"]
            }),
        },
        // Write tools (require permissions)
        ToolDefinition {
            name: "create_task".to_string(),
            description: "Create a new task in Meridian. Requires MCP write permission.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "The project ID to create the task in"
                    },
                    "title": {
                        "type": "string",
                        "description": "Task title"
                    },
                    "description": {
                        "type": "string",
                        "description": "Task description"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Task priority"
                    },
                    "assignee": {
                        "type": "string",
                        "description": "Person assigned to the task"
                    },
                    "due_date": {
                        "type": "string",
                        "format": "date",
                        "description": "Due date (YYYY-MM-DD)"
                    }
                },
                "required": ["project_id", "title"]
            }),
        },
        ToolDefinition {
            name: "update_task".to_string(),
            description: "Update an existing task. Requires MCP write permission.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The task ID to update"
                    },
                    "title": {
                        "type": "string",
                        "description": "New task title"
                    },
                    "description": {
                        "type": "string",
                        "description": "New task description"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["open", "in_progress", "done", "cancelled"],
                        "description": "New task status"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "New task priority"
                    },
                    "assignee": {
                        "type": "string",
                        "description": "New assignee"
                    },
                    "due_date": {
                        "type": "string",
                        "format": "date",
                        "description": "New due date (YYYY-MM-DD)"
                    }
                },
                "required": ["task_id"]
            }),
        },
        ToolDefinition {
            name: "create_meeting_note".to_string(),
            description: "Create a meeting note/transcript entry. Requires MCP write permission.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "The project ID"
                    },
                    "title": {
                        "type": "string",
                        "description": "Meeting title"
                    },
                    "content": {
                        "type": "string",
                        "description": "Meeting notes or transcript"
                    }
                },
                "required": ["project_id", "title", "content"]
            }),
        },
        ToolDefinition {
            name: "run_skill".to_string(),
            description: "Queue a skill for execution. Requires MCP write permission.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "skill_id": {
                        "type": "string",
                        "description": "The skill ID to run"
                    }
                },
                "required": ["skill_id"]
            }),
        },
    ];

    Ok(json!({ "tools": tools }))
}

fn handle_tools_call(params: Option<Value>) -> Result<Value, RpcError> {
    let params = params.ok_or_else(|| RpcError::invalid_params("missing params"))?;

    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing tool name"))?;

    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    info!("Calling tool: {}", name);

    let result = match name {
        "list_projects" => tool_list_projects(),
        "list_tasks" => tool_list_tasks(args),
        "get_task" => tool_get_task(args),
        "list_meetings" => tool_list_meetings(args),
        "get_meeting" => tool_get_meeting(args),
        "get_task_context" => tool_get_task_context(args),
        // Write tools with permission checks
        "create_task" => tool_create_task(args),
        "update_task" => tool_update_task(args),
        "create_meeting_note" => tool_create_meeting_note(args),
        "run_skill" => tool_run_skill(args),
        _ => Err(RpcError::invalid_params(&format!("unknown tool: {}", name))),
    }?;

    // MCP tools/call expects result wrapped in content array
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
        }]
    }))
}

fn get_connection() -> Result<rusqlite::Connection, RpcError> {
    connection::init_db().map_err(|e| RpcError::internal_error(&e))
}

fn tool_list_projects() -> Result<Value, RpcError> {
    let conn = get_connection()?;
    let projects = repositories::projects::get_all_projects(&conn)
        .map_err(|e| RpcError::internal_error(&e))?;

    Ok(json!({
        "projects": projects,
        "count": projects.len()
    }))
}

fn tool_list_tasks(args: Value) -> Result<Value, RpcError> {
    let conn = get_connection()?;

    let project_id = args.get("project_id").and_then(|v| v.as_str());
    let include_archived = args.get("include_archived").and_then(|v| v.as_bool()).unwrap_or(false);

    let filters = TaskFilters {
        status: args.get("status").and_then(|v| v.as_str()).map(String::from),
        priority: args.get("priority").and_then(|v| v.as_str()).map(String::from),
        assignee: args.get("assignee").and_then(|v| v.as_str()).map(String::from),
        search_query: args.get("text_search").and_then(|v| v.as_str()).map(String::from),
        date_from: args.get("due_after").and_then(|v| v.as_str()).map(String::from),
        date_to: args.get("due_before").and_then(|v| v.as_str()).map(String::from),
        show_archived: Some(include_archived),
        tags: None,
    };

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

    let mut tasks = if let Some(pid) = project_id {
        repositories::tasks::get_tasks_for_project(&conn, pid, &filters)
            .map_err(|e| RpcError::internal_error(&e))?
    } else {
        repositories::tasks::get_all_tasks(&conn, &filters)
            .map_err(|e| RpcError::internal_error(&e))?
    };

    // Apply due_before/due_after filters (not handled by backend for all cases)
    if let Some(due_before) = args.get("due_before").and_then(|v| v.as_str()) {
        tasks.retain(|t| {
            t.due_date.as_ref().map(|d| d.as_str() <= due_before).unwrap_or(false)
        });
    }
    if let Some(due_after) = args.get("due_after").and_then(|v| v.as_str()) {
        tasks.retain(|t| {
            t.due_date.as_ref().map(|d| d.as_str() >= due_after).unwrap_or(false)
        });
    }

    // Truncate to limit
    tasks.truncate(limit);

    // Return light version (no description/notes to save tokens)
    let light_tasks: Vec<Value> = tasks
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "title": t.title,
                "status": t.status,
                "priority": t.priority,
                "assignee": t.assignee,
                "due_date": t.due_date,
                "project_id": t.project_id,
                "meeting_id": t.meeting_id,
                "created_at": t.created_at,
                "updated_at": t.updated_at
            })
        })
        .collect();

    Ok(json!({
        "tasks": light_tasks,
        "count": light_tasks.len(),
        "truncated": tasks.len() >= limit
    }))
}

fn tool_get_task(args: Value) -> Result<Value, RpcError> {
    let task_id = args
        .get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing task_id"))?;

    let conn = get_connection()?;

    let task = repositories::tasks::get_task(&conn, task_id)
        .map_err(|e| RpcError::internal_error(&e))?;

    // Get project name
    let project = repositories::projects::get_project(&conn, &task.project_id)
        .map_err(|e| RpcError::internal_error(&e))?;

    // Get meeting title if linked
    let meeting_title = if let Some(mid) = &task.meeting_id {
        repositories::meetings::get_meeting(&conn, mid)
            .ok()
            .flatten()
            .map(|m| m.title)
    } else {
        None
    };

    Ok(json!({
        "task": task,
        "project_name": project.map(|p| p.name),
        "meeting_title": meeting_title
    }))
}

fn tool_list_meetings(args: Value) -> Result<Value, RpcError> {
    let conn = get_connection()?;

    let include_archived = args.get("include_archived").and_then(|v| v.as_bool()).unwrap_or(false);

    let meetings = if let Some(project_id) = args.get("project_id").and_then(|v| v.as_str()) {
        repositories::meetings::get_meetings_for_project(&conn, project_id, include_archived)
            .map_err(|e| RpcError::internal_error(&e))?
    } else {
        // Get all meetings across projects
        let projects = repositories::projects::get_all_projects(&conn)
            .map_err(|e| RpcError::internal_error(&e))?;

        let mut all_meetings = Vec::new();
        for p in projects {
            let meetings = repositories::meetings::get_meetings_for_project(&conn, &p.id, include_archived)
                .map_err(|e| RpcError::internal_error(&e))?;
            all_meetings.extend(meetings);
        }
        all_meetings
    };

    // Return light version (no transcript)
    let light_meetings: Vec<Value> = meetings
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "title": m.title,
                "project_id": m.project_id,
                "platform": m.platform,
                "meeting_at": m.meeting_at,
                "ai_summary": m.ai_summary,
                "attendees": m.attendees,
                "health_score": m.health_score
            })
        })
        .collect();

    Ok(json!({
        "meetings": light_meetings,
        "count": light_meetings.len()
    }))
}

fn tool_get_meeting(args: Value) -> Result<Value, RpcError> {
    let meeting_id = args
        .get("meeting_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing meeting_id"))?;

    let conn = get_connection()?;

    let meeting = repositories::meetings::get_meeting(&conn, meeting_id)
        .map_err(|e| RpcError::internal_error(&e))?
        .ok_or_else(|| RpcError::invalid_params("meeting not found"))?;

    // Get project name
    let project = repositories::projects::get_project(&conn, &meeting.project_id)
        .map_err(|e| RpcError::internal_error(&e))?;

    // Get tasks linked to this meeting
    let filters = TaskFilters::default();
    let all_tasks = repositories::tasks::get_tasks_for_project(&conn, &meeting.project_id, &filters)
        .map_err(|e| RpcError::internal_error(&e))?;

    let linked_tasks: Vec<Value> = all_tasks
        .iter()
        .filter(|t| t.meeting_id.as_ref() == Some(&meeting.id))
        .map(|t| {
            json!({
                "id": t.id,
                "title": t.title,
                "status": t.status,
                "priority": t.priority,
                "assignee": t.assignee
            })
        })
        .collect();

    Ok(json!({
        "meeting": meeting,
        "project_name": project.map(|p| p.name),
        "extracted_tasks": linked_tasks,
        "task_count": linked_tasks.len()
    }))
}

fn tool_get_task_context(args: Value) -> Result<Value, RpcError> {
    let task_id = args
        .get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing task_id"))?;

    let conn = get_connection()?;

    // Get task
    let task = repositories::tasks::get_task(&conn, task_id)
        .map_err(|e| RpcError::internal_error(&e))?;

    // Get project
    let project = repositories::projects::get_project(&conn, &task.project_id)
        .map_err(|e| RpcError::internal_error(&e))?;

    // Get source meeting with relevant excerpt
    let meeting_context = if let Some(mid) = &task.meeting_id {
        if let Ok(Some(meeting)) = repositories::meetings::get_meeting(&conn, mid) {
            // Try to extract relevant excerpt using source quotes
            let excerpt = extract_relevant_excerpt(&task, &meeting);

            Some(json!({
                "id": meeting.id,
                "title": meeting.title,
                "meeting_at": meeting.meeting_at,
                "ai_summary": meeting.ai_summary,
                "decisions": meeting.decisions,
                "attendees": meeting.attendees,
                "transcript_excerpt": excerpt
            }))
        } else {
            None
        }
    } else {
        None
    };

    Ok(json!({
        "task": task,
        "project": project,
        "source_meeting": meeting_context
    }))
}

/// Extract a relevant portion of the transcript for a task
fn extract_relevant_excerpt(
    task: &meridian_lib::models::task::Task,
    meeting: &meridian_lib::models::meeting::Meeting,
) -> Option<String> {
    let transcript = meeting.raw_transcript.as_ref()?;

    // Use source quotes to find relevant sections
    let search_terms: Vec<&str> = [
        task.assignee_source_quote.as_deref(),
        task.due_source_quote.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    if !search_terms.is_empty() {
        // Find the first matching quote in transcript
        for quote in &search_terms {
            if let Some(pos) = transcript.find(quote) {
                // Extract context around the quote (500 chars before/after)
                let start = pos.saturating_sub(500);
                let end = (pos + quote.len() + 500).min(transcript.len());

                let mut excerpt = String::new();
                if start > 0 {
                    excerpt.push_str("...");
                }
                excerpt.push_str(&transcript[start..end]);
                if end < transcript.len() {
                    excerpt.push_str("...");
                }
                return Some(excerpt);
            }
        }
    }

    // Fallback: search for task title keywords in transcript
    let title_words: Vec<&str> = task.title.split_whitespace().take(5).collect();
    for word in &title_words {
        if word.len() < 4 {
            continue;
        }
        if let Some(pos) = transcript.to_lowercase().find(&word.to_lowercase()) {
            let start = pos.saturating_sub(300);
            let end = (pos + 600).min(transcript.len());

            let mut excerpt = String::new();
            if start > 0 {
                excerpt.push_str("...");
            }
            excerpt.push_str(&transcript[start..end]);
            if end < transcript.len() {
                excerpt.push_str("...");
            }
            return Some(excerpt);
        }
    }

    // Last resort: first 1000 chars
    if transcript.len() > 1000 {
        Some(format!("{}...", &transcript[..1000]))
    } else {
        Some(transcript.clone())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Write Tools (MCP permission required)
// ─────────────────────────────────────────────────────────────────────────────

fn check_rate_limit() -> Result<(), RpcError> {
    let mut limiter = RATE_LIMITER.lock().map_err(|_| RpcError::internal_error("rate limiter lock failed"))?;
    if !limiter.check() {
        return Err(RpcError {
            code: 429,
            message: "Rate limit exceeded (100 ops/minute)".to_string(),
            data: None,
        });
    }
    Ok(())
}

fn check_permission(conn: &rusqlite::Connection, permission: &str) -> Result<(), RpcError> {
    let permissions_json: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'mcp_permissions'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(json_str) = permissions_json {
        if let Ok(perms) = serde_json::from_str::<HashMap<String, bool>>(&json_str) {
            if perms.get(permission).copied().unwrap_or(false) {
                return Ok(());
            }
        }
    }

    Err(RpcError {
        code: 403,
        message: format!("MCP permission '{}' not granted. Enable in Settings > MCP.", permission),
        data: None,
    })
}

fn log_mcp_action(conn: &rusqlite::Connection, action: &str, entity_type: &str, entity_id: &str, details: &str) {
    let _ = conn.execute(
        "INSERT INTO audit_log (id, action_type, entity_type, entity_id, details, agent_initiated, risk_level, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 'medium', datetime('now'))",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            action,
            entity_type,
            entity_id,
            details
        ],
    );
}

fn tool_create_task(args: Value) -> Result<Value, RpcError> {
    check_rate_limit()?;
    let conn = get_connection()?;
    check_permission(&conn, "create_task")?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing project_id"))?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing title"))?;

    let input = CreateTaskInput {
        project_id: project_id.to_string(),
        meeting_id: None,
        parent_task_id: None,
        title: title.to_string(),
        description: args.get("description").and_then(|v| v.as_str()).map(String::from),
        assignee: args.get("assignee").and_then(|v| v.as_str()).map(String::from),
        assignee_confidence: None,
        assignee_source_quote: None,
        due_date: args.get("due_date").and_then(|v| v.as_str()).map(String::from),
        due_confidence: None,
        due_source_quote: None,
        priority: args.get("priority").and_then(|v| v.as_str()).map(String::from),
        confidence_score: None,
        tags: None,
        kanban_column: None,
        notes: None,
        is_duplicate: None,
        duplicate_of_id: None,
    };

    let task = repositories::tasks::create_task(&conn, &input)
        .map_err(|e| RpcError::internal_error(&e))?;

    log_mcp_action(&conn, "mcp_create_task", "task", &task.id, &format!("Created task: {}", title));

    Ok(json!({
        "success": true,
        "task": task
    }))
}

fn tool_update_task(args: Value) -> Result<Value, RpcError> {
    check_rate_limit()?;
    let conn = get_connection()?;
    check_permission(&conn, "update_task")?;

    let task_id = args
        .get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing task_id"))?;

    let input = UpdateTaskInput {
        id: task_id.to_string(),
        title: args.get("title").and_then(|v| v.as_str()).map(String::from),
        description: args.get("description").and_then(|v| v.as_str()).map(String::from),
        assignee: args.get("assignee").and_then(|v| v.as_str()).map(String::from),
        assignee_confidence: None,
        due_date: args.get("due_date").and_then(|v| v.as_str()).map(String::from),
        due_confidence: None,
        status: args.get("status").and_then(|v| v.as_str()).map(String::from),
        priority: args.get("priority").and_then(|v| v.as_str()).map(String::from),
        tags: None,
        kanban_column: None,
        kanban_order: None,
        notes: None,
        meeting_id: None,
    };

    let task = repositories::tasks::update_task(&conn, &input)
        .map_err(|e| RpcError::internal_error(&e))?;

    log_mcp_action(&conn, "mcp_update_task", "task", &task.id, "Updated task via MCP");

    Ok(json!({
        "success": true,
        "task": task
    }))
}

fn tool_create_meeting_note(args: Value) -> Result<Value, RpcError> {
    check_rate_limit()?;
    let conn = get_connection()?;
    check_permission(&conn, "create_meeting_note")?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing project_id"))?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing title"))?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing content"))?;

    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO meetings (id, project_id, title, platform, raw_transcript, ingested_at)
         VALUES (?1, ?2, ?3, 'mcp', ?4, datetime('now'))",
        rusqlite::params![id, project_id, title, content],
    )
    .map_err(|e| RpcError::internal_error(&e.to_string()))?;

    let meeting = repositories::meetings::get_meeting(&conn, &id)
        .map_err(|e| RpcError::internal_error(&e))?;

    log_mcp_action(&conn, "mcp_create_meeting", "meeting", &id, &format!("Created meeting note: {}", title));

    Ok(json!({
        "success": true,
        "meeting": meeting
    }))
}

fn tool_run_skill(args: Value) -> Result<Value, RpcError> {
    check_rate_limit()?;
    let conn = get_connection()?;
    check_permission(&conn, "run_skill")?;

    let skill_id = args
        .get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing skill_id"))?;

    // Check skill exists
    let skill_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM skills WHERE id = ?1 AND enabled = 1)",
            [skill_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !skill_exists {
        return Err(RpcError::invalid_params("skill not found or disabled"));
    }

    // Create a skill run
    let run_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO skill_runs (id, skill_id, status, trigger_type, trigger_context, created_at)
         VALUES (?1, ?2, 'pending', 'mcp', ?3, datetime('now'))",
        rusqlite::params![run_id, skill_id, json!({"source": "mcp"}).to_string()],
    )
    .map_err(|e| RpcError::internal_error(&e.to_string()))?;

    // Queue the job for daemon
    let job_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO daemon_jobs (id, job_type, payload, status, priority, created_at)
         VALUES (?1, 'run_skill', ?2, 'pending', 5, datetime('now'))",
        rusqlite::params![job_id, json!({"skill_id": skill_id, "run_id": run_id}).to_string()],
    )
    .map_err(|e| RpcError::internal_error(&e.to_string()))?;

    log_mcp_action(&conn, "mcp_run_skill", "skill", skill_id, &format!("Queued skill run: {}", run_id));

    Ok(json!({
        "success": true,
        "run_id": run_id,
        "status": "pending",
        "message": "Skill queued for execution"
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Resources
// ─────────────────────────────────────────────────────────────────────────────

fn handle_resources_list() -> Result<Value, RpcError> {
    let resources = vec![
        ResourceDefinition {
            uri: "meridian://projects".to_string(),
            name: "Projects".to_string(),
            description: "List all Meridian projects".to_string(),
            mime_type: Some("application/json".to_string()),
        },
        ResourceDefinition {
            uri: "meridian://tasks".to_string(),
            name: "Tasks".to_string(),
            description: "List all tasks (use tools/call with list_tasks for filtering)".to_string(),
            mime_type: Some("application/json".to_string()),
        },
        ResourceDefinition {
            uri: "meridian://meetings".to_string(),
            name: "Meetings".to_string(),
            description: "List all meetings".to_string(),
            mime_type: Some("application/json".to_string()),
        },
    ];

    Ok(json!({ "resources": resources }))
}

fn handle_resources_read(params: Option<Value>) -> Result<Value, RpcError> {
    let params = params.ok_or_else(|| RpcError::invalid_params("missing params"))?;

    let uri = params
        .get("uri")
        .and_then(|u| u.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing uri"))?;

    info!("Reading resource: {}", uri);

    let content = match uri {
        "meridian://projects" => {
            let result = tool_list_projects()?;
            serde_json::to_string_pretty(&result).unwrap()
        }
        "meridian://tasks" => {
            let result = tool_list_tasks(json!({}))?;
            serde_json::to_string_pretty(&result).unwrap()
        }
        "meridian://meetings" => {
            let result = tool_list_meetings(json!({}))?;
            serde_json::to_string_pretty(&result).unwrap()
        }
        _ => {
            return Err(RpcError::invalid_params(&format!("unknown resource: {}", uri)));
        }
    };

    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": content
        }]
    }))
}
