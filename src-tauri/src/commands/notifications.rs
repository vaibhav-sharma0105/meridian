use crate::db::repositories::notifications as repo;
use crate::models::ai_settings::AppNotification;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_notifications(state: State<'_, AppState>) -> Result<Vec<AppNotification>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_notifications(&conn)
}

#[tauri::command]
pub async fn mark_notification_read(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::mark_read(&conn, &id)
}

#[tauri::command]
pub async fn mark_all_read(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::mark_all_read(&conn)
}

#[tauri::command]
pub async fn create_notification(
    notification_type: String,
    title: String,
    body: String,
    task_id: Option<String>,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<AppNotification, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::create_notification(
        &conn,
        &notification_type,
        &title,
        &body,
        task_id.as_deref(),
        project_id.as_deref(),
    )
}
