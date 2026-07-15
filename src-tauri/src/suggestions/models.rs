use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub title: String,
    pub description: Option<String>,
    pub reasoning: Option<String>,
    pub action_config: Option<String>,
    pub severity: String,
    pub status: String,
    pub project_id: Option<String>,
    pub created_at: String,
    pub acted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSuggestionInput {
    pub suggestion_type: String,
    pub title: String,
    pub description: Option<String>,
    pub reasoning: Option<String>,
    pub action_config: Option<String>,
    pub severity: Option<String>,
    pub project_id: Option<String>,
}
