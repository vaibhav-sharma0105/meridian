use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SLACK_SOCKET_URL: &str = "https://slack.com/api/apps.connections.open";
const MAX_RECONNECT_DELAY_SECS: u64 = 300;
const INITIAL_RECONNECT_DELAY_SECS: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackEvent {
    pub event_type: String,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub text: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub text: String,
    pub detected_type: ActionItemType,
    pub ts: String,
    pub thread_ts: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionItemType {
    Mention,
    Question,
    Request,
    Deadline,
    Followup,
}

impl ActionItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionItemType::Mention => "mention",
            ActionItemType::Question => "question",
            ActionItemType::Request => "request",
            ActionItemType::Deadline => "deadline",
            ActionItemType::Followup => "followup",
        }
    }
}

pub struct SocketModeClient {
    app_token: String,
    running: Arc<AtomicBool>,
    reconnect_count: Arc<AtomicU32>,
    event_tx: Option<mpsc::Sender<SlackEvent>>,
}

impl SocketModeClient {
    pub fn new(app_token: String) -> Self {
        Self {
            app_token,
            running: Arc::new(AtomicBool::new(false)),
            reconnect_count: Arc::new(AtomicU32::new(0)),
            event_tx: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn reconnect_count(&self) -> u32 {
        self.reconnect_count.load(Ordering::SeqCst)
    }

    pub async fn connect(&mut self) -> Result<mpsc::Receiver<SlackEvent>, String> {
        if !self.app_token.starts_with("xapp-") {
            return Err("Invalid app token: must start with 'xapp-'".to_string());
        }

        let ws_url = self.get_websocket_url().await?;
        let (tx, rx) = mpsc::channel(100);
        self.event_tx = Some(tx.clone());
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let reconnect_count = self.reconnect_count.clone();
        let app_token = self.app_token.clone();

        tokio::spawn(async move {
            Self::run_socket_loop(ws_url, app_token, tx, running, reconnect_count).await;
        });

        Ok(rx)
    }

    pub fn disconnect(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn get_websocket_url(&self) -> Result<String, String> {
        let client = reqwest::Client::new();

        let response = client
            .post(SLACK_SOCKET_URL)
            .header("Authorization", format!("Bearer {}", self.app_token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Slack: {}", e))?;

        #[derive(Deserialize)]
        struct ConnectionResponse {
            ok: bool,
            url: Option<String>,
            error: Option<String>,
        }

        let result: ConnectionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !result.ok {
            return Err(format!(
                "Slack API error: {}",
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        result.url.ok_or_else(|| "No WebSocket URL returned".to_string())
    }

    async fn run_socket_loop(
        initial_url: String,
        app_token: String,
        tx: mpsc::Sender<SlackEvent>,
        running: Arc<AtomicBool>,
        reconnect_count: Arc<AtomicU32>,
    ) {
        let mut ws_url = initial_url;
        let mut delay = INITIAL_RECONNECT_DELAY_SECS;

        while running.load(Ordering::SeqCst) {
            match Self::connect_and_handle(&ws_url, &tx, &running).await {
                Ok(()) => {
                    // Clean disconnect, reset delay
                    delay = INITIAL_RECONNECT_DELAY_SECS;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}. Reconnecting in {}s...", e, delay);
                    reconnect_count.fetch_add(1, Ordering::SeqCst);

                    // Wait with exponential backoff
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    delay = (delay * 2).min(MAX_RECONNECT_DELAY_SECS);

                    // Get fresh WebSocket URL
                    if running.load(Ordering::SeqCst) {
                        let client = reqwest::Client::new();
                        if let Ok(response) = client
                            .post(SLACK_SOCKET_URL)
                            .header("Authorization", format!("Bearer {}", app_token))
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .send()
                            .await
                        {
                            #[derive(Deserialize)]
                            struct ConnectionResponse {
                                ok: bool,
                                url: Option<String>,
                            }

                            if let Ok(result) = response.json::<ConnectionResponse>().await {
                                if result.ok {
                                    if let Some(url) = result.url {
                                        ws_url = url;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn connect_and_handle(
        ws_url: &str,
        tx: &mpsc::Sender<SlackEvent>,
        running: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        let (mut write, mut read) = ws_stream.split();

        while running.load(Ordering::SeqCst) {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = Self::handle_message(&text, tx, &mut write).await {
                                eprintln!("Error handling message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            return Ok(());
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Err(e)) => {
                            return Err(format!("WebSocket error: {}", e));
                        }
                        None => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    // Send ping to keep connection alive
                    if write.send(Message::Ping(vec![])).await.is_err() {
                        return Err("Failed to send ping".to_string());
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_message(
        text: &str,
        tx: &mpsc::Sender<SlackEvent>,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
    ) -> Result<(), String> {
        #[derive(Deserialize)]
        struct SocketMessage {
            #[serde(rename = "type")]
            msg_type: String,
            envelope_id: Option<String>,
            payload: Option<serde_json::Value>,
        }

        let msg: SocketMessage =
            serde_json::from_str(text).map_err(|e| format!("Parse error: {}", e))?;

        // Acknowledge the message within 3 seconds
        if let Some(envelope_id) = &msg.envelope_id {
            let ack = serde_json::json!({ "envelope_id": envelope_id });
            write
                .send(Message::Text(ack.to_string()))
                .await
                .map_err(|e| format!("Ack failed: {}", e))?;
        }

        match msg.msg_type.as_str() {
            "hello" => {
                // Connection established
            }
            "events_api" => {
                if let Some(payload) = msg.payload {
                    if let Some(event) = payload.get("event") {
                        let event_type = event
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let slack_event = SlackEvent {
                            event_type,
                            channel_id: event
                                .get("channel")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            user_id: event
                                .get("user")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            text: event
                                .get("text")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            ts: event.get("ts").and_then(|v| v.as_str()).map(String::from),
                            thread_ts: event
                                .get("thread_ts")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        };

                        let _ = tx.send(slack_event).await;
                    }
                }
            }
            "disconnect" => {
                // Server requesting disconnect, will reconnect
                return Err("Server requested disconnect".to_string());
            }
            _ => {}
        }

        Ok(())
    }
}

pub fn detect_action_items(text: &str, bot_user_id: Option<&str>) -> Vec<ActionItemType> {
    let mut items = Vec::new();
    let text_lower = text.to_lowercase();

    // Check for direct mentions
    if let Some(bot_id) = bot_user_id {
        if text.contains(&format!("<@{}>", bot_id)) {
            items.push(ActionItemType::Mention);
        }
    }

    // Check for questions
    if text.contains('?')
        || text_lower.contains("can you")
        || text_lower.contains("could you")
        || text_lower.contains("would you")
        || text_lower.contains("do you know")
        || text_lower.contains("what is")
        || text_lower.contains("how do")
    {
        items.push(ActionItemType::Question);
    }

    // Check for requests
    let request_patterns = [
        "please",
        "can you",
        "could you",
        "would you mind",
        "need you to",
        "want you to",
        "help me",
        "help with",
        "take a look",
        "review",
        "check",
        "update",
        "fix",
        "create",
        "add",
        "remove",
        "delete",
    ];

    if request_patterns.iter().any(|p| text_lower.contains(p)) {
        if !items.contains(&ActionItemType::Request) {
            items.push(ActionItemType::Request);
        }
    }

    // Check for deadlines
    let deadline_patterns = [
        "by end of day",
        "by eod",
        "by tomorrow",
        "by monday",
        "by tuesday",
        "by wednesday",
        "by thursday",
        "by friday",
        "asap",
        "urgent",
        "priority",
        "deadline",
        "due",
    ];

    if deadline_patterns.iter().any(|p| text_lower.contains(p)) {
        items.push(ActionItemType::Deadline);
    }

    // Check for follow-up patterns
    let followup_patterns = [
        "follow up",
        "following up",
        "circling back",
        "checking in",
        "any update",
        "any progress",
        "status update",
        "where are we",
        "reminder",
    ];

    if followup_patterns.iter().any(|p| text_lower.contains(p)) {
        items.push(ActionItemType::Followup);
    }

    items
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketModeStatus {
    pub connected: bool,
    pub app_token_configured: bool,
    pub last_event_at: Option<String>,
    pub reconnect_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mention() {
        let items = detect_action_items("Hey <@U12345>, can you help?", Some("U12345"));
        assert!(items.contains(&ActionItemType::Mention));
    }

    #[test]
    fn test_detect_question() {
        let items = detect_action_items("What is the status of the project?", None);
        assert!(items.contains(&ActionItemType::Question));
    }

    #[test]
    fn test_detect_request() {
        let items = detect_action_items("Please review this PR", None);
        assert!(items.contains(&ActionItemType::Request));
    }

    #[test]
    fn test_detect_deadline() {
        let items = detect_action_items("We need this by EOD", None);
        assert!(items.contains(&ActionItemType::Deadline));
    }

    #[test]
    fn test_detect_followup() {
        let items = detect_action_items("Just following up on the earlier discussion", None);
        assert!(items.contains(&ActionItemType::Followup));
    }

    #[test]
    fn test_detect_multiple() {
        let items = detect_action_items(
            "<@U12345> can you please fix this by tomorrow?",
            Some("U12345"),
        );
        assert!(items.contains(&ActionItemType::Mention));
        assert!(items.contains(&ActionItemType::Request));
        assert!(items.contains(&ActionItemType::Deadline));
    }

    #[test]
    fn test_no_detection() {
        let items = detect_action_items("Thanks for your help yesterday!", None);
        assert!(items.is_empty());
    }
}
