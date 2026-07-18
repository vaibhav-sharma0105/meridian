use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    #[serde(rename = "type")]
    pub integration_type: String,
    pub name: String,
    pub config: IntegrationConfig,
    pub permissions: Option<IntegrationPermissions>,
    pub autonomy_mode: String,
    pub status: String,
    pub last_sync: Option<String>,
    pub sync_interval_minutes: i32,
    pub webhook_token: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationConfig {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub api_token: Option<String>,
    pub base_url: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub repositories: Option<Vec<String>>,
    pub projects: Option<Vec<String>>,
    pub channels: Option<Vec<ChannelConfig>>,
    pub bot_token: Option<String>,
    pub user_token: Option<String>,
    pub app_token: Option<String>,
    pub socket_mode_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    pub autonomy_mode: String,
    pub is_external: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationPermissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
    pub admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCache {
    pub id: String,
    pub integration_id: String,
    pub external_type: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub data: serde_json::Value,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationLink {
    pub id: String,
    pub integration_id: String,
    pub local_type: String,
    pub local_id: String,
    pub external_type: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub sync_enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntegrationInput {
    pub integration_type: String,
    pub name: String,
    pub config: IntegrationConfig,
    pub permissions: Option<IntegrationPermissions>,
    pub autonomy_mode: Option<String>,
    pub sync_interval_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIntegrationInput {
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub config: Option<IntegrationConfig>,
    pub permissions: Option<IntegrationPermissions>,
    pub autonomy_mode: Option<String>,
    pub status: Option<String>,
    pub sync_interval_minutes: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLinkInput {
    pub integration_id: String,
    pub local_type: String,
    pub local_id: String,
    pub external_type: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub sync_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub integration_id: String,
    pub status: String,
    pub last_sync: Option<String>,
    pub items_synced: i32,
    pub items_new: i32,
    pub items_updated: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub integration_type: String,
    pub state: String,
    pub code_verifier: Option<String>,
    pub redirect_uri: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}
