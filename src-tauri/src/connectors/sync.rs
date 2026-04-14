use crate::connectors::{sheets_relay, zoom};
use crate::db::repositories::connections as conn_repo;
use crate::models::connection::PendingImport;
use rusqlite::Connection as DbConn;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(serde::Serialize, Debug)]
pub struct SyncResult {
    pub new_imports: usize,
    pub skipped_duplicates: usize,
    pub errors: Vec<String>,
}

pub async fn sync_all_connections(db: &Mutex<DbConn>) -> Result<SyncResult, String> {
    let mut result = SyncResult {
        new_imports: 0,
        skipped_duplicates: 0,
        errors: vec![],
    };

    // ─── Zoom sync ────────────────────────────────────────────────────────────
    let zoom_conn_opt = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "zoom")?
    };

    if let Some(zoom_conn) = zoom_conn_opt {
        match sync_zoom(db, &zoom_conn).await {
            Ok((new, skipped)) => {
                result.new_imports += new;
                result.skipped_duplicates += skipped;
            }
            Err(e) => result.errors.push(format!("Zoom sync failed: {}", e)),
        }
    }

    // ─── Sheets Relay sync ───────────────────────────────────────────────────
    let relay_conn_opt = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "sheets_relay")?
    };

    if let Some(_relay_conn) = relay_conn_opt {
        match sync_sheets_relay(db).await {
            Ok((new, skipped)) => {
                result.new_imports += new;
                result.skipped_duplicates += skipped;
            }
            Err(e) => result.errors.push(format!("Sheets relay sync failed: {}", e)),
        }
    }

    Ok(result)
}

async fn sync_zoom(
    db: &Mutex<DbConn>,
    zoom_conn: &crate::models::connection::Connection,
) -> Result<(usize, usize), String> {
    // Read access token from keyring
    let access_token = keyring::Entry::new("meridian", "zoom-token")
        .map_err(|e| e.to_string())?
        .get_password()
        .map_err(|_| "Zoom not connected — please reconnect in Settings > Connections".to_string())?;

    let refresh_token = keyring::Entry::new("meridian", "zoom-refresh")
        .map_err(|e| e.to_string())?
        .get_password()
        .unwrap_or_default();

    // Refresh token if expired
    let access_token =
        refresh_zoom_if_needed(db, &access_token, &refresh_token, zoom_conn).await?;

    // Determine sync window (last_sync_at date, or 14 days ago on first sync)
    let since = match &zoom_conn.last_sync_at {
        Some(dt) => dt.get(..10).unwrap_or(dt).to_string(),
        None => {
            let date = chrono::Utc::now() - chrono::Duration::days(14);
            date.format("%Y-%m-%d").to_string()
        }
    };

    let meetings = zoom::list_past_meetings(&access_token, &since).await?;
    let mut new_count = 0usize;
    let mut skipped_count = 0usize;

    for meeting in meetings {
        let meeting_id_str = meeting.id.to_string();

        // Courtesy throttle: 100 ms between per-meeting API calls
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let summary = zoom::get_meeting_summary(&access_token, meeting.id)
            .await
            .unwrap_or(None);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let transcript = zoom::get_meeting_transcript(&access_token, meeting.id)
            .await
            .unwrap_or(None);

        let import = PendingImport {
            id: Uuid::new_v4().to_string(),
            provider: "zoom".to_string(),
            external_meeting_id: Some(meeting_id_str),
            title: meeting.topic,
            meeting_date: Some(meeting.start_time),
            duration_minutes: Some(meeting.duration as i32),
            attendees: None,
            summary_preview: summary.as_ref().map(|s| s.chars().take(500).collect()),
            summary_full: summary,
            transcript_available: transcript.is_some(),
            transcript_content: transcript,
            zoom_join_url: meeting.join_url,
            source_email_id: None,
            status: "pending".to_string(),
            imported_meeting_id: None,
            project_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let inserted = {
            let conn = db.lock().map_err(|e| e.to_string())?;
            conn_repo::upsert_pending_import(&conn, &import)?
        };
        if inserted {
            new_count += 1;
        } else {
            skipped_count += 1;
        }
    }

    // Update last_sync_at timestamp
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::update_last_sync(&conn, "zoom")?;
    }

    Ok((new_count, skipped_count))
}

