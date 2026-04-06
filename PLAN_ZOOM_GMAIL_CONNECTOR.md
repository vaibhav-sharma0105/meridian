# Zoom + Gmail Connector — Execution Plan

## Overview

Automatically detect completed Zoom meetings, fetch their AI summaries and VTT transcripts, and present them as pending imports in Meridian's notification system. Users approve each import, select a target project, and the existing AI pipeline (task extraction, health scoring) runs on import.

**Two connectors cover all scenarios:**

| Scenario | Zoom API | Gmail API |
|---|---|---|
| Meetings user **hosts** | Full VTT transcript + AI summary | AI summary (email) |
| Meetings user **attends** (same Zoom org) | Likely summary access, maybe transcript | AI summary (email) |
| Meetings user **attends** (external host) | No access | AI summary (email) |

Zoom API is the primary path (richer data). Gmail is the fallback guaranteeing coverage for non-hosted meetings.

---

## Phase 1: Database Migration (v005)

### File: `src-tauri/src/db/migrations/v005_connectors.rs` (NEW)

```rust
pub const SQL: &str = r#"
-- OAuth connections (Zoom, Gmail)
CREATE TABLE IF NOT EXISTS connections (
    id              TEXT PRIMARY KEY,
    provider        TEXT NOT NULL,          -- 'zoom' | 'gmail'
    account_email   TEXT,
    scopes          TEXT,
    token_expires_at TEXT,
    last_sync_at    TEXT,                   -- ISO 8601, NULL = never synced
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Meetings detected but not yet imported
CREATE TABLE IF NOT EXISTS pending_imports (
    id                    TEXT PRIMARY KEY,
    provider              TEXT NOT NULL,       -- 'zoom' | 'gmail'
    external_meeting_id   TEXT,                -- Zoom meeting ID (for dedup)
    title                 TEXT NOT NULL,
    meeting_date          TEXT,                -- ISO 8601
    duration_minutes      INTEGER,
    attendees             TEXT,                -- JSON array string
    summary_preview       TEXT,                -- first ~500 chars
    summary_full          TEXT,                -- complete AI summary
    transcript_available  INTEGER NOT NULL DEFAULT 0,
    transcript_content    TEXT,                -- full VTT content (NULL until fetched)
    zoom_join_url         TEXT,                -- deep link for "Audit" button
    source_email_id       TEXT,                -- Gmail message ID (for dedup)
    status                TEXT NOT NULL DEFAULT 'pending',  -- pending | imported | dismissed
    imported_meeting_id   TEXT,                -- FK to meetings.id after import
    project_id            TEXT,                -- set when user imports
    created_at            TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Prevent duplicates
CREATE UNIQUE INDEX IF NOT EXISTS idx_pending_ext_meeting ON pending_imports(external_meeting_id)
    WHERE external_meeting_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_pending_email ON pending_imports(source_email_id)
    WHERE source_email_id IS NOT NULL;
"#;
```

**NOTE:** OAuth tokens (`access_token`, `refresh_token`) are NOT stored in SQLite. They are stored in the OS keychain via the `keyring` crate, following the existing pattern used for AI API keys in `commands/ai.rs:38-43`. The keychain key format is `"meridian-{provider}-token"` for access tokens and `"meridian-{provider}-refresh"` for refresh tokens.

### File: `src-tauri/src/db/migrations/mod.rs` — MODIFY

Add to the module declarations and `get_all_migrations()`:

```rust
pub mod v005_connectors;

// In get_all_migrations(), append:
Migration {
    version: 5,
    sql: v005_connectors::SQL,
},
```

---

## Phase 2: Rust Models

