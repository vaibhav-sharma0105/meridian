use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::models::ActionHistory;
use super::repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub success: bool,
    pub undo_action_id: Option<String>,
    pub message: String,
    pub reversal_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Task,
    Meeting,
    Project,
    Skill,
    Integration,
    SlackMessage,
    GithubIssue,
    JiraIssue,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Task => "task",
            EntityType::Meeting => "meeting",
            EntityType::Project => "project",
            EntityType::Skill => "skill",
            EntityType::Integration => "integration",
            EntityType::SlackMessage => "slack_message",
            EntityType::GithubIssue => "github_issue",
            EntityType::JiraIssue => "jira_issue",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "task" => Some(EntityType::Task),
            "meeting" => Some(EntityType::Meeting),
            "project" => Some(EntityType::Project),
            "skill" => Some(EntityType::Skill),
            "integration" => Some(EntityType::Integration),
            "slack_message" => Some(EntityType::SlackMessage),
            "github_issue" => Some(EntityType::GithubIssue),
            "jira_issue" => Some(EntityType::JiraIssue),
            _ => None,
        }
    }

    pub fn is_external(&self) -> bool {
        matches!(
            self,
            EntityType::SlackMessage | EntityType::GithubIssue | EntityType::JiraIssue
        )
    }
}

pub fn capture_action_state(
    conn: &Connection,
    action_type: &str,
    entity_type: &str,
    entity_id: &str,
    before_state: Option<&str>,
    after_state: Option<&str>,
    audit_log_id: Option<&str>,
) -> Result<String, String> {
    let entity = EntityType::from_str(entity_type);
    let undoable = entity.map(|e| !e.is_external()).unwrap_or(true)
        && before_state.is_some()
        && action_type != "delete";

    repository::create_action_history(
        conn,
        action_type,
        entity_type,
        entity_id,
        before_state,
        after_state,
        undoable,
        audit_log_id,
    )
}

pub fn get_action_history(
    conn: &Connection,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    limit: Option<i32>,
) -> Result<Vec<ActionHistory>, String> {
    repository::get_action_history(conn, entity_type, entity_id, limit)
}

pub fn get_undoable_actions(conn: &Connection, limit: Option<i32>) -> Result<Vec<ActionHistory>, String> {
    repository::get_undoable_actions(conn, limit)
}

pub fn undo_action(conn: &Connection, action_id: &str) -> Result<UndoResult, String> {
    let actions = repository::get_action_history(conn, None, None, Some(1000))?;
    let action = actions
        .iter()
        .find(|a| a.id == action_id)
        .ok_or_else(|| format!("Action {} not found", action_id))?;

    if !action.undoable {
        return Ok(UndoResult {
            success: false,
            undo_action_id: None,
            message: "This action cannot be undone".to_string(),
            reversal_type: None,
        });
    }

    if action.undo_action_id.is_some() {
        return Ok(UndoResult {
            success: false,
            undo_action_id: None,
            message: "This action has already been undone".to_string(),
            reversal_type: None,
        });
    }

    let before_state = action
        .before_state
        .as_ref()
        .ok_or("Cannot undo: no before_state recorded")?;

    let reversal_type = determine_reversal_type(&action.action_type);

    let undo_action_id = match execute_reversal(
        conn,
        &action.entity_type,
        &action.entity_id,
        &reversal_type,
        before_state,
        action.after_state.as_deref(),
    ) {
        Ok(id) => id,
        Err(e) => {
            return Ok(UndoResult {
                success: false,
                undo_action_id: None,
                message: format!("Failed to execute undo: {}", e),
                reversal_type: Some(reversal_type),
            });
        }
    };

    repository::mark_action_undone(conn, action_id, &undo_action_id)?;

    Ok(UndoResult {
        success: true,
        undo_action_id: Some(undo_action_id),
        message: format!("Successfully undone {} on {}", action.action_type, action.entity_type),
        reversal_type: Some(reversal_type),
    })
}

fn determine_reversal_type(action_type: &str) -> String {
    match action_type.to_lowercase().as_str() {
        "create" | "insert" | "add" => "delete".to_string(),
        "delete" | "remove" => "create".to_string(),
        "update" | "edit" | "modify" => "update".to_string(),
        _ => "update".to_string(),
    }
}

fn execute_reversal(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    reversal_type: &str,
    before_state: &str,
    after_state: Option<&str>,
) -> Result<String, String> {
    let undo_action_id = repository::create_action_history(
        conn,
        &format!("undo_{}", reversal_type),
        entity_type,
        entity_id,
        after_state,
        Some(before_state),
        false,
        None,
    )?;

    match entity_type {
        "task" => execute_task_reversal(conn, entity_id, reversal_type, before_state)?,
        "meeting" => execute_meeting_reversal(conn, entity_id, reversal_type, before_state)?,
        "project" => execute_project_reversal(conn, entity_id, reversal_type, before_state)?,
        _ => {
            return Err(format!("Undo not implemented for entity type: {}", entity_type));
        }
    }

    Ok(undo_action_id)
}

