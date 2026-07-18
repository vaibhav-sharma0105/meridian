use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::models::{Integration, IntegrationConfig, OAuthTokenResponse};
use super::{FetchedItem, FetchResult, IntegrationProvider};

const SLACK_AUTHORIZE_URL: &str = "https://slack.com/oauth/v2/authorize";
const SLACK_TOKEN_URL: &str = "https://slack.com/api/oauth.v2.access";
const SLACK_API_URL: &str = "https://slack.com/api";

pub struct SlackProvider {
    client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackChannel {
    id: String,
    name: String,
    is_private: bool,
    is_member: bool,
    is_shared: bool,
    is_ext_shared: bool,
}

#[derive(Debug, Deserialize)]
struct SlackChannelListResponse {
    ok: bool,
    channels: Option<Vec<SlackChannel>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackPostMessageResponse {
    ok: bool,
    ts: Option<String>,
    error: Option<String>,
}

impl SlackProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn get_client_id(&self) -> String {
        std::env::var("SLACK_CLIENT_ID")
            .or_else(|_| std::env::var("MERIDIAN_SLACK_CLIENT_ID"))
            .unwrap_or_else(|_| "placeholder_slack_client_id".to_string())
    }

    fn get_client_secret(&self) -> String {
        std::env::var("SLACK_CLIENT_SECRET")
            .or_else(|_| std::env::var("MERIDIAN_SLACK_CLIENT_SECRET"))
            .unwrap_or_else(|_| "placeholder_slack_client_secret".to_string())
    }

    async fn fetch_channels(&self, access_token: &str) -> Result<Vec<SlackChannel>, String> {
        let url = format!(
            "{}/conversations.list?types=public_channel,private_channel&exclude_archived=true&limit=200",
            SLACK_API_URL
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let result: SlackChannelListResponse =
            response.json().await.map_err(|e| e.to_string())?;

        if !result.ok {
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(result.channels.unwrap_or_default())
    }
}

impl Default for SlackProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for SlackProvider {
    fn integration_type(&self) -> &'static str {
        "slack"
    }

    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String> {
        let scopes = self.get_scopes().join(",");
        let client_id = self.get_client_id();

        let url = format!(
            "{}?client_id={}&scope={}&redirect_uri={}&state={}",
            SLACK_AUTHORIZE_URL,
            client_id,
            urlencoding::encode(&scopes),
            urlencoding::encode(redirect_uri),
            state
        );

        Ok((url, None))
    }

    async fn exchange_token(
        &self,
        code: &str,
        redirect_uri: &str,
        _code_verifier: Option<&str>,
    ) -> Result<OAuthTokenResponse, String> {
        let client_id = self.get_client_id();
        let client_secret = self.get_client_secret();

        let response = self
            .client
            .post(SLACK_TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Deserialize)]
        struct SlackTokenResponse {
            ok: bool,
            access_token: Option<String>,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            token_type: Option<String>,
            error: Option<String>,
        }

        let result: SlackTokenResponse = response.json().await.map_err(|e| e.to_string())?;

        if !result.ok {
            return Err(result.error.unwrap_or_else(|| "Token exchange failed".to_string()));
        }

        Ok(OAuthTokenResponse {
            access_token: result.access_token.ok_or("No access token")?,
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
            token_type: result.token_type.unwrap_or_else(|| "bearer".to_string()),
            scope: None,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse, String> {
        let client_id = self.get_client_id();
        let client_secret = self.get_client_secret();

        let response = self
            .client
            .post(SLACK_TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Deserialize)]
        struct SlackTokenResponse {
            ok: bool,
            access_token: Option<String>,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            error: Option<String>,
        }

        let result: SlackTokenResponse = response.json().await.map_err(|e| e.to_string())?;

        if !result.ok {
            return Err(result.error.unwrap_or_else(|| "Token refresh failed".to_string()));
        }

        Ok(OAuthTokenResponse {
            access_token: result.access_token.ok_or("No access token")?,
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
            token_type: "bearer".to_string(),
            scope: None,
        })
    }

    async fn fetch_data(&self, integration: &Integration) -> Result<FetchResult, String> {
        let access_token = integration
            .config
            .access_token
            .as_ref()
            .ok_or("No access token")?;

        let mut items = Vec::new();
        let mut errors = Vec::new();

        match self.fetch_channels(access_token).await {
            Ok(channels) => {
                for channel in channels {
                    let data = serde_json::to_value(&channel).unwrap_or_default();
                    items.push(FetchedItem {
                        external_type: "channel".to_string(),
                        external_id: channel.id,
                        external_url: None,
                        data,
                    });
                }
            }
            Err(e) => errors.push(e),
        }

        Ok(FetchResult { items, errors })
    }

    fn get_scopes(&self) -> Vec<&'static str> {
        vec![
            "channels:read",
            "channels:history",
            "chat:write",
            "chat:write.public",
            "users:read",
        ]
    }

    fn validate_config(&self, config: &IntegrationConfig) -> Result<(), String> {
        if config.access_token.is_none() && config.bot_token.is_none() {
            return Err("Access token or bot token is required".to_string());
        }
        Ok(())
    }
}

pub async fn send_message(
    access_token: &str,
    channel_id: &str,
    text: &str,
) -> Result<String, String> {
    let client = Client::new();
    let url = format!("{}/chat.postMessage", SLACK_API_URL);

    #[derive(Serialize)]
    struct PostMessage<'a> {
        channel: &'a str,
        text: &'a str,
    }

    let body = PostMessage {
        channel: channel_id,
        text,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json; charset=utf-8")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: SlackPostMessageResponse = response.json().await.map_err(|e| e.to_string())?;

    if !result.ok {
        return Err(result.error.unwrap_or_else(|| "Failed to send message".to_string()));
    }

    Ok(result.ts.unwrap_or_default())
}

pub fn get_channel_autonomy(
    config: &IntegrationConfig,
    channel_id: &str,
) -> String {
    if let Some(channels) = &config.channels {
        if let Some(channel) = channels.iter().find(|c| c.id == channel_id) {
            return channel.autonomy_mode.clone();
        }
    }
    "draft".to_string()
}

pub fn is_high_risk_channel(channel: &SlackChannel) -> bool {
    channel.is_ext_shared || channel.is_shared
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackDraft {
    pub id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub text: String,
    pub created_at: String,
    pub send_at: Option<String>,
    pub status: String,
}

pub struct DraftQueue {
    drafts: Vec<SlackDraft>,
}

impl DraftQueue {
    pub fn new() -> Self {
        Self { drafts: Vec::new() }
    }

    pub fn add_draft(&mut self, draft: SlackDraft) {
        self.drafts.push(draft);
    }

    pub fn get_pending(&self) -> Vec<&SlackDraft> {
        self.drafts.iter().filter(|d| d.status == "pending").collect()
    }

    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(draft) = self.drafts.iter_mut().find(|d| d.id == id) {
            draft.status = "cancelled".to_string();
            true
        } else {
            false
        }
    }

    pub fn mark_sent(&mut self, id: &str) -> bool {
        if let Some(draft) = self.drafts.iter_mut().find(|d| d.id == id) {
            draft.status = "sent".to_string();
            true
        } else {
            false
        }
    }
}

impl Default for DraftQueue {
    fn default() -> Self {
        Self::new()
    }
}
