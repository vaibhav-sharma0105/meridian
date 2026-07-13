use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error};

mod ipc;
mod jobs;
mod scheduler;

use ipc::IpcServer;
use scheduler::Scheduler;

#[derive(Debug, Clone)]
pub struct DaemonState {
    pub running: bool,
    pub jobs_processed: u64,
    pub last_error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self {
            running: true,
            jobs_processed: 0,
            last_error: None,
            started_at: chrono::Utc::now(),
        }
    }
}

fn get_socket_path() -> PathBuf {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meridian");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("daemon.sock")
}

fn get_pid_path() -> PathBuf {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meridian");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("daemon.pid")
}

fn write_pid_file() -> std::io::Result<()> {
    let pid = std::process::id();
    std::fs::write(get_pid_path(), pid.to_string())
}

fn cleanup_pid_file() {
    let _ = std::fs::remove_file(get_pid_path());
}

fn cleanup_socket() {
    let socket_path = get_socket_path();
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("meridian_daemon=info".parse()?)
        )
        .init();

    info!("Meridian daemon starting...");

    // Clean up stale socket if exists
    cleanup_socket();

    // Write PID file
    write_pid_file()?;

    // Initialize state
    let state = Arc::new(RwLock::new(DaemonState::default()));

    // Create channels for coordination
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // Start IPC server
    let socket_path = get_socket_path();
    let ipc_state = state.clone();
    let ipc_shutdown = shutdown_tx.clone();
    let _ipc_handle = tokio::spawn(async move {
        if let Err(e) = IpcServer::new(&socket_path, ipc_state, ipc_shutdown).run().await {
            error!("IPC server error: {}", e);
        }
    });

    // Start scheduler
    let scheduler_state = state.clone();
    let _scheduler_handle = tokio::spawn(async move {
        let mut scheduler = Scheduler::new(scheduler_state);
        scheduler.run().await;
    });

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            info!("Shutdown signal received");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received");
        }
    }

    // Cleanup
    info!("Daemon shutting down...");
    {
        let mut s = state.write().await;
        s.running = false;
    }

    // Give tasks time to finish
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    cleanup_socket();
    cleanup_pid_file();

    info!("Daemon stopped");
    Ok(())
}
