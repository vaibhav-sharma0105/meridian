pub const SQL: &str = r#"
-- OAuth connections (Zoom, Gmail)
CREATE TABLE IF NOT EXISTS connections (
    id              TEXT PRIMARY KEY,
    provider        TEXT NOT NULL UNIQUE,
    account_email   TEXT,
    scopes          TEXT,
    token_expires_at TEXT,
    last_sync_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Meetings detected but not yet imported
CREATE TABLE IF NOT EXISTS pending_imports (
    id                    TEXT PRIMARY KEY,
    provider              TEXT NOT NULL,
    external_meeting_id   TEXT,
    title                 TEXT NOT NULL,
    meeting_date          TEXT,
    duration_minutes      INTEGER,
    attendees             TEXT,
    summary_preview       TEXT,
    summary_full          TEXT,
    transcript_available  INTEGER NOT NULL DEFAULT 0,
    transcript_content    TEXT,
    zoom_join_url         TEXT,
    source_email_id       TEXT,
    status                TEXT NOT NULL DEFAULT 'pending',
    imported_meeting_id   TEXT,
    project_id            TEXT,
    created_at            TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_pending_ext_meeting ON pending_imports(external_meeting_id)
    WHERE external_meeting_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_pending_email ON pending_imports(source_email_id)
    WHERE source_email_id IS NOT NULL;
"#;
