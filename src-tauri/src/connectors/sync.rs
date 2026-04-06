use crate::connectors::zoom;
use crate::db::repositories::connections as conn_repo;
use crate::models::connection::PendingImport;
use rusqlite::Connection as DbConn;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(serde::Serialize, Debug)]
pub struct SyncResult {
    pub new_imports: usize,
    pub errors: Vec<String>,
}

pub async fn sync_all_connections(db: &Mutex<DbConn>) -> Result<SyncResult, String> {
    let mut result = SyncResult {
        new_imports: 0,
        errors: vec![],
    };

    // ─── Zoom sync ────────────────────────────────────────────────────────────
    let zoom_conn_opt = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "zoom")?
    };

    if let Some(zoom_conn) = zoom_conn_opt {
        match sync_zoom(db, &zoom_conn).await {
            Ok(count) => result.new_imports += count,
            Err(e) => result.errors.push(format!("Zoom sync failed: {}", e)),
        }
    }

    // ─── Gmail sync (stub — no-op) ────────────────────────────────────────────
    let gmail_conn_opt = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "gmail")?
    };

    if gmail_conn_opt.is_some() {
        // Gmail sync not yet implemented; silently skip
    }

    Ok(result)
}

async fn sync_zoom(
    db: &Mutex<DbConn>,
    zoom_conn: &crate::models::connection::Connection,
) -> Result<usize, String> {
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
        }
    }

    // Update last_sync_at timestamp
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn_repo::update_last_sync(&conn, "zoom")?;
    }

    Ok(new_count)
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