async fn refresh_zoom_if_needed(
    db: &Mutex<DbConn>,
    access_token: &str,
    refresh_token: &str,
    zoom_conn: &crate::models::connection::Connection,
) -> Result<String, String> {
    // If expiry is unknown (NULL or unparseable), proactively refresh
    let is_expired = zoom_conn
        .token_expires_at
        .as_ref()
        .and_then(|exp| chrono::DateTime::parse_from_rfc3339(exp).ok())
        .map(|t| t < chrono::Utc::now())
        .unwrap_or(true);

    if is_expired {
        let new_tokens = zoom::refresh_access_token(refresh_token).await?;
        let expires_at = (chrono::Utc::now()
            + chrono::Duration::seconds(new_tokens.expires_in as i64))
        .to_rfc3339();
        {
            let conn = db.lock().map_err(|e| e.to_string())?;
            conn_repo::update_tokens_with_db(
                &conn,
                "zoom",
                &new_tokens.access_token,
                &new_tokens.refresh_token,
                Some(&expires_at),
            )?;
        }
        Ok(new_tokens.access_token)
    } else {
        Ok(access_token.to_string())
    }
}

// ─── Sheets Relay sync ────────────────────────────────────────────────────────

async fn sync_sheets_relay(db: &Mutex<DbConn>) -> Result<(usize, usize), String> {
    // Read the Apps Script URL (stored as account_email on the connection record)
    let script_url = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "sheets_relay")?
            .and_then(|c| c.account_email)
            .ok_or_else(|| "Sheets relay URL not configured".to_string())?
    };

    // Read the secret key from app_settings (stored there to avoid Keychain prompts)
    let secret_key = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'sheets_relay_secret'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| {
            "Sheets relay secret not found — please reconnect in Settings > Connections"
                .to_string()
        })?
    };

    // Determine the last-synced timestamp (epoch ms, stored in app_settings)
    let since_ms: i64 = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'sheets_relay_last_sync_ms'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
    };

    // Fetch new rows from the Apps Script endpoint
    let rows = sheets_relay::fetch_new_rows(&script_url, &secret_key, since_ms).await?;

    if rows.is_empty() {
        // Still update last_sync_at even with no new rows
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::update_last_sync(&conn, "sheets_relay")?;
        return Ok((0, 0));
    }

    // Track the highest created_at timestamp seen so we don't re-process rows
    let mut max_ts_ms = since_ms;
    let mut new_count = 0usize;
    let mut skipped_count = 0usize;

    for row in &rows {
        // Parse the row's timestamp to update our watermark
        if let Some(ts_str) = &row.created_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                let ts_ms = dt.timestamp_millis();
                if ts_ms > max_ts_ms {
                    max_ts_ms = ts_ms;
                }
            }
        }

        let import = sheets_relay::row_to_pending_import(row);

        // source_email_id is used as the dedup key (set in row_to_pending_import)
        let inserted = {
            let conn = db.lock().map_err(|e| e.to_string())?;
            conn_repo::upsert_pending_import(&conn, &import)?
        };
        if inserted {
            new_count += 1;
        } else {
            skipped_count += 1;
        }
    }

    // Persist the new high-water mark so next sync only fetches newer rows
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('sheets_relay_last_sync_ms', ?1)",
            rusqlite::params![max_ts_ms.to_string()],
        )
        .map_err(|e| e.to_string())?;
        conn_repo::update_last_sync(&conn, "sheets_relay")?;
    }

    Ok((new_count, skipped_count))
}
