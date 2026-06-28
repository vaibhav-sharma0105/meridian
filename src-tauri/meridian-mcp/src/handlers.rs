//! MCP request handlers
//!
//! Dispatches incoming requests to appropriate handlers and returns responses.

use crate::protocol::{
    InitializeResult, Request, ResourceContent, ResourceDefinition, Response, RpcError,
    ServerCapabilities, ServerInfo, ToolDefinition, ToolsCapability, ResourcesCapability,
};
use meridian_lib::db::{connection, repositories};
use meridian_lib::models::task::TaskFilters;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

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
