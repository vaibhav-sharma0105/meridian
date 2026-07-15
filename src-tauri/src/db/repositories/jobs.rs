use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonJob {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub priority: i32,
    pub payload: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentPayload {
    pub document_id: String,
    pub project_id: String,
}

pub fn create_job(
    conn: &Connection,
    job_type: &str,
    payload: Option<&str>,
    priority: i32,
) -> Result<DaemonJob, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO daemon_jobs (id, job_type, status, priority, payload, scheduled_at, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?5)",
        params![id, job_type, priority, payload, now],
    )
    .map_err(|e| format!("Failed to create job: {}", e))?;

    get_job(conn, &id)?.ok_or_else(|| "Failed to retrieve created job".to_string())
}

pub fn create_job_scheduled(
    conn: &Connection,
    job_type: &str,
    payload: Option<&str>,
    priority: i32,
    scheduled_at: &str,
) -> Result<DaemonJob, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO daemon_jobs (id, job_type, status, priority, payload, scheduled_at, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6)",
        params![id, job_type, priority, payload, scheduled_at, now],
    )
    .map_err(|e| format!("Failed to create scheduled job: {}", e))?;

    get_job(conn, &id)?.ok_or_else(|| "Failed to retrieve created job".to_string())
}

pub fn get_pending_jobs_by_type(conn: &Connection, job_type: &str) -> Result<Vec<DaemonJob>, String> {
    get_jobs_by_type(conn, job_type, Some("pending"))
}

pub fn get_job(conn: &Connection, id: &str) -> Result<Option<DaemonJob>, String> {
    let result = conn.query_row(
        "SELECT id, job_type, status, priority, payload, result, error, attempts, max_attempts,
                scheduled_at, started_at, completed_at, created_at
         FROM daemon_jobs WHERE id = ?1",
        params![id],
        |row| {
            Ok(DaemonJob {
                id: row.get(0)?,
                job_type: row.get(1)?,
                status: row.get(2)?,
                priority: row.get(3)?,
                payload: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                attempts: row.get(7)?,
                max_attempts: row.get(8)?,
                scheduled_at: row.get(9)?,
                started_at: row.get(10)?,
                completed_at: row.get(11)?,
                created_at: row.get(12)?,
            })
        },
    );

    match result {
        Ok(job) => Ok(Some(job)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_pending_jobs(conn: &Connection, limit: i32) -> Result<Vec<DaemonJob>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, job_type, status, priority, payload, result, error, attempts, max_attempts,
                    scheduled_at, started_at, completed_at, created_at
             FROM daemon_jobs
             WHERE status = 'pending' AND scheduled_at <= datetime('now')
             ORDER BY priority DESC, scheduled_at ASC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let jobs = stmt
        .query_map(params![limit], |row| {
            Ok(DaemonJob {
                id: row.get(0)?,
                job_type: row.get(1)?,
                status: row.get(2)?,
                priority: row.get(3)?,
                payload: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                attempts: row.get(7)?,
                max_attempts: row.get(8)?,
                scheduled_at: row.get(9)?,
                started_at: row.get(10)?,
                completed_at: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(jobs)
}

pub fn get_jobs_by_type(
    conn: &Connection,
    job_type: &str,
    status: Option<&str>,
) -> Result<Vec<DaemonJob>, String> {
    let sql = if let Some(s) = status {
        format!(
            "SELECT id, job_type, status, priority, payload, result, error, attempts, max_attempts,
                    scheduled_at, started_at, completed_at, created_at
             FROM daemon_jobs
             WHERE job_type = ?1 AND status = '{}'
             ORDER BY created_at DESC",
            s
        )
    } else {
        "SELECT id, job_type, status, priority, payload, result, error, attempts, max_attempts,
                scheduled_at, started_at, completed_at, created_at
         FROM daemon_jobs
         WHERE job_type = ?1
         ORDER BY created_at DESC".to_string()
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let jobs = stmt
        .query_map(params![job_type], |row| {
            Ok(DaemonJob {
                id: row.get(0)?,
                job_type: row.get(1)?,
                status: row.get(2)?,
                priority: row.get(3)?,
                payload: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                attempts: row.get(7)?,
                max_attempts: row.get(8)?,
                scheduled_at: row.get(9)?,
                started_at: row.get(10)?,
                completed_at: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(jobs)
}

pub fn update_job_status(
    conn: &Connection,
    id: &str,
    status: &str,
    result: Option<&str>,
    error: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    let (started_at, completed_at) = match status {
        "running" => (Some(now.as_str()), None),
        "completed" | "failed" => (None, Some(now.as_str())),
        _ => (None, None),
    };

    if status == "running" {
        conn.execute(
            "UPDATE daemon_jobs SET status = ?1, started_at = ?2, attempts = attempts + 1 WHERE id = ?3",
            params![status, started_at, id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE daemon_jobs SET status = ?1, result = ?2, error = ?3, completed_at = ?4 WHERE id = ?5",
            params![status, result, error, completed_at, id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn queue_embed_document_job(
    conn: &Connection,
    document_id: &str,
    project_id: &str,
    priority: i32,
) -> Result<DaemonJob, String> {
    let payload = EmbedDocumentPayload {
        document_id: document_id.to_string(),
        project_id: project_id.to_string(),
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

    create_job(conn, "embed_document", Some(&payload_json), priority)
}

pub fn get_embedding_job_for_document(
    conn: &Connection,
    document_id: &str,
) -> Result<Option<DaemonJob>, String> {
    let result = conn.query_row(
        "SELECT id, job_type, status, priority, payload, result, error, attempts, max_attempts,
                scheduled_at, started_at, completed_at, created_at
         FROM daemon_jobs
         WHERE job_type = 'embed_document'
           AND payload LIKE ?1
           AND status IN ('pending', 'running')
         ORDER BY created_at DESC
         LIMIT 1",
        params![format!("%\"document_id\":\"{}\"", document_id)],
        |row| {
            Ok(DaemonJob {
                id: row.get(0)?,
                job_type: row.get(1)?,
                status: row.get(2)?,
                priority: row.get(3)?,
                payload: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                attempts: row.get(7)?,
                max_attempts: row.get(8)?,
                scheduled_at: row.get(9)?,
                started_at: row.get(10)?,
                completed_at: row.get(11)?,
                created_at: row.get(12)?,
            })
        },
    );

    match result {
        Ok(job) => Ok(Some(job)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn cleanup_old_jobs(conn: &Connection, days_old: i32) -> Result<usize, String> {
    let result = conn.execute(
        "DELETE FROM daemon_jobs
         WHERE status IN ('completed', 'failed')
           AND completed_at < datetime('now', ?1)",
        params![format!("-{} days", days_old)],
    );

    match result {
        Ok(count) => Ok(count),
        Err(e) => Err(e.to_string()),
    }
}
