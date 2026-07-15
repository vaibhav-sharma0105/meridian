use tauri::State;

use crate::commands::ai::{get_api_key_from_db, get_litellm_client_pub};
use crate::db::repositories::{ai_settings as ai_settings_repo, tasks as tasks_repo};
use crate::models::task::{CreateTaskInput, UpdateTaskInput};
use crate::patterns::models::CreateObservationInput;
use crate::patterns::repository as patterns_repo;
use crate::plans::evaluation::{evaluate_task_complexity, generate_simple_action, has_action_keywords};
use crate::plans::models::TaskPlan;
use crate::AppState;

#[tauri::command]
pub async fn evaluate_task_plan(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TaskPlan, String> {
    let (task, ai_settings, api_key) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let task = tasks_repo::get_task(&conn, &task_id)?;
        let ai_settings = ai_settings_repo::get_active_settings(&conn)?
            .ok_or_else(|| "No AI settings configured".to_string())?;
        let api_key = get_api_key_from_db(&conn, &ai_settings.label);
        (task, ai_settings, api_key)
    };

    let client = get_litellm_client_pub(&ai_settings, &api_key);
    let description = task.description.clone().unwrap_or_default();

    let evaluation = evaluate_task_complexity(&client, &task.title, &description).await?;

    let suggested_action = if evaluation.complexity == "simple" && has_action_keywords(&task.title) {
        generate_simple_action(&task.title)
    } else {
        None
    };

    let plan = TaskPlan {
        complexity: evaluation.complexity,
        reasoning: evaluation.reasoning,
        suggested_subtasks: evaluation.suggested_subtasks,
        suggested_action,
    };

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let plan_data = serde_json::to_string(&plan).map_err(|e| e.to_string())?;
        tasks_repo::update_task_plan(&conn, &task_id, &plan.complexity, &plan_data)?;
    }

    Ok(plan)
}

#[tauri::command]
pub async fn get_task_plan(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Option<TaskPlan>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let task = tasks_repo::get_task(&conn, &task_id)?;

    match task.plan_data {
        Some(data) => {
            let plan: TaskPlan = serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse plan data: {}", e))?;
            Ok(Some(plan))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn accept_plan(
    state: State<'_, AppState>,
    task_id: String,
    subtask_titles: Vec<String>,
) -> Result<Vec<crate::models::task::Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let parent_task = tasks_repo::get_task(&conn, &task_id)?;

    let mut created_tasks = vec![];

    for title in subtask_titles.iter() {
        let input = CreateTaskInput {
            project_id: parent_task.project_id.clone(),
            meeting_id: parent_task.meeting_id.clone(),
            parent_task_id: Some(task_id.clone()),
            title: title.clone(),
            description: None,
            assignee: parent_task.assignee.clone(),
            assignee_confidence: None,
            assignee_source_quote: None,
            due_date: parent_task.due_date.clone(),
            due_confidence: None,
            due_source_quote: None,
            priority: Some(parent_task.priority.clone()),
            confidence_score: None,
            tags: None,
            notes: None,
            kanban_column: None,
            is_duplicate: None,
            duplicate_of_id: None,
        };

        let task = tasks_repo::create_task(&conn, &input)?;
        created_tasks.push(task);
    }

    Ok(created_tasks)
}

#[tauri::command]
pub async fn record_plan_correction(
    state: State<'_, AppState>,
    task_id: String,
    original_subtasks: Vec<String>,
    edited_subtasks: Vec<String>,
    action: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let task = tasks_repo::get_task(&conn, &task_id)?;

    patterns_repo::insert_observation(
        &conn,
        CreateObservationInput {
            observation_type: "plan_correction".to_string(),
            entity_type: Some("task".to_string()),
            entity_id: Some(task_id),
            project_id: Some(task.project_id),
            context_data: serde_json::json!({
                "original_subtasks": original_subtasks,
                "edited_subtasks": edited_subtasks,
                "action": action,
                "task_title": task.title,
            }),
        },
    )?;

    Ok(())
}
