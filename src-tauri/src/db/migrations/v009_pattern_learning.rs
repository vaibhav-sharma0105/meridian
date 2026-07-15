pub const SQL: &str = r#"
-- Pattern observations table for recording user actions
CREATE TABLE IF NOT EXISTS pattern_observations (
    id TEXT PRIMARY KEY,
    observation_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    project_id TEXT,
    context_data TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    processed_at TEXT
);

-- Pattern models table for aggregated learning
CREATE TABLE IF NOT EXISTS pattern_models (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    project_id TEXT,
    model_data TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.0,
    observation_count INTEGER NOT NULL DEFAULT 0,
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pattern_type, project_id)
);

-- Indexes for efficient observation processing
CREATE INDEX IF NOT EXISTS idx_pattern_observations_type
    ON pattern_observations(observation_type);

CREATE INDEX IF NOT EXISTS idx_pattern_observations_project
    ON pattern_observations(project_id);

CREATE INDEX IF NOT EXISTS idx_pattern_observations_unprocessed
    ON pattern_observations(processed_at)
    WHERE processed_at IS NULL;

-- Indexes for pattern model lookup
CREATE INDEX IF NOT EXISTS idx_pattern_models_type_project
    ON pattern_models(pattern_type, project_id);
"#;
