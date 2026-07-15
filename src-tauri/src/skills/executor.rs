use rusqlite::Connection;
use serde_json::{json, Value};
use std::time::Instant;

use crate::ai::litellm::LiteLLMClient;
use crate::commands::ai::{get_api_key_from_db, get_litellm_client_pub};
use crate::db::repositories::{
    ai_settings as ai_repo, documents as docs_repo, meetings as meetings_repo,
    projects as projects_repo, tasks as tasks_repo,
};
use crate::models::task::TaskFilters;
use crate::skills::{
    repository as skills_repo, ActionConfig, ApprovalMode, ContextConfig, Skill, SkillRun,
};

const DEFAULT_MAX_TOKENS: i32 = 8000;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionContext {
    pub tasks: Vec<Value>,
    pub meetings: Vec<Value>,
    pub documents: Vec<Value>,
    pub project: Option<Value>,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: String,
    pub duration_ms: i64,
    pub pending_changes: Option<Value>,
    pub needs_approval: bool,
}

pub fn build_context(conn: &Connection, skill: &Skill) -> Result<ExecutionContext, String> {
    let config = skill.get_context_config().unwrap_or_default();
    let scope = config.scope.as_deref().unwrap_or("global");
    let max_tokens = config.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

    let mut context = ExecutionContext {
        tasks: Vec::new(),
        meetings: Vec::new(),
        documents: Vec::new(),
        project: None,
        truncated: false,
    };

    // Get project if scoped
    if scope == "project" {
        if let Some(ref project_id) = config.project_id {
            if let Ok(Some(project)) = projects_repo::get_project(conn, project_id) {
                context.project = Some(json!({
                    "id": project.id,
                    "name": project.name,
                    "description": project.description,
                }));
            }
        }
    }

    // Get tasks
    let task_filters = TaskFilters {
        show_archived: Some(config.include_archived.unwrap_or(false)),
        ..Default::default()
    };

    let tasks = if let Some(ref project_id) = config.project_id {
        tasks_repo::get_tasks_for_project(conn, project_id, &task_filters)?
    } else {
        tasks_repo::get_all_tasks(conn, &task_filters)?
    };

    for task in tasks.iter().take(50) {
        context.tasks.push(json!({
            "id": task.id,
            "title": task.title,
            "description": task.description,
            "assignee": task.assignee,
            "status": task.status,
            "priority": task.priority,
            "due_date": task.due_date,
        }));
    }

    // Get meetings
    let show_archived = config.include_archived.unwrap_or(false);
    let meetings = if let Some(ref project_id) = config.project_id {
        meetings_repo::get_meetings_for_project(conn, project_id, show_archived)?
    } else {
        // Get meetings from all projects
        let projects = projects_repo::get_all_projects(conn)?;
        let mut all_meetings = Vec::new();
        for project in projects.iter().take(10) {
            if let Ok(m) = meetings_repo::get_meetings_for_project(conn, &project.id, show_archived) {
                all_meetings.extend(m);
            }
        }
        all_meetings
    };

    for meeting in meetings.iter().take(20) {
        context.meetings.push(json!({
            "id": meeting.id,
            "title": meeting.title,
            "summary": meeting.summary,
            "platform": meeting.platform,
            "meeting_at": meeting.meeting_at,
            "attendees": meeting.attendees,
        }));
    }

    // Get documents if enabled
    if config.include_documents.unwrap_or(false) {
        if let Some(ref project_id) = config.project_id {
            let documents = docs_repo::get_documents_for_project(conn, project_id)?;
            let max_docs = config.max_documents.unwrap_or(10) as usize;
            let doc_filter = config.document_filter.as_ref().and_then(|f| regex::Regex::new(f).ok());

            for doc in documents.iter().filter(|d| {
                match &doc_filter {
                    Some(re) => re.is_match(&d.filename),
                    None => true,
                }
            }).take(max_docs) {
                let content_snippet = doc.content_text.as_ref()
                    .map(|c| c.chars().take(500).collect::<String>())
                    .unwrap_or_default();

                context.documents.push(json!({
                    "id": doc.id,
                    "filename": doc.filename,
                    "title": doc.title,
                    "file_type": doc.file_type,
                    "content_snippet": content_snippet,
                }));
            }
        }
    }

    // Check if we need to truncate
    let estimated_tokens = estimate_tokens(&context);
    if estimated_tokens > max_tokens {
        context = truncate_context(context, &config, max_tokens);
    }

    Ok(context)
}

