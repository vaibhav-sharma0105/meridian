use crate::models::connection::{Connection, PendingImport, SaveConnectionInput};
use rusqlite::{params, Connection as DbConn};
use uuid::Uuid;

fn row_to_connection(row: &rusqlite::Row<'_>) -> rusqlite::Result<Connection> {
    Ok(Connection {
        id: row.get(0)?,
        provider: row.get(1)?,
        account_email: row.get(2)?,
        scopes: row.get(3)?,
        token_expires_at: row.get(4)?,
        last_sync_at: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn row_to_pending_import(row: &rusqlite::Row<'_>) -> rusqlite::Result<PendingImport> {
    let transcript_available: i64 = row.get(9)?;
    Ok(PendingImport {
        id: row.get(0)?,
        provider: row.get(1)?,
        external_meeting_id: row.get(2)?,
        title: row.get(3)?,
        meeting_date: row.get(4)?,
        duration_minutes: row.get(5)?,
        attendees: row.get(6)?,
        summary_preview: row.get(7)?,
        summary_full: row.get(8)?,
        transcript_available: transcript_available != 0,
        transcript_content: row.get(10)?,
        zoom_join_url: row.get(11)?,
        source_email_id: row.get(12)?,
        status: row.get(13)?,
        imported_meeting_id: row.get(14)?,
        project_id: row.get(15)?,
        created_at: row.get(16)?,
    })
}

pub fn get_connection(conn: &DbConn, provider: &str) -> Result<Option<Connection>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, provider, account_email, scopes, token_expires_at, last_sync_at,
             created_at, updated_at FROM connections WHERE provider = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(params![provider], row_to_connection)
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn save_connection(conn: &DbConn, input: &SaveConnectionInput) -> Result<Connection, String> {
    // Store tokens in OS keyring
    keyring::Entry::new("meridian", &format!("{}-token", input.provider))
        .map_err(|e| e.to_string())?
        .set_password(&input.access_token)
        .map_err(|e| e.to_string())?;
    keyring::Entry::new("meridian", &format!("{}-refresh", input.provider))
        .map_err(|e| e.to_string())?
        .set_password(&input.refresh_token)
        .map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO connections (id, provider, account_email, scopes, token_expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(provider) DO UPDATE SET
             account_email    = excluded.account_email,
             scopes           = excluded.scopes,
             token_expires_at = excluded.token_expires_at,
             updated_at       = datetime('now')",
        params![
            id,
            input.provider,
            input.account_email,
            input.scopes,
            input.token_expires_at
        ],
    )
    .map_err(|e| e.to_string())?;

    get_connection(conn, &input.provider)?
        .ok_or_else(|| "Connection not found after save".to_string())
}

pub fn delete_connection(conn: &DbConn, provider: &str) -> Result<(), String> {
    let _ = keyring::Entry::new("meridian", &format!("{}-token", provider))
        .and_then(|e| e.delete_password());
    let _ = keyring::Entry::new("meridian", &format!("{}-refresh", provider))
        .and_then(|e| e.delete_password());
    conn.execute("DELETE FROM connections WHERE provider = ?1", params![provider])
        .map_err(|e| e.to_string())?;
    // Dismiss orphaned pending imports so they don't show stale notifications
    conn.execute(
        "UPDATE pending_imports SET status = 'dismissed' WHERE provider = ?1 AND status = 'pending'",
        params![provider],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_last_sync(conn: &DbConn, provider: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE connections SET last_sync_at = datetime('now'), updated_at = datetime('now') WHERE provider = ?1",
        params![provider],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_tokens_with_db(
    conn: &DbConn,
    provider: &str,
    access_token: &str,
    refresh_token: &str,
    expires_at: Option<&str>,
) -> Result<(), String> {
    keyring::Entry::new("meridian", &format!("{}-token", provider))
        .map_err(|e| e.to_string())?
        .set_password(access_token)
        .map_err(|e| e.to_string())?;
    keyring::Entry::new("meridian", &format!("{}-refresh", provider))
        .map_err(|e| e.to_string())?
        .set_password(refresh_token)
        .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE connections SET token_expires_at = ?1, updated_at = datetime('now') WHERE provider = ?2",
        params![expires_at, provider],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_pending_imports(conn: &DbConn) -> Result<Vec<PendingImport>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, provider, external_meeting_id, title, meeting_date, duration_minutes,
             attendees, summary_preview, summary_full, transcript_available, transcript_content,
             zoom_join_url, source_email_id, status, imported_meeting_id, project_id, created_at
             FROM pending_imports WHERE status = 'pending' ORDER BY meeting_date DESC",
        )
        .map_err(|e| e.to_string())?;
    let imports = stmt
        .query_map([], row_to_pending_import)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(imports)
}

pub fn upsert_pending_import(conn: &DbConn, import: &PendingImport) -> Result<bool, String> {
    let rows = conn
        .execute(
            "INSERT OR IGNORE INTO pending_imports
             (id, provider, external_meeting_id, title, meeting_date, duration_minutes,
              attendees, summary_preview, summary_full, transcript_available, transcript_content,
              zoom_join_url, source_email_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                import.id,
                import.provider,
                import.external_meeting_id,
                import.title,
                import.meeting_date,
                import.duration_minutes,
                import.attendees,
                import.summary_preview,
                import.summary_full,
                import.transcript_available as i64,
                import.transcript_content,
                import.zoom_join_url,
                import.source_email_id,
                import.status,
            ],
        )
        .map_err(|e| e.to_string())?;
    Ok(rows > 0)
}

pub fn update_pending_import_status(
    conn: &DbConn,
    id: &str,
    status: &str,
    imported_meeting_id: Option<&str>,
    project_id: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE pending_imports SET status = ?1, imported_meeting_id = ?2, project_id = ?3 WHERE id = ?4",
        params![status, imported_meeting_id, project_id, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_pending_import(conn: &DbConn, id: &str) -> Result<Option<PendingImport>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, provider, external_meeting_id, title, meeting_date, duration_minutes,
             attendees, summary_preview, summary_full, transcript_available, transcript_content,
             zoom_join_url, source_email_id, status, imported_meeting_id, project_id, created_at
             FROM pending_imports WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(params![id], row_to_pending_import)
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn external_meeting_id_exists(conn: &DbConn, external_id: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pending_imports WHERE external_meeting_id = ?1",
            params![external_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn source_email_id_exists(conn: &DbConn, email_id: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pending_imports WHERE source_email_id = ?1",
            params![email_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn count_pending_imports(conn: &DbConn) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM pending_imports WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}
