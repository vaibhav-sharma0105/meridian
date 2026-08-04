use crate::ai::{litellm::LiteLLMClient, prompts};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
