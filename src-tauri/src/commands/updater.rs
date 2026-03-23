use crate::utils::backup;
use serde_json::{json, Value};

#[tauri::command]
pub async fn check_for_updates() -> Result<Value, String> {
    // This would normally use tauri-plugin-updater to check GitHub releases
    // For now return a stub that can be wired up with the actual endpoint
    Ok(json!({
        "update_available": false,
        "version": null,
        "release_notes": null,
    }))
}

#[tauri::command]
pub async fn backup_database() -> Result<String, String> {
    backup::backup_database()
}