### File: `src-tauri/src/models/connection.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub provider: String,           // "zoom" | "gmail"
    pub account_email: Option<String>,
    pub scopes: Option<String>,
    pub token_expires_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveConnectionInput {
    pub provider: String,
    pub account_email: Option<String>,
    pub access_token: String,       // stored in keyring, NOT in DB
    pub refresh_token: String,      // stored in keyring, NOT in DB
    pub scopes: Option<String>,
    pub token_expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingImport {
    pub id: String,
    pub provider: String,
    pub external_meeting_id: Option<String>,
    pub title: String,
    pub meeting_date: Option<String>,
    pub duration_minutes: Option<i32>,
    pub attendees: Option<String>,
    pub summary_preview: Option<String>,
    pub summary_full: Option<String>,
    pub transcript_available: bool,
    pub transcript_content: Option<String>,
    pub zoom_join_url: Option<String>,
    pub source_email_id: Option<String>,
    pub status: String,
    pub imported_meeting_id: Option<String>,
    pub project_id: Option<String>,
    pub created_at: String,
}

/// What the frontend sends when user clicks "Import Summary" or "Import Transcript"
#[derive(Debug, Deserialize)]
pub struct ImportApproval {
    pub pending_import_id: String,
    pub project_id: String,
    pub import_type: String,        // "summary" | "transcript"
}
```

### File: `src-tauri/src/models/mod.rs` — MODIFY

```rust
pub mod connection;  // ADD this line
```

---

## Phase 3: Rust DB Repositories

### File: `src-tauri/src/db/repositories/connections.rs` (NEW)

Implements CRUD for `connections` table and `pending_imports` table.

**Functions to implement:**

```rust
// connections table
pub fn get_connection(conn: &Connection, provider: &str) -> Result<Option<models::Connection>, String>
pub fn save_connection(conn: &Connection, input: &SaveConnectionInput) -> Result<models::Connection, String>
    // UPSERT by provider — only one Zoom connection, one Gmail connection at a time
    // Stores access_token and refresh_token in keyring, NOT in DB:
    //   keyring::Entry::new("meridian", &format!("{}-token", input.provider)).set_password(&input.access_token)
    //   keyring::Entry::new("meridian", &format!("{}-refresh", input.provider)).set_password(&input.refresh_token)
pub fn delete_connection(conn: &Connection, provider: &str) -> Result<(), String>
    // Also deletes keyring entries
pub fn update_last_sync(conn: &Connection, provider: &str) -> Result<(), String>
    // Sets last_sync_at = datetime('now')
pub fn update_tokens(provider: &str, access_token: &str, refresh_token: &str, expires_at: Option<&str>) -> Result<(), String>
    // Updates keyring entries + token_expires_at in DB (called after token refresh)

// pending_imports table
pub fn get_pending_imports(conn: &Connection) -> Result<Vec<PendingImport>, String>
    // WHERE status = 'pending' ORDER BY meeting_date DESC
pub fn upsert_pending_import(conn: &Connection, import: &PendingImport) -> Result<(), String>
    // INSERT OR IGNORE — dedup on external_meeting_id or source_email_id
pub fn update_pending_import_status(conn: &Connection, id: &str, status: &str, imported_meeting_id: Option<&str>, project_id: Option<&str>) -> Result<(), String>
pub fn get_pending_import(conn: &Connection, id: &str) -> Result<Option<PendingImport>, String>
pub fn count_pending_imports(conn: &Connection) -> Result<i64, String>
    // COUNT(*) WHERE status = 'pending'
```

**Pattern reference:** Follow `src-tauri/src/db/repositories/notifications.rs` for row mapping and `tasks.rs` for parameterized queries.

### File: `src-tauri/src/db/repositories/mod.rs` — MODIFY

```rust
pub mod connections;  // ADD this line
```

---

## Phase 4: Zoom Connector (Rust)

### File: `src-tauri/src/connectors/mod.rs` (NEW)

```rust
pub mod zoom;
pub mod gmail;
pub mod sync;
```

### File: `src-tauri/src/connectors/zoom.rs` (NEW)

This module handles all Zoom API interactions.

#### 4a. Zoom OAuth App Setup (One-time, manual)

Before any code runs, someone must register a Zoom OAuth app:

1. Go to https://marketplace.zoom.us/develop/create → Choose "General App" (OAuth type)
2. Set redirect URL to `http://127.0.0.1:19274/callback` (arbitrary port, hardcoded in Meridian)
3. Required scopes: `meeting:read:list_past_meetings`, `meeting:read:summary`, `cloud_recording:read:list_recording_files`
4. Note the `client_id` and `client_secret`
5. These are bundled in the app binary as compile-time constants (or in a config file). They are NOT user secrets — they identify the Meridian app itself. Every OAuth desktop app bundles these (Slack, VS Code, etc.)

#### 4b. OAuth Flow

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;

const ZOOM_CLIENT_ID: &str = env!("ZOOM_CLIENT_ID");       // set at build time
const ZOOM_CLIENT_SECRET: &str = env!("ZOOM_CLIENT_SECRET");
const ZOOM_REDIRECT_PORT: u16 = 19274;
const ZOOM_AUTH_URL: &str = "https://zoom.us/oauth/authorize";
const ZOOM_TOKEN_URL: &str = "https://zoom.us/oauth/token";
const ZOOM_SCOPES: &str = "meeting:read:list_past_meetings meeting:read:summary cloud_recording:read:list_recording_files";

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,        // seconds
    token_type: String,
}

#[derive(Deserialize)]
struct ZoomUser {
    email: String,
}

/// Starts the OAuth flow:
/// 1. Generates PKCE code_verifier + code_challenge
/// 2. Spawns a temporary local HTTP server on 127.0.0.1:19274
/// 3. Returns the authorization URL to open in the browser
/// 4. Waits for the callback with the auth code
/// 5. Exchanges the code for tokens
/// 6. Fetches user email from /v2/users/me
/// 7. Returns (access_token, refresh_token, expires_at, email)
pub async fn start_oauth_flow() -> Result<(String, String, String, String), String>

/// Refresh an expired access token using the refresh token
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String>
```

**Implementation details for the local HTTP server:**
- Use `tokio::net::TcpListener` bound to `127.0.0.1:19274`
- Accept exactly one connection, parse the `?code=XXX` from the GET request
- Respond with a simple HTML page: "Connected to Meridian! You can close this tab."
- Shut down the listener immediately after
- Use a `oneshot::channel` to send the auth code back to the calling async function
- Timeout after 120 seconds if user doesn't complete auth

**PKCE flow:**
- Generate random 43-128 char `code_verifier` using `rand` crate (add to Cargo.toml)
- `code_challenge` = base64url(SHA256(code_verifier)) — use `sha2` crate
- Pass `code_challenge_method=S256` in the authorization URL

#### 4c. Zoom API Client

```rust
#[derive(Deserialize)]
pub struct ZoomMeeting {
    pub id: u64,              // Zoom meeting ID (numeric)
    pub uuid: String,
    pub topic: String,        // meeting title
    pub start_time: String,   // ISO 8601
    pub duration: u32,        // minutes
    pub join_url: Option<String>,
}

#[derive(Deserialize)]
pub struct ZoomMeetingList {
    pub meetings: Vec<ZoomMeeting>,
    pub next_page_token: Option<String>,
}

#[derive(Deserialize)]
pub struct ZoomMeetingSummary {
    pub summary_details: Option<ZoomSummaryDetails>,
}

