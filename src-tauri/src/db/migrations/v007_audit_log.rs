pub const SQL: &str = r#"
-- Audit log table for tracking all actions
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    action_type TEXT NOT NULL,  -- 'create', 'update', 'delete', 'send', 'approve', 'reject', 'sync', 'login'
    entity_type TEXT NOT NULL,  -- 'task', 'meeting', 'project', 'document', 'skill', 'message', 'integration', 'settings'
    entity_id TEXT,
    details TEXT,  -- JSON blob with additional context
    agent_initiated INTEGER DEFAULT 0,  -- 0 = user action, 1 = agent action
    autonomy_mode TEXT,  -- 'manual', 'supervised', 'autonomous' (for agent actions)
    risk_level TEXT,  -- 'low', 'medium', 'high', 'critical'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for efficient filtering
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(action_type);
CREATE INDEX IF NOT EXISTS idx_audit_agent ON audit_log(agent_initiated);
CREATE INDEX IF NOT EXISTS idx_audit_risk ON audit_log(risk_level);

-- Combined index for common filter patterns
CREATE INDEX IF NOT EXISTS idx_audit_type_time ON audit_log(entity_type, timestamp);
"#;
