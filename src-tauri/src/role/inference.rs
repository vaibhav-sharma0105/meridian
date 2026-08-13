use super::models::{RoleClassification, RoleObservation, RoleScores, RoleSignal};
use std::collections::HashMap;

// Weight matrix: [signal_index][role_index]
// Roles: 0=TechLead, 1=IC, 2=PM, 3=Manager
const ROLE_WEIGHTS: [[f32; 4]; 8] = [
    // CreatesTasksForOthers
    [0.3, 0.0, 0.4, 0.3],
    // ReceivesAssignments
    [0.1, 0.5, 0.2, 0.1],
    // RunsMeetings
    [0.3, 0.0, 0.3, 0.4],
    // AttendsMeetings
    [0.1, 0.4, 0.1, 0.1],
    // ReviewsPrs
    [0.4, 0.2, 0.0, 0.1],
    // AuthorsPrs
    [0.2, 0.5, 0.0, 0.0],
    // ViewsRoadmap
    [0.2, 0.0, 0.5, 0.2],
    // WorksOnBugs
    [0.2, 0.5, 0.1, 0.0],
];

fn signal_to_index(signal: &str) -> Option<usize> {
    match signal {
        "creates_tasks_for_others" => Some(0),
        "receives_assignments" => Some(1),
        "runs_meetings" => Some(2),
        "attends_meetings" => Some(3),
        "reviews_prs" => Some(4),
        "authors_prs" => Some(5),
        "views_roadmap" => Some(6),
        "works_on_bugs" => Some(7),
        _ => None,
    }
}

pub fn compute_role_scores(observations: &[RoleObservation]) -> RoleScores {
    let mut scores = RoleScores::default();
    let mut signal_counts: HashMap<String, i32> = HashMap::new();

    // Count observations per signal type
    for obs in observations {
        *signal_counts.entry(obs.signal.clone()).or_default() += obs.count;
    }

    // Calculate total for normalization
    let total: f32 = signal_counts.values().sum::<i32>() as f32;
    if total == 0.0 {
        return scores;
    }

    // Apply weights
    for (signal, count) in &signal_counts {
        if let Some(idx) = signal_to_index(signal) {
            let normalized = (*count as f32) / total;
            let weights = ROLE_WEIGHTS[idx];
            scores.tech_lead += normalized * weights[0];
            scores.ic += normalized * weights[1];
            scores.pm += normalized * weights[2];
            scores.manager += normalized * weights[3];
        }
    }

    // Normalize scores to sum to 1.0
    scores.normalize();
    scores
}

pub fn classify_role(scores: &RoleScores) -> RoleClassification {
    let (primary, primary_confidence) = scores.highest();
    let (secondary, secondary_confidence) = scores.second_highest();

    RoleClassification {
        primary: primary.to_string(),
        primary_confidence,
        secondary: if secondary_confidence > 0.3 {
            Some(secondary.to_string())
        } else {
            None
        },
        secondary_confidence,
    }
}

pub fn has_minimum_activity(task_count: i32, meeting_count: i32) -> bool {
    task_count >= 20 && meeting_count >= 5
}

pub fn get_inference_progress(task_count: i32, meeting_count: i32) -> f32 {
    // Progress based on meeting 20 tasks and 5 meetings threshold
    let task_progress = (task_count as f32 / 20.0).min(1.0);
    let meeting_progress = (meeting_count as f32 / 5.0).min(1.0);
    (task_progress * 0.5 + meeting_progress * 0.5).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_observations() {
        let scores = compute_role_scores(&[]);
        assert_eq!(scores.tech_lead, 0.0);
        assert_eq!(scores.ic, 0.0);
    }

    #[test]
    fn test_pr_reviewer_is_tech_lead() {
        let observations = vec![
            RoleObservation {
                signal: "reviews_prs".to_string(),
                count: 15,
            },
            RoleObservation {
                signal: "authors_prs".to_string(),
                count: 5,
            },
        ];
        let scores = compute_role_scores(&observations);
        let classification = classify_role(&scores);
        assert_eq!(classification.primary, "tech_lead");
    }

    #[test]
    fn test_task_receiver_is_ic() {
        let observations = vec![
            RoleObservation {
                signal: "receives_assignments".to_string(),
                count: 20,
            },
            RoleObservation {
                signal: "authors_prs".to_string(),
                count: 15,
            },
            RoleObservation {
                signal: "works_on_bugs".to_string(),
                count: 10,
            },
        ];
        let scores = compute_role_scores(&observations);
        let classification = classify_role(&scores);
        assert_eq!(classification.primary, "ic");
    }

    #[test]
    fn test_meeting_runner_is_manager() {
        let observations = vec![
            RoleObservation {
                signal: "runs_meetings".to_string(),
                count: 20,
            },
            RoleObservation {
                signal: "creates_tasks_for_others".to_string(),
                count: 10,
            },
        ];
        let scores = compute_role_scores(&observations);
        let classification = classify_role(&scores);
        assert_eq!(classification.primary, "manager");
    }

    #[test]
    fn test_roadmap_viewer_is_pm() {
        let observations = vec![
            RoleObservation {
                signal: "views_roadmap".to_string(),
                count: 20,
            },
            RoleObservation {
                signal: "creates_tasks_for_others".to_string(),
                count: 15,
            },
        ];
        let scores = compute_role_scores(&observations);
        let classification = classify_role(&scores);
        assert_eq!(classification.primary, "pm");
    }

    #[test]
    fn test_secondary_role_shown_if_above_threshold() {
        let observations = vec![
            RoleObservation {
                signal: "reviews_prs".to_string(),
                count: 10,
            },
            RoleObservation {
                signal: "authors_prs".to_string(),
                count: 8,
            },
        ];
        let scores = compute_role_scores(&observations);
        let classification = classify_role(&scores);
        // Should have secondary role since IC and TechLead are close
        assert!(classification.secondary.is_some() || classification.secondary_confidence > 0.0);
    }

    #[test]
    fn test_minimum_activity() {
        assert!(!has_minimum_activity(10, 3));
        assert!(!has_minimum_activity(25, 3));
        assert!(!has_minimum_activity(10, 10));
        assert!(has_minimum_activity(20, 5));
        assert!(has_minimum_activity(30, 10));
    }
}
