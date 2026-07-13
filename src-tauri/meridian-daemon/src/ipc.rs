use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, ListenerOptions,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};

use crate::DaemonState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "shutdown")]
    Shutdown,
    #[serde(rename = "health")]
    Health,
    #[serde(rename = "run_job")]
    RunJob { job_type: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

pub struct IpcServer {
    socket_path: std::path::PathBuf,
    state: Arc<RwLock<DaemonState>>,
    shutdown_tx: mpsc::Sender<()>,
}

impl IpcServer {
    pub fn new(
        socket_path: &Path,
        state: Arc<RwLock<DaemonState>>,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            socket_path: socket_path.to_path_buf(),
            state,
            shutdown_tx,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = self.socket_path.clone().to_fs_name::<GenericFilePath>()?;
        let opts = ListenerOptions::new().name(name);
        let listener = opts.create_tokio()?;

        info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok(stream) => {
                    let state = self.state.clone();
                    let shutdown_tx = self.shutdown_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state, shutdown_tx).await {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    // Check if we should stop
                    let s = self.state.read().await;
                    if !s.running {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: Stream,
    state: Arc<RwLock<DaemonState>>,
    shutdown_tx: mpsc::Sender<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        debug!("Received: {}", line.trim());

        let response = match serde_json::from_str::<IpcRequest>(&line) {
            Ok(request) => handle_request(request, &state, &shutdown_tx).await,
            Err(e) => IpcResponse::error(format!("Invalid request: {}", e)),
        };

        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;
        writer.flush().await?;

        line.clear();
    }

    Ok(())
}

async fn handle_request(
    request: IpcRequest,
    state: &Arc<RwLock<DaemonState>>,
    shutdown_tx: &mpsc::Sender<()>,
) -> IpcResponse {
    match request {
        IpcRequest::Status => {
            let s = state.read().await;
            IpcResponse::ok(serde_json::json!({
                "running": s.running,
                "jobs_processed": s.jobs_processed,
                "last_error": s.last_error,
                "started_at": s.started_at.to_rfc3339(),
                "uptime_seconds": (chrono::Utc::now() - s.started_at).num_seconds(),
            }))
        }
        IpcRequest::Health => {
            let s = state.read().await;
            IpcResponse::ok(serde_json::json!({
                "healthy": s.running && s.last_error.is_none(),
                "running": s.running,
            }))
        }
        IpcRequest::Shutdown => {
            info!("Shutdown requested via IPC");
            let _ = shutdown_tx.send(()).await;
            IpcResponse::ok(serde_json::json!({ "message": "Shutdown initiated" }))
        }
        IpcRequest::RunJob { job_type } => {
            info!("Manual job run requested: {}", job_type);
            // TODO: Actually run the job
            IpcResponse::ok(serde_json::json!({
                "message": format!("Job '{}' queued", job_type)
            }))
        }
    }
}
