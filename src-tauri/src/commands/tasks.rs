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

    // Record role observations
    if task.assignee.is_some() && task.assignee.as_deref() != Some("") {
        // Creating a task for someone else suggests leadership role
        let _ = crate::role::repository::record_role_observation(
            &conn,
            "creates_tasks_for_others",
            &task.id,
        );
    }

    // Bug tasks suggest IC work
    if task.title.to_lowercase().contains("bug")
        || task.title.to_lowercase().contains("fix")
        || task.title.to_lowercase().contains("issue")
    {
        let _ = crate::role::repository::record_role_observation(&conn, "works_on_bugs", &task.id);
    }

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

    if let Some(old) = &old_task {
        if let Some(new_status) = &input.status {
            if new_status == "done" && old.status != "done" {
                record_completion_observation(&conn, &task);
            }
        }

        if let Some(new_priority) = &input.priority {
            if new_priority != &old.priority {
                record_priority_observation(&conn, &task, &old.priority, new_priority);
            }
        }

        if let Some(new_assignee) = &input.assignee {
            let old_assignee = old.assignee.as_deref().unwrap_or("");
            if new_assignee != old_assignee {
                record_assignee_observation(&conn, &task, old_assignee, new_assignee);

                // Being assigned a task suggests IC role
                if !new_assignee.is_empty() {
                    let _ = crate::role::repository::record_role_observation(
                        &conn,
                        "receives_assignments",
                        &task.id,
                    );
                }
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

fn categorize_task(title: &str, description: Option<&str>) -> String {
    let text = format!("{} {}", title.to_lowercase(), description.unwrap_or("").to_lowercase());

    // Meeting-related keywords
    if text.contains("meeting") || text.contains("call") || text.contains("sync")
        || text.contains("standup") || text.contains("review") || text.contains("1:1")
        || text.contains("1-on-1") || text.contains("retro")
    {
        return "meetings".to_string();
    }

    // Quick task keywords
    if text.contains("email") || text.contains("respond") || text.contains("reply")
        || text.contains("quick") || text.contains("fix typo") || text.contains("update doc")
        || text.contains("slack") || text.contains("message")
    {
        return "quick_tasks".to_string();
    }

    // Default to focus work
    "focus_work".to_string()
}

/// Records a task_completion pattern observation, fires the skill-trigger
/// event, and nudges assignee expertise — shared by both the single-task
/// and bulk-update paths so completing several tasks at once teaches the
/// system just as much as completing them one at a time.
fn record_completion_observation(conn: &rusqlite::Connection, task: &Task) {
    let _ = patterns_repo::insert_observation(
        conn,
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

    let _ = EventDispatcher::fire_task_completed(conn, &task.id, &task.project_id, &task.title);

    // Record productivity observation for time-based patterns
    let category = categorize_task(&task.title, task.description.as_deref());
    let _ = crate::productivity::patterns::record_completion_with_time(conn, &task.id, &category);

    // Learn expertise: completing a task nudges its assignee(s) toward the
    // task's keywords, promoted after repeated hits.
    if let Some(assignee_str) = &task.assignee {
        let keywords = crate::team::assignee::extract_keywords(&task.title, task.description.as_deref());
        if !keywords.is_empty() {
            for name in assignee_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if let Ok(Some(member)) = crate::team::repository::get_team_member_by_name(conn, name) {
                    if let Err(e) = crate::team::repository::record_expertise_observation(conn, &member.id, &keywords) {
                        eprintln!(
                            "expertise learning failed for member {} on task {}: {}",
                            member.id, task.id, e
                        );
                    }
                }
            }
        }
    }
}

fn record_priority_observation(conn: &rusqlite::Connection, task: &Task, old_priority: &str, new_priority: &str) {
    let _ = patterns_repo::insert_observation(
        conn,
        CreateObservationInput {
            observation_type: "priority_set".to_string(),
            entity_type: Some("task".to_string()),
            entity_id: Some(task.id.clone()),
            project_id: Some(task.project_id.clone()),
            context_data: json!({
                "old_priority": old_priority,
                "new_priority": new_priority,
                "task_title": task.title,
                "task_keywords": extract_keywords(&task.title)
            }),
        },
    );
}

fn record_assignee_observation(conn: &rusqlite::Connection, task: &Task, old_assignee: &str, new_assignee: &str) {
    let _ = patterns_repo::insert_observation(
        conn,
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

#[tauri::command]
pub async fn bulk_update_tasks(
    task_ids: Vec<String>,
    updates: PartialTaskUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Snapshot "before" state so completion/assignee changes can be learned
    // from, same as the single-task update_task path.
    let old_tasks: std::collections::HashMap<String, Task> = task_ids
        .iter()
        .filter_map(|id| repo::get_task(&conn, id).ok().map(|t| (id.clone(), t)))
        .collect();

    repo::bulk_update_tasks(&conn, &task_ids, &updates)?;

    for id in &task_ids {
        let Some(old) = old_tasks.get(id) else { continue };
        let Ok(task) = repo::get_task(&conn, id) else { continue };

        if let Some(new_status) = &updates.status {
            if new_status == "done" && old.status != "done" {
                record_completion_observation(&conn, &task);
            }
        }

        if let Some(new_assignee) = &updates.assignee {
            let old_assignee = old.assignee.as_deref().unwrap_or("");
            if new_assignee != old_assignee {
                record_assignee_observation(&conn, &task, old_assignee, new_assignee);
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE pattern_observations (
                id TEXT PRIMARY KEY,
                observation_type TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                project_id TEXT,
                context_data TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                processed_at TEXT
            );
            CREATE TABLE team_members (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                avatar_url TEXT,
                source TEXT NOT NULL,
                source_id TEXT,
                role TEXT DEFAULT 'member',
                expertise TEXT,
                workload_score REAL,
                metadata TEXT,
                expertise_pending TEXT,
                last_synced_at TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE skills (
                id TEXT PRIMARY KEY,
                trigger_type TEXT,
                trigger_config TEXT,
                enabled INTEGER DEFAULT 1
            );"
        ).unwrap();
        conn
    }

    fn make_task(id: &str, title: &str, description: Option<&str>, assignee: Option<&str>) -> Task {
        Task {
            id: id.to_string(),
            project_id: "proj-1".to_string(),
            meeting_id: None,
            parent_task_id: None,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            assignee: assignee.map(|s| s.to_string()),
            assignee_confidence: "committed".to_string(),
            assignee_source_quote: None,
            due_date: None,
            due_confidence: "none".to_string(),
            due_source_quote: None,
            status: "done".to_string(),
            priority: "medium".to_string(),
            confidence_score: None,
            tags: "[]".to_string(),
            kanban_column: "done".to_string(),
            kanban_order: 0,
            is_duplicate: false,
            duplicate_of_id: None,
            notes: None,
            plan_complexity: None,
            plan_data: None,
            plan_generated_at: None,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            completed_at: Some("2026-01-01".to_string()),
            archived_at: None,
        }
    }

    #[test]
    fn test_completion_observation_is_recorded() {
        let conn = setup_test_db();
        let task = make_task("t1", "Fix billing issue", Some("Customer can't pay"), None);

        record_completion_observation(&conn, &task);

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM pattern_observations WHERE observation_type = 'task_completion'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "completing a task should record exactly one task_completion observation");
    }

    #[test]
    fn test_expertise_learning_actually_fires_from_task_completion() {
        // Full pipeline: task completion -> keyword extraction -> team member
        // lookup -> expertise counter -> promotion after N repeats. This is
        // the thing that was previously only unit-tested in isolation.
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO team_members (id, name, source, role, created_at) VALUES ('tm1', 'Alice', 'manual', 'member', '2026-01-01')",
            [],
        ).unwrap();

        for i in 0..crate::team::repository::EXPERTISE_PROMOTION_THRESHOLD {
            let task = make_task(
                &format!("t{}", i),
                "Fix billing issue",
                Some("Customer invoice is wrong"),
                Some("Alice"),
            );
            record_completion_observation(&conn, &task);
        }

        let expertise_json: Option<String> = conn
            .query_row("SELECT expertise FROM team_members WHERE id = 'tm1'", [], |row| row.get(0))
            .unwrap();
        let expertise: Vec<String> = expertise_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        assert!(
            expertise.iter().any(|e| e == "billing"),
            "after {} completions mentioning 'billing', Alice should have it as an expertise tag — got {:?}",
            crate::team::repository::EXPERTISE_PROMOTION_THRESHOLD,
            expertise
        );
    }

    #[test]
    fn test_expertise_learning_is_silent_noop_for_unknown_assignee() {
        // Assignee string that doesn't match any roster member should not
        // error out — it should just skip learning for that name.
        let conn = setup_test_db();
        let task = make_task("t1", "Fix billing issue", None, Some("Someone Not In Roster"));

        record_completion_observation(&conn, &task);

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM pattern_observations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "the task_completion observation should still be recorded even if the assignee isn't a known team member");
    }
}
