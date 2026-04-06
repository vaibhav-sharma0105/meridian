/// Gmail connector — stub implementation.
/// Full Gmail OAuth + email parsing will be added in a future sprint.
/// The sync engine calls into this module; all functions return safe no-ops.

pub struct ParsedZoomEmail {
    pub title: String,
    pub meeting_date: String,
    pub summary_text: String,
    pub external_meeting_id: Option<String>,
    pub source_email_id: String,
}

/// Stub: always returns an error (Gmail not yet implemented).
pub async fn start_oauth_flow() -> Result<(String, String, String, String), String> {
    Err("Gmail connector is not yet implemented. Please connect via Zoom for now.".to_string())
}

/// Stub: returns empty list — no Gmail emails to process.
pub async fn find_zoom_summary_emails(
    _access_token: &str,
    _since_epoch: i64,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

/// Stub: parse a Zoom summary email (not implemented).
pub async fn parse_zoom_summary_email(
    _access_token: &str,
    _message_id: &str,
) -> Result<ParsedZoomEmail, String> {
    Err("Gmail email parsing not yet implemented".to_string())
}
