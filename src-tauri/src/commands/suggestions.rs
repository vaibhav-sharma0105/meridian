use tauri::State;

use crate::suggestions::models::{CreateSuggestionInput, Suggestion};
use crate::suggestions::repository;
use crate::patterns::models::CreateObservationInput;
use crate::patterns::repository as patterns_repo;
use crate::AppState;

#[tauri::command]
pub async fn get_pending_suggestions(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<Suggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_pending_suggestions(&conn, project_id.as_deref())
}

#[tauri::command]
pub async fn accept_suggestion(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_suggestion_status(&conn, &id, "accepted")
}

#[tauri::command]
pub async fn dismiss_suggestion(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_suggestion_status(&conn, &id, "dismissed")
}

#[tauri::command]
pub async fn stop_suggesting(
    state: State<'_, AppState>,
    id: String,
    suggestion_type: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    repository::update_suggestion_status(&conn, &id, "dismissed")?;

    patterns_repo::insert_observation(
        &conn,
        CreateObservationInput {
            observation_type: "suggestion_rejection".to_string(),
            entity_type: Some("suggestion".to_string()),
            entity_id: Some(id.clone()),
            project_id: None,
            context_data: serde_json::json!({
                "suggestion_id": id,
                "suggestion_type": suggestion_type,
                "action": "stop_suggesting"
            }),
        },
    )?;

    Ok(())
}

#[tauri::command]
pub async fn create_suggestion(
    state: State<'_, AppState>,
    input: CreateSuggestionInput,
) -> Result<Suggestion, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_suggestion(&conn, input)
}

#[tauri::command]
pub async fn get_suggestion_count_today(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_suggestions_count_today(&conn)
}
