pub const SQL: &str = r#"
-- Skills table for automation workflows
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL,           -- 'schedule', 'event', 'manual'
    trigger_config TEXT,                   -- JSON: cron, timezone, event_type, filter
    context_config TEXT,                   -- JSON: scope, project_id, documents, instructions
    action_config TEXT,                    -- JSON: action_type, format, template
    approval_mode TEXT NOT NULL DEFAULT 'notify',  -- 'auto', 'notify', 'approve_first', 'approve_always'
    enabled INTEGER NOT NULL DEFAULT 1,
    shared INTEGER NOT NULL DEFAULT 0,
    owner_id TEXT,
    category TEXT,                         -- 'productivity', 'communication', 'reporting', 'custom'
    icon TEXT,                             -- emoji or icon identifier
    tags TEXT,                             -- JSON array
    next_run_at TEXT,                      -- computed next scheduled execution
    cloned_from_id TEXT REFERENCES skills(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Skill execution history
CREATE TABLE IF NOT EXISTS skill_runs (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'running', 'completed', 'failed', 'partial_failure', 'cancelled', 'approval_pending'
    trigger_type TEXT NOT NULL,              -- what triggered this run
    trigger_context TEXT,                    -- JSON: event payload, schedule time, etc.
    output TEXT,                             -- generated output
    error TEXT,                              -- error message if failed
    pending_changes TEXT,                    -- JSON: tasks to create, drafts, etc. (for approval)
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    approval_decision TEXT,                  -- 'approved', 'rejected', 'timeout'
    approval_reason TEXT,                    -- rejection reason if rejected
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add skill_run_id to suggestions for skill-generated suggestions
ALTER TABLE suggestions ADD COLUMN skill_run_id TEXT REFERENCES skill_runs(id) ON DELETE SET NULL;

-- Add skill_run_id to notifications for skill notifications
ALTER TABLE notifications ADD COLUMN skill_run_id TEXT REFERENCES skill_runs(id) ON DELETE SET NULL;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_skills_enabled ON skills(enabled);
CREATE INDEX IF NOT EXISTS idx_skills_trigger_type ON skills(trigger_type);
CREATE INDEX IF NOT EXISTS idx_skills_next_run_at ON skills(next_run_at);
CREATE INDEX IF NOT EXISTS idx_skills_shared ON skills(shared);
CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);

CREATE INDEX IF NOT EXISTS idx_skill_runs_skill_id ON skill_runs(skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_runs_status ON skill_runs(status);
CREATE INDEX IF NOT EXISTS idx_skill_runs_created_at ON skill_runs(created_at);
"#;