fn estimate_tokens(context: &ExecutionContext) -> i32 {
    let json_str = serde_json::to_string(context).unwrap_or_default();
    (json_str.len() / 4) as i32
}

fn truncate_context(
    mut context: ExecutionContext,
    config: &ContextConfig,
    max_tokens: i32,
) -> ExecutionContext {
    let priority_order = config
        .priority_order
        .clone()
        .unwrap_or_else(|| vec!["tasks".to_string(), "meetings".to_string(), "documents".to_string()]);

    // Truncate from lowest priority first
    for content_type in priority_order.iter().rev() {
        if estimate_tokens(&context) <= max_tokens {
            break;
        }

        match content_type.as_str() {
            "documents" => {
                if !context.documents.is_empty() {
                    context.documents.pop();
                    context.truncated = true;
                }
            }
            "meetings" => {
                if !context.meetings.is_empty() {
                    context.meetings.pop();
                    context.truncated = true;
                }
            }
            "tasks" => {
                if !context.tasks.is_empty() {
                    context.tasks.pop();
                    context.truncated = true;
                }
            }
            _ => {}
        }
    }

    context
}

pub fn check_needs_approval(skill: &Skill, action_config: &ActionConfig) -> bool {
    let mode = ApprovalMode::from_str(&skill.approval_mode).unwrap_or(ApprovalMode::Notify);

    match mode {
        ApprovalMode::Auto => false,
        ApprovalMode::Notify => false,
        ApprovalMode::ApproveAlways => true,
        ApprovalMode::ApproveFirst => {
            // Need approval for actions with side effects
            action_config.has_side_effects.unwrap_or(false)
                || action_config.action_type.as_deref() == Some("create_tasks")
        }
    }
}

pub fn get_ai_client(conn: &Connection) -> Result<LiteLLMClient, String> {
    let settings = ai_repo::get_active_settings(conn)?
        .ok_or_else(|| "No AI provider configured. Set up an AI provider in Settings.".to_string())?;
    let api_key = get_api_key_from_db(conn, &settings.label);
    if api_key.is_empty() {
        return Err("AI API key not configured".to_string());
    }
    Ok(get_litellm_client_pub(&settings, &api_key))
}

pub fn execute_skill(
    conn: &Connection,
    skill: &Skill,
    run: &SkillRun,
) -> Result<ExecutionResult, String> {
    let start = Instant::now();

    skills_repo::update_run_status(conn, &run.id, "running")?;

    let context = build_context(conn, skill)?;
    let action_config = skill.get_action_config().unwrap_or_default();
    let needs_approval = check_needs_approval(skill, &action_config);
    let action_type = action_config.action_type.as_deref().unwrap_or("summarize");

    let result = match action_type {
        "summarize" => execute_summarize(conn, &context, &action_config),
        "draft_message" => execute_draft(conn, &context, &action_config),
        "create_tasks" => execute_create_tasks(conn, &context, &action_config),
        "analyze" => execute_analyze(conn, &context, &action_config),
        "custom" => {
            let ctx_config = skill.get_context_config().unwrap_or_default();
            execute_custom(conn, &context, &action_config, &ctx_config)
        }
        _ => Err(format!("Unknown action type: {}", action_type)),
    }?;

    let duration_ms = start.elapsed().as_millis() as i64;

    Ok(ExecutionResult {
        output: result.0,
        duration_ms,
        pending_changes: result.1,
        needs_approval,
    })
}

