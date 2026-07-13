use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    SyncConnections,
    PruneAuditLog,
    CleanupBackups,
    RefreshEmbeddings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32,
    pub payload: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Job {
    pub fn new(job_type: JobType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_type,
            status: JobStatus::Pending,
            priority: 0,
            payload: None,
            result: None,
            error: None,
            attempts: 0,
            max_attempts: 3,
            scheduled_at: now,
            started_at: None,
            completed_at: None,
            created_at: now,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn scheduled_for(mut self, at: DateTime<Utc>) -> Self {
        self.scheduled_at = at;
        self
    }
}

pub async fn execute_job(job: &mut Job) -> Result<serde_json::Value, String> {
    job.status = JobStatus::Running;
    job.started_at = Some(Utc::now());
    job.attempts += 1;

    let result: Result<serde_json::Value, String> = match job.job_type {
        JobType::SyncConnections => {
            // TODO: Call into main app sync logic via IPC or shared library
            Ok(serde_json::json!({ "synced": true }))
        }
        JobType::PruneAuditLog => {
            // TODO: Call into main app audit log pruning
            Ok(serde_json::json!({ "pruned_count": 0 }))
        }
        JobType::CleanupBackups => {
            // TODO: Clean up old database backups
            Ok(serde_json::json!({ "cleaned_count": 0 }))
        }
        JobType::RefreshEmbeddings => {
            // TODO: Refresh document embeddings
            Ok(serde_json::json!({ "refreshed_count": 0 }))
        }
    };

    match result {
        Ok(data) => {
            job.status = JobStatus::Completed;
            job.completed_at = Some(Utc::now());
            job.result = Some(data.clone());
            Ok(data)
        }
        Err(e) => {
            job.error = Some(e.clone());
            if job.attempts >= job.max_attempts {
                job.status = JobStatus::Failed;
            } else {
                job.status = JobStatus::Pending;
            }
            job.completed_at = Some(Utc::now());
            Err(e)
        }
    }
}
