use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

use super::models::OAuthState;

lazy_static::lazy_static! {
    static ref OAUTH_STATES: Mutex<HashMap<String, OAuthState>> = Mutex::new(HashMap::new());
}

pub struct OAuthHelper;

impl OAuthHelper {
    pub fn generate_state() -> String {
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub fn generate_pkce_pair() -> (String, String) {
        let verifier: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
        (verifier, challenge)
    }

    pub fn store_oauth_state(state: OAuthState) {
        let mut states = OAUTH_STATES.lock().unwrap();
        states.insert(state.state.clone(), state);
    }

    pub fn get_oauth_state(state: &str) -> Option<OAuthState> {
        let states = OAUTH_STATES.lock().unwrap();
        states.get(state).cloned()
    }

    pub fn remove_oauth_state(state: &str) -> Option<OAuthState> {
        let mut states = OAUTH_STATES.lock().unwrap();
        states.remove(state)
    }

    pub fn cleanup_expired_states() {
        let mut states = OAUTH_STATES.lock().unwrap();
        let now = chrono::Utc::now();
        states.retain(|_, v| {
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&v.created_at) {
                now.signed_duration_since(created).num_minutes() < 10
            } else {
                false
            }
        });
    }
}

pub struct IntegrationRegistry {
    pub available: Vec<IntegrationMeta>,
}

#[derive(Debug, Clone)]
pub struct IntegrationMeta {
    pub integration_type: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub auth_type: AuthType,
    pub capabilities: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub enum AuthType {
    OAuth2,
    OAuth2WithPkce,
    ApiToken,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self {
            available: vec![
                IntegrationMeta {
                    integration_type: "github",
                    name: "GitHub",
                    description: "Sync issues, PRs, and create bidirectional task links",
                    icon: "github",
                    auth_type: AuthType::OAuth2WithPkce,
                    capabilities: vec![
                        "fetch_issues",
                        "fetch_prs",
                        "create_issue",
                        "comment",
                        "bidirectional_link",
                    ],
                },
                IntegrationMeta {
                    integration_type: "jira",
                    name: "Jira",
                    description: "Sync Jira issues and sprints with bidirectional linking",
                    icon: "jira",
                    auth_type: AuthType::OAuth2,
                    capabilities: vec![
                        "fetch_issues",
                        "fetch_sprints",
                        "create_issue",
                        "transition_issue",
                        "bidirectional_link",
                    ],
                },
                IntegrationMeta {
                    integration_type: "slack",
                    name: "Slack",
                    description: "Send messages, monitor channels for action items",
                    icon: "slack",
                    auth_type: AuthType::OAuth2,
                    capabilities: vec![
                        "send_message",
                        "channel_monitor",
                        "draft_mode",
                        "channel_autonomy",
                    ],
                },
            ],
        }
    }

    pub fn get_meta(&self, integration_type: &str) -> Option<&IntegrationMeta> {
        self.available
            .iter()
            .find(|m| m.integration_type == integration_type)
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CacheManager;

impl CacheManager {
    pub fn is_cache_stale(synced_at: &str, max_age_minutes: i64) -> bool {
        if let Ok(synced) = chrono::DateTime::parse_from_rfc3339(synced_at) {
            let now = chrono::Utc::now();
            now.signed_duration_since(synced).num_minutes() > max_age_minutes
        } else {
            true
        }
    }
}
