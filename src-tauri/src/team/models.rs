use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub source: String,       // 'manual', 'slack', 'google'
    pub source_id: Option<String>,
    pub role: String,         // 'admin', 'member'
    pub expertise: Option<Vec<String>>,
    pub workload_score: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamMemberInput {
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub source: String,
    pub source_id: Option<String>,
    pub role: Option<String>,
    pub expertise: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamMemberInput {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
    pub expertise: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneeSuggestion {
    pub member: TeamMember,
    pub score: f64,
    pub confidence: String,   // 'high', 'medium', 'low'
    pub reason: String,       // Primary reason for suggestion
    pub factors: AssigneeFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneeFactors {
    pub pattern_score: f64,
    pub workload_score: f64,
    pub expertise_score: f64,
    pub recency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneeWeights {
    pub pattern: f64,
    pub workload: f64,
    pub expertise: f64,
    pub recency: f64,
}

impl Default for AssigneeWeights {
    fn default() -> Self {
        Self {
            pattern: 0.35,
            workload: 0.25,
            expertise: 0.25,
            recency: 0.15,
        }
    }
}
