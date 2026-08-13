use tauri::State;

use crate::role::{self, InferenceStatus, UserProfile};
use crate::role::models::RoleDriftAlert;
use crate::AppState;

#[tauri::command]
pub fn get_user_profile(state: State<AppState>) -> Result<UserProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::get_or_create_user_profile(&conn)
}

#[tauri::command]
pub fn confirm_role(
    state: State<AppState>,
    role: String,
    custom_description: Option<String>,
) -> Result<UserProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::confirm_role(&conn, &role, custom_description.as_deref())
}

#[tauri::command]
pub fn change_role(
    state: State<AppState>,
    role: String,
    custom_description: Option<String>,
) -> Result<UserProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::change_role(&conn, &role, custom_description.as_deref())
}

/// Sets who "me" is, so role-based My Activity ordering can tell the user's own
/// items apart from their team's. Omitted fields are left unchanged.
#[tauri::command]
pub fn update_user_identity(
    state: State<AppState>,
    display_name: Option<String>,
    user_email: Option<String>,
    user_aliases: Option<Vec<String>>,
) -> Result<UserProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::repository::update_user_identity(
        &conn,
        display_name.as_deref(),
        user_email.as_deref(),
        user_aliases.as_deref(),
    )
}

#[tauri::command]
pub fn get_role_inference_status(state: State<AppState>) -> Result<InferenceStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::get_inference_status(&conn)
}

#[tauri::command]
pub fn dismiss_role_drift_alert(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::dismiss_drift_alert(&conn)
}

/// Returns a drift alert if recent behaviour diverges from the confirmed role.
/// Polled by the frontend — the daemon cannot emit Tauri events, so this is a
/// query rather than an event subscription.
#[tauri::command]
pub fn get_role_drift_alert(state: State<AppState>) -> Result<Option<RoleDriftAlert>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::repository::check_role_drift(&conn)
}

#[tauri::command]
pub fn run_role_inference(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    role::run_role_inference(&conn)
}

#[tauri::command]
pub fn update_retention_settings(
    state: State<AppState>,
    ai_context_days: Option<i64>,
    message_retention: Option<String>,
    productivity_tracking_enabled: Option<bool>,
    archive_old_files: Option<bool>,
    archive_after_days: Option<i64>,
) -> Result<UserProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    let mut updates = vec![];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(days) = ai_context_days {
        updates.push(format!("ai_context_days = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(days));
    }

    if let Some(retention) = message_retention {
        updates.push(format!("message_retention = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(retention));
    }

    if let Some(enabled) = productivity_tracking_enabled {
        updates.push(format!(
            "productivity_tracking_enabled = ?{}",
            params_vec.len() + 1
        ));
        params_vec.push(Box::new(enabled as i32));
    }

    if let Some(enabled) = archive_old_files {
        updates.push(format!("archive_old_files = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(enabled as i32));
    }

    if let Some(days) = archive_after_days {
        updates.push(format!("archive_after_days = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(days));
    }

    if updates.is_empty() {
        return role::get_user_profile(&conn);
    }

    updates.push(format!("updated_at = ?{}", params_vec.len() + 1));
    params_vec.push(Box::new(now));

    let sql = format!(
        "UPDATE user_profile SET {} WHERE id = 'default'",
        updates.join(", ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params_refs.as_slice())
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    role::get_user_profile(&conn)
}