#[derive(Deserialize)]
pub struct ZoomSummaryDetails {
    pub summary_overview: Option<String>,
    pub next_steps: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ZoomRecordingFile {
    pub file_type: String,       // "TRANSCRIPT" for VTT
    pub download_url: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ZoomRecordingList {
    pub recording_files: Option<Vec<ZoomRecordingFile>>,
}

/// Fetch past meetings since a given date
/// GET https://api.zoom.us/v2/users/me/meetings?type=previous_meetings&from={date}&page_size=50
/// Handles pagination via next_page_token
pub async fn list_past_meetings(access_token: &str, since: &str) -> Result<Vec<ZoomMeeting>, String>

/// Fetch AI companion meeting summary
/// GET https://api.zoom.us/v2/meetings/{meeting_id}/meeting_summary
/// Returns None if summary not available (not hosted / feature disabled)
pub async fn get_meeting_summary(access_token: &str, meeting_id: u64) -> Result<Option<String>, String>

/// Fetch recording files, filter for TRANSCRIPT type, download VTT content
/// GET https://api.zoom.us/v2/meetings/{meeting_id}/recordings
/// Then GET the download_url with ?access_token=XXX to fetch VTT content
/// Returns None if no transcript available
pub async fn get_meeting_transcript(access_token: &str, meeting_id: u64) -> Result<Option<String>, String>
```

**Error handling:**
- 401 → token expired, trigger refresh and retry once
- 404 on summary/recordings → meeting not hosted by user or feature disabled, return `Ok(None)`
- 429 → Zoom rate limit (low chance for personal use), log warning, skip meeting, continue with others
- All reqwest errors → convert to `String` via `.map_err(|e| e.to_string())`

**Rate limiting:** Zoom allows ~10 requests/second for OAuth apps. Since sync runs once on launch, we won't hit this unless the user has 100+ meetings since last sync. Add a small `tokio::time::sleep(100ms)` between per-meeting API calls as courtesy throttling.

---

## Phase 5: Gmail Connector (Rust)

### File: `src-tauri/src/connectors/gmail.rs` (NEW)

#### 5a. Gmail OAuth App Setup (One-time, manual)

1. Go to https://console.cloud.google.com → Create project "Meridian"
2. Enable Gmail API
3. Create OAuth 2.0 credentials → Desktop application
4. Redirect URI: `http://127.0.0.1:19275/callback` (different port from Zoom)
5. Required scope: `https://www.googleapis.com/auth/gmail.readonly`
6. Bundle `client_id` and `client_secret` as compile-time constants

#### 5b. OAuth Flow

Same PKCE pattern as Zoom but targeting Google endpoints:
- Auth URL: `https://accounts.google.com/o/oauth2/v2/auth`
- Token URL: `https://oauth2.googleapis.com/token`
- Port: `19275`
- User info: `GET https://www.googleapis.com/gmail/v1/users/me/profile` (returns email)

```rust
const GMAIL_CLIENT_ID: &str = env!("GMAIL_CLIENT_ID");
const GMAIL_CLIENT_SECRET: &str = env!("GMAIL_CLIENT_SECRET");
const GMAIL_REDIRECT_PORT: u16 = 19275;

pub async fn start_oauth_flow() -> Result<(String, String, String, String), String>
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String>
```

#### 5c. Gmail API Client

```rust
#[derive(Deserialize)]
struct GmailMessageList {
    messages: Option<Vec<GmailMessageRef>>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
struct GmailMessageRef {
    id: String,
}

#[derive(Deserialize)]
struct GmailMessage {
    id: String,
    payload: GmailPayload,
    internal_date: String,     // epoch ms
}

#[derive(Deserialize)]
struct GmailPayload {
    headers: Vec<GmailHeader>,
    body: Option<GmailBody>,
    parts: Option<Vec<GmailPart>>,
}

// ... (standard Gmail message structure types)

/// Search for Zoom summary emails since a given date
/// GET https://gmail.googleapis.com/gmail/v1/users/me/messages?q=from:no-reply@zoom.us subject:"AI Companion" after:{epoch_seconds}
/// Returns a list of message IDs
pub async fn find_zoom_summary_emails(access_token: &str, since_epoch: i64) -> Result<Vec<String>, String>

/// Fetch a single email and parse out:
/// - Meeting title (from subject line, strip "Meeting Summary Available: " prefix)
/// - Meeting date (from internal_date)
/// - Summary content (from email HTML body, stripped to text)
/// - External meeting ID (parse from email body links, e.g., zoom.us/j/XXXXX)
pub async fn parse_zoom_summary_email(access_token: &str, message_id: &str) -> Result<ParsedZoomEmail, String>

pub struct ParsedZoomEmail {
    pub title: String,
    pub meeting_date: String,
    pub summary_text: String,
    pub external_meeting_id: Option<String>,  // extracted from zoom URLs in email body
    pub source_email_id: String,
}
```

**Email parsing strategy:**
- Zoom summary emails have a consistent format with subject like "Meeting Summary Available: {title}"
- The HTML body contains the AI summary in structured sections
- Strip HTML tags to get plain text summary (use a simple regex-based HTML stripper, no heavy dependency needed)
- Extract Zoom meeting ID from URLs like `https://zoom.us/j/12345678` in the email body for dedup against Zoom API results

---

## Phase 6: Sync Engine (Rust)

### File: `src-tauri/src/connectors/sync.rs` (NEW)

This is the orchestrator that runs on app launch.

```rust
use crate::db::repositories::connections as conn_repo;
use crate::connectors::{zoom, gmail};
use crate::models::connection::PendingImport;
use uuid::Uuid;

/// Main sync function — called from a Tauri command
/// Returns the number of new pending imports found
pub async fn sync_all_connections(db_conn: &rusqlite::Connection) -> Result<SyncResult, String> {
    let mut result = SyncResult { new_imports: 0, errors: vec![] };

    // --- Zoom sync ---
    if let Some(zoom_conn) = conn_repo::get_connection(db_conn, "zoom")? {
        match sync_zoom(db_conn, &zoom_conn).await {
            Ok(count) => result.new_imports += count,
            Err(e) => result.errors.push(format!("Zoom sync failed: {}", e)),
        }
    }

    // --- Gmail sync ---
    if let Some(gmail_conn) = conn_repo::get_connection(db_conn, "gmail")? {
        match sync_gmail(db_conn, &gmail_conn).await {
            Ok(count) => result.new_imports += count,
            Err(e) => result.errors.push(format!("Gmail sync failed: {}", e)),
        }
    }

    Ok(result)
}

#[derive(serde::Serialize)]
pub struct SyncResult {
    pub new_imports: usize,
    pub errors: Vec<String>,
}
```

**`sync_zoom` logic:**

```
1. Read access_token from keyring::Entry::new("meridian", "zoom-token")
2. Read refresh_token from keyring::Entry::new("meridian", "zoom-refresh")
3. Check token_expires_at — if expired, call zoom::refresh_access_token()
   and update keyring + DB via conn_repo::update_tokens()
4. Determine `since` date:
   - If last_sync_at is set → use it
   - If NULL (first sync) → use 14 days ago (reasonable default to not overwhelm)
5. Call zoom::list_past_meetings(access_token, since)
6. For each meeting:
   a. Check if external_meeting_id already exists in pending_imports → skip if so
   b. Call zoom::get_meeting_summary() → summary text (may be None)
   c. Call zoom::get_meeting_transcript() → VTT content (may be None)
   d. Build PendingImport {
        id: Uuid::new_v4(),
        provider: "zoom",
        external_meeting_id: Some(meeting.id.to_string()),
        title: meeting.topic,
        meeting_date: Some(meeting.start_time),
        duration_minutes: Some(meeting.duration as i32),
        summary_preview: summary.as_ref().map(|s| s.chars().take(500).collect()),
        summary_full: summary,
        transcript_available: transcript.is_some(),
        transcript_content: transcript,
        zoom_join_url: meeting.join_url,
        status: "pending",
        ...
      }
   e. conn_repo::upsert_pending_import(db_conn, &import)
7. conn_repo::update_last_sync(db_conn, "zoom")
8. Return count of newly inserted imports
```

**`sync_gmail` logic:**

```
1. Read tokens from keyring (same pattern as Zoom)
2. Refresh if expired
3. Determine `since` — same logic as Zoom
4. Call gmail::find_zoom_summary_emails(access_token, since_epoch)
5. For each email message_id:
   a. Check if source_email_id already exists in pending_imports → skip
   b. Call gmail::parse_zoom_summary_email(access_token, message_id)
   c. If parsed.external_meeting_id is Some, check if that external_meeting_id
      already exists in pending_imports (already imported via Zoom API) → skip
   d. Build PendingImport with provider: "gmail", transcript_available: false
   e. conn_repo::upsert_pending_import(db_conn, &import)
6. conn_repo::update_last_sync(db_conn, "gmail")
7. Return count
```

**Deduplication flow:** Gmail sync runs AFTER Zoom sync. If the same meeting was already found via Zoom API (matched by `external_meeting_id`), the Gmail version is skipped. This ensures Zoom's richer data (with transcript) takes precedence.

### File: `src-tauri/src/lib.rs` — MODIFY

Add module declaration:

```rust
pub mod connectors;  // ADD after `pub mod utils;`
```

---

## Phase 7: Tauri Commands

### File: `src-tauri/src/commands/connections.rs` (NEW)

```rust
use crate::connectors::{zoom, gmail, sync};
use crate::db::repositories::connections as conn_repo;
use crate::models::connection::{Connection, PendingImport, ImportApproval};
use crate::AppState;
use tauri::{Emitter, State};

// ─── Connection Management ───────────────────────────────────────────────

/// Start Zoom OAuth flow: opens browser, waits for callback, saves tokens
#[tauri::command]
pub async fn connect_zoom(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<Connection, String> {
    // 1. Start OAuth flow (spawns local server, returns auth URL)
    // 2. Open browser via tauri_plugin_shell::ShellExt
    //    app_handle.shell().open(&auth_url, None).map_err(|e| e.to_string())?;
    // 3. Await token exchange
    // 4. Save connection via conn_repo::save_connection() (tokens go to keyring)
    // 5. Return the Connection record
}

/// Start Gmail OAuth flow: same pattern as Zoom
#[tauri::command]
pub async fn connect_gmail(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<Connection, String> {
    // Same as connect_zoom but for Gmail
}

/// Get connection status for a provider
#[tauri::command]
pub async fn get_connection(provider: String, state: State<'_, AppState>) -> Result<Option<Connection>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::get_connection(&conn, &provider)
}

/// Disconnect a provider (remove tokens from keyring + delete DB row)
#[tauri::command]
pub async fn disconnect_provider(provider: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::delete_connection(&conn, &provider)
}

// ─── Sync ────────────────────────────────────────────────────────────────

/// Trigger a sync for all connected providers
/// Called on app launch from frontend + manually from UI
#[tauri::command]
pub async fn sync_connections(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<sync::SyncResult, String> {
    // NOTE: sync needs to call async Zoom/Gmail APIs which are not compatible with
    // holding the DB mutex. Strategy:
    //
    // 1. Lock DB briefly to read connection records
    // 2. Drop the lock
    // 3. Run async API calls (zoom::list_past_meetings, gmail::find_zoom_summary_emails, etc.)
    // 4. Lock DB again to write pending_imports
    //
    // This avoids holding Mutex<Connection> across await points.
    // See the detailed sync implementation in Phase 6.

    let result = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        // Read connections, determine what to sync
        // ... then drop conn
    };

    // Run async API calls here (no DB lock held)

    // Lock again to write results
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // ... write pending imports

    // Emit event to frontend with count
    let _ = app_handle.emit("sync_complete", &result);

    Ok(result)
}

// ─── Pending Imports ─────────────────────────────────────────────────────

/// Get all pending imports
#[tauri::command]
pub async fn get_pending_imports(state: State<'_, AppState>) -> Result<Vec<PendingImport>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::get_pending_imports(&conn)
}

/// Count pending imports (for badge)
#[tauri::command]
pub async fn count_pending_imports(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::count_pending_imports(&conn)
}

/// User approves an import — creates a Meeting record and triggers AI pipeline
#[tauri::command]
pub async fn approve_import(
    input: ImportApproval,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // 1. Fetch the pending import
    // 2. Determine raw_transcript:
    //    - If input.import_type == "transcript" && pending.transcript_content.is_some()
    //      → use transcript_content (VTT)
    //    - If input.import_type == "summary"
    //      → use summary_full
    //    - Fallback: use summary_full
    // 3. Call the existing meeting ingest pipeline:
    //    - Create meeting via meetings::create_meeting()
    //    - Run AI extraction (summary, tasks, health score)
    //    - This reuses ALL existing logic from commands/meetings.rs::ingest_meeting()
    // 4. Update pending_import status = 'imported', set imported_meeting_id, project_id
    // 5. Create a notification: "Meeting '{title}' imported with {N} tasks"
    // 6. Return the same JSON as ingest_meeting: { meeting, tasks }
}

/// User dismisses an import
#[tauri::command]
pub async fn dismiss_import(
    pending_import_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn_repo::update_pending_import_status(&conn, &pending_import_id, "dismissed", None, None)
}
```

### Reusing the existing ingest pipeline

The `approve_import` command should NOT duplicate the AI extraction logic. Instead, refactor `commands/meetings.rs::ingest_meeting` to extract a shared helper:

**File: `src-tauri/src/commands/meetings.rs` — MODIFY**

Extract the core logic (lines 24-173 of current file) into a public helper function:

```rust
/// Core meeting ingest logic — used by both manual ingest and auto-import
pub async fn ingest_meeting_core(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: &State<'_, AppState>,
) -> Result<Value, String> {
    // ... existing logic from ingest_meeting, unchanged
}

#[tauri::command]
pub async fn ingest_meeting(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    ingest_meeting_core(project_id, title, platform, raw_transcript, attendees, duration_minutes, meeting_at, &state).await
}
```

Then `approve_import` calls `ingest_meeting_core()` with the pending import's data. Platform is set to `"zoom"`.

### File: `src-tauri/src/commands/mod.rs` — MODIFY

```rust
pub mod connections;  // ADD this line
```

### File: `src-tauri/src/lib.rs` — MODIFY invoke_handler

Add to the `tauri::generate_handler![...]` block:

```rust
// Connections
commands::connections::connect_zoom,
commands::connections::connect_gmail,
commands::connections::get_connection,
commands::connections::disconnect_provider,
commands::connections::sync_connections,
commands::connections::get_pending_imports,
commands::connections::count_pending_imports,
commands::connections::approve_import,
commands::connections::dismiss_import,
```

---

## Phase 8: Cargo.toml Dependencies

### File: `src-tauri/Cargo.toml` — MODIFY

Add these dependencies:

```toml
# For PKCE code_challenge generation (SHA-256)
sha2 = "0.10"
# For PKCE code_verifier random generation
rand = "0.8"
# For base64url encoding of code_challenge
base64 = "0.22"
```

`reqwest` (already present), `tokio` (already present with `full` features), `serde` (already present) are sufficient for all HTTP and async operations.

---

## Phase 9: Frontend Types

### File: `src/lib/tauri.ts` — MODIFY

Add types and invoke wrappers at the end of the relevant sections:

```typescript
// ─── Types (add after existing types) ────────────────────────────────────

export interface Connection {
  id: string;
  provider: "zoom" | "gmail";
  account_email: string | null;
  scopes: string | null;
  token_expires_at: string | null;
  last_sync_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface PendingImport {
  id: string;
  provider: "zoom" | "gmail";
  external_meeting_id: string | null;
  title: string;
  meeting_date: string | null;
  duration_minutes: number | null;
  attendees: string | null;
  summary_preview: string | null;
  summary_full: string | null;
  transcript_available: boolean;
  transcript_content: string | null;
  zoom_join_url: string | null;
  source_email_id: string | null;
  status: "pending" | "imported" | "dismissed";
  imported_meeting_id: string | null;
  project_id: string | null;
  created_at: string;
}

export interface ImportApproval {
  pending_import_id: string;
  project_id: string;
  import_type: "summary" | "transcript";
}

export interface SyncResult {
  new_imports: number;
  errors: string[];
}

// ─── Connections (add new section) ───────────────────────────────────────

export const connectZoom = () =>
  invoke<Connection>("connect_zoom");
export const connectGmail = () =>
  invoke<Connection>("connect_gmail");
export const getConnection = (provider: string) =>
  invoke<Connection | null>("get_connection", { provider });
export const disconnectProvider = (provider: string) =>
  invoke<void>("disconnect_provider", { provider });
export const syncConnections = () =>
  invoke<SyncResult>("sync_connections");
export const getPendingImports = () =>
  invoke<PendingImport[]>("get_pending_imports");
export const countPendingImports = () =>
  invoke<number>("count_pending_imports");
export const approveImport = (input: ImportApproval) =>
  invoke<IngestMeetingResult>("approve_import", { input });
export const dismissImport = (pendingImportId: string) =>
  invoke<void>("dismiss_import", { pendingImportId });

// ─── Event Listeners (add to existing section) ──────────────────────────

export const onSyncComplete = (
  callback: (data: SyncResult) => void
) => {
  return listen<SyncResult>(
    "sync_complete",
    (event) => callback(event.payload)
  );
};
```

---

## Phase 10: Frontend Hooks

### File: `src/hooks/useConnections.ts` (NEW)

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";

export function useConnections() {
  const qc = useQueryClient();

  const zoomQuery = useQuery({
    queryKey: ["connection", "zoom"],
    queryFn: () => api.getConnection("zoom"),
  });

  const gmailQuery = useQuery({
    queryKey: ["connection", "gmail"],
    queryFn: () => api.getConnection("gmail"),
  });

  const connectZoom = useMutation({
    mutationFn: api.connectZoom,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "zoom"] }),
  });

  const connectGmail = useMutation({
    mutationFn: api.connectGmail,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "gmail"] }),
  });

  const disconnect = useMutation({
    mutationFn: api.disconnectProvider,
    onSuccess: (_, provider) => qc.invalidateQueries({ queryKey: ["connection", provider] }),
  });

  return {
    zoom: zoomQuery.data ?? null,
    gmail: gmailQuery.data ?? null,
    isLoadingZoom: zoomQuery.isLoading,
    isLoadingGmail: gmailQuery.isLoading,
    connectZoom: connectZoom.mutateAsync,
    isConnectingZoom: connectZoom.isPending,
    connectGmail: connectGmail.mutateAsync,
    isConnectingGmail: connectGmail.isPending,
    disconnect: disconnect.mutateAsync,
  };
}
```

### File: `src/hooks/usePendingImports.ts` (NEW)

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { ImportApproval } from "@/lib/tauri";

export function usePendingImports() {
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["pending-imports"],
    queryFn: api.getPendingImports,
  });

  const countQuery = useQuery({
    queryKey: ["pending-imports-count"],
    queryFn: api.countPendingImports,
  });

  const approveMutation = useMutation({
    mutationFn: (input: ImportApproval) => api.approveImport(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pending-imports"] });
      qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
      // Also refresh meetings + tasks since a new meeting was imported
      qc.invalidateQueries({ queryKey: ["meetings"] });
      qc.invalidateQueries({ queryKey: ["tasks"] });
    },
  });

  const dismissMutation = useMutation({
    mutationFn: api.dismissImport,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pending-imports"] });
      qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
    },
  });

  return {
    pendingImports: query.data ?? [],
    pendingCount: countQuery.data ?? 0,
    isLoading: query.isLoading,
    approveImport: approveMutation.mutateAsync,
    isApproving: approveMutation.isPending,
    dismissImport: dismissMutation.mutateAsync,
  };
}
```

