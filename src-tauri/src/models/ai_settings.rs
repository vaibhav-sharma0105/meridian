use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub id: String,
    pub label: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model_id: Option<String>,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub embedding_provider: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AiSettingsInput {
    pub id: Option<String>,
    pub label: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_id: Option<String>,
    pub ollama_base_url: Option<String>,
    pub ollama_model: Option<String>,
    pub embedding_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub output_format: String,
    pub is_default: bool,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNotification {
    pub id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub task_id: Option<String>,
    pub project_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub project_id: Option<String>,
    pub meeting_id: Option<String>,
    pub role: String,
    pub content: String,
    pub template_id: Option<String>,
    pub created_at: String,
}
