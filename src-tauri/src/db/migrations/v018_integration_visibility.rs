pub const SQL: &str = r#"
-- Map external repos/projects to Meridian projects
CREATE TABLE IF NOT EXISTS integration_project_mapping (
    id TEXT PRIMARY KEY,
    integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    external_key TEXT NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    UNIQUE(integration_id, external_key)
);

-- Pre-computed attention items
CREATE TABLE IF NOT EXISTS attention_items (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    category TEXT NOT NULL,
    reason_text TEXT,
    matched_skill_id TEXT,
    computed_at TEXT NOT NULL,
    dismissed_at TEXT,
    UNIQUE(source_type, source_id, category)
);

CREATE INDEX IF NOT EXISTS idx_attention_active
    ON attention_items(dismissed_at) WHERE dismissed_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_attention_severity
    ON attention_items(severity, computed_at DESC);

-- Extend skills table for filter config
ALTER TABLE skills ADD COLUMN filter_config JSON;

-- Extend integration_cache for filter results and lifecycle
ALTER TABLE integration_cache ADD COLUMN attention_score REAL;
ALTER TABLE integration_cache ADD COLUMN attention_reason TEXT;
ALTER TABLE integration_cache ADD COLUMN evaluated_at TEXT;
ALTER TABLE integration_cache ADD COLUMN archived_at TEXT;
ALTER TABLE integration_cache ADD COLUMN expires_at TEXT;

CREATE INDEX IF NOT EXISTS idx_cache_attention
    ON integration_cache(attention_score DESC) WHERE archived_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_cache_type_sync
    ON integration_cache(integration_id, external_type, synced_at DESC);

-- Default settings
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('cache_retention_days', '30');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('attention_refresh_minutes', '5');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('ai_integration_context_tokens', '4000');
"#;
