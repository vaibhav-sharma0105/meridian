use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{
    ActionHistory, ApprovalStatus, GovernanceMetrics, PendingApproval, RiskAdjustment,
};

pub fn create_pending_approval(
    conn: &Connection,
    action_type: &str,
    action_config: &str,
    source_type: Option<&str>,
    source_id: Option<&str>,
    risk_level: &str,
    autonomy_mode: &str,
    context: Option<&str>,
    timeout_at: Option<&str>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO pending_approvals (id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at],
    )
    .map_err(|e| format!("Failed to create pending approval: {}", e))?;
    Ok(id)
}

pub fn get_pending_approval(conn: &Connection, id: &str) -> Result<Option<PendingApproval>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at, status, resolved_by, resolution_reason, created_at, resolved_at
             FROM pending_approvals WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt
        .query_row(params![id], |row| {
            Ok(PendingApproval {
                id: row.get(0)?,
                action_type: row.get(1)?,
                action_config: row.get(2)?,
                source_type: row.get(3)?,
                source_id: row.get(4)?,
                risk_level: row.get(5)?,
                autonomy_mode: row.get(6)?,
                context: row.get(7)?,
                timeout_at: row.get(8)?,
                status: row.get(9)?,
                resolved_by: row.get(10)?,
                resolution_reason: row.get(11)?,
                created_at: row.get(12)?,
                resolved_at: row.get(13)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn get_pending_approvals(
    conn: &Connection,
    status: Option<&str>,
    limit: Option<i32>,
) -> Result<Vec<PendingApproval>, String> {
    let limit = limit.unwrap_or(50);

    if let Some(s) = status {
        let mut stmt = conn
            .prepare(
                "SELECT id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at, status, resolved_by, resolution_reason, created_at, resolved_at
                 FROM pending_approvals WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        let rows = stmt
            .query_map(params![s, limit], |row| {
                Ok(PendingApproval {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    action_config: row.get(2)?,
                    source_type: row.get(3)?,
                    source_id: row.get(4)?,
                    risk_level: row.get(5)?,
                    autonomy_mode: row.get(6)?,
                    context: row.get(7)?,
                    timeout_at: row.get(8)?,
                    status: row.get(9)?,
                    resolved_by: row.get(10)?,
                    resolution_reason: row.get(11)?,
                    created_at: row.get(12)?,
                    resolved_at: row.get(13)?,
                })
            })
            .map_err(|e| format!("Failed to query pending approvals: {}", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect pending approvals: {}", e))
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at, status, resolved_by, resolution_reason, created_at, resolved_at
                 FROM pending_approvals ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(PendingApproval {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    action_config: row.get(2)?,
                    source_type: row.get(3)?,
                    source_id: row.get(4)?,
                    risk_level: row.get(5)?,
                    autonomy_mode: row.get(6)?,
                    context: row.get(7)?,
                    timeout_at: row.get(8)?,
                    status: row.get(9)?,
                    resolved_by: row.get(10)?,
                    resolution_reason: row.get(11)?,
                    created_at: row.get(12)?,
                    resolved_at: row.get(13)?,
                })
            })
            .map_err(|e| format!("Failed to query pending approvals: {}", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect pending approvals: {}", e))
    }
}

pub fn update_approval_status(
    conn: &Connection,
    id: &str,
    status: ApprovalStatus,
    resolved_by: Option<&str>,
    resolution_reason: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE pending_approvals SET status = ?2, resolved_by = ?3, resolution_reason = ?4, resolved_at = datetime('now')
         WHERE id = ?1",
        params![id, status.as_str(), resolved_by, resolution_reason],
    )
    .map_err(|e| format!("Failed to update approval status: {}", e))?;
    Ok(())
}

pub fn get_expired_approvals(conn: &Connection) -> Result<Vec<PendingApproval>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, action_type, action_config, source_type, source_id, risk_level, autonomy_mode, context, timeout_at, status, resolved_by, resolution_reason, created_at, resolved_at
             FROM pending_approvals
             WHERE status = 'pending' AND timeout_at IS NOT NULL AND timeout_at < datetime('now')",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PendingApproval {
                id: row.get(0)?,
                action_type: row.get(1)?,
                action_config: row.get(2)?,
                source_type: row.get(3)?,
                source_id: row.get(4)?,
                risk_level: row.get(5)?,
                autonomy_mode: row.get(6)?,
                context: row.get(7)?,
                timeout_at: row.get(8)?,
                status: row.get(9)?,
                resolved_by: row.get(10)?,
                resolution_reason: row.get(11)?,
                created_at: row.get(12)?,
                resolved_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Failed to query expired approvals: {}", e))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect expired approvals: {}", e))
}

pub fn create_action_history(
    conn: &Connection,
    action_type: &str,
    entity_type: &str,
    entity_id: &str,
    before_state: Option<&str>,
    after_state: Option<&str>,
    undoable: bool,
    audit_log_id: Option<&str>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO action_history (id, action_type, entity_type, entity_id, before_state, after_state, undoable, audit_log_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, action_type, entity_type, entity_id, before_state, after_state, undoable, audit_log_id],
    )
    .map_err(|e| format!("Failed to create action history: {}", e))?;
    Ok(id)
}

pub fn get_action_history(
    conn: &Connection,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    limit: Option<i32>,
) -> Result<Vec<ActionHistory>, String> {
    let limit = limit.unwrap_or(50);

    match (entity_type, entity_id) {
        (Some(et), Some(ei)) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, action_type, entity_type, entity_id, before_state, after_state, undoable, undo_action_id, audit_log_id, created_at
                     FROM action_history WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at DESC LIMIT ?3",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;
            let rows = stmt.query_map(params![et, ei, limit], |row| {
                Ok(ActionHistory {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    before_state: row.get(4)?,
                    after_state: row.get(5)?,
                    undoable: row.get::<_, i32>(6)? == 1,
                    undo_action_id: row.get(7)?,
                    audit_log_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("Failed to query action history: {}", e))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect action history: {}", e))
        }
        (Some(et), None) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, action_type, entity_type, entity_id, before_state, after_state, undoable, undo_action_id, audit_log_id, created_at
                     FROM action_history WHERE entity_type = ?1 ORDER BY created_at DESC LIMIT ?2",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;
            let rows = stmt.query_map(params![et, limit], |row| {
                Ok(ActionHistory {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    before_state: row.get(4)?,
                    after_state: row.get(5)?,
                    undoable: row.get::<_, i32>(6)? == 1,
                    undo_action_id: row.get(7)?,
                    audit_log_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("Failed to query action history: {}", e))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect action history: {}", e))
        }
        (None, Some(ei)) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, action_type, entity_type, entity_id, before_state, after_state, undoable, undo_action_id, audit_log_id, created_at
                     FROM action_history WHERE entity_id = ?1 ORDER BY created_at DESC LIMIT ?2",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;
            let rows = stmt.query_map(params![ei, limit], |row| {
                Ok(ActionHistory {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    before_state: row.get(4)?,
                    after_state: row.get(5)?,
                    undoable: row.get::<_, i32>(6)? == 1,
                    undo_action_id: row.get(7)?,
                    audit_log_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("Failed to query action history: {}", e))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect action history: {}", e))
        }
        (None, None) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, action_type, entity_type, entity_id, before_state, after_state, undoable, undo_action_id, audit_log_id, created_at
                     FROM action_history ORDER BY created_at DESC LIMIT ?1",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(ActionHistory {
                    id: row.get(0)?,
                    action_type: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    before_state: row.get(4)?,
                    after_state: row.get(5)?,
                    undoable: row.get::<_, i32>(6)? == 1,
                    undo_action_id: row.get(7)?,
                    audit_log_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| format!("Failed to query action history: {}", e))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect action history: {}", e))
        }
    }
}

pub fn get_undoable_actions(conn: &Connection, limit: Option<i32>) -> Result<Vec<ActionHistory>, String> {
    let limit = limit.unwrap_or(10);
    let mut stmt = conn
        .prepare(
            "SELECT id, action_type, entity_type, entity_id, before_state, after_state, undoable, undo_action_id, audit_log_id, created_at
             FROM action_history
             WHERE undoable = 1 AND undo_action_id IS NULL
             ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(ActionHistory {
                id: row.get(0)?,
                action_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                before_state: row.get(4)?,
                after_state: row.get(5)?,
                undoable: row.get::<_, i32>(6)? == 1,
                undo_action_id: row.get(7)?,
                audit_log_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query undoable actions: {}", e))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect undoable actions: {}", e))
}

pub fn mark_action_undone(conn: &Connection, action_id: &str, undo_action_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE action_history SET undo_action_id = ?2 WHERE id = ?1",
        params![action_id, undo_action_id],
    )
    .map_err(|e| format!("Failed to mark action undone: {}", e))?;
    Ok(())
}

pub fn upsert_governance_metric(
    conn: &Connection,
    date: &str,
    metric_type: &str,
    breakdown_key: Option<&str>,
    value: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO governance_metrics (date, metric_type, breakdown_key, value)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(date, metric_type, breakdown_key) DO UPDATE SET value = ?4",
        params![date, metric_type, breakdown_key, value],
    )
    .map_err(|e| format!("Failed to upsert governance metric: {}", e))?;
    Ok(())
}

pub fn get_governance_metrics(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
    metric_type: Option<&str>,
) -> Result<Vec<GovernanceMetrics>, String> {
    if let Some(mt) = metric_type {
        let mut stmt = conn
            .prepare(
                "SELECT date, metric_type, breakdown_key, value FROM governance_metrics
                 WHERE date >= ?1 AND date <= ?2 AND metric_type = ?3
                 ORDER BY date DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        let rows = stmt.query_map(params![start_date, end_date, mt], |row| {
            Ok(GovernanceMetrics {
                date: row.get(0)?,
                metric_type: row.get(1)?,
                breakdown_key: row.get(2)?,
                value: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to query governance metrics: {}", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect governance metrics: {}", e))
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT date, metric_type, breakdown_key, value FROM governance_metrics
                 WHERE date >= ?1 AND date <= ?2
                 ORDER BY date DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok(GovernanceMetrics {
                date: row.get(0)?,
                metric_type: row.get(1)?,
                breakdown_key: row.get(2)?,
                value: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to query governance metrics: {}", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect governance metrics: {}", e))
    }
}

pub fn create_risk_adjustment(
    conn: &Connection,
    adjustment_type: &str,
    target_type: &str,
    target_id: &str,
    risk_delta: i32,
    reason: Option<&str>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO risk_adjustments (id, adjustment_type, target_type, target_id, risk_delta, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(adjustment_type, target_type, target_id) DO UPDATE SET risk_delta = ?5, reason = ?6",
        params![id, adjustment_type, target_type, target_id, risk_delta, reason],
    )
    .map_err(|e| format!("Failed to create risk adjustment: {}", e))?;
    Ok(id)
}

pub fn get_risk_adjustment(
    conn: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<Option<RiskAdjustment>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, adjustment_type, target_type, target_id, risk_delta, reason, created_at
             FROM risk_adjustments WHERE target_type = ?1 AND target_id = ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt
        .query_row(params![target_type, target_id], |row| {
            Ok(RiskAdjustment {
                id: row.get(0)?,
                adjustment_type: row.get(1)?,
                target_type: row.get(2)?,
                target_id: row.get(3)?,
                risk_delta: row.get(4)?,
                reason: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn delete_risk_adjustment(conn: &Connection, target_type: &str, target_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM risk_adjustments WHERE target_type = ?1 AND target_id = ?2",
        params![target_type, target_id],
    )
    .map_err(|e| format!("Failed to delete risk adjustment: {}", e))?;
    Ok(())
}

pub fn get_pending_approval_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM pending_approvals WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to count pending approvals: {}", e))
}
