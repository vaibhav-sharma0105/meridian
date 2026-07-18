use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

#[derive(Debug)]
pub struct CronParseResult {
    pub is_valid: bool,
    pub error: Option<String>,
    pub next_run: Option<String>,
    pub human_readable: Option<String>,
}

pub fn parse_cron(expression: &str) -> CronParseResult {
    match Schedule::from_str(expression) {
        Ok(schedule) => {
            let next = schedule.upcoming(Utc).next();
            CronParseResult {
                is_valid: true,
                error: None,
                next_run: next.map(|dt| dt.to_rfc3339()),
                human_readable: Some(describe_cron(expression)),
            }
        }
        Err(e) => CronParseResult {
            is_valid: false,
            error: Some(e.to_string()),
            next_run: None,
            human_readable: None,
        },
    }
}

pub fn validate_cron_expression(expression: &str) -> Result<(), String> {
    Schedule::from_str(expression)
        .map(|_| ())
        .map_err(|e| format!("Invalid cron expression: {}", e))
}

pub fn compute_next_run(expression: &str, timezone: Option<&str>) -> Result<String, String> {
    let schedule = Schedule::from_str(expression)
        .map_err(|e| format!("Invalid cron expression: {}", e))?;

    let next = if let Some(tz_str) = timezone {
        let tz: Tz = tz_str.parse().map_err(|_| format!("Invalid timezone: {}", tz_str))?;
        let now_in_tz = Utc::now().with_timezone(&tz);
        schedule
            .after(&now_in_tz)
            .next()
            .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
    } else {
        schedule.upcoming(Utc).next().map(|dt| dt.to_rfc3339())
    };

    next.ok_or_else(|| "No upcoming runs".to_string())
}

pub fn compute_next_run_after(
    expression: &str,
    after: &DateTime<Utc>,
    timezone: Option<&str>,
) -> Result<String, String> {
    let schedule = Schedule::from_str(expression)
        .map_err(|e| format!("Invalid cron expression: {}", e))?;

    let next = if let Some(tz_str) = timezone {
        let tz: Tz = tz_str.parse().map_err(|_| format!("Invalid timezone: {}", tz_str))?;
        let after_in_tz = after.with_timezone(&tz);
        schedule
            .after(&after_in_tz)
            .next()
            .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
    } else {
        schedule.after(after).next().map(|dt| dt.to_rfc3339())
    };

    next.ok_or_else(|| "No upcoming runs".to_string())
}

fn describe_cron(expression: &str) -> String {
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() < 5 {
        return "Custom schedule".to_string();
    }

    // Common patterns
    match expression.trim() {
        "0 * * * *" => "Every hour".to_string(),
        "0 0 * * *" => "Daily at midnight".to_string(),
        "0 9 * * *" => "Daily at 9:00 AM".to_string(),
        "0 17 * * *" => "Daily at 5:00 PM".to_string(),
        "0 9 * * 1" => "Every Monday at 9:00 AM".to_string(),
        "0 9 * * 5" => "Every Friday at 9:00 AM".to_string(),
        "0 9 * * 1-5" => "Weekdays at 9:00 AM".to_string(),
        "0 0 1 * *" => "Monthly on the 1st".to_string(),
        _ => format!("Cron: {}", expression),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: cron crate uses 7-field syntax (sec min hour day month weekday year)
    // We test with 7-field syntax here
    #[test]
    fn test_valid_cron() {
        let result = parse_cron("0 0 9 * * * *"); // Every day at 9:00 (7-field)
        assert!(result.is_valid, "Expected valid cron, got error: {:?}", result.error);
        assert!(result.next_run.is_some());
    }

    #[test]
    fn test_invalid_cron() {
        let result = parse_cron("invalid");
        assert!(!result.is_valid);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_validate_cron() {
        assert!(validate_cron_expression("0 0 9 * * * *").is_ok()); // 7-field
        assert!(validate_cron_expression("bad").is_err());
    }

    #[test]
    fn test_compute_next_run() {
        let next = compute_next_run("0 0 9 * * * *", None); // 7-field
        assert!(next.is_ok(), "Expected ok, got error: {:?}", next.err());
    }

    #[test]
    fn test_compute_next_run_with_timezone() {
        let next = compute_next_run("0 0 9 * * * *", Some("America/New_York")); // 7-field
        assert!(next.is_ok(), "Expected ok, got error: {:?}", next.err());
    }

    #[test]
    fn test_describe_common_patterns() {
        // describe_cron uses 5-field syntax for display only
        assert_eq!(describe_cron("0 9 * * *"), "Daily at 9:00 AM");
        assert_eq!(describe_cron("0 9 * * 1"), "Every Monday at 9:00 AM");
    }
}
