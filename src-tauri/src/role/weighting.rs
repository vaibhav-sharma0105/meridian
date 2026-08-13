use crate::suggestions::models::Suggestion;

/// Role weights for suggestion ordering, from the Phase 10 spec's Suggestion
/// Weighting table.
///
/// The spec's table names four rows; only two correspond to suggestion types
/// this codebase actually produces:
///
/// | Spec row | Producer |
/// |---|---|
/// | Task overdue | `overdue_task` ✓ |
/// | Meeting follow-up | `meeting_followup` ✓ |
/// | PR review needed | none — no job creates PR-review suggestions |
/// | Team velocity drop | none — no job creates velocity suggestions |
///
/// `stale_task` and `workflow_sequence` are produced but absent from the table.
/// Every type without an entry weighs 1.0, so unmapped suggestions keep their
/// relative order rather than being silently demoted.
fn weight_for(role: &str, suggestion_type: &str) -> f32 {
    match (role, suggestion_type) {
        ("tech_lead", "overdue_task") => 1.0,
        ("ic", "overdue_task") => 1.5,
        ("pm", "overdue_task") => 1.2,
        ("manager", "overdue_task") => 1.0,

        ("tech_lead", "meeting_followup") => 1.2,
        ("ic", "meeting_followup") => 1.0,
        ("pm", "meeting_followup") => 1.3,
        ("manager", "meeting_followup") => 1.5,

        _ => 1.0,
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 0,
        "warning" => 1,
        _ => 2,
    }
}

/// Reorders suggestions by role weight within each severity band.
///
/// Severity stays the primary key: weighting decides what matters most among
/// equally urgent suggestions, it never promotes an info item above a critical
/// one. Returns the input untouched for unrecognised roles.
pub fn weight_suggestions(mut suggestions: Vec<Suggestion>, role: &str) -> Vec<Suggestion> {
    if !matches!(role, "tech_lead" | "ic" | "pm" | "manager") {
        return suggestions;
    }

    suggestions.sort_by(|a, b| {
        let wa = weight_for(role, &a.suggestion_type);
        let wb = weight_for(role, &b.suggestion_type);

        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            // Higher weight first; NaN is impossible here since weights are literals.
            .then(
                wb.partial_cmp(&wa)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            // Newest first, matching the repository's ORDER BY.
            .then(b.created_at.cmp(&a.created_at))
    });

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion(id: &str, stype: &str, severity: &str) -> Suggestion {
        Suggestion {
            id: id.to_string(),
            suggestion_type: stype.to_string(),
            title: format!("{} suggestion", stype),
            description: None,
            reasoning: None,
            action_config: None,
            severity: severity.to_string(),
            status: "pending".to_string(),
            project_id: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            acted_at: None,
        }
    }

    #[test]
    fn test_manager_prioritizes_meeting_followup_over_overdue() {
        let items = vec![
            suggestion("a", "overdue_task", "warning"),
            suggestion("b", "meeting_followup", "warning"),
        ];
        let out = weight_suggestions(items, "manager");
        assert_eq!(out[0].suggestion_type, "meeting_followup");
    }

    #[test]
    fn test_ic_prioritizes_overdue_over_meeting_followup() {
        let items = vec![
            suggestion("a", "meeting_followup", "warning"),
            suggestion("b", "overdue_task", "warning"),
        ];
        let out = weight_suggestions(items, "ic");
        assert_eq!(out[0].suggestion_type, "overdue_task");
    }

    #[test]
    fn test_severity_outranks_weighting() {
        // IC weights overdue_task above meeting_followup, but a critical
        // follow-up must still come first.
        let items = vec![
            suggestion("a", "overdue_task", "warning"),
            suggestion("b", "meeting_followup", "critical"),
        ];
        let out = weight_suggestions(items, "ic");
        assert_eq!(out[0].severity, "critical");
    }

    #[test]
    fn test_unmapped_types_weigh_one_and_keep_order() {
        let items = vec![
            suggestion("a", "stale_task", "warning"),
            suggestion("b", "workflow_sequence", "warning"),
        ];
        let out = weight_suggestions(items, "tech_lead");
        assert_eq!(out[0].id, "a");
        assert_eq!(out[1].id, "b");
    }

    #[test]
    fn test_unknown_role_is_a_noop() {
        let items = vec![
            suggestion("a", "overdue_task", "warning"),
            suggestion("b", "meeting_followup", "warning"),
        ];
        let out = weight_suggestions(items, "");
        assert_eq!(out[0].id, "a");
        assert_eq!(out[1].id, "b");
    }

    #[test]
    fn test_pm_and_tech_lead_differ() {
        let items = vec![
            suggestion("a", "overdue_task", "warning"),
            suggestion("b", "meeting_followup", "warning"),
        ];
        // PM: meeting_followup 1.3 > overdue 1.2
        let pm = weight_suggestions(items.clone(), "pm");
        assert_eq!(pm[0].suggestion_type, "meeting_followup");
        // Tech Lead: meeting_followup 1.2 > overdue 1.0
        let tl = weight_suggestions(items, "tech_lead");
        assert_eq!(tl[0].suggestion_type, "meeting_followup");
    }
}
