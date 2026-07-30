use chrono::{Duration, Utc};
use rusqlite::Connection;

use super::models::{ApprovalStatus, AutonomyMode, CreatePendingApprovalInput, PendingApproval, RiskLevel};
use super::repository;

const DEFAULT_TIMEOUT_MINUTES: i64 = 1440;

pub fn create_pending_approval(
    conn: &Connection,
    input: CreatePendingApprovalInput,
) -> Result<String, String> {
    let timeout_minutes = input.timeout_minutes.unwrap_or(DEFAULT_TIMEOUT_MINUTES);
    let timeout_at = Utc::now()
        .checked_add_signed(Duration::minutes(timeout_minutes))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());

    repository::create_pending_approval(
        conn,
        &input.action_type,
        &input.action_config,
        input.source_type.as_deref(),
        input.source_id.as_deref(),
        input.risk_level.as_str(),
        input.autonomy_mode.as_str(),
        input.context.as_deref(),
        timeout_at.as_deref(),
    )
}

pub fn get_pending_approvals(
    conn: &Connection,
    status: Option<&str>,
    limit: Option<i32>,
) -> Result<Vec<PendingApproval>, String> {
    repository::get_pending_approvals(conn, status, limit)
}

pub fn get_pending_approval(conn: &Connection, id: &str) -> Result<Option<PendingApproval>, String> {
    repository::get_pending_approval(conn, id)
}

pub fn approve_action(conn: &Connection, id: &str, resolved_by: &str) -> Result<PendingApproval, String> {
    let approval = repository::get_pending_approval(conn, id)?
        .ok_or_else(|| format!("Pending approval {} not found", id))?;

    if approval.status != "pending" {
        return Err(format!(
            "Cannot approve: approval {} is already {}",
            id, approval.status
        ));
    }

    repository::update_approval_status(conn, id, ApprovalStatus::Approved, Some(resolved_by), None)?;

    repository::get_pending_approval(conn, id)?
        .ok_or_else(|| "Failed to retrieve updated approval".to_string())
}

pub fn reject_action(
    conn: &Connection,
    id: &str,
    resolved_by: &str,
    reason: Option<&str>,
) -> Result<PendingApproval, String> {
    let approval = repository::get_pending_approval(conn, id)?
        .ok_or_else(|| format!("Pending approval {} not found", id))?;

    if approval.status != "pending" {
        return Err(format!(
            "Cannot reject: approval {} is already {}",
            id, approval.status
        ));
    }

    repository::update_approval_status(conn, id, ApprovalStatus::Rejected, Some(resolved_by), reason)?;

    repository::get_pending_approval(conn, id)?
        .ok_or_else(|| "Failed to retrieve updated approval".to_string())
}

pub fn archive_expired_approvals(conn: &Connection) -> Result<Vec<String>, String> {
    let expired = repository::get_expired_approvals(conn)?;
    let mut archived_ids = Vec::new();

    for approval in expired {
        repository::update_approval_status(
            conn,
            &approval.id,
            ApprovalStatus::Archived,
            Some("timeout"),
            Some("Approval timed out"),
        )?;
        archived_ids.push(approval.id);
    }

    Ok(archived_ids)
}

pub fn bulk_approve(conn: &Connection, ids: &[String], resolved_by: &str) -> Result<Vec<String>, String> {
    let mut approved_ids = Vec::new();

    for id in ids {
        if let Ok(approval) = approve_action(conn, id, resolved_by) {
            approved_ids.push(approval.id);
        }
    }

    Ok(approved_ids)
}

pub fn bulk_reject(
    conn: &Connection,
    ids: &[String],
    resolved_by: &str,
    reason: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut rejected_ids = Vec::new();

    for id in ids {
        if let Ok(approval) = reject_action(conn, id, resolved_by, reason) {
            rejected_ids.push(approval.id);
        }
    }

    Ok(rejected_ids)
}

pub fn mark_executed(conn: &Connection, id: &str) -> Result<(), String> {
    repository::update_approval_status(conn, id, ApprovalStatus::Executed, Some("system"), None)
}

