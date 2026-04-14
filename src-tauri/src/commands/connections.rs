use crate::connectors::{sheets_relay, sync};
use crate::db::repositories::connections as conn_repo;
use crate::models::connection::{Connection, ImportApproval, PendingImport, SaveConnectionInput};
use crate::AppState;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use tauri::{Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ─── Utility ─────────────────────────────────────────────────────────────────

/// Open a URL in the system's default browser.
#[tauri::command]
pub async fn open_url(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    #[allow(deprecated)]
    app_handle.shell().open(&url, None).map_err(|e| e.to_string())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn parse_code_from_http_request(request: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;
    for param in query.split('&') {
        let mut parts = param.splitn(2, '=');
        if parts.next() == Some("code") {
            return parts
                .next()
                .map(|v| urlencoding::decode(v).unwrap_or_default().into_owned());
        }
    }
    None
}

async fn run_oauth_flow(
    app_handle: &tauri::AppHandle,
    client_id: &str,
    client_secret: &str,
    auth_base_url: &str,
    token_url: &str,
    port: u16,
    scopes: &str,
) -> Result<(String, String, String), String> {
    use tauri_plugin_shell::ShellExt;

    // Generate PKCE pair
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    let state_param: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}&scope={}",
        auth_base_url,
        client_id,
        urlencoding::encode(&redirect_uri),
        code_challenge,
        state_param,
        urlencoding::encode(scopes),
    );

    // Bind callback server before opening browser (avoids race)
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|e| format!("Cannot bind OAuth callback port {}: {}", port, e))?;

    // Open browser
    app_handle
        .shell()
        .open(&auth_url, None)
        .map_err(|e| e.to_string())?;

    // Wait for callback (120 s timeout)
    let (mut stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        listener.accept(),
    )
    .await
    .map_err(|_| "OAuth timed out — no response from browser within 2 minutes".to_string())?
    .map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
    let request = String::from_utf8_lossy(&buf[..n]).to_string();

    let code = parse_code_from_http_request(&request)
        .ok_or_else(|| "No authorization code in callback".to_string())?;

    let html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body style='font-family:sans-serif;text-align:center;padding:60px'>\
        <h2>Connected to Meridian!</h2><p>You can close this tab.</p></body></html>";
    let _ = stream.write_all(html.as_bytes()).await;
    drop(stream);
    drop(listener);

    // Exchange code for tokens
    #[derive(serde::Deserialize)]
    struct Tokens {
        access_token: String,
        refresh_token: String,
        expires_in: u64,
    }

    let client = reqwest::Client::new();
    let tokens: Tokens = client
        .post(token_url)
        .basic_auth(client_id, Some(client_secret))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", code_verifier.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Token parse failed: {}", e))?;

    let expires_at =
        (chrono::Utc::now() + chrono::Duration::seconds(tokens.expires_in as i64)).to_rfc3339();

    Ok((tokens.access_token, tokens.refresh_token, expires_at))
}

// ─── Connection Management ────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect_zoom(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Connection, String> {
    let client_id = option_env!("ZOOM_CLIENT_ID").unwrap_or("placeholder_zoom_client_id");
    let client_secret =
        option_env!("ZOOM_CLIENT_SECRET").unwrap_or("placeholder_zoom_client_secret");

    if client_id == "placeholder_zoom_client_id" {
        return Err(
            "Zoom credentials not configured. See CREDENTIALS_SETUP.md for setup instructions."
                .to_string(),
        );
    }

    let (access_token, refresh_token, expires_at) = run_oauth_flow(
        &app_handle,
        client_id,
        client_secret,
        "https://zoom.us/oauth/authorize",
        "https://zoom.us/oauth/token",
        19274,
        "meeting:read:list_past_meetings meeting:read:summary cloud_recording:read:list_recording_files",
    )
    .await?;

    // Fetch user email
    #[derive(serde::Deserialize)]
    struct ZoomMe {
        email: String,
    }
    let me: ZoomMe = reqwest::Client::new()
        .get("https://api.zoom.us/v2/users/me")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("User info failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("User parse failed: {}", e))?;

    let input = SaveConnectionInput {
        provider: "zoom".to_string(),
        account_email: Some(me.email),
        access_token,
        refresh_token,
        scopes: Some(
            "meeting:read:list_past_meetings meeting:read:summary cloud_recording:read:list_recording_files"
                .to_string(),
        ),
        token_expires_at: Some(expires_at),
    };

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::save_connection(&conn, &input)
}

