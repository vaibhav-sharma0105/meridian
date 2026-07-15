use crate::daemon::jobs::{init_skill_jobs, process_job_sync, JobContext, JobResult};
use crate::db::repositories::jobs as jobs_repo;
use crate::vectors::qdrant::QdrantClient;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL_MS: u64 = 2000;
const MAX_JOBS_PER_POLL: i32 = 5;

pub struct WorkerConfig {
    pub model_dir: PathBuf,
    pub qdrant_url: Option<String>,
    pub embedding_provider: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
}

pub struct Worker {
    config: WorkerConfig,
    running: Arc<AtomicBool>,
    jobs_processed: Arc<std::sync::atomic::AtomicU64>,
}

impl Worker {
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            jobs_processed: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn jobs_processed(&self) -> u64 {
        self.jobs_processed.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub async fn run(&self, conn: &Connection) {
        self.running.store(true, Ordering::SeqCst);

        // Initialize skill polling jobs on startup
        init_skill_jobs(conn);

        let qdrant = QdrantClient::new(self.config.qdrant_url.as_deref());

        let ctx = JobContext {
            model_dir: self.config.model_dir.clone(),
            qdrant,
            embedding_provider: self.config.embedding_provider.clone(),
            ollama_base_url: self.config.ollama_base_url.clone(),
            ollama_model: self.config.ollama_model.clone(),
        };

        while self.running.load(Ordering::SeqCst) {
            match self.poll_and_process(conn, &ctx) {
                Ok(processed) => {
                    if processed == 0 {
                        tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
                    }
                }
                Err(e) => {
                    eprintln!("Worker error: {}", e);
                    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS * 2)).await;
                }
            }
        }
    }

    fn poll_and_process(&self, conn: &Connection, ctx: &JobContext) -> Result<usize, String> {
        let jobs = jobs_repo::get_pending_jobs(conn, MAX_JOBS_PER_POLL)?;

        for job in &jobs {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            let result = process_job_sync(conn, job, ctx);
            self.finalize_job(conn, &job.id, result)?;
            self.jobs_processed.fetch_add(1, Ordering::SeqCst);
        }

        Ok(jobs.len())
    }

    fn finalize_job(&self, conn: &Connection, job_id: &str, result: JobResult) -> Result<(), String> {
        let status = if result.success { "completed" } else { "failed" };
        let result_json = serde_json::to_string(&result).ok();

        jobs_repo::update_job_status(
            conn,
            job_id,
            status,
            result_json.as_deref(),
            result.error.as_deref(),
        )
    }
}

pub struct InProcessWorker {
    worker: Arc<Worker>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl InProcessWorker {
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            worker: Arc::new(Worker::new(config)),
            handle: None,
        }
    }

    pub fn start(&mut self, db_path: PathBuf) -> Result<(), String> {
        if self.worker.is_running() {
            return Ok(());
        }

        let worker = self.worker.clone();

        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            let conn = rusqlite::Connection::open(&db_path)
                .expect("Failed to open database connection");

            rt.block_on(worker.run(&conn));
        });

        self.handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.worker.stop();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.worker.is_running()
    }

    pub fn jobs_processed(&self) -> u64 {
        self.worker.jobs_processed()
    }
}

impl Drop for InProcessWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
