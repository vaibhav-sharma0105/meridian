pub const SQL: &str = r#"
-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    source TEXT NOT NULL,  -- 'manual', 'slack', 'google'
    source_id TEXT,        -- external ID for dedup
    role TEXT DEFAULT 'member',  -- 'admin', 'member'
    expertise TEXT,        -- JSON array of tags
    workload_score REAL,   -- computed from assigned tasks
    metadata TEXT,         -- JSON for source-specific data
    last_synced_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source, source_id)
);

CREATE INDEX IF NOT EXISTS idx_team_members_source ON team_members(source);
CREATE INDEX IF NOT EXISTS idx_team_members_email ON team_members(email);

-- Pattern contributions table for anonymized shared patterns
CREATE TABLE IF NOT EXISTS pattern_contributions (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    observation_hash TEXT NOT NULL,  -- anonymized observation
    contributed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pattern_type, observation_hash)
);

CREATE INDEX IF NOT EXISTS idx_pattern_contributions_type ON pattern_contributions(pattern_type);
"#;

pub fn run_post_migration(conn: &rusqlite::Connection) -> Result<(), String> {
    // Add scope column to pattern_models if not exists
    crate::db::migrations::add_column_if_not_exists(conn, "pattern_models", "scope", "TEXT DEFAULT 'personal'")?;
    // Add contributor_count column to pattern_models if not exists
    crate::db::migrations::add_column_if_not_exists(conn, "pattern_models", "contributor_count", "INTEGER DEFAULT 1")?;
    Ok(())
}
