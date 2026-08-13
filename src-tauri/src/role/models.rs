use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoleScores {
    pub tech_lead: f32,
    pub ic: f32,
    pub pm: f32,
    pub manager: f32,
}

impl RoleScores {
    pub fn normalize(&mut self) {
        let total = self.tech_lead + self.ic + self.pm + self.manager;
        if total > 0.0 {
            self.tech_lead /= total;
            self.ic /= total;
            self.pm /= total;
            self.manager /= total;
        }
    }

    pub fn highest(&self) -> (&'static str, f32) {
        let roles = [
            ("tech_lead", self.tech_lead),
            ("ic", self.ic),
            ("pm", self.pm),
            ("manager", self.manager),
        ];
        roles
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or(("ic", 0.0))
    }

    pub fn second_highest(&self) -> (&'static str, f32) {
        let mut roles = [
            ("tech_lead", self.tech_lead),
            ("ic", self.ic),
            ("pm", self.pm),
            ("manager", self.manager),
        ];
        roles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        roles.get(1).copied().unwrap_or(("ic", 0.0))
    }

    pub fn difference(&self, other: &RoleScores) -> RoleScores {
        RoleScores {
            tech_lead: (self.tech_lead - other.tech_lead).abs(),
            ic: (self.ic - other.ic).abs(),
            pm: (self.pm - other.pm).abs(),
            manager: (self.manager - other.manager).abs(),
        }
    }

    pub fn max_delta(&self) -> f32 {
        [self.tech_lead, self.ic, self.pm, self.manager]
            .into_iter()
            .fold(0.0f32, |acc, x| acc.max(x))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleClassification {
    pub primary: String,
    pub primary_confidence: f32,
    pub secondary: Option<String>,
    pub secondary_confidence: f32,
}

/// Internally tagged so the frontend receives a discriminated union
/// (`{"type":"Learning",...}`). Serde's default external tagging would emit
/// `{"Learning":{...}}`, which no TypeScript consumer here expects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InferenceStatus {
    Learning {
        message: String,
        progress: f32,
    },
    PendingConfirmation {
        inferred: String,
        confidence: f32,
    },
    Confirmed {
        role: String,
        secondary: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub inferred_role: Option<String>,
    pub secondary_role: Option<String>,
    pub custom_role_description: Option<String>,
    pub role_confirmed: bool,
    pub role_confirmed_at: Option<String>,
    pub role_scores: Option<RoleScores>,
    pub last_inference_at: Option<String>,
    pub productivity_patterns: Option<serde_json::Value>,
    pub productivity_tracking_enabled: bool,
    pub ai_context_days: i64,
    pub message_retention: String,
    pub archive_old_files: bool,
    pub archive_after_days: i64,
    pub display_name: Option<String>,
    pub user_email: Option<String>,
    pub user_aliases: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl UserProfile {
    /// Every name/handle that should count as "me" when matching against a
    /// task's free-text assignee string.
    pub fn identity_tokens(&self) -> Vec<String> {
        let mut tokens: Vec<String> = Vec::new();
        for candidate in [self.display_name.as_ref(), self.user_email.as_ref()]
            .into_iter()
            .flatten()
        {
            let t = candidate.trim().to_lowercase();
            if !t.is_empty() {
                tokens.push(t);
            }
        }
        for alias in &self.user_aliases {
            let t = alias.trim().to_lowercase();
            if !t.is_empty() {
                tokens.push(t);
            }
        }
        tokens.sort();
        tokens.dedup();
        tokens
    }

    /// True when the profile carries enough information to tell "my items"
    /// apart from everyone else's. Role-based ordering falls back to
    /// severity-only ordering when this is false.
    pub fn has_identity(&self) -> bool {
        !self.identity_tokens().is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDriftAlert {
    pub previous_role: String,
    pub suggested_role: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleObservation {
    pub signal: String,
    pub count: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum RoleSignal {
    CreatesTasksForOthers,
    ReceivesAssignments,
    RunsMeetings,
    AttendsMeetings,
    ReviewsPrs,
    AuthorsPrs,
    ViewsRoadmap,
    WorksOnBugs,
}

impl RoleSignal {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleSignal::CreatesTasksForOthers => "creates_tasks_for_others",
            RoleSignal::ReceivesAssignments => "receives_assignments",
            RoleSignal::RunsMeetings => "runs_meetings",
            RoleSignal::AttendsMeetings => "attends_meetings",
            RoleSignal::ReviewsPrs => "reviews_prs",
            RoleSignal::AuthorsPrs => "authors_prs",
            RoleSignal::ViewsRoadmap => "views_roadmap",
            RoleSignal::WorksOnBugs => "works_on_bugs",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "creates_tasks_for_others" => Some(RoleSignal::CreatesTasksForOthers),
            "receives_assignments" => Some(RoleSignal::ReceivesAssignments),
            "runs_meetings" => Some(RoleSignal::RunsMeetings),
            "attends_meetings" => Some(RoleSignal::AttendsMeetings),
            "reviews_prs" => Some(RoleSignal::ReviewsPrs),
            "authors_prs" => Some(RoleSignal::AuthorsPrs),
            "views_roadmap" => Some(RoleSignal::ViewsRoadmap),
            "works_on_bugs" => Some(RoleSignal::WorksOnBugs),
            _ => None,
        }
    }
}
