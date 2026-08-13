use chrono::{Duration, Utc};
use rusqlite::Connection;

/// Counts backing a digest. All figures come from local tables — a digest never
/// calls out to an AI provider, so it works offline and costs nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestStats {
    pub completed: i64,
    pub created: i64,
    pub overdue: i64,
    pub meetings: i64,
}

impl DigestStats {
    /// A digest with nothing in it is noise; the job skips those.
    pub fn is_empty(&self) -> bool {
        self.completed == 0 && self.created == 0 && self.overdue == 0 && self.meetings == 0
    }
}

/// `period` is "daily" or "weekly" and controls the look-back window.
pub fn collect_stats(conn: &Connection, period: &str) -> Result<DigestStats, String> {
    let days = if period == "weekly" { 7 } else { 1 };
    let since = (Utc::now() - Duration::days(days)).to_rfc3339();
    let now = Utc::now().to_rfc3339();

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE status = 'done' AND updated_at >= ?1 AND archived_at IS NULL",
            rusqlite::params![since],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count completed tasks: {}", e))?;

    let created: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE created_at >= ?1 AND archived_at IS NULL",
            rusqlite::params![since],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count created tasks: {}", e))?;

    let overdue: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE due_date IS NOT NULL AND due_date < ?1
               AND status != 'done' AND archived_at IS NULL",
            rusqlite::params![now],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count overdue tasks: {}", e))?;

    let meetings: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM meetings
             WHERE meeting_at >= ?1 AND archived_at IS NULL",
            rusqlite::params![since],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count meetings: {}", e))?;

    Ok(DigestStats {
        completed,
        created,
        overdue,
        meetings,
    })
}

pub fn render_markdown(stats: &DigestStats, period: &str) -> String {
    let window = if period == "weekly" {
        "this week"
    } else {
        "today"
    };
    format!(
        "## {} digest\n\n\
         - **{}** task(s) completed {}\n\
         - **{}** task(s) created {}\n\
         - **{}** meeting(s) {}\n\
         - **{}** task(s) currently overdue\n",
        if period == "weekly" { "Weekly" } else { "Daily" },
        stats.completed,
        window,
        stats.created,
        window,
        stats.meetings,
        window,
        stats.overdue,
    )
}

pub fn title_for(period: &str) -> String {
    let date = Utc::now().format("%b %d, %Y");
    if period == "weekly" {
        format!("Weekly digest — {}", date)
    } else {
        format!("Daily digest — {}", date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stats_detected() {
        let stats = DigestStats {
            completed: 0,
            created: 0,
            overdue: 0,
            meetings: 0,
        };
        assert!(stats.is_empty());
    }

    #[test]
    fn test_overdue_alone_is_not_empty() {
        // An overdue backlog is worth a digest even on a day with no activity.
        let stats = DigestStats {
            completed: 0,
            created: 0,
            overdue: 3,
            meetings: 0,
        };
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_render_includes_counts_and_period() {
        let stats = DigestStats {
            completed: 2,
            created: 5,
            overdue: 1,
            meetings: 3,
        };
        let daily = render_markdown(&stats, "daily");
        assert!(daily.contains("Daily digest"));
        assert!(daily.contains("**2** task(s) completed today"));
        assert!(daily.contains("**1** task(s) currently overdue"));

        let weekly = render_markdown(&stats, "weekly");
        assert!(weekly.contains("Weekly digest"));
        assert!(weekly.contains("this week"));
    }

    #[test]
    fn test_title_reflects_period() {
        assert!(title_for("weekly").starts_with("Weekly digest — "));
        assert!(title_for("daily").starts_with("Daily digest — "));
    }
}

#[cfg(test)]
mod weekly_tests {
    use super::*;

    #[test]
    fn test_weekly_window_differs_from_daily() {
        // Guards the scheduling split: both periods must render distinctly, or
        // a weekly digest is indistinguishable from a daily one in the UI.
        let stats = DigestStats {
            completed: 1,
            created: 1,
            overdue: 0,
            meetings: 0,
        };
        assert_ne!(
            render_markdown(&stats, "daily"),
            render_markdown(&stats, "weekly")
        );
        assert_ne!(title_for("daily"), title_for("weekly"));
    }
}
