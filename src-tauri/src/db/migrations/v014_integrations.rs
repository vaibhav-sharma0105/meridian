pub const SQL: &str = r#"
-- Integrations table: stores connected external services
CREATE TABLE IF NOT EXISTS integrations (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,                     -- 'github', 'jira', 'slack'
    name TEXT NOT NULL,                     -- user-facing name
    config TEXT NOT NULL,                   -- JSON: { access_token, refresh_token, expires_at, ... }
    permissions TEXT,                       -- JSON: { read: true, write: false, ... }
    autonomy_mode TEXT DEFAULT 'manual',    -- 'manual', 'supervised', 'autonomous'
    status TEXT DEFAULT 'disconnected',     -- 'connected', 'disconnected', 'error', 'syncing'
    last_sync TEXT,                         -- last successful sync timestamp
    sync_interval_minutes INTEGER DEFAULT 15,
    webhook_token TEXT,                     -- security token for webhook callbacks
    error_message TEXT,                     -- last error if status is 'error'
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Integration cache: stores fetched external data
CREATE TABLE IF NOT EXISTS integration_cache (
    id TEXT PRIMARY KEY,
    integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    external_type TEXT NOT NULL,            -- 'issue', 'pr', 'jira_issue', 'channel', etc.
    external_id TEXT NOT NULL,              -- ID in external system
    external_url TEXT,                      -- URL to external item
    data TEXT NOT NULL,                     -- JSON: full external item data
    synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(integration_id, external_type, external_id)
);

-- Integration links: bidirectional links between Meridian entities and external items
CREATE TABLE IF NOT EXISTS integration_links (
    id TEXT PRIMARY KEY,
    integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    local_type TEXT NOT NULL,               -- 'task', 'meeting'
    local_id TEXT NOT NULL,                 -- Meridian task/meeting ID
    external_type TEXT NOT NULL,            -- 'issue', 'pr', 'jira_issue'
    external_id TEXT NOT NULL,              -- ID in external system
    external_url TEXT,
    sync_enabled INTEGER DEFAULT 1,         -- whether to sync status changes
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(integration_id, local_id, external_id)
);

-- Add severity and desktop columns to notifications table
ALTER TABLE notifications ADD COLUMN severity TEXT DEFAULT 'info';
ALTER TABLE notifications ADD COLUMN desktop INTEGER DEFAULT 0;
ALTER TABLE notifications ADD COLUMN integration_id TEXT REFERENCES integrations(id) ON DELETE SET NULL;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_integrations_type ON integrations(type);
CREATE INDEX IF NOT EXISTS idx_integrations_status ON integrations(status);

CREATE INDEX IF NOT EXISTS idx_integration_cache_integration ON integration_cache(integration_id);
CREATE INDEX IF NOT EXISTS idx_integration_cache_type ON integration_cache(external_type);
CREATE INDEX IF NOT EXISTS idx_integration_cache_synced_at ON integration_cache(synced_at);

CREATE INDEX IF NOT EXISTS idx_integration_links_integration ON integration_links(integration_id);
CREATE INDEX IF NOT EXISTS idx_integration_links_local ON integration_links(local_type, local_id);
CREATE INDEX IF NOT EXISTS idx_integration_links_external ON integration_links(external_type, external_id);
"#;
