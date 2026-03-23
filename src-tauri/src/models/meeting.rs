use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub platform: String,
    pub raw_transcript: Option<String>,
    pub ai_summary: Option<String>,
    pub summary: Option<String>,       // alias for ai_summary, set by serde
    pub decisions: Option<String>,
    pub health_score: Option<i32>,
    pub health_breakdown: Option<String>,
    pub attendees: Option<String>,
    pub duration_minutes: Option<i32>,
    pub meeting_at: Option<String>,
    pub ingested_at: String,
    pub created_at: String,            // alias for ingested_at
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMeetingInput {
    pub project_id: String,
    pub title: String,
    pub platform: String,
    pub raw_transcript: String,
    pub attendees: Option<String>,
    pub duration_minutes: Option<i32>,
    pub meeting_at: Option<String>,
}
