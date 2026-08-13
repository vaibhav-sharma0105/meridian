use tauri::State;

use crate::messages::{
    self, CleanupStats, CreateMessageInput, Message, MessageFilters, PaginatedMessages,
    StorageStats,
};
use crate::AppState;

#[tauri::command]
pub fn get_messages(
    state: State<AppState>,
    filters: Option<MessageFilters>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<PaginatedMessages, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let filters = filters.unwrap_or_default();
    messages::get_messages(&conn, &filters, page.unwrap_or(1), per_page.unwrap_or(20))
}

#[tauri::command]
pub fn get_message(state: State<AppState>, id: String) -> Result<Message, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::repository::get_message(&conn, &id)
}

#[tauri::command]
pub fn pin_message(
    state: State<AppState>,
    source_type: String,
    source_id: String,
    title: String,
    content: Option<String>,
    project_id: Option<String>,
) -> Result<Message, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::pin_from_source(
        &conn,
        &source_type,
        &source_id,
        &title,
        content.as_deref(),
        project_id.as_deref(),
    )
}

#[tauri::command]
pub fn create_message(
    state: State<AppState>,
    input: CreateMessageInput,
) -> Result<Message, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::create_message(&conn, input)
}

#[tauri::command]
pub fn delete_message(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::soft_delete_message(&conn, &id)
}

#[tauri::command]
pub fn restore_message(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::restore_message(&conn, &id)
}

#[tauri::command]
pub fn get_storage_stats(state: State<AppState>) -> Result<StorageStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::calculate_storage_usage(&conn)
}

#[tauri::command]
pub fn get_messages_for_ai_context(
    state: State<AppState>,
    project_id: Option<String>,
    ai_context_days: Option<i64>,
) -> Result<Vec<Message>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::get_messages_for_ai_context(
        &conn,
        project_id.as_deref(),
        ai_context_days.unwrap_or(30),
    )
}

#[tauri::command]
pub fn cleanup_messages(state: State<AppState>) -> Result<CleanupStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::cleanup_expired_messages(&conn)
}

#[tauri::command]
pub fn get_deleted_messages(
    state: State<AppState>,
    limit: Option<i64>,
) -> Result<Vec<Message>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    messages::get_soft_deleted_messages(&conn, limit.unwrap_or(50))
}
