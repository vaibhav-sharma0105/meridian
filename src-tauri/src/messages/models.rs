use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    SkillResult,
    Digest,
    PinnedChat,
    IntegrationSync,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::SkillResult => "skill_result",
            MessageType::Digest => "digest",
            MessageType::PinnedChat => "pinned_chat",
            MessageType::IntegrationSync => "integration_sync",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "skill_result" => Some(MessageType::SkillResult),
            "digest" => Some(MessageType::Digest),
            "pinned_chat" => Some(MessageType::PinnedChat),
            "integration_sync" => Some(MessageType::IntegrationSync),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub project_id: Option<String>,
    pub message_type: String,
    pub title: String,
    pub content: Option<String>,
    pub source_id: Option<String>,
    pub source_type: Option<String>,
    pub auto_pinned: bool,
    pub pinned_reason: Option<String>,
    pub file_refs: Option<Vec<String>>,
    pub ai_visible_until: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageInput {
    pub project_id: Option<String>,
    pub message_type: String,
    pub title: String,
    pub content: Option<String>,
    pub source_id: Option<String>,
    pub source_type: Option<String>,
    pub auto_pinned: Option<bool>,
    pub pinned_reason: Option<String>,
    pub file_refs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageFilters {
    pub project_id: Option<String>,
    pub message_type: Option<String>,
    pub search: Option<String>,
    pub include_deleted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedMessages {
    pub messages: Vec<Message>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_messages: i64,
    pub total_files: i64,
    pub storage_bytes: u64,
    pub oldest_message: Option<String>,
    pub newest_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub soft_deleted: i64,
    pub hard_deleted: i64,
    pub files_removed: i64,
}