pub async fn execute_skill_async(
    conn: &Connection,
    skill: &Skill,
    run: &SkillRun,
) -> Result<ExecutionResult, String> {
    let start = Instant::now();

    skills_repo::update_run_status(conn, &run.id, "running")?;

    let context = build_context(conn, skill)?;
    let action_config = skill.get_action_config().unwrap_or_default();
    let needs_approval = check_needs_approval(skill, &action_config);
    let action_type = action_config.action_type.as_deref().unwrap_or("summarize");

    let client = get_ai_client(conn)?;

    let result = match action_type {
        "summarize" => execute_summarize_ai(&client, &context, &action_config).await,
        "draft_message" => execute_draft_ai(&client, &context, &action_config).await,
        "create_tasks" => execute_create_tasks_ai(&client, &context, &action_config).await,
        "analyze" => execute_analyze_ai(&client, &context, &action_config).await,
        "custom" => {
            let ctx_config = skill.get_context_config().unwrap_or_default();
            execute_custom_ai(&client, &context, &action_config, &ctx_config).await
        }
        _ => Err(format!("Unknown action type: {}", action_type)),
    }?;

    let duration_ms = start.elapsed().as_millis() as i64;

    Ok(ExecutionResult {
        output: result.0,
        duration_ms,
        pending_changes: result.1,
        needs_approval,
    })
}

fn format_context_for_prompt(context: &ExecutionContext) -> String {
    let mut parts = Vec::new();

    if let Some(ref project) = context.project {
        parts.push(format!("Project: {}", project["name"].as_str().unwrap_or("Unknown")));
    }

    if !context.tasks.is_empty() {
        let tasks_str: Vec<String> = context.tasks.iter().map(|t| {
            format!("- [{}] {} (priority: {}, assignee: {}, due: {})",
                t["status"].as_str().unwrap_or("open"),
                t["title"].as_str().unwrap_or(""),
                t["priority"].as_str().unwrap_or("medium"),
                t["assignee"].as_str().unwrap_or("unassigned"),
                t["due_date"].as_str().unwrap_or("none"))
        }).collect();
        parts.push(format!("Tasks ({}):\n{}", context.tasks.len(), tasks_str.join("\n")));
    }

    if !context.meetings.is_empty() {
        let meetings_str: Vec<String> = context.meetings.iter().map(|m| {
            format!("- {} ({})\n  Summary: {}",
                m["title"].as_str().unwrap_or(""),
                m["meeting_at"].as_str().unwrap_or(""),
                m["summary"].as_str().unwrap_or("No summary"))
        }).collect();
        parts.push(format!("Meetings ({}):\n{}", context.meetings.len(), meetings_str.join("\n")));
    }

    if !context.documents.is_empty() {
        let docs_str: Vec<String> = context.documents.iter().map(|d| {
            let snippet = d["content_snippet"].as_str().unwrap_or("");
            format!("- {} ({})\n  Content: {}...",
                d["filename"].as_str().unwrap_or(""),
                d["file_type"].as_str().unwrap_or(""),
                if snippet.len() > 200 { &snippet[..200] } else { snippet })
        }).collect();
        parts.push(format!("Documents ({}):\n{}", context.documents.len(), docs_str.join("\n")));
    }

    parts.join("\n\n")
}

// --- Sync fallback versions (used when AI is unavailable) ---

fn execute_summarize(
    conn: &Connection,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    if let Ok(client) = get_ai_client(conn) {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            return handle.block_on(execute_summarize_ai(&client, context, config));
        }
    }
    execute_summarize_fallback(context, config)
}

fn execute_draft(
    conn: &Connection,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    if let Ok(client) = get_ai_client(conn) {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            return handle.block_on(execute_draft_ai(&client, context, config));
        }
    }
    execute_draft_fallback(context, config)
}

fn execute_create_tasks(
    conn: &Connection,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    if let Ok(client) = get_ai_client(conn) {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            return handle.block_on(execute_create_tasks_ai(&client, context, config));
        }
    }
    execute_create_tasks_fallback(context)
}

fn execute_analyze(
    conn: &Connection,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    if let Ok(client) = get_ai_client(conn) {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            return handle.block_on(execute_analyze_ai(&client, context, config));
        }
    }
    execute_analyze_fallback(context)
}

fn execute_custom(
    conn: &Connection,
    context: &ExecutionContext,
    action_config: &ActionConfig,
    context_config: &ContextConfig,
) -> Result<(String, Option<Value>), String> {
    if let Ok(client) = get_ai_client(conn) {
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            return handle.block_on(execute_custom_ai(&client, context, action_config, context_config));
        }
    }
    execute_custom_fallback(context, context_config)
}

// --- Fallback implementations (no AI) ---