### File: `src/hooks/useSync.ts` (NEW)

```typescript
import { useEffect, useCallback, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import toast from "react-hot-toast";

/**
 * Runs sync on mount (app launch) and provides manual sync trigger.
 * Listens to the "sync_complete" event from the backend.
 * Call this once in AppShell.
 */
export function useSync() {
  const qc = useQueryClient();
  const hasSynced = useRef(false);

  const runSync = useCallback(async () => {
    try {
      const result = await api.syncConnections();
      if (result.new_imports > 0) {
        toast(`${result.new_imports} new meeting${result.new_imports > 1 ? "s" : ""} found`, { icon: "📥" });
        qc.invalidateQueries({ queryKey: ["pending-imports"] });
        qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
        qc.invalidateQueries({ queryKey: ["notifications"] });
      }
      if (result.errors.length > 0) {
        console.warn("Sync errors:", result.errors);
      }
    } catch (e) {
      console.error("Sync failed:", e);
    }
  }, [qc]);

  // Auto-sync on first mount (app launch)
  useEffect(() => {
    if (!hasSynced.current) {
      hasSynced.current = true;
      runSync();
    }
  }, [runSync]);

  return { runSync };
}
```

---

## Phase 11: Frontend UI Components

### 11a. Connections Settings Tab

#### File: `src/components/connections/ConnectionsSettings.tsx` (NEW)

