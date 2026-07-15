use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Create,
    Update,
    Delete,
    Send,
    Approve,
    Reject,
    Sync,
    Login,
    Export,
    Import,
    SensitiveDetected,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Create => "create",
            ActionType::Update => "update",
            ActionType::Delete => "delete",
            ActionType::Send => "send",
            ActionType::Approve => "approve",
            ActionType::Reject => "reject",
            ActionType::Sync => "sync",
            ActionType::Login => "login",
            ActionType::Export => "export",
            ActionType::Import => "import",
            ActionType::SensitiveDetected => "sensitive_detected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Task,
    Meeting,
    Project,
    Document,
    Skill,
    SkillRun,
    Message,
    Integration,
    Settings,
    Notification,
    Draft,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Task => "task",
            EntityType::Meeting => "meeting",
            EntityType::Project => "project",
            EntityType::Document => "document",
            EntityType::Skill => "skill",
            EntityType::SkillRun => "skill_run",
            EntityType::Message => "message",
            EntityType::Integration => "integration",
            EntityType::Settings => "settings",
            EntityType::Notification => "notification",
            EntityType::Draft => "draft",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyMode {
    Manual,
    Supervised,
    Autonomous,
}

impl AutonomyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AutonomyMode::Manual => "manual",
            AutonomyMode::Supervised => "supervised",
            AutonomyMode::Autonomous => "autonomous",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: Option<String>,
    pub agent_initiated: bool,
    pub autonomy_mode: Option<String>,
    pub risk_level: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Default)]
pub struct LogActionParams {
    pub action_type: ActionType,
    pub entity_type: EntityType,
    pub entity_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub agent_initiated: bool,
    pub autonomy_mode: Option<AutonomyMode>,
    pub risk_level: Option<RiskLevel>,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::Update
    }
}

impl Default for EntityType {
    fn default() -> Self {
        EntityType::Task
    }
}

/// Classify risk level based on action type and entity type.
pub fn classify_risk(action: &ActionType, entity: &EntityType, is_external: bool) -> RiskLevel {
    match action {
        ActionType::Delete => {
            if is_external {
                RiskLevel::Critical
            } else {
                RiskLevel::High
            }
        }
        ActionType::Send => {
            if is_external {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            }
        }
        ActionType::Create | ActionType::Update => {
            if is_external {
                RiskLevel::High
            } else {
                match entity {
                    EntityType::Settings | EntityType::Integration => RiskLevel::Medium,
                    _ => RiskLevel::Low,
                }
            }
        }
        ActionType::Approve | ActionType::Reject => RiskLevel::Medium,
        ActionType::Sync | ActionType::Login | ActionType::Export | ActionType::Import => {
            RiskLevel::Low
        }
        ActionType::SensitiveDetected => RiskLevel::High,
    }
}

/// Log an action to the audit log.
pub fn log_action(conn: &Connection, params: LogActionParams) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let details_str = params.details.map(|d| d.to_string());
    let autonomy_str = params.autonomy_mode.as_ref().map(|a| a.as_str().to_string());
    let risk_str = params.risk_level.as_ref().map(|r| r.as_str().to_string());

    conn.execute(
        "INSERT INTO audit_log (id, timestamp, action_type, entity_type, entity_id, details, agent_initiated, autonomy_mode, risk_level)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id,
            timestamp,
            params.action_type.as_str(),
            params.entity_type.as_str(),
            params.entity_id,
            details_str,
            params.agent_initiated as i32,
            autonomy_str,
            risk_str,
        ],
    )
    .map_err(|e| format!("Failed to log action: {}", e))?;

    Ok(id)
}

/// Helper to log a user action (non-agent).
pub fn log_user_action(
    conn: &Connection,
    action: ActionType,
    entity: EntityType,
    entity_id: Option<String>,
    details: Option<serde_json::Value>,
) -> Result<String, String> {
    let risk = classify_risk(&action, &entity, false);
    log_action(
        conn,
        LogActionParams {
            action_type: action,
            entity_type: entity,
            entity_id,
            details,
            agent_initiated: false,
            autonomy_mode: None,
            risk_level: Some(risk),
        },
    )
}

/// Helper to log an agent action.
pub fn log_agent_action(
    conn: &Connection,
    action: ActionType,
    entity: EntityType,
    entity_id: Option<String>,
    details: Option<serde_json::Value>,
    autonomy_mode: AutonomyMode,
    is_external: bool,
) -> Result<String, String> {
    let risk = classify_risk(&action, &entity, is_external);
    log_action(
        conn,
        LogActionParams {
            action_type: action,
            entity_type: entity,
            entity_id,
            details,
            agent_initiated: true,
            autonomy_mode: Some(autonomy_mode),
            risk_level: Some(risk),
        },
    )
}

