use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionItem {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub severity: String,
    pub category: String,
    pub reason_text: Option<String>,
    pub matched_skill_id: Option<String>,
    pub computed_at: String,
    pub dismissed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttentionFilters {
    pub severity: Option<String>,
    pub source_type: Option<String>,
    pub category: Option<String>,
    pub include_dismissed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionItemWithDetails {
    pub item: AttentionItem,
    pub title: String,
    pub subtitle: Option<String>,
    pub external_url: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
}