This is a settings modal/tab accessible from the Settings panel. It follows the same pattern as `AISettings.tsx` (receives `open`/`onClose` props).

**However**, the current settings UI uses `uiStore.settingsOpen` with a single `settingsTab` string. To add the Connections tab:

**File: `src/stores/uiStore.ts` — MODIFY**

The existing `settingsTab` state already supports arbitrary string values. No change needed to the store — just pass `"connections"` as the tab value.

**File: `src/components/layout/AppShell.tsx` — MODIFY**

Add the ConnectionsSettings component alongside AISettings:

```tsx
import ConnectionsSettings from "@/components/connections/ConnectionsSettings";

// In the JSX, after <AISettings ... />:
<ConnectionsSettings
  open={settingsOpen && settingsTab === "connections"}
  onClose={() => setSettingsOpen(false)}
/>
```

Also modify the AISettings rendering to be tab-aware:

```tsx
<AISettings
  open={settingsOpen && settingsTab === "ai"}
  onClose={() => setSettingsOpen(false)}
/>
```

**File: `src/components/layout/Sidebar.tsx` — MODIFY**

Add a "Connections" button in the sidebar settings area (near the existing settings gear icon), or add it as a navigation option within the settings modal.

**ConnectionsSettings.tsx content:**

```
┌─────────────────────────────────────────────────────┐
│  Connections                                    [X]  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  🎥 Zoom                                      │  │
│  │                                                │  │
│  │  [Connected as user@company.com]               │  │
│  │  Last synced: Mar 23, 2:15 PM                  │  │
│  │                                                │  │
│  │  [Sync Now]  [Disconnect]                      │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  ✉️  Gmail                                     │  │
│  │                                                │  │
│  │  Not connected                                 │  │
│  │                                                │  │
│  │  [Connect Gmail]                               │  │
│  │                                                │  │
│  │  Used to import meeting summaries from Zoom    │  │
│  │  emails for meetings you attend but don't host │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Key behaviors:**
- "Connect Zoom" → calls `connectZoom()` → shows loading spinner → browser opens → on success, shows connected state
- "Sync Now" → calls `syncConnections()` → shows loading spinner → on complete, shows toast with count
- "Disconnect" → confirmation dialog → calls `disconnect("zoom")` → clears keyring tokens
- Each card uses `useConnections()` hook for state

### 11b. Pending Import Cards in Notification Center

#### File: `src/components/notifications/PendingImportCard.tsx` (NEW)

This component renders inside the NotificationCenter drawer for each pending import.

```
┌──────────────────────────────────────────────────────┐
│  📥 Weekly Product Sync                              │
│  via Zoom · Mar 21, 2:00 PM · 45 min                │
│                                                       │
│  "Discussed Q2 roadmap priorities. Key decisions..."  │
│                                                       │
│  Project: [ Select project         ▾]                │
│                                                       │
│  [Audit ↗]  [Import Summary]  [Import Transcript]    │
│                                          [Dismiss]    │
└──────────────────────────────────────────────────────┘
```

**Props:**

```typescript
interface Props {
  import: PendingImport;
  projects: Project[];
  onApprove: (pendingImportId: string, projectId: string, importType: "summary" | "transcript") => void;
  onDismiss: (pendingImportId: string) => void;
}
```

**Key behaviors:**
- Project dropdown defaults to no selection, user must pick one before import buttons are enabled
- "Import Transcript" button is disabled + shows tooltip "No transcript available" if `transcript_available === false`
- "Audit" opens `zoom_join_url` in the browser via `@tauri-apps/plugin-shell` → `open(url)`
- "Import Summary" calls `onApprove(id, projectId, "summary")`
- "Import Transcript" calls `onApprove(id, projectId, "transcript")`
- While importing, show a spinner on the clicked button
- After successful import, the card transitions to a "Imported ✓" state and fades out (or immediately disappears from pending list since status changes)
- "Dismiss" marks as dismissed, card disappears

#### File: `src/components/notifications/NotificationCenter.tsx` — MODIFY

Add a "Pending Imports" section at the top of the notification list:

```tsx
import { usePendingImports } from "@/hooks/usePendingImports";
import { useProjectStore } from "@/stores/projectStore";
import PendingImportCard from "./PendingImportCard";

