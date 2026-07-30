pub const SQL: &str = r#"
-- Pending approvals queue for actions requiring user approval
CREATE TABLE IF NOT EXISTS pending_approvals (
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

CREATE INDEX IF NOT EXISTS idx_pending_approvals_status ON pending_approvals(status);
CREATE INDEX IF NOT EXISTS idx_pending_approvals_timeout ON pending_approvals(timeout_at);
CREATE INDEX IF NOT EXISTS idx_pending_approvals_source ON pending_approvals(source_type, source_id);

-- Action history for undo tracking
CREATE TABLE IF NOT EXISTS action_history (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    before_state TEXT,
    after_state TEXT,
    undoable INTEGER NOT NULL DEFAULT 1,
    undo_action_id TEXT,
    audit_log_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_action_history_entity ON action_history(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_action_history_undoable ON action_history(undoable, created_at);
CREATE INDEX IF NOT EXISTS idx_action_history_undo_action ON action_history(undo_action_id);

-- Governance metrics for dashboard aggregates
CREATE TABLE IF NOT EXISTS governance_metrics (
    date TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    breakdown_key TEXT,
    value INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, metric_type, breakdown_key)
);

-- Note: ALTER TABLE ADD COLUMN statements are handled programmatically
-- in the migration runner to check if columns exist first

-- Risk adjustments for learned user preferences
CREATE TABLE IF NOT EXISTS risk_adjustments (
    id TEXT PRIMARY KEY,
    adjustment_type TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    risk_delta INTEGER NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(adjustment_type, target_type, target_id)
);

CREATE INDEX IF NOT EXISTS idx_risk_adjustments_target ON risk_adjustments(target_type, target_id);
"#;
