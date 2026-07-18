use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::models::{Integration, IntegrationConfig, OAuthTokenResponse};
use super::{FetchedItem, FetchResult, IntegrationProvider};

const GITHUB_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_URL: &str = "https://api.github.com";

pub struct GitHubProvider {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubIssue {
    id: u64,
    number: u32,
    title: String,
    html_url: String,
    state: String,
    body: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubPR {
    id: u64,
    number: u32,
    title: String,
    html_url: String,
    state: String,
    body: Option<String>,
    created_at: String,
    updated_at: String,
}

impl GitHubProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn get_client_id(&self) -> String {
        std::env::var("GITHUB_CLIENT_ID")
            .or_else(|_| std::env::var("MERIDIAN_GITHUB_CLIENT_ID"))
            .unwrap_or_else(|_| "placeholder_github_client_id".to_string())
    }

    fn get_client_secret(&self) -> String {
        std::env::var("GITHUB_CLIENT_SECRET")
            .or_else(|_| std::env::var("MERIDIAN_GITHUB_CLIENT_SECRET"))
            .unwrap_or_else(|_| "placeholder_github_client_secret".to_string())
    }

    async fn fetch_assigned_issues(
        &self,
        access_token: &str,
        repos: &[String],
    ) -> Result<Vec<GitHubIssue>, String> {
        let mut all_issues = Vec::new();

        for repo in repos {
            let url = format!("{}/repos/{}/issues?assignee=@me&state=all", GITHUB_API_URL, repo);
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "Meridian-Desktop")
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if response.status().is_success() {
                let issues: Vec<GitHubIssue> = response.json().await.map_err(|e| e.to_string())?;
                all_issues.extend(issues);
            }
        }

        Ok(all_issues)
    }

    async fn fetch_user_prs(
        &self,
        access_token: &str,
    ) -> Result<Vec<GitHubPR>, String> {
        let url = format!("{}/search/issues?q=type:pr+author:@me+state:open", GITHUB_API_URL);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Meridian-Desktop")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct SearchResult {
                items: Vec<GitHubPR>,
            }
            let result: SearchResult = response.json().await.map_err(|e| e.to_string())?;
            Ok(result.items)
        } else {
            Ok(Vec::new())
        }
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for GitHubProvider {
    fn integration_type(&self) -> &'static str {
        "github"
    }

    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String> {
        let (verifier, challenge) = super::framework::OAuthHelper::generate_pkce_pair();
        let scopes = self.get_scopes().join(" ");
        let client_id = self.get_client_id();

        let url = format!(
            "{}?client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            GITHUB_AUTHORIZE_URL,
            client_id,
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scopes),
            state,
            challenge
        );

        Ok((url, Some(verifier)))
    }

    async fn exchange_token(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<OAuthTokenResponse, String> {
        let client_id = self.get_client_id();
        let client_secret = self.get_client_secret();

        let mut params = vec![
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        if let Some(verifier) = code_verifier {
            params.push(("code_verifier", verifier));
        }

        let response = self
            .client
            .post(GITHUB_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", error));
        }

        let token: OAuthTokenResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(token)
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokenResponse, String> {
        Err("GitHub OAuth tokens don't expire and don't support refresh".to_string())
    }

    async fn fetch_data(&self, integration: &Integration) -> Result<FetchResult, String> {
        let access_token = integration
            .config
            .access_token
            .as_ref()
            .ok_or("No access token")?;

        let repos = integration.config.repositories.clone().unwrap_or_default();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        match self.fetch_assigned_issues(access_token, &repos).await {
            Ok(issues) => {
                for issue in issues {
                    let data = serde_json::to_value(&issue).unwrap_or_default();
                    items.push(FetchedItem {
                        external_type: "issue".to_string(),
                        external_id: issue.id.to_string(),
                        external_url: Some(issue.html_url),
                        data,
                    });
                }
            }
            Err(e) => errors.push(format!("Issues: {}", e)),
        }

        match self.fetch_user_prs(access_token).await {
            Ok(prs) => {
                for pr in prs {
                    let data = serde_json::to_value(&pr).unwrap_or_default();
                    items.push(FetchedItem {
                        external_type: "pr".to_string(),
                        external_id: pr.id.to_string(),
                        external_url: Some(pr.html_url),
                        data,
                    });
                }
            }
            Err(e) => errors.push(format!("PRs: {}", e)),
        }

        Ok(FetchResult { items, errors })
    }

    fn get_scopes(&self) -> Vec<&'static str> {
        vec!["repo", "read:user"]
    }

    fn validate_config(&self, config: &IntegrationConfig) -> Result<(), String> {
        if config.access_token.is_none() {
            return Err("Access token is required".to_string());
        }
        Ok(())
    }
}

pub async fn create_issue(
    access_token: &str,
    repo: &str,
    title: &str,
    body: Option<&str>,
    labels: Option<Vec<&str>>,
) -> Result<GitHubIssue, String> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues", GITHUB_API_URL, repo);

    #[derive(Serialize)]
    struct CreateIssue<'a> {
        title: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        labels: Option<Vec<&'a str>>,
    }

    let body_json = CreateIssue {
        title,
        body,
        labels,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Meridian-Desktop")
        .json(&body_json)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create issue: {}", error));
    }

    response.json().await.map_err(|e| e.to_string())
}

pub async fn add_comment(
    access_token: &str,
    repo: &str,
    issue_number: u32,
    body: &str,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues/{}/comments", GITHUB_API_URL, repo, issue_number);

    #[derive(Serialize)]
    struct Comment<'a> {
        body: &'a str,
    }

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Meridian-Desktop")
        .json(&Comment { body })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Failed to add comment: {}", error));
    }

    Ok(())
}
