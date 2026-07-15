use serde::{Deserialize, Serialize};

// ─── Trigger Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Schedule,
    Event,
    Manual,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerType::Schedule => "schedule",
            TriggerType::Event => "event",
            TriggerType::Manual => "manual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "schedule" => Some(TriggerType::Schedule),
            "event" => Some(TriggerType::Event),
            "manual" => Some(TriggerType::Manual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggerConfig {
    // Schedule trigger
    pub cron: Option<String>,
    pub timezone: Option<String>,

    // Event trigger
    pub event_type: Option<String>,
    pub filter: Option<serde_json::Value>,
}

// ─── Context Configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    pub scope: Option<String>,              // "global" or "project"
    pub project_id: Option<String>,
    pub include_documents: Option<bool>,
    pub document_filter: Option<String>,    // regex pattern
    pub max_documents: Option<i32>,
    pub include_archived: Option<bool>,
    pub system_prompt: Option<String>,
    pub output_instructions: Option<String>,
    pub persona: Option<String>,
    pub max_tokens: Option<i32>,
    pub priority_order: Option<Vec<String>>,
}

// ─── Action Configuration ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Summarize,
    DraftMessage,
    CreateTasks,
    Analyze,
    Custom,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Summarize => "summarize",
            ActionType::DraftMessage => "draft_message",
            ActionType::CreateTasks => "create_tasks",
            ActionType::Analyze => "analyze",
            ActionType::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionConfig {
    pub action_type: Option<String>,
    pub format: Option<String>,         // "markdown", "json", "html", "plaintext"
    pub template: Option<String>,       // Handlebars template
    pub max_length: Option<i32>,
    pub channel: Option<String>,        // for draft_message: "email", "slack"
    pub recipient: Option<String>,
    pub has_side_effects: Option<bool>,
}

// ─── Approval Mode ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    Auto,
    Notify,
    ApproveFirst,
    ApproveAlways,
}

impl Default for ApprovalMode {
    fn default() -> Self {
        ApprovalMode::Notify
    }
}

impl ApprovalMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalMode::Auto => "auto",
            ApprovalMode::Notify => "notify",
            ApprovalMode::ApproveFirst => "approve_first",
            ApprovalMode::ApproveAlways => "approve_always",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(ApprovalMode::Auto),
            "notify" => Some(ApprovalMode::Notify),
            "approve_first" => Some(ApprovalMode::ApproveFirst),
            "approve_always" => Some(ApprovalMode::ApproveAlways),
            _ => None,
        }
    }
}

// ─── Skill ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<String>,     // JSON
    pub context_config: Option<String>,     // JSON
    pub action_config: Option<String>,      // JSON
    pub approval_mode: String,
    pub enabled: bool,
    pub shared: bool,
    pub owner_id: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<String>,               // JSON array
    pub next_run_at: Option<String>,
    pub cloned_from_id: Option<String>,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Skill {
    pub fn get_trigger_config(&self) -> Option<TriggerConfig> {
        self.trigger_config.as_ref().and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn get_context_config(&self) -> Option<ContextConfig> {
        self.context_config.as_ref().and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn get_action_config(&self) -> Option<ActionConfig> {
        self.action_config.as_ref().and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn get_tags(&self) -> Vec<String> {
        self.tags.as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSkillInput {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<TriggerConfig>,
    pub context_config: Option<ContextConfig>,
    pub action_config: Option<ActionConfig>,
    pub approval_mode: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub shared: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSkillInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<TriggerConfig>,
    pub context_config: Option<ContextConfig>,
    pub action_config: Option<ActionConfig>,
    pub approval_mode: Option<String>,
    pub enabled: Option<bool>,
    pub shared: Option<bool>,
    pub category: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SkillFilters {
    pub enabled: Option<bool>,
    pub shared: Option<bool>,
    pub category: Option<String>,
    pub trigger_type: Option<String>,
    pub search: Option<String>,
}

// ─── Skill Run ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    PartialFailure,
    Cancelled,
    ApprovalPending,
}

impl SkillRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillRunStatus::Pending => "pending",
            SkillRunStatus::Running => "running",
            SkillRunStatus::Completed => "completed",
            SkillRunStatus::Failed => "failed",
            SkillRunStatus::PartialFailure => "partial_failure",
            SkillRunStatus::Cancelled => "cancelled",
            SkillRunStatus::ApprovalPending => "approval_pending",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(SkillRunStatus::Pending),
            "running" => Some(SkillRunStatus::Running),
            "completed" => Some(SkillRunStatus::Completed),
            "failed" => Some(SkillRunStatus::Failed),
            "partial_failure" => Some(SkillRunStatus::PartialFailure),
            "cancelled" => Some(SkillRunStatus::Cancelled),
            "approval_pending" => Some(SkillRunStatus::ApprovalPending),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRun {
    pub id: String,
    pub skill_id: String,
    pub status: String,
    pub trigger_type: String,
    pub trigger_context: Option<String>,    // JSON
    pub output: Option<String>,
    pub error: Option<String>,
    pub pending_changes: Option<String>,    // JSON
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub approval_decision: Option<String>,
    pub approval_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSkillRunInput {
    pub skill_id: String,
    pub trigger_type: String,
    pub trigger_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStats {
    pub total_runs: i64,
    pub completed_runs: i64,
    pub failed_runs: i64,
    pub success_rate: f64,
    pub avg_duration_ms: Option<f64>,
    pub last_run_at: Option<String>,
}
