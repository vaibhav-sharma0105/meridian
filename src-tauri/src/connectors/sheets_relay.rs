/// Google Sheets Relay connector.
///
/// Architecture: Google Workspace AI Studio watches Gmail for Zoom AI summary
/// emails and appends rows to a private Google Sheet. A tiny Apps Script web
/// app on that Sheet exposes the rows as JSON, protected by a user-chosen
/// secret key. Meridian polls that Apps Script URL every 15 minutes to pull
/// new rows and create PendingImport records.
///
/// Sheet column contract (order does not matter — matched by header name):
///   import_id      — unique identifier for deduplication (may carry the full JSON payload)
///   created_at     — ISO-8601 timestamp (used for incremental sync)
///   title          — meeting title (may carry the full JSON payload)
///   meeting_date   — date/time of the meeting
///   summary        — AI summary / email body text (may carry the full JSON payload)
///   action_items   — extracted action items (may carry the full JSON payload)
///   source_subject — original email subject; used as canonical meeting title
///
/// The Workspace automation may embed the full meeting JSON as the cell value
/// for several columns at once (import_id, title, summary, action_items all
/// carry the same blob). `extract_embedded_json` detects this and unpacks the
/// real field values so the rest of the pipeline sees clean data.

use crate::models::connection::PendingImport;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

// ─── Response / row types ─────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct SheetRow {
    pub import_id: Option<String>,
    pub created_at: Option<String>,
    pub title: Option<String>,
    pub meeting_date: Option<String>,
    pub summary: Option<String>,
    pub action_items: Option<String>,
    pub source_subject: Option<String>,
}

#[derive(Deserialize)]
struct SheetResponse {
    rows: Option<Vec<SheetRow>>,
    error: Option<String>,
    #[serde(default)]
    ok: bool,
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Scan SheetRow string fields for an embedded JSON payload produced by the
/// Workspace automation (several columns may all carry the same JSON blob).
/// Returns the parsed Value when any field is a JSON object that contains at
/// least one of the canonical meeting keys.
fn extract_embedded_json(row: &SheetRow) -> Option<serde_json::Value> {
    let candidates = [
        row.import_id.as_deref(),
        row.title.as_deref(),
        row.summary.as_deref(),
        row.action_items.as_deref(),
    ];
    for candidate in candidates.into_iter().flatten() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(candidate) {
            if v.is_object()
                && ["import_id", "title", "summary", "action_items"]
                    .iter()
                    .any(|k| v.get(k).is_some())
            {
                return Some(v);
            }
        }
    }
    None
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Test whether the Apps Script endpoint is reachable and the secret is valid.
/// Returns a human-readable success message or an error string.
pub async fn test_connection(script_url: &str, secret_key: &str) -> Result<String, String> {
    let url = format!(
        "{}?key={}&test=1",
        script_url,
        urlencoding::encode(secret_key)
    );

    let resp = Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Could not reach the Apps Script URL: {}", e))?;

    let status = resp.status();
    let body_text = resp
        .text()
        .await
        .unwrap_or_else(|_| "(no body)".to_string());

    if !status.is_success() {
        return Err(format!(
            "Apps Script returned HTTP {} — check the deployment URL",
            status
        ));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&body_text).map_err(|_| {
            "Apps Script returned non-JSON — make sure you deployed it as a Web App with \
             execution set to 'Me' and access set to 'Anyone'"
                .to_string()
        })?;

    if parsed.get("error").and_then(|e| e.as_str()) == Some("unauthorized") {
        return Err(
            "Secret key rejected — make sure the key in Meridian matches the one you set \
             with setSecretKey() in the Apps Script"
                .to_string(),
        );
    }

    if parsed.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        return Ok(
            "Connected! Meridian will start pulling Zoom summaries from your Sheet on the \
             next sync."
                .to_string(),
        );
    }

    Ok("Endpoint reachable. Ready to sync.".to_string())
}

