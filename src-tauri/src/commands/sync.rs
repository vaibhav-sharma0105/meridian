use std::path::PathBuf;
use tauri::State;

use crate::sync::{
    export::{export_data, export_skill_standalone, ExportOptions, ExportResult},
    import::{
        import_data, import_skill_standalone, preview_import, ConflictResolution, ImportOptions,
        ImportPreview, ImportResult,
    },
};
use crate::AppState;

#[tauri::command]
pub async fn export_all_data(
    state: State<'_, AppState>,
    output_path: String,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&output_path);
    export_data(&conn, &path, &options)
}

#[tauri::command]
pub async fn export_single_skill(
    state: State<'_, AppState>,
    skill_id: String,
    output_path: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&output_path);
    export_skill_standalone(&conn, &skill_id, &path)
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
    state: State<'_, AppState>,
    archive_path: String,
    options: ImportOptions,
    conflict_resolutions: std::collections::HashMap<String, String>,
) -> Result<ImportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&archive_path);

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

    import_data(&conn, &path, &options, &resolutions)
}

#[tauri::command]
pub async fn import_single_skill(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<crate::skills::models::Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&file_path);
    import_skill_standalone(&conn, &path)
}
