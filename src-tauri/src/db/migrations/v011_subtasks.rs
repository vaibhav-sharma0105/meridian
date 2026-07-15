pub const SQL: &str = r#"
-- Add parent_task_id column for subtask hierarchy
ALTER TABLE tasks ADD COLUMN parent_task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE;

-- Index for efficient subtask lookups
CREATE INDEX IF NOT EXISTS idx_tasks_parent_task_id ON tasks(parent_task_id);
"#;
