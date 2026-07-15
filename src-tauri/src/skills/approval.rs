use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde_json::{json, Value};

use crate::db::repositories::{notifications as notifications_repo, tasks as tasks_repo};
use crate::models::task::CreateTaskInput;
use crate::skills::{repository as skills_repo, ActionConfig, ApprovalMode, Skill, SkillRun};

const APPROVAL_TIMEOUT_HOURS: i64 = 24;

pub fn check_needs_approval(skill: &Skill, action_config: &ActionConfig) -> bool {
    let mode = ApprovalMode::from_str(&skill.approval_mode).unwrap_or(ApprovalMode::Notify);

    match mode {
        ApprovalMode::Auto => false,
        ApprovalMode::Notify => false,
        ApprovalMode::ApproveAlways => true,
        ApprovalMode::ApproveFirst => {
            // Need approval for actions with side effects
            action_config.has_side_effects.unwrap_or(false)
                || action_config.action_type.as_deref() == Some("create_tasks")
        }
    }
}

pub fn create_approval_notification(
    conn: &Connection,
    skill: &Skill,
    run: &SkillRun,
    pending_changes: &Value,
) -> Result<String, String> {
    let change_summary = summarize_pending_changes(pending_changes);

    let notification = notifications_repo::create_notification(
        conn,
        "skill_approval_needed",
        &format!("Skill '{}' needs approval", skill.name),
        &format!(
            "The skill '{}' has completed and is waiting for your approval. {}",
            skill.name, change_summary
        ),
        Some(&run.id),
        None,
    )?;

    let notification_id = notification.id.clone();

    // Link notification to skill run (skill_run_id column may not exist yet)
    let _ = conn.execute(
        "UPDATE notifications SET skill_run_id = ?1 WHERE id = ?2",
        rusqlite::params![run.id, notification_id],
    );

    Ok(notification_id)
}

fn summarize_pending_changes(changes: &Value) -> String {
    if let Some(change_type) = changes.get("type").and_then(|t| t.as_str()) {
        match change_type {
            "create_tasks" => {
                if let Some(tasks) = changes.get("tasks").and_then(|t| t.as_array()) {
                    format!("{} task(s) will be created.", tasks.len())
                } else {
                    "Tasks will be created.".to_string()
                }
            }
            "send_message" => "A message will be sent.".to_string(),
            "update_tasks" => "Tasks will be updated.".to_string(),
            _ => format!("Action type: {}", change_type),
        }
    } else {
        "Changes are pending.".to_string()
    }
}

pub fn approve_skill_run(
    conn: &Connection,
    run_id: &str,
    project_id: Option<&str>,
) -> Result<Value, String> {
    let run = skills_repo::get_skill_run(conn, run_id)?;

    if run.status != "approval_pending" {
        return Err(format!(
            "Skill run is not pending approval (status: {})",
            run.status
        ));
    }

    let pending_changes = run
        .pending_changes
        .as_ref()
        .ok_or("No pending changes found")?;

    let changes: Value =
        serde_json::from_str(pending_changes).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Apply the pending changes
    let result = apply_pending_changes(conn, &changes, project_id)?;

    // Mark run as completed
    skills_repo::set_approval_decision(conn, run_id, "approved", None)?;

    // Record pattern observation
    record_approval_observation(conn, run_id, "approved", None)?;

    Ok(result)
}

pub fn reject_skill_run(conn: &Connection, run_id: &str, reason: Option<&str>) -> Result<(), String> {
    let run = skills_repo::get_skill_run(conn, run_id)?;

    if run.status != "approval_pending" {
        return Err(format!(
            "Skill run is not pending approval (status: {})",
            run.status
        ));
    }

    // Mark run as cancelled with rejection reason
    skills_repo::set_approval_decision(conn, run_id, "rejected", reason)?;

    // Record pattern observation
    record_approval_observation(conn, run_id, "rejected", reason)?;

    Ok(())
}

fn apply_pending_changes(
    conn: &Connection,
    changes: &Value,
    project_id: Option<&str>,
) -> Result<Value, String> {
    let change_type = changes
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    match change_type {
        "create_tasks" => {
            let tasks = changes
                .get("tasks")
                .and_then(|t| t.as_array())
                .ok_or("No tasks array in pending changes")?;

            let mut created_ids = Vec::new();
            let pid = project_id.unwrap_or("default").to_string();

            for task in tasks {
                let input = CreateTaskInput {
                    project_id: pid.clone(),
                    meeting_id: task.get("meeting_id").and_then(|m| m.as_str()).map(String::from),
                    parent_task_id: task
                        .get("parent_task_id")
                        .and_then(|p| p.as_str())
                        .map(String::from),
                    title: task
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("Untitled")
                        .to_string(),
                    description: task.get("description").and_then(|d| d.as_str()).map(String::from),
                    assignee: task.get("assignee").and_then(|a| a.as_str()).map(String::from),
                    assignee_confidence: None,
                    assignee_source_quote: None,
                    due_date: task.get("due_date").and_then(|d| d.as_str()).map(String::from),
                    due_confidence: None,
                    due_source_quote: None,
                    priority: task
                        .get("priority")
                        .and_then(|p| p.as_str())
                        .map(String::from),
                    confidence_score: None,
                    tags: None,
                    kanban_column: None,
                    notes: None,
                    is_duplicate: None,
                    duplicate_of_id: None,
                };

                let created_task = tasks_repo::create_task(conn, &input)?;
                created_ids.push(created_task.id);
            }

            Ok(json!({
                "type": "tasks_created",
                "count": created_ids.len(),
                "task_ids": created_ids,
            }))
        }
        _ => Ok(json!({
            "type": "no_action",
            "message": format!("Unknown change type: {}", change_type),
        })),
    }
}

pub fn check_expired_approvals(conn: &Connection) -> Result<Vec<String>, String> {
    let cutoff = Utc::now() - Duration::hours(APPROVAL_TIMEOUT_HOURS);
    let cutoff_str = cutoff.to_rfc3339();

    let expired_ids: Vec<String> = conn
        .prepare(
            "SELECT id FROM skill_runs
             WHERE status = 'approval_pending'
             AND created_at < ?1",
        )
        .map_err(|e| e.to_string())?
        .query_map([&cutoff_str], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for run_id in &expired_ids {
        skills_repo::set_approval_decision(
            conn,
            run_id,
            "expired",
            Some("Approval timed out after 24 hours"),
        )?;
    }

    Ok(expired_ids)
}

fn record_approval_observation(
    conn: &Connection,
    run_id: &str,
    decision: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let observation_type = match decision {
        "approved" => "skill_approved",
        "rejected" => "skill_correction",
        "expired" => "skill_expired",
        _ => return Ok(()),
    };

    let context = json!({
        "skill_run_id": run_id,
        "decision": decision,
        "reason": reason,
    });

    conn.execute(
        "INSERT INTO pattern_observations (id, observation_type, entity_type, entity_id, context, created_at)
         VALUES (?1, ?2, 'skill_run', ?3, ?4, ?5)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            observation_type,
            run_id,
            context.to_string(),
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_create_tasks() {
        let changes = json!({
            "type": "create_tasks",
            "tasks": [
                {"title": "Task 1"},
                {"title": "Task 2"},
            ]
        });

        let summary = summarize_pending_changes(&changes);
        assert!(summary.contains("2 task(s)"));
    }

    #[test]
    fn test_summarize_unknown() {
        let changes = json!({
            "type": "custom_action",
        });

        let summary = summarize_pending_changes(&changes);
        assert!(summary.contains("custom_action"));
    }
}
