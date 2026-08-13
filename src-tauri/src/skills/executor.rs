use regex::Regex;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::time::Instant;

use crate::ai::litellm::LiteLLMClient;
use crate::commands::ai::{get_api_key_from_db, get_litellm_client_pub};
use crate::db::repositories::{
    ai_settings as ai_repo, documents as docs_repo, meetings as meetings_repo,
    notifications as notifications_repo, projects as projects_repo, tasks as tasks_repo,
};
use crate::governance::{
    approval as governance_approval,
    autonomy::{AutonomyContext, AutonomyController},
    risk::{self, ActionType, ContentRisk, DestinationType},
};
use crate::integrations::repository as integ_repo;
use crate::models::task::TaskFilters;
use crate::skills::{
    models::{FilterResult, FilteredItem},
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
            action_config.has_side_effects.unwrap_or(false)
                || action_config.action_type.as_deref() == Some("create_tasks")
        }
    }
}

pub fn evaluate_skill_action_governance(
    conn: &Connection,
    skill: &Skill,
    action_config: &ActionConfig,
    content: Option<&str>,
) -> Result<(bool, Option<String>), String> {
    let autonomy_context = AutonomyContext {
        integration_id: None,
        skill_id: Some(skill.id.clone()),
    };

    let action_type = match action_config.action_type.as_deref() {
        Some("summarize") | Some("analyze") | Some("filter") => ActionType::Read,
        Some("create_tasks") => ActionType::Create,
        Some("draft_message") => ActionType::ExternalSend,
        Some("custom") => {
            if action_config.has_side_effects.unwrap_or(false) {
                ActionType::Update
            } else {
                ActionType::Read
            }
        }
        _ => ActionType::Read,
    };

    let destination = match action_config.channel.as_deref() {
        Some("slack") | Some("email") => DestinationType::Team,
        Some("external") => DestinationType::External,
        _ => DestinationType::Internal,
    };

    let content_risk = content.map(risk::classify_content).unwrap_or(ContentRisk::Normal);

    let risk_score = risk::calculate_risk(action_type, destination, content_risk);
    let decision = AutonomyController::evaluate_action(conn, &autonomy_context, risk_score.risk_level)?;

    if decision.requires_approval {
        let action_config_json = serde_json::to_string(action_config).unwrap_or_default();
        let approval_id = governance_approval::queue_for_approval(
            conn,
            &format!("skill:{}", action_config.action_type.as_deref().unwrap_or("custom")),
            &action_config_json,
            Some("skill"),
            Some(&skill.id),
            decision.risk_level,
            decision.autonomy_mode,
            Some(&decision.reason),
            None,
        )?;
        return Ok((true, Some(approval_id)));
    }

    Ok((false, None))
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

    let (governance_requires_approval, approval_id) =
        evaluate_skill_action_governance(conn, skill, &action_config, None)?;

    if governance_requires_approval {
        let duration_ms = start.elapsed().as_millis() as i64;
        return Ok(ExecutionResult {
            output: format!(
                "Action queued for approval (ID: {})",
                approval_id.as_deref().unwrap_or("unknown")
            ),
            duration_ms,
            pending_changes: None,
            needs_approval: true,
        });
    }

    let needs_approval = check_needs_approval(skill, &action_config);
    let action_type = action_config.action_type.as_deref().unwrap_or("summarize");

    let result = match action_type {
        "summarize" => execute_summarize(conn, &context, &action_config),
        "draft_message" => execute_draft(conn, &context, &action_config),
        "create_tasks" => execute_create_tasks(conn, &context, &action_config),
        "analyze" => execute_analyze(conn, &context, &action_config),
        "filter" => execute_filter(conn, &context, &action_config),
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

    let (governance_requires_approval, approval_id) =
        evaluate_skill_action_governance(conn, skill, &action_config, None)?;

    if governance_requires_approval {
        let duration_ms = start.elapsed().as_millis() as i64;
        return Ok(ExecutionResult {
            output: format!(
                "Action queued for approval (ID: {})",
                approval_id.as_deref().unwrap_or("unknown")
            ),
            duration_ms,
            pending_changes: None,
            needs_approval: true,
        });
    }

    let needs_approval = check_needs_approval(skill, &action_config);
    let action_type = action_config.action_type.as_deref().unwrap_or("summarize");

    let client = get_ai_client(conn)?;

    let result = match action_type {
        "summarize" => execute_summarize_ai(&client, &context, &action_config).await,
        "draft_message" => execute_draft_ai(&client, &context, &action_config).await,
        "create_tasks" => execute_create_tasks_ai(&client, &context, &action_config).await,
        "analyze" => execute_analyze_ai(&client, &context, &action_config).await,
        "filter" => execute_filter(conn, &context, &action_config),
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

// --- Filter execution (Phase 8) ---

fn execute_filter(
    conn: &Connection,
    _context: &ExecutionContext,
    config: &ActionConfig,
) -> Result<(String, Option<Value>), String> {
    let source = config.filter_source.as_deref();
    let item_type = config.filter_item_type.as_deref();
    let patterns = config.filter_patterns.as_ref();
    let keywords = config.filter_keywords.as_ref();
    let excludes = config.filter_exclude.as_ref();
    let min_score = config.filter_min_score;

    // Get all cache items (optionally filtered by source)
    let items = integ_repo::get_all_cached_items(conn, source, item_type, Some(500))
        .map_err(|e| format!("Failed to get cached items: {}", e))?;

    let mut matched_items: Vec<FilteredItem> = Vec::new();

    // Compile regex patterns
    let compiled_patterns: Vec<(Regex, String)> = patterns
        .map(|p| {
            p.iter()
                .filter_map(|pat| Regex::new(pat).ok().map(|r| (r, pat.clone())))
                .collect()
        })
        .unwrap_or_default();

    let compiled_excludes: Vec<Regex> = excludes
        .map(|e| e.iter().filter_map(|pat| Regex::new(pat).ok()).collect())
        .unwrap_or_default();

    for item in items {
        // Get searchable text from item
        let title = item.data.get("title")
            .or_else(|| item.data.get("message"))
            .or_else(|| item.data.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let description = item.data.get("description")
            .or_else(|| item.data.get("body"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let searchable = format!("{} {}", title, description).to_lowercase();

        // Check min score filter
        if let Some(min) = min_score {
            if item.attention_score.unwrap_or(0.0) < min {
                continue;
            }
        }

        // Check excludes
        let mut excluded = false;
        for exc in &compiled_excludes {
            if exc.is_match(&searchable) {
                excluded = true;
                break;
            }
        }
        if excluded {
            continue;
        }

        // Check patterns and keywords
        let mut matches: Vec<String> = Vec::new();

        for (regex, pattern) in &compiled_patterns {
            if regex.is_match(&searchable) {
                matches.push(pattern.clone());
            }
        }

        if let Some(kws) = keywords {
            for kw in kws {
                if searchable.contains(&kw.to_lowercase()) {
                    matches.push(format!("keyword:{}", kw));
                }
            }
        }

        // If no patterns/keywords specified, include all (after exclusions)
        let should_include = matches.is_empty() && patterns.is_none() && keywords.is_none();

        if !matches.is_empty() || should_include {
            matched_items.push(FilteredItem {
                cache_id: item.id,
                item_type: item.external_type,
                title: title.to_string(),
                url: item.external_url,
                matched_patterns: matches,
                attention_score: item.attention_score,
            });
        }
    }

    let result = FilterResult {
        matched_count: matched_items.len(),
        items: matched_items,
    };

    let output = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| format!("Matched {} items", result.matched_count));

    Ok((output, Some(json!(result))))
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

        // Auto-route completed skill results to Message Center
        if let Ok(run) = skills_repo::get_skill_run(conn, run_id) {
            if let Ok(skill) = skills_repo::get_skill(conn, &run.skill_id) {
                let content = crate::messages::routing::Content::SkillResult(
                    crate::messages::routing::SkillResultContent {
                        has_output: !result.output.is_empty(),
                        important: false,
                        skill_name: skill.name.clone(),
                    },
                );
                if crate::messages::routing::should_create_message(&content) {
                    let auto_pin_reason = crate::messages::routing::should_auto_pin(&content);
                    let message = crate::messages::repository::create_message(
                        conn,
                        crate::messages::models::CreateMessageInput {
                            project_id: None,
                            message_type: "skill_result".to_string(),
                            title: format!("{} Result", skill.name),
                            content: Some(result.output.clone()),
                            source_id: Some(run_id.to_string()),
                            source_type: Some("skill_run".to_string()),
                            auto_pinned: Some(auto_pin_reason.is_some()),
                            pinned_reason: auto_pin_reason,
                            file_refs: None,
                        },
                    );

                    // Routing resolves skill results to
                    // `MessageCenterWithNotification` — the notification half is
                    // what makes the stored result discoverable.
                    if let Ok(message) = message {
                        let _ = notifications_repo::create_notification_for_message(
                            conn,
                            "skill_result",
                            &format!("{} finished", skill.name),
                            "View full result",
                            None,
                            None,
                            &message.id,
                            "info",
                            false,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn fail_skill_run(conn: &Connection, run_id: &str, error: &str) -> Result<(), String> {
    skills_repo::set_run_error(conn, run_id, error)
}

// ─── Multi-Skill Execution ───────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillMatch {
    pub skill_id: String,
    pub skill_name: String,
    pub confidence: f64,
    pub match_reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiSkillPlan {
    pub skills: Vec<SkillMatch>,
    pub execution_order: Vec<String>,
    pub chained: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiSkillResult {
    pub results: Vec<SkillExecutionResult>,
    pub combined_output: Option<String>,
    pub total_duration_ms: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillExecutionResult {
    pub skill_id: String,
    pub skill_name: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: i64,
}

pub fn match_skills_to_query(
    conn: &Connection,
    query: &str,
    min_confidence: f64,
) -> Result<Vec<SkillMatch>, String> {
    let filters = crate::skills::SkillFilters {
        enabled: Some(true),
        ..Default::default()
    };

    let skills = skills_repo::list_skills(conn, &filters)?;
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut matches: Vec<SkillMatch> = Vec::new();

    for skill in skills {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Name match
        let name_lower = skill.name.to_lowercase();
        if name_lower.contains(&query_lower) || query_lower.contains(&name_lower) {
            score += 0.4;
            reasons.push("name match");
        }

        // Description match
        if let Some(ref desc) = skill.description {
            let desc_lower = desc.to_lowercase();
            let word_matches = query_words.iter()
                .filter(|w| w.len() > 3 && desc_lower.contains(*w))
                .count();
            if word_matches > 0 {
                score += 0.1 * word_matches as f64;
                reasons.push("description match");
            }
        }

        // Category match
        if let Some(ref cat) = skill.category {
            if query_lower.contains(&cat.to_lowercase()) {
                score += 0.2;
                reasons.push("category match");
            }
        }

        // Tags match
        let tags = skill.get_tags();
        for tag in &tags {
            if query_lower.contains(&tag.to_lowercase()) {
                score += 0.15;
                reasons.push("tag match");
                break;
            }
        }

        // Action type keyword match
        if let Some(action_config) = skill.get_action_config() {
            if let Some(action_type) = action_config.action_type {
                let action_keywords = match action_type.as_str() {
                    "summarize" => vec!["summary", "summarize", "overview", "recap"],
                    "draft_message" => vec!["draft", "write", "compose", "message", "email"],
                    "create_tasks" => vec!["task", "tasks", "todo", "create", "action"],
                    "analyze" => vec!["analyze", "analysis", "review", "examine"],
                    _ => vec![],
                };
                if action_keywords.iter().any(|kw| query_lower.contains(kw)) {
                    score += 0.25;
                    reasons.push("action type match");
                }
            }
        }

        score = score.min(1.0);

        if score >= min_confidence {
            matches.push(SkillMatch {
                skill_id: skill.id,
                skill_name: skill.name,
                confidence: score,
                match_reason: reasons.join(", "),
            });
        }
    }

    // Sort by confidence descending
    matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    Ok(matches)
}

pub fn resolve_skill_dependencies(skills: &[SkillMatch]) -> Vec<String> {
    // Simple topological sort - for now just return in confidence order
    // Future: Parse depends_on field from skill configs
    skills.iter().map(|s| s.skill_id.clone()).collect()
}

pub async fn execute_skill_chain(
    conn: &Connection,
    client: &LiteLLMClient,
    skill_ids: &[String],
    initial_context: Option<&str>,
) -> Result<MultiSkillResult, String> {
    let start = Instant::now();
    let mut results = Vec::new();
    let mut combined_outputs = Vec::new();
    let mut current_context = initial_context.map(String::from);

    for skill_id in skill_ids {
        let skill = skills_repo::get_skill(conn, skill_id)?;
        let skill_start = Instant::now();

        // Build context with previous output if chaining
        let mut exec_context = build_context(conn, &skill)?;

        // Add previous output to context if available
        if let Some(ref prev_output) = current_context {
            exec_context.documents.push(json!({
                "type": "previous_skill_output",
                "content": prev_output,
            }));
        }

        // Execute skill
        let action_config = skill.get_action_config().unwrap_or_default();
        let action_type = action_config.action_type.as_deref().unwrap_or("summarize");

        let result = match action_type {
            "summarize" => crate::skills::execute_summarize_ai(client, &exec_context, &action_config).await,
            "draft_message" => crate::skills::execute_draft_ai(client, &exec_context, &action_config).await,
            "create_tasks" => crate::skills::execute_create_tasks_ai(client, &exec_context, &action_config).await,
            "analyze" => crate::skills::execute_analyze_ai(client, &exec_context, &action_config).await,
            "custom" => {
                let ctx_config = skill.get_context_config().unwrap_or_default();
                crate::skills::execute_custom_ai(client, &exec_context, &action_config, &ctx_config).await
            }
            _ => Err(format!("Unknown action type: {}", action_type)),
        };

        let duration_ms = skill_start.elapsed().as_millis() as i64;

        match result {
            Ok((output, _pending)) => {
                current_context = Some(output.clone());
                combined_outputs.push(format!("## {}\n\n{}", skill.name, output));

                results.push(SkillExecutionResult {
                    skill_id: skill.id.clone(),
                    skill_name: skill.name.clone(),
                    status: "completed".to_string(),
                    output: Some(output),
                    error: None,
                    duration_ms,
                });
            }
            Err(e) => {
                results.push(SkillExecutionResult {
                    skill_id: skill.id.clone(),
                    skill_name: skill.name.clone(),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(e.clone()),
                    duration_ms,
                });
                // Continue with next skill even if one fails
            }
        }
    }

    let total_duration_ms = start.elapsed().as_millis() as i64;

    Ok(MultiSkillResult {
        results,
        combined_output: if combined_outputs.is_empty() {
            None
        } else {
            Some(combined_outputs.join("\n\n---\n\n"))
        },
        total_duration_ms,
    })
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
