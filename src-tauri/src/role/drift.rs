use super::models::{RoleDriftAlert, RoleScores};

const DRIFT_THRESHOLD: f32 = 0.2;
const DRIFT_WINDOW_DAYS: i64 = 14;

pub fn detect_role_drift(
    current_scores: &RoleScores,
    historical_scores: &RoleScores,
    window_days: i64,
) -> Option<RoleDriftAlert> {
    if window_days < DRIFT_WINDOW_DAYS {
        return None;
    }

    let score_change = current_scores.difference(historical_scores);

    // Significant drift if any role score changed by > 0.2 over 2 weeks
    if score_change.max_delta() > DRIFT_THRESHOLD {
        let (previous_role, _) = historical_scores.highest();
        let (suggested_role, confidence) = current_scores.highest();

        // Only alert if the primary role changed
        if previous_role != suggested_role {
            return Some(RoleDriftAlert {
                previous_role: previous_role.to_string(),
                suggested_role: suggested_role.to_string(),
                confidence,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_with_stable_scores() {
        let current = RoleScores {
            tech_lead: 0.4,
            ic: 0.3,
            pm: 0.2,
            manager: 0.1,
        };
        let historical = RoleScores {
            tech_lead: 0.38,
            ic: 0.32,
            pm: 0.18,
            manager: 0.12,
        };

        let drift = detect_role_drift(&current, &historical, 14);
        assert!(drift.is_none());
    }

    #[test]
    fn test_drift_detected_with_role_change() {
        let current = RoleScores {
            tech_lead: 0.5,
            ic: 0.2,
            pm: 0.2,
            manager: 0.1,
        };
        let historical = RoleScores {
            tech_lead: 0.2,
            ic: 0.5,
            pm: 0.2,
            manager: 0.1,
        };

        let drift = detect_role_drift(&current, &historical, 14);
        assert!(drift.is_some());
        let alert = drift.unwrap();
        assert_eq!(alert.previous_role, "ic");
        assert_eq!(alert.suggested_role, "tech_lead");
    }

    #[test]
    fn test_no_drift_below_window() {
        let current = RoleScores {
            tech_lead: 0.5,
            ic: 0.2,
            pm: 0.2,
            manager: 0.1,
        };
        let historical = RoleScores {
            tech_lead: 0.2,
            ic: 0.5,
            pm: 0.2,
            manager: 0.1,
        };

        // Window too short
        let drift = detect_role_drift(&current, &historical, 7);
        assert!(drift.is_none());
    }

    #[test]
    fn test_no_drift_if_same_primary_role() {
        let current = RoleScores {
            tech_lead: 0.45,
            ic: 0.25,
            pm: 0.2,
            manager: 0.1,
        };
        let historical = RoleScores {
            tech_lead: 0.4,
            ic: 0.3,
            pm: 0.2,
            manager: 0.1,
        };

        // Same primary role, even if scores shifted
        let drift = detect_role_drift(&current, &historical, 14);
        assert!(drift.is_none());
    }
}