#[tauri::command]
pub async fn connect_gmail(
    _app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<Connection, String> {
    Err("Gmail direct OAuth is not available — use the Sheets Relay integration instead.".to_string())
}

/// Save (or update) the Google Sheets relay configuration.
/// Stores the Apps Script URL in the connections table and the secret key
/// in the OS keyring. Deliberately avoids save_connection() because that
/// helper tries to store an empty refresh_token in the keyring, which can
/// fail and silently prevent the row from being written.
#[tauri::command]
pub async fn save_sheets_relay_config(
    script_url: String,
    secret_key: String,
    state: State<'_, AppState>,
) -> Result<Connection, String> {
    if script_url.trim().is_empty() {
        return Err("Apps Script URL cannot be empty".to_string());
    }
    if secret_key.trim().is_empty() {
        return Err("Secret key cannot be empty".to_string());
    }
    if !script_url.starts_with("https://script.google.com/") {
        return Err(
            "That doesn't look like an Apps Script URL — it should start with \
             https://script.google.com/"
                .to_string(),
        );
    }

    // Store secret in app_settings (no Keychain — avoids macOS permission prompts
    // on unsigned dev builds and is appropriate for an API key, not a user password)
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('sheets_relay_secret', ?1)",
            rusqlite::params![secret_key.trim()],
        )
        .map_err(|e| format!("Failed to save secret: {}", e))?;

        // Write the connection row
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO connections (id, provider, account_email, scopes)
             VALUES (?1, 'sheets_relay', ?2, 'sheets:read')
             ON CONFLICT(provider) DO UPDATE SET
                 account_email = excluded.account_email,
                 updated_at    = datetime('now')",
            rusqlite::params![id, script_url.trim()],
        )
        .map_err(|e| format!("Failed to save connection: {}", e))?;
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::get_connection(&conn, "sheets_relay")?
        .ok_or_else(|| "Connection not found after save — please try again".to_string())
}

/// Test the Sheets relay connection and return a human-readable result.
/// Also does a real row-fetch (since=0) so we can confirm data is reachable.
#[tauri::command]
pub async fn test_sheets_relay(state: State<'_, AppState>) -> Result<String, String> {
    let script_url = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_connection(&conn, "sheets_relay")?
            .and_then(|c| c.account_email)
            .ok_or_else(|| "No Sheets relay configured — save your settings first".to_string())?
    };

    let secret_key = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'sheets_relay_secret'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "Secret key not found — please save your Sheets relay settings again".to_string())?
    };

    // Ping the endpoint first
    sheets_relay::test_connection(&script_url, &secret_key).await?;

    // Do a real fetch with since=-1 so rows with unparseable dates are included too
    let rows = sheets_relay::fetch_new_rows(&script_url, &secret_key, -1).await?;
    let row_count = rows.len();

    if row_count == 0 {
        Ok("Connected! No rows found in the sheet yet — make sure at least one data row exists below the header row.".to_string())
    } else {
        Ok(format!(
            "Connected! Found {} row{} in the sheet. Click Sync Now to import.",
            row_count,
            if row_count == 1 { "" } else { "s" }
        ))
    }
}

/// Reset the high-water mark so the next sync re-fetches all rows from the Sheet.
/// Also called on disconnect to clean up the stored secret.
#[tauri::command]
pub async fn reset_sheets_relay_sync(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM app_settings WHERE key IN ('sheets_relay_last_sync_ms', 'sheets_relay_secret')",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_connection(
    provider: String,
    state: State<'_, AppState>,
) -> Result<Option<Connection>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::get_connection(&conn, &provider)
}

#[tauri::command]
pub async fn disconnect_provider(
    provider: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::delete_connection(&conn, &provider)
}

// ─── Sync ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn sync_connections(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<sync::SyncResult, String> {
    let result = sync::sync_all_connections(&state.db).await?;
    let _ = app_handle.emit("sync_complete", &result);
    Ok(result)
}

// ─── Pending Imports ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_pending_imports(
    state: State<'_, AppState>,
) -> Result<Vec<PendingImport>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::get_pending_imports(&conn)
}

#[tauri::command]
pub async fn count_pending_imports(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::count_pending_imports(&conn)
}

#[tauri::command]
pub async fn approve_import(
    input: ImportApproval,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Fetch pending import (short lock)
    let pending = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn_repo::get_pending_import(&conn, &input.pending_import_id)?
            .ok_or_else(|| "Pending import not found".to_string())?
    };

    // Choose content: prefer transcript when requested and available
    let raw_content = match input.import_type.as_str() {
        "transcript" if pending.transcript_content.is_some() => {
            pending.transcript_content.clone().unwrap()
        }
        _ => pending
            .summary_full
            .clone()
            .or_else(|| pending.summary_preview.clone())
            .ok_or_else(|| "No content available to import".to_string())?,
    };

    // Run the AI ingest pipeline (skip 50-word minimum — connector content is machine-generated)
    let result = crate::commands::meetings::ingest_meeting_core_from_connector(
        input.project_id.clone(),
        pending.title.clone(),
        "zoom".to_string(),
        raw_content,
        pending.attendees.clone(),
        pending.duration_minutes,
        pending.meeting_date.clone(),
        &state,
    )
    .await?;

    // Extract created meeting ID (required for backlink)
    let meeting_id = result
        .get("meeting")
        .and_then(|m| m.get("id"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| "Ingest succeeded but returned no meeting ID".to_string())?
        .to_string();

    // Update pending import status
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn_repo::update_pending_import_status(
            &conn,
            &input.pending_import_id,
            "imported",
            Some(&meeting_id),
            Some(&input.project_id),
        )?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn dismiss_import(
    pending_import_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::update_pending_import_status(&conn, &pending_import_id, "dismissed", None, None)
}
