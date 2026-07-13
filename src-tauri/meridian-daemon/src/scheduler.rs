use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use crate::DaemonState;
use crate::jobs::{Job, JobType, JobStatus, execute_job};

#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub job_type: JobType,
    pub cron_expression: String,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub enabled: bool,
}

impl ScheduledJob {
    pub fn new(job_type: JobType, interval_minutes: i64) -> Self {
        Self {
            job_type,
            cron_expression: format!("every {} minutes", interval_minutes),
            last_run: None,
            next_run: Utc::now() + Duration::minutes(interval_minutes),
            enabled: true,
        }
    }

    pub fn update_next_run(&mut self, interval_minutes: i64) {
        self.last_run = Some(Utc::now());
        self.next_run = Utc::now() + Duration::minutes(interval_minutes);
    }

    pub fn is_missed(&self, max_missed_minutes: i64) -> bool {
        let now = Utc::now();
        if self.next_run < now {
            let missed_duration = now - self.next_run;
            missed_duration.num_minutes() <= max_missed_minutes
        } else {
            false
        }
    }

    pub fn with_last_run(mut self, last_run: Option<DateTime<Utc>>) -> Self {
        self.last_run = last_run;
        if let Some(lr) = last_run {
            // If we have a last run time, calculate next run from that
            let interval = self.get_interval_minutes();
            self.next_run = lr + Duration::minutes(interval);
        }
        self
    }

