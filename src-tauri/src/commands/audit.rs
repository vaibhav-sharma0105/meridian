use crate::audit::{query_audit_log, count_audit_log, prune_audit_log, AuditEntry, AuditLogFilter};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct AuditLogFilterInput {
    pub action_type: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub agent_initiated: Option<bool>,
    pub risk_level: Option<String>,
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<AuditLogFilterInput> for AuditLogFilter {
    fn from(input: AuditLogFilterInput) -> Self {
        AuditLogFilter {
            action_type: input.action_type,
            entity_type: input.entity_type,
            entity_id: input.entity_id,
            agent_initiated: input.agent_initiated,
            risk_level: input.risk_level,
            from_timestamp: input.from_timestamp,
            to_timestamp: input.to_timestamp,
            limit: input.limit,
            offset: input.offset,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
    pub has_more: bool,
}

#[tauri::command]
pub async fn get_audit_log(
    filter: Option<AuditLogFilterInput>,
    state: State<'_, AppState>,
) -> Result<AuditLogResponse, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let filter = filter.map(AuditLogFilter::from).unwrap_or_default();
    let limit = filter.limit.unwrap_or(50);

    let entries = query_audit_log(&conn, AuditLogFilter {
        limit: Some(limit + 1), // Fetch one extra to check has_more
        ..filter.clone()
    })?;

    let has_more = entries.len() > limit as usize;
    let entries: Vec<AuditEntry> = entries.into_iter().take(limit as usize).collect();

    let total = count_audit_log(&conn, &filter)?;

    Ok(AuditLogResponse {
        entries,
        total,
        has_more,
    })
}

#[tauri::command]
pub async fn export_audit_log(
    filter: Option<AuditLogFilterInput>,
    format: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let filter = filter.map(AuditLogFilter::from).unwrap_or_default();

    // Remove limit for export
    let export_filter = AuditLogFilter {
        limit: None,
        offset: None,
        ..filter
    };

    let entries = query_audit_log(&conn, export_filter)?;

    match format.as_str() {
        "json" => {
            serde_json::to_string_pretty(&entries)
                .map_err(|e| format!("Failed to serialize: {}", e))
        }
        "csv" => {
            let mut csv_output = String::from("id,timestamp,action_type,entity_type,entity_id,details,agent_initiated,autonomy_mode,risk_level,created_at\n");
            for entry in entries {
                csv_output.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    entry.id,
                    entry.timestamp,
                    entry.action_type,
                    entry.entity_type,
                    entry.entity_id.unwrap_or_default(),
                    entry.details.unwrap_or_default().replace(',', ";").replace('\n', " "),
                    entry.agent_initiated,
                    entry.autonomy_mode.unwrap_or_default(),
                    entry.risk_level.unwrap_or_default(),
                    entry.created_at,
                ));
            }
            Ok(csv_output)
        }
        _ => Err("Unsupported format. Use 'json' or 'csv'.".to_string()),
    }
}

#[tauri::command]
pub async fn prune_old_audit_logs(state: State<'_, AppState>) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Calculate timestamp for 2 years ago
    let two_years_ago = chrono::Utc::now() - chrono::Duration::days(730);
    let cutoff = two_years_ago.to_rfc3339();

    prune_audit_log(&conn, &cutoff)
}

#[tauri::command]
pub async fn get_audit_log_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count audit log: {}", e))?;

    let oldest: Option<String> = conn.query_row(
        "SELECT MIN(timestamp) FROM audit_log",
        [],
        |row| row.get(0),
    ).ok();

    let newest: Option<String> = conn.query_row(
        "SELECT MAX(timestamp) FROM audit_log",
        [],
        |row| row.get(0),
    ).ok();

    let agent_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE agent_initiated = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(serde_json::json!({
        "total_entries": total,
        "oldest_entry": oldest,
        "newest_entry": newest,
        "agent_initiated_count": agent_count,
        "retention_days": 730
    }))
}
