pub const SQL: &str = r#"
-- Message Center table for persistent storage of skill results, digests, pinned chat
CREATE TABLE IF NOT EXISTS message_center (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    message_type TEXT NOT NULL,  -- 'skill_result' | 'digest' | 'pinned_chat' | 'integration_sync'
    title TEXT NOT NULL,
    content TEXT,                -- markdown content or summary
    source_id TEXT,              -- skill_run_id, chat_message_id, etc.
    source_type TEXT,            -- 'skill' | 'ai_chat' | 'integration'
    auto_pinned INTEGER DEFAULT 0,
    pinned_reason TEXT,          -- 'file_attachment' | 'long_response' | 'important_skill'
    file_refs TEXT,              -- JSON array of file paths in created_files/
    ai_visible_until TEXT,       -- ISO timestamp; NULL = always visible to AI
    deleted_at TEXT,             -- soft-delete timestamp
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_message_center_project ON message_center(project_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_center_type ON message_center(message_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_message_center_deleted ON message_center(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_message_center_ai_visible ON message_center(ai_visible_until) WHERE ai_visible_until IS NOT NULL;

-- User Profile table for role inference and productivity patterns
CREATE TABLE IF NOT EXISTS user_profile (
    id TEXT PRIMARY KEY DEFAULT 'default',  -- single-user app
    inferred_role TEXT,           -- 'tech_lead' | 'ic' | 'pm' | 'manager' | 'other'
    secondary_role TEXT,          -- if confidence > 0.3
    custom_role_description TEXT, -- free text for "Other"
    role_confirmed INTEGER DEFAULT 0,
    role_confirmed_at TEXT,
    role_scores TEXT,             -- JSON: {"tech_lead": 0.4, "ic": 0.3, ...}
    last_inference_at TEXT,
    productivity_patterns TEXT,   -- JSON: peak hours, completions by hour
    productivity_tracking_enabled INTEGER DEFAULT 1,
    ai_context_days INTEGER DEFAULT 30,
    message_retention TEXT DEFAULT 'forever',  -- '90d' | '1y' | 'forever'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Extend pattern_observations with role and productivity signals
ALTER TABLE pattern_observations ADD COLUMN role_signal TEXT;
ALTER TABLE pattern_observations ADD COLUMN completion_hour INTEGER;
ALTER TABLE pattern_observations ADD COLUMN completion_day_of_week INTEGER;
ALTER TABLE pattern_observations ADD COLUMN task_category TEXT;

-- Indexes for role and productivity queries
CREATE INDEX IF NOT EXISTS idx_pattern_obs_role ON pattern_observations(role_signal)
    WHERE role_signal IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_pattern_obs_productivity ON pattern_observations(completion_hour, task_category)
    WHERE completion_hour IS NOT NULL;
"#;
