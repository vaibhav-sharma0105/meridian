use crate::db::repositories::prompt_templates as tpl_repo;
use crate::models::ai_settings::PromptTemplate;
use crate::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?;

    let settings: HashMap<String, String> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect();

    Ok(settings)
}

#[tauri::command]
pub async fn set_app_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_prompt_templates(state: State<'_, AppState>) -> Result<Vec<PromptTemplate>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    tpl_repo::get_all_templates(&conn)
}

#[tauri::command]
pub async fn save_prompt_template(
    template: PromptTemplate,
    state: State<'_, AppState>,
) -> Result<PromptTemplate, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    tpl_repo::save_template(&conn, &template)
}
