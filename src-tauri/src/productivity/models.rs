use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductivityPatterns {
    pub task_completions_by_hour: HashMap<String, [u32; 24]>,
    pub peak_hours: HashMap<String, Vec<u8>>,
    pub low_productivity_hours: Vec<u8>,
    pub total_completions: u32,
    pub last_aggregation: Option<String>,
    pub tracking_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSuggestion {
    pub suggested_hour: u8,
    pub reason: String,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Confidence {
    High,
    Default,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityInsights {
    pub patterns: ProductivityPatterns,
    pub status: ProductivityStatus,
    pub storage_warning: Option<String>,
}

/// Internally tagged — see the note on `role::models::InferenceStatus`.
/// Without this, the unit variants would serialize as the bare strings
/// `"Ready"` / `"Disabled"` rather than `{"type":"Ready"}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProductivityStatus {
    Learning { completions_needed: u32 },
    Ready,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityExport {
    pub peak_hours: HashMap<String, Vec<u8>>,
    pub total_data_points: u32,
    pub tracking_since: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskCategory {
    FocusWork,
    Meetings,
    QuickTasks,
}

impl TaskCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskCategory::FocusWork => "focus_work",
            TaskCategory::Meetings => "meetings",
            TaskCategory::QuickTasks => "quick_tasks",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "focus_work" => Some(TaskCategory::FocusWork),
            "meetings" => Some(TaskCategory::Meetings),
            "quick_tasks" => Some(TaskCategory::QuickTasks),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TaskCategory::FocusWork => "focus work",
            TaskCategory::Meetings => "meetings",
            TaskCategory::QuickTasks => "quick tasks",
        }
    }
}

pub fn classify_task_category(
    title: &str,
    estimated_minutes: Option<i32>,
    source_type: Option<&str>,
) -> TaskCategory {
    // Check if it's meeting-related
    let title_lower = title.to_lowercase();
    if source_type == Some("meeting")
        || title_lower.contains("meeting")
        || title_lower.contains("standup")
        || title_lower.contains("sync")
        || title_lower.contains("1:1")
        || title_lower.contains("review meeting")
    {
        return TaskCategory::Meetings;
    }

    // Check duration for focus work vs quick tasks
    match estimated_minutes {
        Some(mins) if mins >= 60 => TaskCategory::FocusWork,
        Some(_) => TaskCategory::QuickTasks,
        None => {
            // Heuristic based on title complexity
            if title.len() > 50 || title_lower.contains("implement") || title_lower.contains("design") {
                TaskCategory::FocusWork
            } else {
                TaskCategory::QuickTasks
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivitySettings {
    pub tracking_enabled: bool,
    pub show_suggestions: bool,
    pub data_retention_days: u32,
}

impl Default for ProductivitySettings {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            show_suggestions: true,
            data_retention_days: 365,
        }
    }
}
