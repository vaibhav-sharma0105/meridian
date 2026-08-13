use super::models::{Confidence, ProductivityPatterns, TaskCategory, TimeSuggestion};
use super::patterns::{get_effective_peak_hours, DEFAULT_PEAK_HOURS, MINIMUM_COMPLETIONS};

pub fn suggest_task_time(
    patterns: &ProductivityPatterns,
    task_category: TaskCategory,
) -> Option<TimeSuggestion> {
    let category_str = task_category.as_str();
    let peak_hours = get_effective_peak_hours(patterns, category_str);

    if peak_hours.is_empty() {
        return None;
    }

    let suggested_hour = peak_hours[0];
    let confidence = if patterns.total_completions >= MINIMUM_COMPLETIONS {
        Confidence::High
    } else {
        Confidence::Default
    };

    let reason = if confidence == Confidence::High {
        format!(
            "You typically complete {} best around {}",
            task_category.display_name(),
            format_hour(suggested_hour)
        )
    } else {
        format!(
            "Research suggests {} is good for {}",
            format_hour(suggested_hour),
            task_category.display_name()
        )
    };

    Some(TimeSuggestion {
        suggested_hour,
        reason,
        confidence,
    })
}

pub fn suggest_meeting_batching(
    meeting_hours: &[u8],
    patterns: &ProductivityPatterns,
) -> Option<BatchingSuggestion> {
    if meeting_hours.len() < 3 {
        return None;
    }

    // Check if meetings are scattered (gaps > 2 hours)
    let mut sorted_hours = meeting_hours.to_vec();
    sorted_hours.sort();

    let mut scattered = false;
    for window in sorted_hours.windows(2) {
        if window[1] - window[0] > 2 {
            scattered = true;
            break;
        }
    }

    if !scattered {
        return None;
    }

    let suggested_block = patterns
        .peak_hours
        .get("meetings")
        .cloned()
        .unwrap_or_else(|| vec![10, 11, 14]);

    Some(BatchingSuggestion {
        message: "Consider batching your meetings to protect focus time".to_string(),
        suggested_block: Some(suggested_block),
        freed_hours: calculate_freed_hours(meeting_hours),
    })
}

fn format_hour(hour: u8) -> String {
    if hour == 0 {
        "12 AM".to_string()
    } else if hour < 12 {
        format!("{} AM", hour)
    } else if hour == 12 {
        "12 PM".to_string()
    } else {
        format!("{} PM", hour - 12)
    }
}

fn calculate_freed_hours(meeting_hours: &[u8]) -> u8 {
    if meeting_hours.is_empty() {
        return 0;
    }

    let mut sorted = meeting_hours.to_vec();
    sorted.sort();

    // Calculate gaps that could be freed
    let mut freed = 0u8;
    for window in sorted.windows(2) {
        let gap = window[1] - window[0];
        if gap > 1 && gap <= 2 {
            freed += gap - 1;
        }
    }
    freed
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchingSuggestion {
    pub message: String,
    pub suggested_block: Option<Vec<u8>>,
    pub freed_hours: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_patterns(total: u32) -> ProductivityPatterns {
        let mut peak_hours = HashMap::new();
        peak_hours.insert("focus_work".to_string(), vec![9, 10, 11]);
        peak_hours.insert("meetings".to_string(), vec![14, 15, 10]);
        peak_hours.insert("quick_tasks".to_string(), vec![9, 14, 15]);

        ProductivityPatterns {
            task_completions_by_hour: HashMap::new(),
            peak_hours,
            low_productivity_hours: vec![12, 13],
            total_completions: total,
            last_aggregation: None,
            tracking_enabled: true,
        }
    }

    #[test]
    fn test_suggest_focus_work_time() {
        let patterns = make_patterns(100);
        let suggestion = suggest_task_time(&patterns, TaskCategory::FocusWork);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.suggested_hour, 9);
        assert_eq!(s.confidence, Confidence::High);
    }

    #[test]
    fn test_suggest_with_low_completions_uses_defaults() {
        let patterns = make_patterns(10);
        let suggestion = suggest_task_time(&patterns, TaskCategory::FocusWork);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.confidence, Confidence::Default);
    }

    #[test]
    fn test_format_hour() {
        assert_eq!(format_hour(0), "12 AM");
        assert_eq!(format_hour(9), "9 AM");
        assert_eq!(format_hour(12), "12 PM");
        assert_eq!(format_hour(14), "2 PM");
        assert_eq!(format_hour(23), "11 PM");
    }

    #[test]
    fn test_batching_suggestion_scattered() {
        let patterns = make_patterns(100);
        let meeting_hours = vec![9, 11, 14, 16];
        let suggestion = suggest_meeting_batching(&meeting_hours, &patterns);
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_batching_no_suggestion_compact() {
        let patterns = make_patterns(100);
        let meeting_hours = vec![10, 11, 12];
        let suggestion = suggest_meeting_batching(&meeting_hours, &patterns);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_batching_no_suggestion_few_meetings() {
        let patterns = make_patterns(100);
        let meeting_hours = vec![10, 14];
        let suggestion = suggest_meeting_batching(&meeting_hours, &patterns);
        assert!(suggestion.is_none());
    }
}
