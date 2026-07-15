pub const SQL: &str = r#"
-- Suggestions table for proactive agent recommendations
CREATE TABLE IF NOT EXISTS suggestions (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    reasoning TEXT,
    action_config TEXT,
    severity TEXT NOT NULL DEFAULT 'info',
    status TEXT NOT NULL DEFAULT 'pending',
    project_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    acted_at TEXT
);

-- Draft messages table for auto-generated communications
CREATE TABLE IF NOT EXISTS draft_messages (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    channel TEXT NOT NULL,
    recipient TEXT,
    subject TEXT,
    body TEXT NOT NULL,
    ai_signature INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'draft',
    sensitive_warnings TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    sent_at TEXT
);

-- Add plan fields to tasks table
ALTER TABLE tasks ADD COLUMN plan_complexity TEXT;
ALTER TABLE tasks ADD COLUMN plan_data TEXT;
ALTER TABLE tasks ADD COLUMN plan_generated_at TEXT;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_suggestions_status_project
    ON suggestions(status, project_id);

CREATE INDEX IF NOT EXISTS idx_suggestions_created
    ON suggestions(created_at);

CREATE INDEX IF NOT EXISTS idx_draft_messages_task
    ON draft_messages(task_id);

CREATE INDEX IF NOT EXISTS idx_draft_messages_status
    ON draft_messages(status);
"#;