// Inside the component:
const { pendingImports, approveImport, dismissImport } = usePendingImports();
const { projects } = useProjectStore();

// In the JSX, BEFORE the existing notifications list:
{pendingImports.length > 0 && (
  <div className="border-b border-zinc-200 dark:border-zinc-800">
    <div className="px-4 py-2 bg-indigo-50 dark:bg-indigo-900/20">
      <span className="text-xs font-semibold text-indigo-600 dark:text-indigo-400 uppercase tracking-wide">
        {pendingImports.length} New Meeting{pendingImports.length > 1 ? "s" : ""} Found
      </span>
    </div>
    {pendingImports.map((pi) => (
      <PendingImportCard
        key={pi.id}
        import={pi}
        projects={projects}
        onApprove={async (id, projectId, type) => {
          await approveImport({ pending_import_id: id, project_id: projectId, import_type: type });
        }}
        onDismiss={async (id) => {
          await dismissImport(id);
        }}
      />
    ))}
  </div>
)}
```

### 11c. Sidebar Badge for Pending Imports

#### File: `src/components/layout/Sidebar.tsx` — MODIFY

Add a pending import count badge next to the notification bell (or as a separate indicator):

```tsx
import { usePendingImports } from "@/hooks/usePendingImports";

// Inside the component:
const { pendingCount } = usePendingImports();

