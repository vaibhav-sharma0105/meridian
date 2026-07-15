use crate::audit::{log_user_action, ActionType, EntityType};
use crate::db::repositories::tasks as repo;
use crate::models::task::{CreateTaskInput, PartialTaskUpdate, Task, TaskFilters, UpdateTaskInput};
use crate::patterns::models::CreateObservationInput;
use crate::patterns::repository as patterns_repo;
use crate::skills::EventDispatcher;
use crate::AppState;
use serde_json::json;
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
    let task = repo::create_task(&conn, &input)?;

    let _ = log_user_action(
        &conn,
        ActionType::Create,
        EntityType::Task,
        Some(task.id.clone()),
        Some(json!({
            "title": task.title,
            "project_id": task.project_id
        })),
    );

    // Fire event for skill triggers
    let _ = EventDispatcher::fire_task_created(
        &conn,
        &task.id,
        &task.project_id,
        &task.title,
        &task.priority,
        task.assignee.as_deref(),
    );

    Ok(task)
}

#[tauri::command]
pub async fn update_task(
    input: UpdateTaskInput,
    state: State<'_, AppState>,
) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let old_task = repo::get_task(&conn, &input.id).ok();

    let task = repo::update_task(&conn, &input)?;

    let _ = log_user_action(
        &conn,
        ActionType::Update,
        EntityType::Task,
        Some(task.id.clone()),
        Some(json!({
            "title": task.title,
            "status": task.status
        })),
    );

    if let Some(old) = old_task {
        if let Some(new_status) = &input.status {
            if new_status == "done" && old.status != "done" {
                let _ = patterns_repo::insert_observation(
                    &conn,
                    CreateObservationInput {
                        observation_type: "task_completion".to_string(),
                        entity_type: Some("task".to_string()),
                        entity_id: Some(task.id.clone()),
                        project_id: Some(task.project_id.clone()),
                        context_data: json!({
                            "task_title": task.title,
                            "task_keywords": extract_keywords(&task.title),
                            "completed_at": task.completed_at
                        }),
                    },
                );

                // Fire event for skill triggers
                let _ = EventDispatcher::fire_task_completed(
                    &conn,
                    &task.id,
                    &task.project_id,
                    &task.title,
                );
            }
        }

        if let Some(new_priority) = &input.priority {
            if new_priority != &old.priority {
                let _ = patterns_repo::insert_observation(
                    &conn,
                    CreateObservationInput {
                        observation_type: "priority_set".to_string(),
                        entity_type: Some("task".to_string()),
                        entity_id: Some(task.id.clone()),
                        project_id: Some(task.project_id.clone()),
                        context_data: json!({
                            "old_priority": old.priority,
                            "new_priority": new_priority,
                            "task_title": task.title,
                            "task_keywords": extract_keywords(&task.title)
                        }),
                    },
                );
            }
        }

        if let Some(new_assignee) = &input.assignee {
            let old_assignee = old.assignee.as_deref().unwrap_or("");
            if new_assignee != old_assignee {
                let _ = patterns_repo::insert_observation(
                    &conn,
                    CreateObservationInput {
                        observation_type: "assignee_set".to_string(),
                        entity_type: Some("task".to_string()),
                        entity_id: Some(task.id.clone()),
                        project_id: Some(task.project_id.clone()),
                        context_data: json!({
                            "old_assignee": old_assignee,
                            "new_assignee": new_assignee,
                            "task_title": task.title,
                            "task_keywords": extract_keywords(&task.title)
                        }),
                    },
                );
            }
        }
    }

    Ok(task)
}

fn extract_keywords(title: &str) -> Vec<String> {
    let stopwords = ["the", "a", "an", "is", "are", "to", "for", "of", "and", "or", "in", "on", "at", "with"];
    title
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

#[tauri::command]
pub async fn bulk_update_tasks(
    task_ids: Vec<String>,
    updates: PartialTaskUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::bulk_update_tasks(&conn, &task_ids, &updates)?;

    let _ = log_user_action(
        &conn,
        ActionType::Update,
        EntityType::Task,
        None,
        Some(json!({
            "task_ids": task_ids,
            "updates": {
                "status": updates.status,
                "assignee": updates.assignee
            }
        })),
    );

    Ok(())
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
    repo::delete_task(&conn, &id)?;

    let _ = log_user_action(
        &conn,
        ActionType::Delete,
        EntityType::Task,
        Some(id),
        None,
    );

    Ok(())
}

#[tauri::command]
pub async fn move_task_to_project(
    task_id: String,
    new_project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::move_task_to_project(&conn, &task_id, &new_project_id)
}

#[tauri::command]
pub async fn archive_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::archive_task(&conn, &id)
}

#[tauri::command]
pub async fn unarchive_task(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::unarchive_task(&conn, &id)
}
