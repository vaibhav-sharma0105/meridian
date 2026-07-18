use crate::db::repositories::notifications as repo;
use crate::models::ai_settings::AppNotification;
use crate::AppState;
use rusqlite::Connection;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

fn get_app_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    )
    .ok()
}

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

#[tauri::command]
pub async fn create_notification_with_options(
    app: AppHandle,
    notification_type: String,
    title: String,
    body: String,
    task_id: Option<String>,
    project_id: Option<String>,
    skill_run_id: Option<String>,
    integration_id: Option<String>,
    severity: Option<String>,
    desktop: Option<bool>,
    state: State<'_, AppState>,
) -> Result<AppNotification, String> {
    let severity = severity.unwrap_or_else(|| "info".to_string());
    let desktop = desktop.unwrap_or(false);

    let notification = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        repo::create_notification_full(
            &conn,
            &notification_type,
            &title,
            &body,
            task_id.as_deref(),
            project_id.as_deref(),
            skill_run_id.as_deref(),
            integration_id.as_deref(),
            &severity,
            desktop,
        )?
    };

    if desktop {
        let should_send = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            get_app_setting(&conn, "desktop_notifications_enabled")
                .map(|v| v != "false")
                .unwrap_or(true)
        };

        if should_send {
            send_desktop_notification(&app, &title, &body, &severity);
        }
    }

    Ok(notification)
}

fn send_desktop_notification(app: &AppHandle, title: &str, body: &str, severity: &str) {
    let mut builder = app.notification().builder();
    builder = builder.title(title).body(body);

    if severity == "critical" {
        builder = builder.sound("default");
    }

    if let Err(e) = builder.show() {
        eprintln!("Failed to show desktop notification: {}", e);
    }
}

#[tauri::command]
pub async fn check_notification_permission(app: AppHandle) -> Result<bool, String> {
    match app.notification().permission_state() {
        Ok(state) => Ok(state == tauri_plugin_notification::PermissionState::Granted),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn request_notification_permission(app: AppHandle) -> Result<bool, String> {
    match app.notification().request_permission() {
        Ok(state) => Ok(state == tauri_plugin_notification::PermissionState::Granted),
        Err(e) => Err(e.to_string()),
    }
}
