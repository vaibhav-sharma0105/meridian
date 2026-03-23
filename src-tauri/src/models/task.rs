use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub meeting_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub assignee_confidence: String,
    pub assignee_source_quote: Option<String>,
    pub due_date: Option<String>,
    pub due_confidence: String,
    pub due_source_quote: Option<String>,
    pub status: String,
    pub priority: String,        // low, medium, high, critical
    pub confidence_score: Option<f64>,
    pub tags: String, // JSON array as string
    pub kanban_column: String,
    pub kanban_order: i64,
    pub is_duplicate: bool,
    pub duplicate_of_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub project_id: String,
    pub meeting_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub assignee_confidence: Option<String>,
    pub assignee_source_quote: Option<String>,
    pub due_date: Option<String>,
    pub due_confidence: Option<String>,
    pub due_source_quote: Option<String>,
    pub priority: Option<String>,
    pub confidence_score: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub kanban_column: Option<String>,
    pub notes: Option<String>,
    pub is_duplicate: Option<bool>,
    pub duplicate_of_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub assignee_confidence: Option<String>,
    pub due_date: Option<String>,
    pub due_confidence: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub kanban_column: Option<String>,
    pub kanban_order: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TaskFilters {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub search_query: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PartialTaskUpdate {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub kanban_column: Option<String>,
}