    fn get_interval_minutes(&self) -> i64 {
        // Parse interval from cron_expression (simple format: "every N minutes")
        self.cron_expression
            .strip_prefix("every ")
            .and_then(|s| s.strip_suffix(" minutes"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(60)
    }
}

pub struct Scheduler {
    state: Arc<RwLock<DaemonState>>,
    scheduled_jobs: HashMap<String, ScheduledJob>,
}

impl Scheduler {
    pub fn new(state: Arc<RwLock<DaemonState>>) -> Self {
        let mut scheduled_jobs = HashMap::new();

        // Default scheduled jobs
        scheduled_jobs.insert(
            "sync_connections".to_string(),
            ScheduledJob::new(JobType::SyncConnections, 15), // Every 15 minutes
        );
        scheduled_jobs.insert(
            "prune_audit_log".to_string(),
            ScheduledJob::new(JobType::PruneAuditLog, 60 * 24), // Daily
        );
        scheduled_jobs.insert(
            "cleanup_backups".to_string(),
            ScheduledJob::new(JobType::CleanupBackups, 60 * 24 * 7), // Weekly
        );

        Self {
            state,
            scheduled_jobs,
        }
    }

    pub async fn run(&mut self) {
        info!("Scheduler started with {} jobs", self.scheduled_jobs.len());

        // Check for missed jobs on startup (catch-up)
        self.run_missed_jobs().await;

        loop {
            // Check if we should stop
            {
                let s = self.state.read().await;
                if !s.running {
                    info!("Scheduler stopping");
                    break;
                }
            }

            // Check for due jobs
            let now = Utc::now();
            let mut jobs_to_run = Vec::new();

            for (name, scheduled) in &self.scheduled_jobs {
                if scheduled.enabled && scheduled.next_run <= now {
                    debug!("Job '{}' is due", name);
                    jobs_to_run.push(name.clone());
                }
            }

            // Run due jobs
            for name in jobs_to_run {
                if let Some(scheduled) = self.scheduled_jobs.get_mut(&name) {
                    let mut job = Job::new(scheduled.job_type.clone());
                    info!("Running scheduled job: {}", name);

                    match execute_job(&mut job).await {
                        Ok(_) => {
                            info!("Job '{}' completed successfully", name);
                            let mut s = self.state.write().await;
                            s.jobs_processed += 1;
                        }
                        Err(e) => {
                            warn!("Job '{}' failed: {}", name, e);
                            let mut s = self.state.write().await;
                            s.last_error = Some(format!("Job '{}' failed: {}", name, e));
                        }
                    }

                    // Update next run time based on job type
                    let interval = match scheduled.job_type {
                        JobType::SyncConnections => 15,
                        JobType::PruneAuditLog => 60 * 24,
                        JobType::CleanupBackups => 60 * 24 * 7,
                        JobType::RefreshEmbeddings => 60,
                    };
                    scheduled.update_next_run(interval);
                }
            }

            // Sleep for a bit before checking again
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    async fn run_missed_jobs(&mut self) {
        // Max time window to consider a job as "missed" (not stale)
        // Jobs older than this are considered stale and skipped
        const MAX_MISSED_HOURS: i64 = 24;

        let now = Utc::now();
        let mut missed_jobs = Vec::new();

        for (name, scheduled) in &self.scheduled_jobs {
            if scheduled.enabled && scheduled.next_run < now {
                let missed_duration = now - scheduled.next_run;
                if missed_duration.num_hours() <= MAX_MISSED_HOURS {
                    info!(
                        "Job '{}' was missed (due {} ago), will run catch-up",
                        name,
                        format_duration(missed_duration)
                    );
                    missed_jobs.push(name.clone());
                } else {
                    info!(
                        "Job '{}' is stale (due {} ago), skipping catch-up",
                        name,
                        format_duration(missed_duration)
                    );
                }
            }
        }

        // Run missed jobs
        for name in missed_jobs {
            if let Some(scheduled) = self.scheduled_jobs.get_mut(&name) {
                let mut job = Job::new(scheduled.job_type.clone());
                info!("Running catch-up job: {}", name);

                match execute_job(&mut job).await {
                    Ok(_) => {
                        info!("Catch-up job '{}' completed successfully", name);
                        let mut s = self.state.write().await;
                        s.jobs_processed += 1;
                    }
                    Err(e) => {
                        warn!("Catch-up job '{}' failed: {}", name, e);
                        let mut s = self.state.write().await;
                        s.last_error = Some(format!("Catch-up job '{}' failed: {}", name, e));
                    }
                }

                // Update next run time
                let interval = match scheduled.job_type {
                    JobType::SyncConnections => 15,
                    JobType::PruneAuditLog => 60 * 24,
                    JobType::CleanupBackups => 60 * 24 * 7,
                    JobType::RefreshEmbeddings => 60,
                };
                scheduled.update_next_run(interval);
            }
        }
    }

    pub fn enable_job(&mut self, name: &str, enabled: bool) {
        if let Some(job) = self.scheduled_jobs.get_mut(name) {
            job.enabled = enabled;
        }
    }

    pub fn get_status(&self) -> Vec<(String, ScheduledJob)> {
        self.scheduled_jobs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

fn format_duration(d: chrono::Duration) -> String {
    let hours = d.num_hours();
    let mins = d.num_minutes() % 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_job_creation() {
        let job = ScheduledJob::new(JobType::SyncConnections, 15);
        assert!(job.enabled);
        assert!(job.last_run.is_none());
        assert_eq!(job.cron_expression, "every 15 minutes");
    }

    #[test]
    fn test_scheduled_job_interval_parsing() {
        let job = ScheduledJob::new(JobType::PruneAuditLog, 1440);
        assert_eq!(job.get_interval_minutes(), 1440);
    }

    #[test]
    fn test_scheduled_job_is_missed() {
        let mut job = ScheduledJob::new(JobType::SyncConnections, 15);
        // Set next_run to 30 minutes ago
        job.next_run = Utc::now() - Duration::minutes(30);

        // Should be considered missed (within 24 hours)
        assert!(job.is_missed(60 * 24));

        // Set next_run to 25 hours ago
        job.next_run = Utc::now() - Duration::hours(25);
        // Should NOT be considered missed (outside max window)
        assert!(!job.is_missed(60 * 24));
    }

    #[test]
    fn test_scheduled_job_update_next_run() {
        let mut job = ScheduledJob::new(JobType::SyncConnections, 15);
        let before = job.next_run;

        job.update_next_run(15);

        assert!(job.last_run.is_some());
        assert!(job.next_run > before);
    }

    #[test]
    fn test_format_duration_minutes_only() {
        let d = Duration::minutes(45);
        assert_eq!(format_duration(d), "45m");
    }

    #[test]
    fn test_format_duration_hours_and_minutes() {
        let d = Duration::hours(2) + Duration::minutes(30);
        assert_eq!(format_duration(d), "2h 30m");
    }

    #[test]
    fn test_format_duration_exact_hours() {
        let d = Duration::hours(5);
        assert_eq!(format_duration(d), "5h 0m");
    }
}
