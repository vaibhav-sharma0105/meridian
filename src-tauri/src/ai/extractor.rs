use crate::ai::{litellm::LiteLLMClient, prompts};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtractedTask {
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub assignee_confidence: String,
    pub assignee_source_quote: Option<String>,
    pub due_date: Option<String>,
    pub due_confidence: String,
    pub due_source_quote: Option<String>,
    pub priority: Option<String>,
    pub confidence_score: Option<f64>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthData {
    pub had_agenda: bool,
    pub decisions_count: i32,
    pub tasks_count: i32,
    pub attendees_count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractionResult {
    pub summary: String,
    pub decisions: Vec<String>,
    pub tasks: Vec<ExtractedTask>,
    pub attendees: Vec<String>,
    pub health: HealthData,
}

pub async fn extract_tasks(
    litellm: &LiteLLMClient,
    transcript: &str,
    project_name: &str,
    existing_tasks: &[String],
    all_project_names: &[String],
) -> Result<ExtractionResult, String> {
    let existing_tasks_str = existing_tasks.join(", ");
    let all_projects_str = all_project_names.join(", ");
    let user_prompt = prompts::TASK_EXTRACTION_USER_TEMPLATE
        .replace("{{project_name}}", project_name)
        .replace("{{all_projects}}", &all_projects_str)
        .replace("{{existing_tasks}}", &existing_tasks_str)
        .replace("{{transcript}}", transcript);

    let messages = vec![
        json!({"role": "system", "content": prompts::TASK_EXTRACTION_SYSTEM}),
        json!({"role": "user", "content": user_prompt}),
    ];

    let response = litellm.chat_completion(messages.clone(), None).await?;

    // Try to parse the response
    match parse_extraction_response(&response) {
        Ok(result) => Ok(result),
        Err(_) => {
            // Retry with JSON repair instruction
            let repair_messages = vec![
                json!({"role": "system", "content": prompts::TASK_EXTRACTION_SYSTEM}),
                json!({"role": "user", "content": user_prompt}),
                json!({"role": "assistant", "content": response}),
                json!({"role": "user", "content": prompts::JSON_REPAIR_INSTRUCTION}),
            ];

            let repaired = litellm.chat_completion(repair_messages, None).await?;
            parse_extraction_response(&repaired).map_err(|e| {
                format!("Could not parse AI response after retry: {}. Raw response saved.", e)
            })
        }
    }
}

fn parse_extraction_response(response: &str) -> Result<ExtractionResult, String> {
    // Strip markdown code blocks if present
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str::<ExtractionResult>(cleaned)
        .map_err(|e| format!("JSON parse error: {}", e))
}

pub fn build_project_context(
    project_name: &str,
    open_tasks: &[crate::models::task::Task],
    completed_tasks: &[crate::models::task::Task],
    meetings: &[crate::models::meeting::Meeting],
    doc_chunks: &[crate::models::document::SearchResult],
) -> String {
    build_project_context_with_integrations(
        project_name,
        open_tasks,
        completed_tasks,
        meetings,
        doc_chunks,
        &[],
    )
}

pub fn build_project_context_with_integrations(
    project_name: &str,
    open_tasks: &[crate::models::task::Task],
    completed_tasks: &[crate::models::task::Task],
    meetings: &[crate::models::meeting::Meeting],
    doc_chunks: &[crate::models::document::SearchResult],
    integration_items: &[crate::integrations::models::IntegrationCache],
) -> String {
    let mut ctx = format!("Project: {}\n\n", project_name);

    ctx.push_str("=== OPEN TASKS ===\n");
    for t in open_tasks.iter().take(50) {
        let assignee = t.assignee.as_deref().unwrap_or("Unassigned");
        let due = t.due_date.as_deref().unwrap_or("No date");
        ctx.push_str(&format!("- {} | {} | Due: {}\n", t.title, assignee, due));
    }

    ctx.push_str("\n=== RECENTLY COMPLETED ===\n");
    for t in completed_tasks.iter().take(20) {
        let completed = t.completed_at.as_deref().unwrap_or("unknown");
        ctx.push_str(&format!("- {} (completed: {})\n", t.title, completed));
    }

    ctx.push_str("\n=== RECENT MEETINGS ===\n");
    for m in meetings.iter().take(3) {
        if let Some(summary) = &m.ai_summary {
            ctx.push_str(&format!("**{}**: {}\n\n", m.title, summary));
        }
    }

    if !doc_chunks.is_empty() {
        ctx.push_str("\n=== PROJECT DOCUMENTS ===\n");
        let mut sorted_docs: Vec<_> = doc_chunks.iter().collect();
        sorted_docs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        for chunk in sorted_docs.iter().take(10) {
            let preview = if chunk.chunk_text.len() > 2000 {
                format!("{}...", &chunk.chunk_text[..2000])
            } else {
                chunk.chunk_text.clone()
            };
            ctx.push_str(&format!(
                "--- Document: {} ---\n{}\n\n",
                chunk.filename, preview
            ));
        }
    }

    if !integration_items.is_empty() {
        ctx.push_str("\n=== INTEGRATION DATA ===\n");
        for item in integration_items.iter().take(20) {
            let type_label = match item.external_type.as_str() {
                "issue" => "GitHub Issue",
                "pr" | "pull_request" => "GitHub PR",
                "commit" => "Commit",
                "thread" => "Slack Thread",
                "message" => "Slack Message",
                _ => &item.external_type,
            };
            let title = item.data.get("title")
                .or_else(|| item.data.get("message"))
                .or_else(|| item.data.get("subject"))
                .and_then(|v| v.as_str())
                .unwrap_or(&item.external_id);
            let url = item.external_url.as_deref().unwrap_or("");
            ctx.push_str(&format!("- [{}] {} {}\n", type_label, title, url));
            if let Some(desc) = item.data.get("description").or_else(|| item.data.get("body")) {
                if let Some(s) = desc.as_str() {
                    let preview = if s.len() > 200 { format!("{}...", &s[..200]) } else { s.to_string() };
                    ctx.push_str(&format!("  {}\n", preview));
                }
            }
        }
    }

    ctx
}

/// Estimate token count for a string (rough approximation: ~4 chars per token)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Score an integration item for relevance based on multiple factors
fn score_integration_item(
    item: &crate::integrations::models::IntegrationCache,
    query: Option<&str>,
    user_id: Option<&str>,
) -> f64 {
    let mut score = 0.0;

    // 1. Attention score (if present) - highest priority
    if let Some(attention) = item.attention_score {
        score += attention * 100.0;
    }

    // 2. Recency - more recent items score higher
    if let Ok(synced) = DateTime::parse_from_rfc3339(&item.synced_at) {
        let age_hours = (Utc::now() - synced.with_timezone(&Utc)).num_hours() as f64;
        let recency_score = (168.0 - age_hours.min(168.0)) / 168.0 * 30.0; // Up to 30 points for items < 7 days old
        score += recency_score;
    }

    // 3. User assignment - items assigned to current user score higher
    if let Some(uid) = user_id {
        if let Some(assignee) = item.data.get("assignee").or(item.data.get("user")) {
            if let Some(assignee_str) = assignee.as_str() {
                if assignee_str.to_lowercase().contains(&uid.to_lowercase()) {
                    score += 50.0;
                }
            }
        }
    }

    // 4. Query relevance - if there's a search query, check if item matches
    if let Some(q) = query {
        let q_lower = q.to_lowercase();
        let title = item.data.get("title")
            .or(item.data.get("message"))
            .or(item.data.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let body = item.data.get("description")
            .or(item.data.get("body"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if title.to_lowercase().contains(&q_lower) {
            score += 80.0; // Direct title match
        } else if body.to_lowercase().contains(&q_lower) {
            score += 40.0; // Body match
        }
    }

    // 5. Type priority - PRs and issues generally more important than commits
    match item.external_type.as_str() {
        "pr" | "pull_request" => score += 10.0,
        "issue" => score += 8.0,
        "thread" => score += 6.0,
        _ => {}
    }

    score
}

/// Build integration context with token budget enforcement
pub fn build_integration_context_with_budget(
    integration_items: &[crate::integrations::models::IntegrationCache],
    token_budget: usize,
    query: Option<&str>,
    user_id: Option<&str>,
) -> String {
    if integration_items.is_empty() {
        return String::new();
    }

    // Score and sort items by relevance
    let mut scored_items: Vec<_> = integration_items
        .iter()
        .map(|item| (item, score_integration_item(item, query, user_id)))
        .collect();
    scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let total_items = scored_items.len();
    let mut ctx = String::from("=== INTEGRATION DATA ===\n");
    ctx.push_str("(Sorted by relevance; freshness timestamps indicate data age)\n\n");
    let mut tokens_used = estimate_tokens(&ctx);
    let mut items_included = 0;

    for (item, _score) in scored_items {
        // Format item with source attribution and timestamp
        let source = match item.external_type.as_str() {
            "issue" | "pr" | "pull_request" | "commit" => "GitHub",
            "thread" | "message" => "Slack",
            _ if item.data.get("jira_key").is_some() => "Jira",
            _ => "External",
        };
        let type_label = match item.external_type.as_str() {
            "issue" => "Issue",
            "pr" | "pull_request" => "PR",
            "commit" => "Commit",
            "thread" => "Thread",
            "message" => "Message",
            _ => &item.external_type,
        };
        let title = item.data.get("title")
            .or_else(|| item.data.get("message"))
            .or_else(|| item.data.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or(&item.external_id);
        let url = item.external_url.as_deref().unwrap_or("");

        // Calculate freshness
        let freshness = if let Ok(synced) = DateTime::parse_from_rfc3339(&item.synced_at) {
            let age = Utc::now() - synced.with_timezone(&Utc);
            if age.num_hours() < 1 {
                format!("{}m ago", age.num_minutes())
            } else if age.num_hours() < 24 {
                format!("{}h ago", age.num_hours())
            } else {
                format!("{}d ago", age.num_days())
            }
        } else {
            "unknown".to_string()
        };

        let mut item_text = format!("- [{}/{}] {} {} (synced: {})\n", source, type_label, title, url, freshness);

        // Add description if available (truncated)
        if let Some(desc) = item.data.get("description").or_else(|| item.data.get("body")) {
            if let Some(s) = desc.as_str() {
                let preview = if s.len() > 150 { format!("{}...", &s[..150]) } else { s.to_string() };
                item_text.push_str(&format!("  {}\n", preview));
            }
        }

        let item_tokens = estimate_tokens(&item_text);
        if tokens_used + item_tokens > token_budget {
            // Add truncation notice
            let remaining = total_items - items_included;
            if remaining > 0 {
                ctx.push_str(&format!("\n[... {} more items truncated due to token budget ...]\n", remaining));
            }
            break;
        }

        ctx.push_str(&item_text);
        tokens_used += item_tokens;
        items_included += 1;
    }

    ctx
}
