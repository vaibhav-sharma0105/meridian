use tauri::State;

use crate::commands::ai::{get_api_key_from_db, get_litellm_client_pub};
use crate::db::repositories::{ai_settings as ai_settings_repo, tasks as tasks_repo};
use crate::drafts::models::{CreateDraftInput, DraftMessage, UpdateDraftInput};
use crate::drafts::repository;
use crate::patterns::repository as patterns_repo;
use crate::sensitive;
use crate::AppState;

#[tauri::command]
pub async fn get_drafts_for_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<DraftMessage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_drafts_for_task(&conn, &task_id)
}

#[tauri::command]
pub async fn generate_draft(
    state: State<'_, AppState>,
    task_id: String,
    channel: String,
) -> Result<DraftMessage, String> {
    let (task, ai_settings, api_key, comm_style) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        let task = tasks_repo::get_task(&conn, &task_id)?;
        let ai_settings = ai_settings_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI settings configured".to_string())?;
        let api_key = get_api_key_from_db(&conn, &ai_settings.label);
        let comm_style = patterns_repo::get_pattern_model_by_type(&conn, "communication_style", None).ok();

        (task, ai_settings, api_key, comm_style)
    };

    let mut style_hints = String::new();
    if let Some(model) = comm_style {
        if let Ok(data) = serde_json::from_str::<crate::patterns::models::CommunicationStyleModelData>(&model.model_data) {
            style_hints.push_str(&format!("- Length preference: {}\n", data.length_preference));
            style_hints.push_str(&format!("- Formality: {}\n", data.formality_level));
            if !data.common_additions.is_empty() {
                let phrases: Vec<_> = data.common_additions.iter().take(5).map(|(p, _)| p.as_str()).collect();
                style_hints.push_str(&format!("- Common phrases to include: {}\n", phrases.join(", ")));
            }
        }
    }

    let prompt = format!(
        r#"Generate a draft {} message for the following task:

Task: {}
Description: {}

Style preferences:
{}

Generate a professional message that accomplishes the task.
Keep it concise and actionable.
End with signature line: "Drafted by Meridian"

Respond with only the message body, no explanations."#,
        channel,
        task.title,
        task.description.clone().unwrap_or_default(),
        if style_hints.is_empty() { "- Use professional tone\n- Be concise".to_string() } else { style_hints }
    );

    let client = get_litellm_client_pub(&ai_settings, &api_key);
    let messages = vec![
        serde_json::json!({
            "role": "user",
            "content": prompt
        })
    ];
    let body = client.chat_completion(messages, None)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_draft(
        &conn,
        CreateDraftInput {
            task_id: Some(task_id),
            channel,
            recipient: None,
            subject: Some(task.title),
            body,
            ai_signature: Some(true),
        },
    )
}

#[tauri::command]
pub async fn update_draft(
    state: State<'_, AppState>,
    id: String,
    input: UpdateDraftInput,
) -> Result<DraftMessage, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_draft(&conn, &id, input)
}

#[tauri::command]
pub async fn delete_draft(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::delete_draft(&conn, &id)
}

#[tauri::command]
pub async fn scan_draft(
    state: State<'_, AppState>,
    content: String,
    draft_id: Option<String>,
) -> Result<Vec<sensitive::SensitiveWarning>, String> {
    let warnings = sensitive::scan_content(&content);

    if !warnings.is_empty() {
        if let Ok(conn) = state.db.lock() {
            let _ = crate::audit::log_user_action(
                &conn,
                crate::audit::ActionType::SensitiveDetected,
                crate::audit::EntityType::Draft,
                draft_id,
                Some(serde_json::json!({
                    "warning_count": warnings.len(),
                    "warning_types": warnings.iter().map(|w| w.warning_type.clone()).collect::<Vec<_>>(),
                    "has_critical": warnings.iter().any(|w| w.severity == "critical"),
                })),
            );
        }
    }

    Ok(warnings)
}
