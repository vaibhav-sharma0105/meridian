use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::models::{Integration, IntegrationConfig, OAuthTokenResponse};
use super::{FetchedItem, FetchResult, IntegrationProvider};

const JIRA_AUTHORIZE_URL: &str = "https://auth.atlassian.com/authorize";
const JIRA_TOKEN_URL: &str = "https://auth.atlassian.com/oauth/token";
const JIRA_API_URL: &str = "https://api.atlassian.com/ex/jira";

pub struct JiraProvider {
    client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
struct JiraIssue {
    id: String,
    key: String,
    fields: JiraIssueFields,
    #[serde(rename = "self")]
    self_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JiraIssueFields {
    summary: String,
    description: Option<serde_json::Value>,
    status: JiraStatus,
    priority: Option<JiraPriority>,
    assignee: Option<JiraUser>,
    created: String,
    updated: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JiraStatus {
    name: String,
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JiraPriority {
    name: String,
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JiraUser {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct JiraSearchResult {
    issues: Vec<JiraIssue>,
    total: u32,
    #[serde(rename = "startAt")]
    start_at: u32,
    #[serde(rename = "maxResults")]
    max_results: u32,
}

#[derive(Debug, Deserialize)]
struct AccessibleResources {
    id: String,
    name: String,
    url: String,
}

impl JiraProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn get_client_id(&self) -> String {
        std::env::var("JIRA_CLIENT_ID")
            .or_else(|_| std::env::var("MERIDIAN_JIRA_CLIENT_ID"))
            .unwrap_or_else(|_| "placeholder_jira_client_id".to_string())
    }

    fn get_client_secret(&self) -> String {
        std::env::var("JIRA_CLIENT_SECRET")
            .or_else(|_| std::env::var("MERIDIAN_JIRA_CLIENT_SECRET"))
            .unwrap_or_else(|_| "placeholder_jira_client_secret".to_string())
    }

    async fn get_cloud_id(&self, access_token: &str) -> Result<String, String> {
        let response = self
            .client
            .get("https://api.atlassian.com/oauth/token/accessible-resources")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err("Failed to get accessible resources".to_string());
        }

        let resources: Vec<AccessibleResources> =
            response.json().await.map_err(|e| e.to_string())?;

        resources
            .first()
            .map(|r| r.id.clone())
            .ok_or_else(|| "No accessible Jira cloud found".to_string())
    }

    async fn fetch_assigned_issues(
        &self,
        access_token: &str,
        cloud_id: &str,
        projects: &[String],
    ) -> Result<Vec<JiraIssue>, String> {
        let project_clause = if projects.is_empty() {
            String::new()
        } else {
            format!(
                " AND project IN ({})",
                projects
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        let jql = format!("assignee = currentUser(){} ORDER BY updated DESC", project_clause);
        let url = format!(
            "{}/{}/rest/api/3/search?jql={}&maxResults=50",
            JIRA_API_URL,
            cloud_id,
            urlencoding::encode(&jql)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Failed to fetch issues: {}", error));
        }

        let result: JiraSearchResult = response.json().await.map_err(|e| e.to_string())?;
        Ok(result.issues)
    }
}

impl Default for JiraProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for JiraProvider {
    fn integration_type(&self) -> &'static str {
        "jira"
    }

    fn auth_url(&self, state: &str, redirect_uri: &str) -> Result<(String, Option<String>), String> {
        let scopes = self.get_scopes().join(" ");
        let client_id = self.get_client_id();

        let url = format!(
            "{}?audience=api.atlassian.com&client_id={}&scope={}&redirect_uri={}&state={}&response_type=code&prompt=consent",
            JIRA_AUTHORIZE_URL,
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

        #[derive(Serialize)]
        struct TokenRequest<'a> {
            grant_type: &'a str,
            client_id: &'a str,
            client_secret: &'a str,
            code: &'a str,
            redirect_uri: &'a str,
        }

        let body = TokenRequest {
            grant_type: "authorization_code",
            client_id: &client_id,
            client_secret: &client_secret,
            code,
            redirect_uri,
        };

        let response = self
            .client
            .post(JIRA_TOKEN_URL)
            .header("Content-Type", "application/json")
            .json(&body)
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

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse, String> {
        let client_id = self.get_client_id();
        let client_secret = self.get_client_secret();

        #[derive(Serialize)]
        struct RefreshRequest<'a> {
            grant_type: &'a str,
            client_id: &'a str,
            client_secret: &'a str,
            refresh_token: &'a str,
        }

        let body = RefreshRequest {
            grant_type: "refresh_token",
            client_id: &client_id,
            client_secret: &client_secret,
            refresh_token,
        };

        let response = self
            .client
            .post(JIRA_TOKEN_URL)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Token refresh failed: {}", error));
        }

        let token: OAuthTokenResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(token)
    }

    async fn fetch_data(&self, integration: &Integration) -> Result<FetchResult, String> {
        let access_token = integration
            .config
            .access_token
            .as_ref()
            .ok_or("No access token")?;

        let cloud_id = self.get_cloud_id(access_token).await?;
        let projects = integration.config.projects.clone().unwrap_or_default();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        match self
            .fetch_assigned_issues(access_token, &cloud_id, &projects)
            .await
        {
            Ok(issues) => {
                for issue in issues {
                    let data = serde_json::to_value(&issue).unwrap_or_default();
                    let url = format!(
                        "https://{}.atlassian.net/browse/{}",
                        cloud_id, issue.key
                    );
                    items.push(FetchedItem {
                        external_type: "jira_issue".to_string(),
                        external_id: issue.id,
                        external_url: Some(url),
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
            "read:jira-work",
            "write:jira-work",
            "read:jira-user",
            "offline_access",
        ]
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
    cloud_id: &str,
    project_key: &str,
    summary: &str,
    description: Option<&str>,
    issue_type: &str,
) -> Result<JiraIssue, String> {
    let client = Client::new();
    let url = format!("{}/{}/rest/api/3/issue", JIRA_API_URL, cloud_id);

    #[derive(Serialize)]
    struct CreateIssue<'a> {
        fields: CreateIssueFields<'a>,
    }

    #[derive(Serialize)]
    struct CreateIssueFields<'a> {
        project: ProjectKey<'a>,
        summary: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<AdfDoc<'a>>,
        issuetype: IssueType<'a>,
    }

    #[derive(Serialize)]
    struct ProjectKey<'a> {
        key: &'a str,
    }

    #[derive(Serialize)]
    struct IssueType<'a> {
        name: &'a str,
    }

    #[derive(Serialize)]
    struct AdfDoc<'a> {
        r#type: &'a str,
        version: u8,
        content: Vec<AdfParagraph<'a>>,
    }

    #[derive(Serialize)]
    struct AdfParagraph<'a> {
        r#type: &'a str,
        content: Vec<AdfText<'a>>,
    }

    #[derive(Serialize)]
    struct AdfText<'a> {
        r#type: &'a str,
        text: &'a str,
    }

    let description_adf = description.map(|d| AdfDoc {
        r#type: "doc",
        version: 1,
        content: vec![AdfParagraph {
            r#type: "paragraph",
            content: vec![AdfText {
                r#type: "text",
                text: d,
            }],
        }],
    });

    let body = CreateIssue {
        fields: CreateIssueFields {
            project: ProjectKey { key: project_key },
            summary,
            description: description_adf,
            issuetype: IssueType { name: issue_type },
        },
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create issue: {}", error));
    }

    response.json().await.map_err(|e| e.to_string())
}

pub async fn transition_issue(
    access_token: &str,
    cloud_id: &str,
    issue_id: &str,
    transition_id: &str,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}/{}/rest/api/3/issue/{}/transitions",
        JIRA_API_URL, cloud_id, issue_id
    );

    #[derive(Serialize)]
    struct TransitionRequest<'a> {
        transition: TransitionId<'a>,
    }

    #[derive(Serialize)]
    struct TransitionId<'a> {
        id: &'a str,
    }

    let body = TransitionRequest {
        transition: TransitionId { id: transition_id },
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Failed to transition issue: {}", error));
    }

    Ok(())
}
