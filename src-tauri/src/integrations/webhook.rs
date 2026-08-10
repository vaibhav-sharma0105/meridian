use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::sync::{broadcast, oneshot};

// Holds the shutdown sender for the currently-running OAuth callback server so
// start_oauth_flow can kill the old one before binding a fresh one.
static ACTIVE_SERVER_SHUTDOWN: Mutex<Option<broadcast::Sender<()>>> = Mutex::new(None);

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

struct ServerState {
    tokens: std::collections::HashMap<String, String>,
    callback: Option<WebhookCallback>,
    app_handle: tauri::AppHandle,
    oauth_done_tx: Arc<std::sync::Mutex<Option<oneshot::Sender<()>>>>,
    // Keeps the broadcast sender alive so shutdown_rx.recv() doesn't resolve
    // immediately when the WebhookServer local variable drops after start().
    _shutdown_tx: broadcast::Sender<()>,
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
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Shut down any previously-running OAuth callback server so we can
        // reclaim port 8765. This handles the case where the user started an
        // OAuth flow but never completed it (no callback received), leaving
        // the server alive and holding the port.
        // NOTE: We must drop the MutexGuard before the `.await` below because
        // std::sync::MutexGuard is not Send and therefore cannot be held
        // across an await point in a Send future.
        let had_prev_server = {
            let mut guard = ACTIVE_SERVER_SHUTDOWN
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let had_prev = if let Some(prev_tx) = guard.take() {
                let _ = prev_tx.send(());
                true
            } else {
                false
            };
            *guard = Some(self.shutdown_tx.clone());
            had_prev
        };
        if had_prev_server {
            // Give the OS a moment to release the port before we bind again.
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let (oauth_done_tx, oauth_done_rx) = oneshot::channel::<()>();

        let state = Arc::new(ServerState {
            tokens,
            callback,
            app_handle,
            oauth_done_tx: Arc::new(std::sync::Mutex::new(Some(oauth_done_tx))),
            _shutdown_tx: self.shutdown_tx.clone(),
        });

        let router = Router::new()
            .route("/webhook/{token}", post(handle_webhook))
            .route("/oauth/callback", get(handle_oauth_callback))
            .with_state(state);

        // Try IPv6 loopback first (macOS resolves "localhost" to ::1 by default),
        // then fall back to IPv4 loopback. Both stay local-only.
        let listener = match tokio::net::TcpListener::bind(
            format!("[::1]:{}", self.port),
        )
        .await
        {
            Ok(l) => l,
            Err(_) => tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port))
                .await
                .map_err(|e| format!("Failed to bind OAuth callback server: {}", e))?,
        };

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {}
                        _ = oauth_done_rx => {}
                    }
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
    State(state): State<Arc<ServerState>>,
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

#[derive(Deserialize)]
struct OAuthCallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn handle_oauth_callback(
    Query(params): Query<OAuthCallbackParams>,
    State(server_state): State<Arc<ServerState>>,
) -> Html<String> {
    let shutdown = || {
        if let Ok(mut guard) = server_state.oauth_done_tx.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }
    };

    if let Some(err) = params.error {
        let description = params
            .error_description
            .unwrap_or_else(|| "OAuth authorization failed".to_string());
        let _ = server_state.app_handle.emit(
            "oauth_callback_received",
            serde_json::json!({ "success": false, "error": format!("{}: {}", err, description) }),
        );
        shutdown();
        return Html(oauth_result_page(
            false,
            "Authorization failed. You can close this tab and return to Meridian.",
        ));
    }

    let (code, oauth_state) = match (params.code, params.state) {
        (Some(c), Some(s)) => (c, s),
        _ => {
            let _ = server_state.app_handle.emit(
                "oauth_callback_received",
                serde_json::json!({ "success": false, "error": "Missing code or state parameter" }),
            );
            shutdown();
            return Html(oauth_result_page(false, "Invalid callback. You can close this tab."));
        }
    };

    // Emit code+state to frontend. The frontend calls handle_oauth_callback
    // (the Tauri command) to do the token exchange and persist the integration,
    // because that command has access to AppState.db.
    let _ = server_state.app_handle.emit(
        "oauth_callback_received",
        serde_json::json!({ "success": true, "code": code, "state": oauth_state }),
    );
    shutdown();
    Html(oauth_result_page(
        true,
        "Authorizing\u{2026} you can close this tab and return to Meridian.",
    ))
}

fn oauth_result_page(success: bool, message: &str) -> String {
    let (icon, color) = if success {
        ("\u{2713}", "#22c55e")
    } else {
        ("\u{2717}", "#ef4444")
    };
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Meridian</title>
<style>body{{font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#09090b;color:#fff}}
.box{{text-align:center}}.icon{{font-size:48px;color:{color}}}.msg{{margin-top:12px;font-size:16px;color:#a1a1aa}}</style></head>
<body><div class="box"><div class="icon">{icon}</div><div class="msg">{message}</div></div></body></html>"#
    )
}

pub fn find_available_port(start: u16) -> u16 {
    for port in start..start + 100 {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    start
}
