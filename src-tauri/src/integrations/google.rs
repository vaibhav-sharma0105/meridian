use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::models::{Integration, IntegrationConfig, OAuthTokenResponse};
use super::{FetchResult, IntegrationProvider};

const GOOGLE_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DIRECTORY_API_URL: &str = "https://admin.googleapis.com/admin/directory/v1/users";

/// Google Workspace integration. Currently only used for team roster sync
/// (Directory API) — not a general fetch_data provider like GitHub/Jira,
/// since there's no "issues/PRs" equivalent for a directory of people.
///
/// Requires a Google Cloud OAuth client with the Admin SDK API enabled, and
/// the `admin.directory.user.readonly` scope approved by a Workspace
/// domain admin (regular Google accounts without a Workspace domain can't
/// grant this scope at all — see CREDENTIALS_SETUP.md).
pub struct GoogleProvider {
    client: Client,
}

impl GoogleProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn get_client_id(&self) -> String {
        std::env::var("GOOGLE_CLIENT_ID")
            .or_else(|_| std::env::var("MERIDIAN_GOOGLE_CLIENT_ID"))
            .unwrap_or_else(|_| "placeholder_google_client_id".to_string())
    }

    fn get_client_secret(&self) -> String {
        std::env::var("GOOGLE_CLIENT_SECRET")
            .or_else(|_| std::env::var("MERIDIAN_GOOGLE_CLIENT_SECRET"))
            .unwrap_or_else(|_| "placeholder_google_client_secret".to_string())
    }
}

impl Default for GoogleProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for GoogleProvider {
    fn integration_type(&self) -> &'static str {
        "google"
    }

    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String> {
        let scopes = self.get_scopes().join(" ");
        let client_id = self.get_client_id();

        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&access_type=offline&prompt=consent&scope={}&state={}",
            GOOGLE_AUTHORIZE_URL,
            client_id,
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scopes),
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
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: Option<String>,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            token_type: Option<String>,
            scope: Option<String>,
            error: Option<String>,
            error_description: Option<String>,
        }

        let result: GoogleTokenResponse = response.json().await.map_err(|e| e.to_string())?;

        if let Some(error) = result.error {
            return Err(result.error_description.unwrap_or(error));
        }

        Ok(OAuthTokenResponse {
            access_token: result.access_token.ok_or("No access token")?,
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
            token_type: result.token_type.unwrap_or_else(|| "bearer".to_string()),
            scope: result.scope,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse, String> {
        let client_id = self.get_client_id();
        let client_secret = self.get_client_secret();

        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: Option<String>,
            expires_in: Option<u64>,
            token_type: Option<String>,
            error: Option<String>,
            error_description: Option<String>,
        }

        let result: GoogleTokenResponse = response.json().await.map_err(|e| e.to_string())?;

        if let Some(error) = result.error {
            return Err(result.error_description.unwrap_or(error));
        }

        Ok(OAuthTokenResponse {
            access_token: result.access_token.ok_or("No access token")?,
            // Google doesn't rotate the refresh token on refresh; the caller
            // keeps using the one it already stored.
            refresh_token: None,
            expires_in: result.expires_in,
            token_type: result.token_type.unwrap_or_else(|| "bearer".to_string()),
            scope: None,
        })
    }

    async fn fetch_data(&self, _integration: &Integration) -> Result<FetchResult, String> {
        // No generic "issues/PRs"-style content for a directory of people —
        // team roster sync uses fetch_workspace_members() directly instead.
        Ok(FetchResult {
            items: Vec::new(),
            errors: Vec::new(),
        })
    }

    fn get_scopes(&self) -> Vec<&'static str> {
        vec!["https://www.googleapis.com/auth/admin.directory.user.readonly"]
    }

    fn validate_config(&self, config: &IntegrationConfig) -> Result<(), String> {
        if config.access_token.is_none() {
            return Err("Access token is required".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct GoogleDirectoryName {
    #[serde(rename = "fullName")]
    full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleDirectoryUser {
    id: String,
    #[serde(rename = "primaryEmail")]
    primary_email: Option<String>,
    name: Option<GoogleDirectoryName>,
    #[serde(rename = "thumbnailPhotoUrl")]
    thumbnail_photo_url: Option<String>,
    suspended: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GoogleDirectoryListResponse {
    users: Option<Vec<GoogleDirectoryUser>>,
    error: Option<GoogleDirectoryError>,
}

#[derive(Debug, Deserialize)]
struct GoogleDirectoryError {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTeamMember {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Fetches active (non-suspended) members of the authenticated user's Google
/// Workspace domain via the Admin SDK Directory API. `customer=my_customer`
/// is a Directory API shortcut for "the caller's own Workspace domain" —
/// avoids a separate lookup to resolve the domain name first.
///
/// Requires the access token to carry the `admin.directory.user.readonly`
/// scope, which only a Workspace domain admin can grant — a personal Google
/// account (no Workspace domain) will get a 403 here, by design.
pub async fn fetch_workspace_members(access_token: &str) -> Result<Vec<GoogleTeamMember>, String> {
    let client = Client::new();
    let url = format!("{}?customer=my_customer&maxResults=200", GOOGLE_DIRECTORY_API_URL);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: GoogleDirectoryListResponse = response.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = result.error {
        return Err(error.message);
    }

    let members = result
        .users
        .unwrap_or_default()
        .into_iter()
        .filter(|u| !u.suspended.unwrap_or(false))
        .map(|u| GoogleTeamMember {
            id: u.id,
            name: u
                .name
                .and_then(|n| n.full_name)
                .or_else(|| u.primary_email.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            email: u.primary_email,
            avatar_url: u.thumbnail_photo_url,
        })
        .collect();

    Ok(members)
}
