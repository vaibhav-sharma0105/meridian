use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct WebhookServer {
    pub port: u16,
    shutdown_tx: broadcast::Sender<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub integration_type: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: Option<String>,
}

pub type WebhookCallback = Arc<dyn Fn(WebhookPayload) + Send + Sync>;

struct AppState {
    tokens: std::collections::HashMap<String, String>,
    callback: Option<WebhookCallback>,
}

impl WebhookServer {
    pub fn new(port: u16) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self { port, shutdown_tx }
    }

    pub async fn start(
        &self,
        tokens: std::collections::HashMap<String, String>,
        callback: Option<WebhookCallback>,
    ) -> Result<(), String> {
        let state = Arc::new(AppState { tokens, callback });

        let app = Router::new()
            .route("/webhook/{token}", post(handle_webhook))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind webhook server: {}", e))?;

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                })
                .await
                .ok();
        });

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(());
    }

    pub fn get_webhook_url(&self, token: &str) -> String {
        format!("http://localhost:{}/webhook/{}", self.port, token)
    }
}

async fn handle_webhook(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<WebhookResponse>, StatusCode> {
    if !state.tokens.values().any(|t| t == &token) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let integration_type = state
        .tokens
        .iter()
        .find(|(_, t)| *t == &token)
        .map(|(k, _)| k.clone())
        .unwrap_or_default();

    let event_type = payload
        .get("event")
        .or_else(|| payload.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let webhook_payload = WebhookPayload {
        integration_type,
        event_type,
        data: payload,
    };

    if let Some(ref callback) = state.callback {
        callback(webhook_payload);
    }

    Ok(Json(WebhookResponse {
        success: true,
        message: None,
    }))
}

pub fn find_available_port(start: u16) -> u16 {
    for port in start..start + 100 {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    start
}