#[derive(Debug, Default, Clone)]
pub struct AuditLogFilter {
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

/// Query audit log with filters.
pub fn query_audit_log(
    conn: &Connection,
    filter: AuditLogFilter,
) -> Result<Vec<AuditEntry>, String> {
    let mut sql = String::from(
        "SELECT id, timestamp, action_type, entity_type, entity_id, details,
                agent_initiated, autonomy_mode, risk_level, created_at
         FROM audit_log WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref action) = filter.action_type {
        sql.push_str(" AND action_type = ?");
        params.push(Box::new(action.clone()));
    }
    if let Some(ref entity) = filter.entity_type {
        sql.push_str(" AND entity_type = ?");
        params.push(Box::new(entity.clone()));
    }
    if let Some(ref entity_id) = filter.entity_id {
        sql.push_str(" AND entity_id = ?");
        params.push(Box::new(entity_id.clone()));
    }
    if let Some(agent) = filter.agent_initiated {
        sql.push_str(" AND agent_initiated = ?");
        params.push(Box::new(agent as i32));
    }
    if let Some(ref risk) = filter.risk_level {
        sql.push_str(" AND risk_level = ?");
        params.push(Box::new(risk.clone()));
    }
    if let Some(ref from) = filter.from_timestamp {
        sql.push_str(" AND timestamp >= ?");
        params.push(Box::new(from.clone()));
    }
    if let Some(ref to) = filter.to_timestamp {
        sql.push_str(" AND timestamp <= ?");
        params.push(Box::new(to.clone()));
    }

    sql.push_str(" ORDER BY timestamp DESC");

    if let Some(limit) = filter.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    if let Some(offset) = filter.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let entries = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action_type: row.get(2)?,
                entity_type: row.get(3)?,
                entity_id: row.get(4)?,
                details: row.get(5)?,
                agent_initiated: row.get::<_, i32>(6)? != 0,
                autonomy_mode: row.get(7)?,
                risk_level: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query audit log: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(entries)
}

/// Delete audit log entries older than the specified timestamp.
/// Used for retention policy enforcement.
pub fn prune_audit_log(conn: &Connection, before_timestamp: &str) -> Result<usize, String> {
    let deleted = conn
        .execute(
            "DELETE FROM audit_log WHERE timestamp < ?1",
            rusqlite::params![before_timestamp],
        )
        .map_err(|e| format!("Failed to prune audit log: {}", e))?;

    Ok(deleted)
}

/// Get count of audit log entries (for pagination).
pub fn count_audit_log(conn: &Connection, filter: &AuditLogFilter) -> Result<i64, String> {
    let mut sql = String::from("SELECT COUNT(*) FROM audit_log WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref action) = filter.action_type {
        sql.push_str(" AND action_type = ?");
        params.push(Box::new(action.clone()));
    }
    if let Some(ref entity) = filter.entity_type {
        sql.push_str(" AND entity_type = ?");
        params.push(Box::new(entity.clone()));
    }
    if let Some(agent) = filter.agent_initiated {
        sql.push_str(" AND agent_initiated = ?");
        params.push(Box::new(agent as i32));
    }
    if let Some(ref risk) = filter.risk_level {
        sql.push_str(" AND risk_level = ?");
        params.push(Box::new(risk.clone()));
    }
    if let Some(ref from) = filter.from_timestamp {
        sql.push_str(" AND timestamp >= ?");
        params.push(Box::new(from.clone()));
    }
    if let Some(ref to) = filter.to_timestamp {
        sql.push_str(" AND timestamp <= ?");
        params.push(Box::new(to.clone()));
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0))
        .map_err(|e| format!("Failed to count audit log: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT,
                details TEXT,
                agent_initiated INTEGER DEFAULT 0,
                autonomy_mode TEXT,
                risk_level TEXT DEFAULT 'low',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_log_user_action() {
        let conn = setup_test_db();
        let result = log_user_action(
            &conn,
            ActionType::Create,
            EntityType::Task,
            Some("task-123".to_string()),
            Some(serde_json::json!({"title": "Test task"})),
        );
        assert!(result.is_ok());

        // Verify it was logged
        let entries = query_audit_log(&conn, AuditLogFilter::default()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action_type, "create");
        assert_eq!(entries[0].entity_type, "task");
        assert!(!entries[0].agent_initiated);
    }

    #[test]
    fn test_log_agent_action() {
        let conn = setup_test_db();
        let result = log_agent_action(
            &conn,
            ActionType::Update,
            EntityType::Meeting,
            Some("mtg-456".to_string()),
            None,
            AutonomyMode::Supervised,
            false,
        );
        assert!(result.is_ok());

        let entries = query_audit_log(&conn, AuditLogFilter::default()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].agent_initiated);
        assert_eq!(entries[0].autonomy_mode, Some("supervised".to_string()));
    }

    #[test]
    fn test_risk_classification() {
        assert_eq!(classify_risk(&ActionType::Create, &EntityType::Task, false), RiskLevel::Low);
        assert_eq!(classify_risk(&ActionType::Delete, &EntityType::Project, false), RiskLevel::High);
        assert_eq!(classify_risk(&ActionType::Send, &EntityType::Message, true), RiskLevel::High);
    }

    #[test]
    fn test_filter_by_action_type() {
        let conn = setup_test_db();

        // Log multiple actions
        log_user_action(&conn, ActionType::Create, EntityType::Task, Some("1".to_string()), None).unwrap();
        log_user_action(&conn, ActionType::Update, EntityType::Task, Some("1".to_string()), None).unwrap();
        log_user_action(&conn, ActionType::Delete, EntityType::Task, Some("1".to_string()), None).unwrap();

        let filter = AuditLogFilter {
            action_type: Some("create".to_string()),
            ..Default::default()
        };
        let entries = query_audit_log(&conn, filter).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action_type, "create");
    }

    #[test]
    fn test_prune_old_entries() {
        let conn = setup_test_db();

        // Insert old entry directly
        conn.execute(
            "INSERT INTO audit_log (id, timestamp, action_type, entity_type, risk_level, created_at)
             VALUES ('old-1', '2020-01-01T00:00:00Z', 'create', 'task', 'low', '2020-01-01T00:00:00Z')",
            [],
        ).unwrap();

        // Insert recent entry
        log_user_action(&conn, ActionType::Create, EntityType::Task, None, None).unwrap();

        let pruned = prune_audit_log(&conn, "2023-01-01T00:00:00Z").unwrap();
        assert_eq!(pruned, 1);

        let remaining = query_audit_log(&conn, AuditLogFilter::default()).unwrap();
        assert_eq!(remaining.len(), 1);
    }
}
