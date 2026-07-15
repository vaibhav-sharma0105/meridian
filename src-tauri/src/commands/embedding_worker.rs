use crate::daemon::worker::{InProcessWorker, WorkerConfig};
use crate::db::connection::get_db_path;
use crate::db::repositories::{ai_settings, jobs as jobs_repo};
use crate::AppState;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct WorkerState {
    worker: Mutex<Option<InProcessWorker>>,
}

impl WorkerState {
    pub fn new() -> Self {
        Self {
            worker: Mutex::new(None),
        }
    }
}

impl Default for WorkerState {
    fn default() -> Self {
        Self::new()
    }
}

fn get_model_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Check multiple possible locations
    let candidates = vec![
        exe_dir.join("resources/models/all-MiniLM-L6-v2"),
        exe_dir.join("../Resources/resources/models/all-MiniLM-L6-v2"), // macOS bundle
        PathBuf::from("src-tauri/resources/models/all-MiniLM-L6-v2"), // dev mode
    ];

    for candidate in candidates {
        if candidate.join("model.onnx").exists() {
            return candidate;
        }
    }

    exe_dir.join("resources/models/all-MiniLM-L6-v2")
}

#[derive(Serialize)]
pub struct IndexingStatus {
    pub worker_running: bool,
    pub jobs_processed: u64,
    pub pending_jobs: usize,
    pub running_jobs: usize,
}

#[tauri::command]
pub async fn start_embedding_worker(
    app_state: State<'_, AppState>,
    worker_state: State<'_, WorkerState>,
) -> Result<(), String> {
    let mut worker_guard = worker_state.worker.lock().map_err(|e| e.to_string())?;

    if worker_guard.as_ref().map(|w| w.is_running()).unwrap_or(false) {
        return Ok(()); // Already running
    }

    // Get settings
    let (embedding_provider, ollama_base_url, ollama_model) = {
        let conn = app_state.db.lock().map_err(|e| e.to_string())?;
        let settings = ai_settings::get_active_settings(&conn)?;
        (
            settings.as_ref().map(|s| s.embedding_provider.clone()).unwrap_or_else(|| "bundled".to_string()),
            settings.as_ref().map(|s| s.ollama_base_url.clone()).unwrap_or_else(|| "http://localhost:11434".to_string()),
            settings.as_ref().map(|s| s.ollama_model.clone()).unwrap_or_else(|| "nomic-embed-text".to_string()),
        )
    };

    let config = WorkerConfig {
        model_dir: get_model_dir(),
        qdrant_url: None,
        embedding_provider,
        ollama_base_url,
        ollama_model,
    };

    let mut worker = InProcessWorker::new(config);
    worker.start(get_db_path())?;
    *worker_guard = Some(worker);

    Ok(())
}

#[tauri::command]
pub async fn stop_embedding_worker(
    worker_state: State<'_, WorkerState>,
) -> Result<(), String> {
    let mut worker_guard = worker_state.worker.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut worker) = *worker_guard {
        worker.stop();
    }
    *worker_guard = None;

    Ok(())
}

#[tauri::command]
pub async fn get_indexing_status(
    app_state: State<'_, AppState>,
    worker_state: State<'_, WorkerState>,
) -> Result<IndexingStatus, String> {
    let worker_guard = worker_state.worker.lock().map_err(|e| e.to_string())?;

    let (worker_running, jobs_processed) = match worker_guard.as_ref() {
        Some(w) => (w.is_running(), w.jobs_processed()),
        None => (false, 0),
    };

    let (pending_jobs, running_jobs) = {
        let conn = app_state.db.lock().map_err(|e| e.to_string())?;

        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daemon_jobs WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let running: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daemon_jobs WHERE status = 'running'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        (pending as usize, running as usize)
    };

    Ok(IndexingStatus {
        worker_running,
        jobs_processed,
        pending_jobs,
        running_jobs,
    })
}

#[tauri::command]
pub async fn process_pending_embeddings(
    app_state: State<'_, AppState>,
    worker_state: State<'_, WorkerState>,
) -> Result<IndexingStatus, String> {
    // Start worker if not running
    start_embedding_worker(app_state.clone(), worker_state.clone()).await?;

    // Return current status
    get_indexing_status(app_state, worker_state).await
}
