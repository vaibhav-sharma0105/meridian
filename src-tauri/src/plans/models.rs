use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub complexity: String,
    pub reasoning: String,
    pub suggested_subtasks: Vec<String>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEvaluationResult {
    pub complexity: String,
    pub reasoning: String,
    pub suggested_subtasks: Vec<String>,
}

impl Default for TaskPlan {
    fn default() -> Self {
        Self {
            complexity: "simple".to_string(),
            reasoning: String::new(),
            suggested_subtasks: vec![],
            suggested_action: None,
        }
    }
}