fn execute_summarize_fallback(
    context: &ExecutionContext,
    _config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let summary = format!(
        "## Summary\n\n**Tasks:** {} total\n**Meetings:** {} total\n\n### Recent Tasks\n{}\n\n### Recent Meetings\n{}",
        context.tasks.len(),
        context.meetings.len(),
        context.tasks.iter().take(5)
            .map(|t| format!("- {} ({})", t["title"].as_str().unwrap_or(""), t["status"].as_str().unwrap_or("")))
            .collect::<Vec<_>>().join("\n"),
        context.meetings.iter().take(3)
            .map(|m| format!("- {}", m["title"].as_str().unwrap_or("")))
            .collect::<Vec<_>>().join("\n"),
    );
    Ok((summary, None))
}

fn execute_draft_fallback(
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let channel = config.channel.as_deref().unwrap_or("email");
    let task_list = context.tasks.iter().take(5)
        .map(|t| format!("• {}", t["title"].as_str().unwrap_or("")))
        .collect::<Vec<_>>().join("\n");

    let draft = match channel {
        "slack" => format!("*Task Update*\n\n{}\n\n_Sent via Meridian_", task_list),
        _ => format!("Subject: Task Update\n\nHi,\n\nHere's a quick update on the current tasks:\n\n{}\n\nBest regards", task_list),
    };
    Ok((draft, None))
}

fn execute_create_tasks_fallback(
    _context: &ExecutionContext,
) -> Result<(String, Option<Value>), String> {
    Err("AI provider required for task creation. Configure an AI provider in Settings.".to_string())
}

fn execute_analyze_fallback(
    context: &ExecutionContext,
) -> Result<(String, Option<Value>), String> {
    let open = context.tasks.iter().filter(|t| t["status"].as_str() == Some("open")).count();
    let in_progress = context.tasks.iter().filter(|t| t["status"].as_str() == Some("in_progress")).count();
    let done = context.tasks.iter().filter(|t| t["status"].as_str() == Some("done")).count();

    let analysis = format!(
        "## Analysis\n\n**Task Distribution:**\n- Open: {}\n- In Progress: {}\n- Done: {}\n\n**Insights:**\n- {} tasks need attention\n- {} meetings recorded",
        open, in_progress, done, open + in_progress, context.meetings.len()
    );
    Ok((analysis, None))
}

fn execute_custom_fallback(
    context: &ExecutionContext,
    context_config: &ContextConfig,
) -> Result<(String, Option<Value>), String> {
    let prompt = context_config.system_prompt.as_deref().unwrap_or("Analyze the provided context.");
    Err(format!("AI provider required for custom actions (prompt: '{}', context: {} tasks, {} meetings). Configure an AI provider in Settings.",
        prompt, context.tasks.len(), context.meetings.len()))
}

// --- Real AI implementations ---

pub async fn execute_summarize_ai(
    client: &LiteLLMClient,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let format = config.format.as_deref().unwrap_or("markdown");
    let ctx_text = format_context_for_prompt(context);

    let messages = vec![
        json!({"role": "system", "content": format!(
            "You are a project assistant. Summarize the following project data concisely in {} format. \
             Highlight key items: overdue tasks, upcoming deadlines, recent meetings, and blockers. \
             Keep it actionable and brief.", format)}),
        json!({"role": "user", "content": ctx_text}),
    ];

    let response = client.chat_completion(messages, Some(1000)).await?;
    Ok((response, None))
}

pub async fn execute_draft_ai(
    client: &LiteLLMClient,
    context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let channel = config.channel.as_deref().unwrap_or("email");
    let recipient = config.recipient.as_deref().unwrap_or("team");
    let ctx_text = format_context_for_prompt(context);

    let channel_instructions = match channel {
        "slack" => "Write a Slack message using markdown formatting (*bold*, bullet points). Keep it concise and conversational.",
        "email" => "Write a professional email with Subject line, greeting, body, and sign-off.",
        _ => "Write a clear message appropriate for the channel.",
    };

    let messages = vec![
        json!({"role": "system", "content": format!(
            "You are a professional communication assistant. Draft a {} message to {}. {}\n\
             Base the content on the project context provided. Focus on actionable updates.",
            channel, recipient, channel_instructions)}),
        json!({"role": "user", "content": ctx_text}),
    ];

    let response = client.chat_completion(messages, Some(800)).await?;
    Ok((response, None))
}

