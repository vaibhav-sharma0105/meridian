use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftMessage {
    pub id: String,
    pub task_id: Option<String>,
    pub channel: String,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub ai_signature: bool,
    pub status: String,
    pub sensitive_warnings: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDraftInput {
    pub task_id: Option<String>,
    pub channel: String,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub ai_signature: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDraftInput {
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub ai_signature: Option<bool>,
    pub sensitive_warnings: Option<String>,
    pub status: Option<String>,
}