/// Fetch all sheet rows with `created_at` newer than `since_ms` (epoch ms).
pub async fn fetch_new_rows(
    script_url: &str,
    secret_key: &str,
    since_ms: i64,
) -> Result<Vec<SheetRow>, String> {
    let url = format!(
        "{}?key={}&since={}",
        script_url,
        urlencoding::encode(secret_key),
        since_ms
    );

    let resp = Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("Sheets relay request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Apps Script returned HTTP {}",
            resp.status()
        ));
    }

    let body: SheetResponse = resp.json().await.map_err(|e| {
        format!(
            "Could not parse Apps Script response: {}. Check the deployment settings.",
            e
        )
    })?;

    if let Some(err) = &body.error {
        if err == "unauthorized" {
            return Err(
                "Invalid secret key — check that the key in Meridian matches the one set \
                 in the Apps Script properties"
                    .to_string(),
            );
        }
        return Err(format!("Apps Script reported an error: {}", err));
    }

    Ok(body.rows.unwrap_or_default())
}

/// Convert a sheet row into a PendingImport record ready to be upserted.
///
/// Handles the case where the Workspace automation embeds the full meeting JSON
/// as the cell value for multiple columns (import_id, title, summary, and
/// action_items all carry the same blob). When detected, real field values are
/// extracted from inside that blob. `source_subject` is always used as the
/// canonical meeting title.
pub fn row_to_pending_import(row: &SheetRow) -> PendingImport {
    // Detect and parse an embedded JSON payload if present.
    let embedded = extract_embedded_json(row);

    // Returns the value for `key` from the embedded JSON blob when available,
    // falling back to the raw SheetRow field.
    let from_embedded = |key: &str, field: &Option<String>| -> Option<String> {
        embedded
            .as_ref()
            .and_then(|v| v.get(key)?.as_str().map(str::to_string))
            .or_else(|| field.clone().filter(|s| !s.is_empty()))
    };

    // Dedup key: embedded JSON import_id → raw field → fresh UUID.
    let dedup_key = from_embedded("import_id", &row.import_id)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // source_subject is the canonical title for new rows produced by the
    // Workspace automation: "Meeting assets for <title> are ready!" → "<title>".
    // Falls back to the JSON-embedded title, then to the raw title field.
    let title = row
        .source_subject
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| {
            let s = s.strip_prefix("Meeting assets for ").unwrap_or(s);
            let s = s.strip_suffix(" are ready!").unwrap_or(s);
            s.trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .or_else(|| from_embedded("title", &row.title))
        .or_else(|| {
            row.source_subject.clone().filter(|s| !s.is_empty()).map(|s| {
                s.trim_start_matches("AI Companion Meeting Summary: ")
                    .trim_start_matches("Meeting Summary: ")
                    .trim_start_matches("Zoom AI Summary: ")
                    .to_string()
            })
        })
        .unwrap_or_else(|| "Zoom Meeting".to_string());

    // Summary and action items — extracted from embedded JSON when present.
    let summary_text = from_embedded("summary", &row.summary);
    let action_items_text = from_embedded("action_items", &row.action_items);

    // Combine summary + action items into one block for the AI pipeline.
    let summary_full = match (&summary_text, &action_items_text) {
        (Some(s), Some(a)) if !a.trim().is_empty() => {
            Some(format!("{}\n\nAction Items:\n{}", s.trim(), a.trim()))
        }
        (Some(s), _) if !s.trim().is_empty() => Some(s.trim().to_string()),
        (_, Some(a)) if !a.trim().is_empty() => {
            Some(format!("Action Items:\n{}", a.trim()))
        }
        _ => None,
    };

    let summary_preview = summary_full
        .as_ref()
        .map(|s| s.chars().take(500).collect::<String>());

    PendingImport {
        id: Uuid::new_v4().to_string(),
        provider: "gmail".to_string(), // displayed as "Gmail" in the UI
        external_meeting_id: None,
        title,
        meeting_date: row.meeting_date.clone().filter(|s| !s.is_empty()),
        duration_minutes: None,
        attendees: None,
        summary_preview,
        summary_full,
        transcript_available: false,
        transcript_content: None,
        zoom_join_url: None,
        source_email_id: Some(dedup_key), // dedup index
        status: "pending".to_string(),
        imported_meeting_id: None,
        project_id: None,
        created_at: Utc::now().to_rfc3339(),
    }
}
