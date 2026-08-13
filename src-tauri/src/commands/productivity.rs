use chrono::Timelike;
use tauri::State;

use crate::productivity::{self, BatchingSuggestion, ProductivityExport, ProductivityInsights, ProductivitySettings, TaskCategory, TimeSuggestion};
use crate::AppState;

#[tauri::command]
pub fn get_productivity_insights(state: State<AppState>) -> Result<ProductivityInsights, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    productivity::get_productivity_insights(&conn)
}

#[tauri::command]
pub fn get_time_suggestion(
    state: State<AppState>,
    task_id: String,
) -> Result<Option<TimeSuggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get task details to determine category
    let (title, estimated_minutes, source_type): (String, Option<i32>, Option<String>) = conn
        .query_row(
            "SELECT title, estimated_minutes, source_type FROM tasks WHERE id = ?1",
            rusqlite::params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Task not found: {}", e))?;

    let category = productivity::classify_task_category(&title, estimated_minutes, source_type.as_deref());
    let patterns = productivity::get_productivity_patterns(&conn)?;

    Ok(productivity::suggest_task_time(&patterns, category))
}

#[tauri::command]
pub fn get_time_suggestion_for_category(
    state: State<AppState>,
    category: String,
) -> Result<Option<TimeSuggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let patterns = productivity::get_productivity_patterns(&conn)?;

    let task_category = TaskCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid category: {}", category))?;

    Ok(productivity::suggest_task_time(&patterns, task_category))
}

#[tauri::command]
pub fn update_productivity_settings(
    state: State<AppState>,
    settings: ProductivitySettings,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE user_profile SET
            productivity_tracking_enabled = ?1,
            show_suggestions = ?2,
            data_retention_days = ?3,
            updated_at = ?4
         WHERE id = 'default'",
        rusqlite::params![
            settings.tracking_enabled as i32,
            settings.show_suggestions as i32,
            settings.data_retention_days as i64,
            now
        ],
    )
    .map_err(|e| format!("Failed to update settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_productivity_settings(state: State<AppState>) -> Result<ProductivitySettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT COALESCE(productivity_tracking_enabled, 1),
                COALESCE(show_suggestions, 1),
                COALESCE(data_retention_days, 365)
         FROM user_profile WHERE id = 'default'",
        [],
        |row| {
            Ok(ProductivitySettings {
                tracking_enabled: row.get::<_, i64>(0)? != 0,
                show_suggestions: row.get::<_, i64>(1)? != 0,
                data_retention_days: row.get::<_, i64>(2)? as u32,
            })
        },
    )
    .map_err(|e| format!("Failed to read productivity settings: {}", e))
}

#[tauri::command]
pub fn export_productivity_data(state: State<AppState>) -> Result<ProductivityExport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    productivity::export_productivity_data(&conn)
}

#[tauri::command]
pub fn clear_productivity_data(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    productivity::clear_productivity_data(&conn)
}

#[tauri::command]
pub fn aggregate_productivity_patterns(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    productivity::aggregate_patterns(&conn)?;
    Ok(())
}

/// Suggests batching meetings when today's schedule is fragmented.
/// Meeting hours are derived from `meetings.meeting_at` for the given day
/// (defaults to today, local date) rather than taken from the caller.
#[tauri::command]
pub fn get_meeting_batching_suggestion(
    state: State<AppState>,
    date: Option<String>,
) -> Result<Option<BatchingSuggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let day = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    let mut stmt = conn
        .prepare(
            "SELECT meeting_at FROM meetings
             WHERE meeting_at IS NOT NULL
               AND substr(meeting_at, 1, 10) = ?1
               AND archived_at IS NULL",
        )
        .map_err(|e| e.to_string())?;

    let meeting_hours: Vec<u8> = stmt
        .query_map(rusqlite::params![day], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(|ts| {
            chrono::DateTime::parse_from_rfc3339(&ts)
                .ok()
                .map(|dt| dt.hour() as u8)
        })
        .collect();

    let patterns = productivity::get_productivity_patterns(&conn)?;
    Ok(productivity::suggest_meeting_batching(&meeting_hours, &patterns))
}