fn execute_task_reversal(
    conn: &Connection,
    entity_id: &str,
    reversal_type: &str,
    before_state: &str,
) -> Result<(), String> {
    let state: Value =
        serde_json::from_str(before_state).map_err(|e| format!("Invalid before_state JSON: {}", e))?;

    match reversal_type {
        "delete" => {
            conn.execute("DELETE FROM tasks WHERE id = ?1", [entity_id])
                .map_err(|e| format!("Failed to delete task: {}", e))?;
        }
        "create" => {
            let title = state["title"].as_str().unwrap_or("");
            let description = state["description"].as_str();
            let status = state["status"].as_str().unwrap_or("todo");
            let priority = state["priority"].as_str().unwrap_or("medium");
            let project_id = state["project_id"].as_str();

            conn.execute(
                "INSERT INTO tasks (id, title, description, status, priority, project_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![entity_id, title, description, status, priority, project_id],
            )
            .map_err(|e| format!("Failed to recreate task: {}", e))?;
        }
        "update" => {
            let title = state["title"].as_str().unwrap_or("");
            let description = state["description"].as_str();
            let status = state["status"].as_str().unwrap_or("todo");
            let priority = state["priority"].as_str().unwrap_or("medium");
            let assignee = state["assignee"].as_str();
            let due_date = state["due_date"].as_str();

            conn.execute(
                "UPDATE tasks SET title = ?2, description = ?3, status = ?4, priority = ?5, assignee = ?6, due_date = ?7, updated_at = datetime('now') WHERE id = ?1",
                rusqlite::params![entity_id, title, description, status, priority, assignee, due_date],
            )
            .map_err(|e| format!("Failed to restore task: {}", e))?;
        }
        _ => return Err(format!("Unknown reversal type: {}", reversal_type)),
    }

    Ok(())
}

fn execute_meeting_reversal(
    conn: &Connection,
    entity_id: &str,
    reversal_type: &str,
    before_state: &str,
) -> Result<(), String> {
    let state: Value =
        serde_json::from_str(before_state).map_err(|e| format!("Invalid before_state JSON: {}", e))?;

    match reversal_type {
        "delete" => {
            conn.execute("DELETE FROM meetings WHERE id = ?1", [entity_id])
                .map_err(|e| format!("Failed to delete meeting: {}", e))?;
        }
        "update" => {
            let title = state["title"].as_str().unwrap_or("");
            let notes = state["notes"].as_str();
            let status = state["status"].as_str().unwrap_or("pending");

            conn.execute(
                "UPDATE meetings SET title = ?2, notes = ?3, status = ?4, updated_at = datetime('now') WHERE id = ?1",
                rusqlite::params![entity_id, title, notes, status],
            )
            .map_err(|e| format!("Failed to restore meeting: {}", e))?;
        }
        _ => return Err(format!("Reversal type {} not supported for meetings", reversal_type)),
    }

    Ok(())
}

fn execute_project_reversal(
    conn: &Connection,
    entity_id: &str,
    reversal_type: &str,
    before_state: &str,
) -> Result<(), String> {
    let state: Value =
        serde_json::from_str(before_state).map_err(|e| format!("Invalid before_state JSON: {}", e))?;

    match reversal_type {
        "update" => {
            let name = state["name"].as_str().unwrap_or("");
            let description = state["description"].as_str();

            conn.execute(
                "UPDATE projects SET name = ?2, description = ?3, updated_at = datetime('now') WHERE id = ?1",
                rusqlite::params![entity_id, name, description],
            )
            .map_err(|e| format!("Failed to restore project: {}", e))?;
        }
        _ => return Err(format!("Reversal type {} not supported for projects", reversal_type)),
    }

    Ok(())
}

pub fn is_action_undoable(entity_type: &str, action_type: &str) -> bool {
    let entity = EntityType::from_str(entity_type);
    if let Some(e) = entity {
        if e.is_external() {
            return false;
        }
    }

    !matches!(action_type.to_lowercase().as_str(), "delete" | "remove" | "destroy")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_external() {
        assert!(EntityType::SlackMessage.is_external());
        assert!(EntityType::GithubIssue.is_external());
        assert!(EntityType::JiraIssue.is_external());
        assert!(!EntityType::Task.is_external());
        assert!(!EntityType::Meeting.is_external());
    }

    #[test]
    fn test_determine_reversal_type() {
        assert_eq!(determine_reversal_type("create"), "delete");
        assert_eq!(determine_reversal_type("delete"), "create");
        assert_eq!(determine_reversal_type("update"), "update");
        assert_eq!(determine_reversal_type("edit"), "update");
    }

    #[test]
    fn test_is_action_undoable() {
        assert!(is_action_undoable("task", "update"));
        assert!(is_action_undoable("task", "create"));
        assert!(!is_action_undoable("task", "delete"));
        assert!(!is_action_undoable("slack_message", "create"));
        assert!(!is_action_undoable("github_issue", "update"));
    }
}
