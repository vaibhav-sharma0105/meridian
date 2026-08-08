pub const SQL: &str = r#"
-- Extend skills table for sync and trust
ALTER TABLE skills ADD COLUMN sync_source TEXT;
ALTER TABLE skills ADD COLUMN sync_path TEXT;
ALTER TABLE skills ADD COLUMN sync_commit TEXT;
ALTER TABLE skills ADD COLUMN trust_state TEXT DEFAULT 'untrusted';
ALTER TABLE skills ADD COLUMN trust_granted_at TEXT;
ALTER TABLE skills ADD COLUMN network_mode TEXT DEFAULT 'none';
ALTER TABLE skills ADD COLUMN network_allowlist TEXT;
ALTER TABLE skills ADD COLUMN last_sync_check TEXT;
ALTER TABLE skills ADD COLUMN content_hash TEXT;
ALTER TABLE skills ADD COLUMN source_conversation_id TEXT;

-- Skill execution outputs
CREATE TABLE IF NOT EXISTS skill_outputs (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    skill_run_id TEXT REFERENCES skill_runs(id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_skill_outputs_skill ON skill_outputs(skill_id, created_at DESC);

-- Skill execution queue for MCP async execution
CREATE TABLE IF NOT EXISTS skill_queue (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL,
    inputs TEXT,
    priority INTEGER DEFAULT 5,
    status TEXT DEFAULT 'pending',
    result TEXT,
    error TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_skill_queue_status ON skill_queue(status, created_at);
CREATE INDEX IF NOT EXISTS idx_skill_queue_skill ON skill_queue(skill_id, created_at DESC);

-- Index for sync source lookups
CREATE INDEX IF NOT EXISTS idx_skills_sync_source ON skills(sync_source) WHERE sync_source IS NOT NULL;
"#;
