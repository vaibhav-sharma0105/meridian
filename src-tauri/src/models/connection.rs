use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub provider: String,
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
    pub access_token: String,
    pub refresh_token: String,
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

#[derive(Debug, Deserialize)]
pub struct ImportApproval {
    pub pending_import_id: String,
    pub project_id: String,
    pub import_type: String,
}
