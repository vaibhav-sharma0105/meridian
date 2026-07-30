use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(RiskLevel::Low),
            "medium" => Some(RiskLevel::Medium),
            "high" => Some(RiskLevel::High),
            "critical" => Some(RiskLevel::Critical),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyMode {
    Manual,
    Supervised,
    Autonomous,
}

impl AutonomyMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "manual" => Some(AutonomyMode::Manual),
            "supervised" => Some(AutonomyMode::Supervised),
            "autonomous" => Some(AutonomyMode::Autonomous),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AutonomyMode::Manual => "manual",
            AutonomyMode::Supervised => "supervised",
            AutonomyMode::Autonomous => "autonomous",
        }
    }
}

impl Default for AutonomyMode {
    fn default() -> Self {
        AutonomyMode::Supervised
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomySource {
    Global,
    Integration,
    Skill,
}

impl AutonomySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            AutonomySource::Global => "global",
            AutonomySource::Integration => "integration",
            AutonomySource::Skill => "skill",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Archived,
    Executed,
}

impl ApprovalStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(ApprovalStatus::Pending),
            "approved" => Some(ApprovalStatus::Approved),
            "rejected" => Some(ApprovalStatus::Rejected),
            "archived" => Some(ApprovalStatus::Archived),
            "executed" => Some(ApprovalStatus::Executed),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            ApprovalStatus::Archived => "archived",
            ApprovalStatus::Executed => "executed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub action_type: String,
    pub action_config: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub risk_level: String,
    pub autonomy_mode: String,
    pub context: Option<String>,
    pub timeout_at: Option<String>,
    pub status: String,
    pub resolved_by: Option<String>,
    pub resolution_reason: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    pub id: String,
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub undoable: bool,
    pub undo_action_id: Option<String>,
    pub audit_log_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceMetrics {
    pub date: String,
    pub metric_type: String,
    pub breakdown_key: Option<String>,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAdjustment {
    pub id: String,
    pub adjustment_type: String,
    pub target_type: String,
    pub target_id: String,
    pub risk_delta: i32,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePendingApprovalInput {
    pub action_type: String,
    pub action_config: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub risk_level: RiskLevel,
    pub autonomy_mode: AutonomyMode,
    pub context: Option<String>,
    pub timeout_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub requires_approval: bool,
    pub risk_level: RiskLevel,
    pub autonomy_mode: AutonomyMode,
    pub autonomy_source: AutonomySource,
    pub reason: String,
}