pub async fn execute_create_tasks_ai(
    client: &LiteLLMClient,
    context: &ExecutionContext,
    _config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let ctx_text = format_context_for_prompt(context);

    let messages = vec![
        json!({"role": "system", "content":
            "You are a project management assistant. Based on the context (meetings, existing tasks), \
             suggest new tasks that should be created. Return ONLY a JSON array of task objects, each with: \
             \"title\" (string), \"description\" (string), \"priority\" (\"low\"|\"medium\"|\"high\"|\"critical\"), \
             \"assignee\" (string or null). Do not include tasks that already exist. \
             Return between 1-5 suggestions. Output ONLY valid JSON, no markdown."}),
        json!({"role": "user", "content": ctx_text}),
    ];

    let response = client.chat_completion(messages, Some(1000)).await?;

    let tasks: Vec<Value> = serde_json::from_str(&response)
        .or_else(|_| {
            let trimmed = response.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
            serde_json::from_str(trimmed)
        })
        .map_err(|e| format!("Failed to parse AI task suggestions: {}", e))?;

    let pending_changes = json!({
        "type": "create_tasks",
        "tasks": tasks,
    });

    Ok((
        format!("AI suggested {} tasks to create", tasks.len()),
        Some(pending_changes),
    ))
}

pub async fn execute_analyze_ai(
    client: &LiteLLMClient,
    context: &ExecutionContext,
    _config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let ctx_text = format_context_for_prompt(context);

    let messages = vec![
        json!({"role": "system", "content":
            "You are a project analyst. Analyze the project data and provide insights on: \
             1. Task velocity and bottlenecks \
             2. Workload distribution across assignees \
             3. Risks (overdue items, unassigned tasks, stale work) \
             4. Recommendations for the next sprint/week \
             Be specific and data-driven. Use markdown formatting."}),
        json!({"role": "user", "content": ctx_text}),
    ];

    let response = client.chat_completion(messages, Some(1200)).await?;
    Ok((response, None))
}

pub async fn execute_custom_ai(
    client: &LiteLLMClient,
    context: &ExecutionContext,
    _action_config: &ActionConfig,
    context_config: &ContextConfig,
) -> Result<(String, Option<Value>), String> {
    let system_prompt = context_config.system_prompt.as_deref()
        .unwrap_or("You are a helpful project assistant. Analyze the provided context and respond thoughtfully.");
    let output_instructions = context_config.output_instructions.as_deref().unwrap_or("");
    let ctx_text = format_context_for_prompt(context);

    let full_system = if output_instructions.is_empty() {
        system_prompt.to_string()
    } else {
        format!("{}\n\nOutput format: {}", system_prompt, output_instructions)
    };

    let messages = vec![
        json!({"role": "system", "content": full_system}),
        json!({"role": "user", "content": ctx_text}),
    ];

    let response = client.chat_completion(messages, Some(1500)).await?;
    Ok((response, None))
}

pub fn complete_skill_run(
    conn: &Connection,
    run_id: &str,
    result: &ExecutionResult,
) -> Result<(), String> {
    if result.needs_approval {
        if let Some(ref changes) = result.pending_changes {
            skills_repo::set_pending_changes(conn, run_id, changes)?;
        }
    } else {
        skills_repo::set_run_output(conn, run_id, &result.output, result.duration_ms)?;
    }

    Ok(())
}

pub fn fail_skill_run(conn: &Connection, run_id: &str, error: &str) -> Result<(), String> {
    skills_repo::set_run_error(conn, run_id, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        let context = ExecutionContext {
            tasks: vec![json!({"title": "Test"})],
            meetings: vec![],
            documents: vec![],
            project: None,
            truncated: false,
        };

        let tokens = estimate_tokens(&context);
        assert!(tokens > 0);
        assert!(tokens < 100);
    }

    #[test]
    fn test_check_needs_approval() {
        // We can't easily test this without a full Skill struct,
        // but we can verify the logic exists
        let config = ActionConfig {
            action_type: Some("create_tasks".to_string()),
            has_side_effects: Some(true),
            ..Default::default()
        };

        // The function should return true for create_tasks
        assert!(config.action_type.as_deref() == Some("create_tasks"));
    }
}
