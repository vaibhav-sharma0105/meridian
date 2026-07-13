use crate::audit::{log_user_action, ActionType, EntityType};
use crate::db::repositories::projects as repo;
use crate::models::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_all_projects(&conn)
}

#[tauri::command]
pub async fn create_project(
    input: CreateProjectInput,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let project = repo::create_project(&conn, &input)?;

    let _ = log_user_action(
        &conn,
        ActionType::Create,
        EntityType::Project,
        Some(project.id.clone()),
        Some(json!({"name": project.name})),
    );

    Ok(project)
}

#[tauri::command]
pub async fn update_project(
    input: UpdateProjectInput,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let project = repo::update_project(&conn, &input)?;

    let _ = log_user_action(
        &conn,
        ActionType::Update,
        EntityType::Project,
        Some(project.id.clone()),
        Some(json!({"name": project.name})),
    );

    Ok(project)
}

#[tauri::command]
pub async fn archive_project(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::archive_project(&conn, &id)?;

    let _ = log_user_action(
        &conn,
        ActionType::Delete,
        EntityType::Project,
        Some(id),
        Some(json!({"type": "archive"})),
    );

    Ok(())
}
