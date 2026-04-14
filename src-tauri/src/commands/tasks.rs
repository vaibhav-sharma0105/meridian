use crate::db::repositories::tasks as repo;
use crate::models::task::{CreateTaskInput, PartialTaskUpdate, Task, TaskFilters, UpdateTaskInput};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_tasks_for_project(
    project_id: String,
    filters: Option<TaskFilters>,
    state: State<'_, AppState>,
) -> Result<Vec<Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let filters = filters.unwrap_or_default();
    repo::get_tasks_for_project(&conn, &project_id, &filters)
}

#[tauri::command]
pub async fn get_all_tasks(
    filters: Option<TaskFilters>,
    state: State<'_, AppState>,
) -> Result<Vec<Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let filters = filters.unwrap_or_default();
    repo::get_all_tasks(&conn, &filters)
}

#[tauri::command]
pub async fn create_task(
    input: CreateTaskInput,
    state: State<'_, AppState>,
) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::create_task(&conn, &input)
}

#[tauri::command]
pub async fn update_task(
    input: UpdateTaskInput,
    state: State<'_, AppState>,
) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::update_task(&conn, &input)
}

#[tauri::command]
pub async fn bulk_update_tasks(
    task_ids: Vec<String>,
    updates: PartialTaskUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::bulk_update_tasks(&conn, &task_ids, &updates)
}

#[tauri::command]
pub async fn reorder_tasks(
    task_id: String,
    new_column: String,
    new_order: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::reorder_task(&conn, &task_id, &new_column, new_order)
}

#[tauri::command]
pub async fn delete_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::delete_task(&conn, &id)
}
