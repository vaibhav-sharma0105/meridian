use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthBreakdown {
    pub had_agenda: bool,
    pub agenda_score: i32,
    pub decisions_score: i32,
    pub tasks_per_attendee_score: i32,
    pub follow_through_score: i32,
    pub duration_score: i32,
    pub total: i32,
}

pub fn calculate_health_score(
    had_agenda: bool,
    decisions_count: i32,
    tasks_count: i32,
    attendees_count: i32,
    follow_through_rate: f32,
    transcript_word_count: i32,
) -> HealthBreakdown {
    // Had agenda: +20 points
    let agenda_score = if had_agenda { 20 } else { 0 };

    // Decisions per topic ratio: up to +25 points
    let decisions_score = if attendees_count > 0 {
        let ratio = decisions_count as f32 / attendees_count.max(1) as f32;
        (ratio * 25.0).min(25.0) as i32
    } else {
        0
    };

    // Tasks per attendee: up to +20 points (healthy = 1-3, penalty for 0 or >5)
    let tasks_per_attendee_score = if attendees_count > 0 {
        let tpa = tasks_count as f32 / attendees_count as f32;
        if tpa == 0.0 {
            0
        } else if tpa <= 3.0 {
            20
        } else if tpa <= 5.0 {
            10
        } else {
            5
        }
    } else {
        if tasks_count > 0 { 10 } else { 0 }
    };

    // Follow-through: up to +25 points
    let follow_through_score = (follow_through_rate * 25.0) as i32;

    // Duration efficiency: up to +10 points
    // Estimated from transcript length: 100-500 words = ideal short meeting
    let duration_score = if transcript_word_count < 50 {
        0
    } else if transcript_word_count < 200 {
        5
    } else if transcript_word_count < 1000 {
        10
    } else if transcript_word_count < 3000 {
        8
    } else {
        5
    };

    let total = (agenda_score + decisions_score + tasks_per_attendee_score + follow_through_score + duration_score).min(100);

    HealthBreakdown {
        had_agenda,
        agenda_score,
        decisions_score,
        tasks_per_attendee_score,
        follow_through_score,
        duration_score,
        total,
    }
}

pub fn health_score_color(score: i32) -> &'static str {
    match score {
        0..=40 => "red",
        41..=70 => "amber",
        _ => "green",
    }
}

pub fn detect_agenda(transcript: &str) -> bool {
    let lower = transcript.to_lowercase();
    let keywords = ["agenda", "today we'll cover", "today we will cover", "on the agenda", "meeting agenda", "let's start with"];
    keywords.iter().any(|kw| lower.contains(kw))
}

pub fn count_words(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}
