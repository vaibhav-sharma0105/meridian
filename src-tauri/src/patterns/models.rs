use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternObservation {
    pub id: String,
    pub observation_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub project_id: Option<String>,
    pub context_data: String,
    pub created_at: String,
    pub processed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateObservationInput {
    pub observation_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub project_id: Option<String>,
    pub context_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternModel {
    pub id: String,
    pub pattern_type: String,
    pub project_id: Option<String>,
    pub model_data: String,
    pub confidence: f64,
    pub observation_count: i64,
    pub last_updated: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertPatternModelInput {
    pub pattern_type: String,
    pub project_id: Option<String>,
    pub model_data: serde_json::Value,
    pub confidence: f64,
    pub observation_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSequence {
    pub trigger_action: String,
    pub follow_action: String,
    pub occurrence_count: i64,
    pub avg_delay_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSequenceModelData {
    pub sequences: Vec<WorkflowSequence>,
    pub negative_sequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityPattern {
    pub keyword: String,
    pub priority: String,
    pub occurrence_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneePattern {
    pub keyword: String,
    pub assignee: String,
    pub occurrence_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDefault {
    pub default_priority: Option<String>,
    pub default_assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaultsModelData {
    pub priority_patterns: Vec<PriorityPattern>,
    pub assignee_patterns: Vec<AssigneePattern>,
    pub project_defaults: std::collections::HashMap<String, ProjectDefault>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyleModelData {
    pub length_preference: String,
    pub formality_level: String,
    pub common_additions: Vec<(String, i64)>,
    pub common_removals: Vec<(String, i64)>,
    pub signature_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSuggestion {
    pub trigger_task_id: String,
    pub suggested_action: String,
    pub confidence: f64,
    pub sequence_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaults {
    pub suggested_priority: Option<String>,
    pub priority_confidence: f64,
    pub suggested_assignee: Option<String>,
    pub assignee_confidence: f64,
    pub source: String,
}
