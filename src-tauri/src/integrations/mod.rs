pub mod framework;
pub mod github;
pub mod jira;
pub mod models;
pub mod repository;
pub mod slack;
pub mod slack_socket;
pub mod webhook;

use async_trait::async_trait;
use models::{Integration, IntegrationConfig, OAuthTokenResponse};

#[derive(Debug, Clone)]
pub struct FetchedItem {
    pub external_type: String,
    pub external_id: String,
    pub external_url: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub items: Vec<FetchedItem>,
    pub errors: Vec<String>,
}

#[async_trait]
pub trait IntegrationProvider: Send + Sync {
    fn integration_type(&self) -> &'static str;

    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String>;

    async fn exchange_token(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<OAuthTokenResponse, String>;

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse, String>;

    async fn fetch_data(&self, integration: &Integration) -> Result<FetchResult, String>;

    fn get_scopes(&self) -> Vec<&'static str>;

    fn validate_config(&self, config: &IntegrationConfig) -> Result<(), String>;
}

pub fn get_provider(integration_type: &str) -> Option<Box<dyn IntegrationProvider>> {
    match integration_type {
        "github" => Some(Box::new(github::GitHubProvider::new())),
        "jira" => Some(Box::new(jira::JiraProvider::new())),
        "slack" => Some(Box::new(slack::SlackProvider::new())),
        _ => None,
    }
}