pub fn queue_for_approval(
    conn: &Connection,
    action_type: &str,
    action_config: &str,
    source_type: Option<&str>,
    source_id: Option<&str>,
    risk_level: RiskLevel,
    autonomy_mode: AutonomyMode,
    context: Option<&str>,
    timeout_minutes: Option<i64>,
) -> Result<String, String> {
    create_pending_approval(
        conn,
        CreatePendingApprovalInput {
            action_type: action_type.to_string(),
            action_config: action_config.to_string(),
            source_type: source_type.map(String::from),
            source_id: source_id.map(String::from),
            risk_level,
            autonomy_mode,
            context: context.map(String::from),
            timeout_minutes,
        },
    )
}

pub fn get_pending_count(conn: &Connection) -> Result<i64, String> {
    repository::get_pending_approval_count(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE pending_approvals (
                id TEXT PRIMARY KEY,
                action_type TEXT NOT NULL,
                action_config TEXT NOT NULL,
                source_type TEXT,
                source_id TEXT,
                risk_level TEXT NOT NULL,
                autonomy_mode TEXT NOT NULL,
                context TEXT,
                timeout_at TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                resolved_by TEXT,
                resolution_reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                resolved_at TEXT
            );
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_approval() {
        let conn = setup_test_db();

        let id = create_pending_approval(
            &conn,
            CreatePendingApprovalInput {
                action_type: "create_task".to_string(),
                action_config: r#"{"title":"Test"}"#.to_string(),
                source_type: Some("skill".to_string()),
                source_id: Some("skill-123".to_string()),
                risk_level: RiskLevel::Medium,
                autonomy_mode: AutonomyMode::Supervised,
                context: None,
                timeout_minutes: Some(60),
            },
        )
        .unwrap();

        let approval = get_pending_approval(&conn, &id).unwrap().unwrap();
        assert_eq!(approval.action_type, "create_task");
        assert_eq!(approval.status, "pending");
        assert_eq!(approval.risk_level, "medium");
    }

    #[test]
    fn test_approve_action() {
        let conn = setup_test_db();

        let id = create_pending_approval(
            &conn,
            CreatePendingApprovalInput {
                action_type: "send_message".to_string(),
                action_config: "{}".to_string(),
                source_type: None,
                source_id: None,
                risk_level: RiskLevel::High,
                autonomy_mode: AutonomyMode::Manual,
                context: None,
                timeout_minutes: None,
            },
        )
        .unwrap();

        let approved = approve_action(&conn, &id, "user").unwrap();
        assert_eq!(approved.status, "approved");
        assert_eq!(approved.resolved_by, Some("user".to_string()));
    }

    #[test]
    fn test_reject_action() {
        let conn = setup_test_db();

        let id = create_pending_approval(
            &conn,
            CreatePendingApprovalInput {
                action_type: "delete_task".to_string(),
                action_config: "{}".to_string(),
                source_type: None,
                source_id: None,
                risk_level: RiskLevel::Critical,
                autonomy_mode: AutonomyMode::Supervised,
                context: None,
                timeout_minutes: None,
            },
        )
        .unwrap();

        let rejected = reject_action(&conn, &id, "user", Some("Too risky")).unwrap();
        assert_eq!(rejected.status, "rejected");
        assert_eq!(rejected.resolution_reason, Some("Too risky".to_string()));
    }

    #[test]
    fn test_cannot_approve_already_approved() {
        let conn = setup_test_db();

        let id = create_pending_approval(
            &conn,
            CreatePendingApprovalInput {
                action_type: "test".to_string(),
                action_config: "{}".to_string(),
                source_type: None,
                source_id: None,
                risk_level: RiskLevel::Low,
                autonomy_mode: AutonomyMode::Manual,
                context: None,
                timeout_minutes: None,
            },
        )
        .unwrap();

        approve_action(&conn, &id, "user").unwrap();
        let result = approve_action(&conn, &id, "user");
        assert!(result.is_err());
    }

    #[test]
    fn test_bulk_approve() {
        let conn = setup_test_db();

        let id1 = queue_for_approval(
            &conn,
            "task1",
            "{}",
            None,
            None,
            RiskLevel::Low,
            AutonomyMode::Manual,
            None,
            None,
        )
        .unwrap();

        let id2 = queue_for_approval(
            &conn,
            "task2",
            "{}",
            None,
            None,
            RiskLevel::Low,
            AutonomyMode::Manual,
            None,
            None,
        )
        .unwrap();

        let approved = bulk_approve(&conn, &[id1.clone(), id2.clone()], "user").unwrap();
        assert_eq!(approved.len(), 2);
    }
}
