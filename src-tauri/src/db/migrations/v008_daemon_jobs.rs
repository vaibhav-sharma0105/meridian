pub const V008_DAEMON_JOBS: &str = r#"
-- Daemon jobs table for background task queue
CREATE TABLE IF NOT EXISTS daemon_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    payload TEXT,
    result TEXT,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Scheduled jobs table for persistent cron-like schedules
CREATE TABLE IF NOT EXISTS scheduled_jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    job_type TEXT NOT NULL,
    cron_expression TEXT,
    interval_minutes INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    payload TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for efficient job queue polling
CREATE INDEX IF NOT EXISTS idx_daemon_jobs_status_scheduled
    ON daemon_jobs(status, scheduled_at)
    WHERE status = 'pending';

-- Index for job type lookup
CREATE INDEX IF NOT EXISTS idx_daemon_jobs_type
    ON daemon_jobs(job_type);

-- Index for scheduled jobs by next run time
CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_next_run
    ON scheduled_jobs(next_run_at)
    WHERE enabled = 1;
"#;
