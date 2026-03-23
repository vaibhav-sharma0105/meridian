use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalytics {
    pub avg_health_score: f64,
    pub tasks_closed_this_week: i64,
    pub tasks_closed_last_week: i64,
    pub follow_through_rate: f64,
    pub active_assignees: i64,
    pub total_open_tasks: i64,
    pub total_done_tasks: i64,
    pub overdue_tasks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyVelocity {
    pub week_start: String,
    pub tasks_closed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneeWorkload {
    pub assignee: String,
    pub week_start: String,
    pub task_count: i64,
}