// Modify the bell icon badge to include pending imports in the count:
// The existing unreadCount badge should also show pendingCount
// Or add a separate indicator next to the bell:
{pendingCount > 0 && (
  <span className="absolute -top-1 -right-1 w-4 h-4 bg-indigo-500 text-white text-[10px] font-bold rounded-full flex items-center justify-center">
    {pendingCount}
  </span>
)}
```

### 11d. Auto-Sync on App Launch

#### File: `src/components/layout/AppShell.tsx` — MODIFY

```tsx
import { useSync } from "@/hooks/useSync";

// Inside the component:
useSync();  // Triggers sync on mount, shows toast when new meetings found
```

This is the single entry point for sync. When the user opens Meridian:
1. `useSync` fires `syncConnections()`
2. Backend queries Zoom API + Gmail API for new meetings
3. Inserts pending_imports
4. Emits `"sync_complete"` event
5. Frontend shows toast: "3 new meetings found"
6. User opens notification center → sees pending import cards
7. User picks project, clicks Import → meeting goes through existing pipeline

---

## Phase 12: Sync Status in Sidebar

### File: `src/components/layout/Sidebar.tsx` — MODIFY

Add a subtle sync indicator that shows when sync is running:

```
[🔄 Syncing...]  ← shows briefly on app launch during sync
[✓ Synced]       ← flashes green for 2 seconds after sync completes
[nothing]        ← default state, no indicator
```

The `useSync` hook can expose `isSyncing` state for this.

---

## File Summary — All Files Changed or Created

### New Files (13)

| File | Purpose |
|---|---|
| `src-tauri/src/db/migrations/v005_connectors.rs` | Schema: connections + pending_imports tables |
| `src-tauri/src/models/connection.rs` | Rust structs: Connection, PendingImport, ImportApproval |
| `src-tauri/src/db/repositories/connections.rs` | CRUD for connections + pending_imports |
| `src-tauri/src/connectors/mod.rs` | Module declarations |
| `src-tauri/src/connectors/zoom.rs` | Zoom OAuth flow + API client |
| `src-tauri/src/connectors/gmail.rs` | Gmail OAuth flow + API client |
| `src-tauri/src/connectors/sync.rs` | Sync engine orchestrator |
| `src-tauri/src/commands/connections.rs` | Tauri commands for connections |
| `src/hooks/useConnections.ts` | React hook for connection state |
| `src/hooks/usePendingImports.ts` | React hook for pending imports |
| `src/hooks/useSync.ts` | Auto-sync on app launch |
| `src/components/connections/ConnectionsSettings.tsx` | Settings UI for Zoom/Gmail connections |
| `src/components/notifications/PendingImportCard.tsx` | Pending import card UI |

### Modified Files (9)

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Add sha2, rand, base64 deps |
| `src-tauri/src/lib.rs` | Add `connectors` module + 9 new commands to invoke_handler |
| `src-tauri/src/db/migrations/mod.rs` | Register v005 migration |
| `src-tauri/src/models/mod.rs` | Add `connection` module |
| `src-tauri/src/db/repositories/mod.rs` | Add `connections` module |
| `src-tauri/src/commands/mod.rs` | Add `connections` module |
| `src-tauri/src/commands/meetings.rs` | Extract `ingest_meeting_core` helper |
| `src/lib/tauri.ts` | Add Connection/PendingImport types + invoke wrappers |
| `src/components/layout/AppShell.tsx` | Add useSync + ConnectionsSettings |
| `src/components/notifications/NotificationCenter.tsx` | Add pending imports section |
| `src/components/layout/Sidebar.tsx` | Add pending count badge + sync indicator |
| `src/stores/uiStore.ts` | No change needed (settingsTab already supports strings) |

---

## Build Order (Recommended Implementation Sequence)

### Sprint 1: Foundation (Backend-only, no UI)

1. **v005 migration** — create tables
2. **models/connection.rs** — structs
3. **repositories/connections.rs** — CRUD functions
4. **connectors/zoom.rs** — OAuth flow + API client
5. **connectors/sync.rs** — sync engine (Zoom only)
6. **commands/connections.rs** — Tauri commands
7. **Refactor meetings.rs** — extract `ingest_meeting_core`
8. **Register everything** in mod.rs + lib.rs

**Verification:** `cargo check` passes. Manually test Zoom OAuth via a temporary test command.

### Sprint 2: Frontend — Connections Settings

9. **tauri.ts** — add types + invoke wrappers
10. **useConnections.ts** — hook
11. **ConnectionsSettings.tsx** — settings UI
12. **AppShell.tsx** — wire up ConnectionsSettings

**Verification:** Can connect Zoom from UI, see connected state, disconnect.

### Sprint 3: Frontend — Sync + Pending Imports

13. **usePendingImports.ts** — hook
14. **useSync.ts** — auto-sync hook
15. **PendingImportCard.tsx** — card component
16. **NotificationCenter.tsx** — add pending imports section
17. **Sidebar.tsx** — add badge + sync indicator
18. **AppShell.tsx** — add `useSync()`

**Verification:** Launch app → sync runs → pending imports appear in notifications → import to project works → meeting + tasks created.

### Sprint 4: Gmail Connector

19. **connectors/gmail.rs** — OAuth flow + API client
20. **sync.rs** — add Gmail sync + dedup logic
21. **ConnectionsSettings.tsx** — add Gmail card
22. **commands/connections.rs** — implement `connect_gmail`

**Verification:** Connect Gmail → sync finds Zoom summary emails → imports work for non-hosted meetings.

---

## Testing Plan

### Unit Tests (Rust)

| Test | File | What it verifies |
|---|---|---|
| `test_upsert_pending_import_dedup` | `repositories/connections.rs` | Same external_meeting_id is not duplicated |
| `test_gmail_zoom_dedup` | `connectors/sync.rs` | Gmail import skips if Zoom API already found it |
| `test_token_refresh_flow` | `connectors/zoom.rs` | 401 → refresh → retry succeeds |
| `test_vtt_parsing` | `connectors/zoom.rs` | VTT content is correctly extracted from recording API response |
| `test_email_summary_parsing` | `connectors/gmail.rs` | Zoom summary email HTML is correctly parsed to text |
| `test_migration_v005` | `db/migrations/` | Tables created, indexes exist |

### Integration Tests (Manual)

| # | Scenario | Steps | Expected |
|---|---|---|---|
| 1 | Zoom OAuth connect | Settings → Connections → Connect Zoom → Approve in browser | Shows "Connected as email" |
| 2 | First sync (Zoom) | Open app after connecting Zoom | Toast: "N new meetings found" |
| 3 | Import with summary | Notification → pick project → Import Summary | Meeting created, tasks extracted |
| 4 | Import with transcript | Notification → pick project → Import Transcript | Meeting created with full VTT |
| 5 | Dismiss import | Notification → Dismiss | Card disappears, won't reappear |
| 6 | Subsequent sync | Close + reopen app | Only new meetings since last sync |
| 7 | Disconnect | Settings → Disconnect Zoom | Tokens cleared, no more sync |
| 8 | Gmail connect | Settings → Connect Gmail → Approve in browser | Shows "Connected as email" |
| 9 | Gmail sync (non-hosted) | Attend a meeting, wait for email, open app | Meeting appears in pending |
| 10 | Dedup: Zoom + Gmail | Host a meeting → both connectors find it | Only one pending import |
| 11 | Token expiry | Wait for token to expire → reopen app | Auto-refreshes, sync works |
| 12 | No connections | Open app with nothing connected | No errors, no sync, no badge |
| 13 | Offline | Open app without internet | Sync fails gracefully, toast/log |

### TypeScript Checks

```bash
npx tsc --noEmit                    # No type errors
cargo check                         # Rust compiles
cargo test                          # All unit tests pass
```

---

## Security Considerations

1. **OAuth tokens in OS keychain** — never in SQLite, never in logs, never sent to frontend
2. **Client ID/secret** — bundled at build time via `env!()`. These are NOT user secrets (same as any OAuth desktop app). For open-source distribution, use a `.env` file excluded from git.
3. **Gmail scope** — `gmail.readonly` only. Meridian never sends emails.
4. **Zoom scopes** — read-only. Meridian never creates/modifies meetings.
5. **Local HTTP server** — only binds to `127.0.0.1`, only accepts one request, times out in 120s. No persistent server.
6. **Token refresh** — handled automatically on 401. If refresh fails (revoked), connection status shows "Reconnect required" and stops syncing.

---

## Environment Variables (Build-time)

```bash
# Required for building with Zoom/Gmail OAuth
ZOOM_CLIENT_ID=your_zoom_client_id
ZOOM_CLIENT_SECRET=your_zoom_client_secret
GMAIL_CLIENT_ID=your_gmail_client_id
GMAIL_CLIENT_SECRET=your_gmail_client_secret
```

These are set in the CI/CD pipeline or a local `.env` file (which is `.gitignore`d). The `env!()` macro reads them at compile time.

**Alternative for development:** Use `option_env!()` with fallback to a config file, so the app can build without OAuth credentials (connectors just show "Not available — build without OAuth credentials").
