use std::path::PathBuf;
use tauri::{Emitter, State};

use crate::sync::{
    export::{build_local_entries, finish_export, ExportOptions, ExportResult},
    import::{
        apply_local_import, finish_import, preview_import,
        ConflictResolution, ImportOptions, ImportPreview, ImportResult,
    },
};
use crate::AppState;

#[tauri::command]
pub async fn export_all_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    output_path: String,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    let path = PathBuf::from(&output_path);
    let progress = |label: &str, current: u32, total: u32| {
        let _ = app.emit(
            "export_progress",
            serde_json::json!({ "step": label, "current": current, "total": total }),
        );
    };

    // Must not hold the DB MutexGuard past this block — build_local_entries
    // is synchronous and finishes (and drops its `conn` borrow) before
    // finish_export's first `.await`. See build_local_entries's doc comment.
    let (zip, contents, checksum_entries) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        build_local_entries(&conn, &options, Some(&progress))?
    };

    finish_export(zip, contents, checksum_entries, &path, &options, Some(&progress)).await
}

#[tauri::command]
pub async fn preview_import_data(
    state: State<'_, AppState>,
    archive_path: String,
    options: ImportOptions,
) -> Result<ImportPreview, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&archive_path);
    preview_import(&conn, &path, &options)
}

#[tauri::command]
pub async fn import_all_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    archive_path: String,
    options: ImportOptions,
    conflict_resolutions: std::collections::HashMap<String, String>,
) -> Result<ImportResult, String> {
    let path = PathBuf::from(&archive_path);
    let progress = |label: &str, current: u32, total: u32| {
        let _ = app.emit(
            "import_progress",
            serde_json::json!({ "step": label, "current": current, "total": total }),
        );
    };

    // Convert string resolutions to enum
    let resolutions: std::collections::HashMap<String, ConflictResolution> = conflict_resolutions
        .into_iter()
        .map(|(k, v)| {
            let res = match v.as_str() {
                "skip" => ConflictResolution::Skip,
                "overwrite" => ConflictResolution::Overwrite,
                _ => ConflictResolution::Ask,
            };
            (k, res)
        })
        .collect();

    // Must not hold the DB MutexGuard past this block — see
    // apply_local_import's doc comment.
    let (result, vector_snapshots) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        apply_local_import(&conn, &path, &options, &resolutions, Some(&progress))?
    };

    Ok(finish_import(result, vector_snapshots, Some(&progress)).await)
}

/// Native "save file" dialog for choosing where an export archive goes.
/// Same osascript/rfd split as `pick_folder_dialog` in commands/skills.rs —
/// `@tauri-apps/plugin-dialog`'s save() doesn't work in this Tauri v2 webview.
#[tauri::command]
pub async fn pick_export_save_path(default_name: String) -> Result<Option<String>, String> {
    let handle = tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "POSIX path of (choose file name with prompt \"Save Meridian export\" default name \"{}\")",
                default_name.replace('"', "")
            );
            let output = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output()
                .map_err(|e| format!("Failed to open save dialog: {}", e))?;

            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() { Ok(None) } else { Ok(Some(path)) }
            } else {
                Ok(None) // user cancelled
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let file = rfd::FileDialog::new()
                .set_title("Save Meridian export")
                .set_file_name(&default_name)
                .add_filter("Meridian Export", &["zip"])
                .save_file();
            Ok(file.map(|p| p.to_string_lossy().to_string()))
        }
    });
    handle.await.map_err(|e| e.to_string())?
}

/// Native "open file" dialog for choosing an export archive to import.
#[tauri::command]
pub async fn pick_import_file_path() -> Result<Option<String>, String> {
    let handle = tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("osascript")
                .args([
                    "-e",
                    "POSIX path of (choose file with prompt \"Select a Meridian export archive\" of type {\"zip\"})",
                ])
                .output()
                .map_err(|e| format!("Failed to open file picker: {}", e))?;

            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() { Ok(None) } else { Ok(Some(path)) }
            } else {
                Ok(None) // user cancelled
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let file = rfd::FileDialog::new()
                .set_title("Select a Meridian export archive")
                .add_filter("Meridian Export", &["zip"])
                .pick_file();
            Ok(file.map(|p| p.to_string_lossy().to_string()))
        }
    });
    handle.await.map_err(|e| e.to_string())?
}
