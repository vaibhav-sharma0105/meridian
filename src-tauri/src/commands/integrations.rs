use tauri::State;

use crate::governance::{
    approval as governance_approval,
    autonomy::{AutonomyContext, AutonomyController},
    risk::{self, ActionType, ContentRisk, DestinationType},
};
use crate::integrations::framework::{IntegrationRegistry, OAuthHelper};
use crate::integrations::models::{
    CreateIntegrationInput, CreateLinkInput, Integration, IntegrationCache, IntegrationLink,
    OAuthState, SyncState, UpdateIntegrationInput,
};
use crate::integrations::{mapping, repository};
use crate::integrations::{get_provider, IntegrationProvider};
use crate::AppState;

#[derive(serde::Serialize)]
pub struct IntegrationWriteResult {
    pub success: bool,
    pub queued_for_approval: bool,
    pub approval_id: Option<String>,
    pub result: Option<serde_json::Value>,
}

fn evaluate_integration_write(
    conn: &rusqlite::Connection,
    integration_id: &str,
    action_type: ActionType,
    destination: DestinationType,
    action_description: &str,
    action_config: &str,
) -> Result<Option<String>, String> {
    let risk_score = risk::calculate_risk(action_type, destination, ContentRisk::Normal);

    let autonomy_context = AutonomyContext {
        integration_id: Some(integration_id.to_string()),
        skill_id: None,
    };

    let decision = AutonomyController::evaluate_action(conn, &autonomy_context, risk_score.risk_level)?;

    if decision.requires_approval {
        let approval_id = governance_approval::queue_for_approval(
            conn,
            action_description,
            action_config,
            Some("integration"),
            Some(integration_id),
            decision.risk_level,
            decision.autonomy_mode,
            Some(&decision.reason),
            None,
        )?;
        return Ok(Some(approval_id));
    }

    Ok(None)
}

#[tauri::command]
pub fn list_integrations(state: State<AppState>) -> Result<Vec<Integration>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::list_integrations(&conn)
}

#[tauri::command]
pub fn get_integration(state: State<AppState>, id: String) -> Result<Option<Integration>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_integration(&conn, &id)
}

#[tauri::command]
pub fn get_available_integrations() -> Vec<serde_json::Value> {
    let registry = IntegrationRegistry::new();
    registry
        .available
        .iter()
        .map(|meta| {
            serde_json::json!({
                "type": meta.integration_type,
                "name": meta.name,
                "description": meta.description,
                "icon": meta.icon,
                "capabilities": meta.capabilities,
            })
        })
        .collect()
}

#[tauri::command]
pub fn create_integration(
    state: State<AppState>,
    input: CreateIntegrationInput,
) -> Result<Integration, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_integration(&conn, input)
}

#[tauri::command]
pub fn update_integration(
    state: State<AppState>,
    input: UpdateIntegrationInput,
) -> Result<Integration, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_integration(&conn, input)
}

#[tauri::command]
pub fn delete_integration(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::delete_integration(&conn, &id)
}

#[tauri::command]
pub fn start_oauth_flow(
    integration_type: String,
    redirect_uri: String,
) -> Result<String, String> {
    let provider = get_provider(&integration_type)
        .ok_or_else(|| format!("Unknown integration type: {}", integration_type))?;

    let state = OAuthHelper::generate_state();
    let (auth_url, code_verifier) = provider.auth_url(&state, &redirect_uri)?;

    let oauth_state = OAuthState {
        integration_type,
        state: state.clone(),
        code_verifier,
        redirect_uri,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    OAuthHelper::store_oauth_state(oauth_state);
    OAuthHelper::cleanup_expired_states();

    Ok(auth_url)
}

#[tauri::command]
pub async fn handle_oauth_callback(
    state: State<'_, AppState>,
    oauth_state: String,
    code: String,
) -> Result<Integration, String> {
    let stored_state = OAuthHelper::remove_oauth_state(&oauth_state)
        .ok_or("Invalid or expired OAuth state")?;

    let provider = get_provider(&stored_state.integration_type)
        .ok_or_else(|| format!("Unknown integration type: {}", stored_state.integration_type))?;

    let token_response = provider
        .exchange_token(
            &code,
            &stored_state.redirect_uri,
            stored_state.code_verifier.as_deref(),
        )
        .await?;

    let expires_at = token_response.expires_in.map(|secs| {
        (chrono::Utc::now() + chrono::Duration::seconds(secs as i64)).to_rfc3339()
    });

    let input = CreateIntegrationInput {
        integration_type: stored_state.integration_type.clone(),
        name: format!(
            "{}",
            IntegrationRegistry::new()
                .get_meta(&stored_state.integration_type)
                .map(|m| m.name)
                .unwrap_or("Integration")
        ),
        config: crate::integrations::models::IntegrationConfig {
            access_token: Some(token_response.access_token),
            refresh_token: token_response.refresh_token,
            expires_at,
            scopes: token_response
                .scope
                .map(|s| s.split_whitespace().map(String::from).collect()),
            ..Default::default()
        },
        permissions: Some(crate::integrations::models::IntegrationPermissions {
            read: true,
            write: false,
            delete: false,
            admin: false,
        }),
        autonomy_mode: Some("manual".to_string()),
        linking_workflow: Some("lazy".to_string()),
        sync_interval_minutes: Some(15),
    };

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut integration = repository::create_integration(&conn, input)?;
    repository::update_integration_status(&conn, &integration.id, "connected", None)?;
    integration.status = "connected".to_string();

    Ok(integration)
}

#[tauri::command]
pub async fn refresh_integration_token(
    state: State<'_, AppState>,
    id: String,
) -> Result<Integration, String> {
    let integration = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        repository::get_integration(&conn, &id)?
            .ok_or_else(|| format!("Integration {} not found", id))?
    };

    let refresh_token = integration
        .config
        .refresh_token
        .as_ref()
        .ok_or("No refresh token available")?;

    let provider = get_provider(&integration.integration_type)
        .ok_or_else(|| format!("Unknown integration type: {}", integration.integration_type))?;

    let token_response = provider.refresh_token(refresh_token).await?;

    let expires_at = token_response.expires_in.map(|secs| {
        (chrono::Utc::now() + chrono::Duration::seconds(secs as i64)).to_rfc3339()
    });

    let mut new_config = integration.config.clone();
    new_config.access_token = Some(token_response.access_token);
    if let Some(new_refresh) = token_response.refresh_token {
        new_config.refresh_token = Some(new_refresh);
    }
    new_config.expires_at = expires_at;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_integration(
        &conn,
        UpdateIntegrationInput {
            id: id.clone(),
            config: Some(new_config),
            status: Some("connected".to_string()),
            ..Default::default()
        },
    )
}

#[tauri::command]
pub async fn sync_integration(
    state: State<'_, AppState>,
    id: String,
) -> Result<SyncState, String> {
    let integration = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        repository::update_integration_status(&conn, &id, "syncing", None)?;
        repository::get_integration(&conn, &id)?
            .ok_or_else(|| format!("Integration {} not found", id))?
    };

    let provider = get_provider(&integration.integration_type)
        .ok_or_else(|| format!("Unknown integration type: {}", integration.integration_type))?;

    let fetch_result = provider.fetch_data(&integration).await;

    match fetch_result {
        Ok(result) => {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            let mut items_synced = 0;
            for item in &result.items {
                if repository::upsert_cache_item(
                    &conn,
                    &id,
                    &item.external_type,
                    &item.external_id,
                    item.external_url.as_deref(),
                    &item.data,
                )
                .is_ok()
                {
                    items_synced += 1;
                }
            }
            repository::update_integration_last_sync(&conn, &id)?;
            repository::update_integration_status(&conn, &id, "connected", None)?;

            Ok(SyncState {
                integration_id: id,
                status: if result.errors.is_empty() {
                    "success".to_string()
                } else {
                    "partial".to_string()
                },
                last_sync: Some(chrono::Utc::now().to_rfc3339()),
                items_synced,
                items_new: 0,
                items_updated: 0,
                errors: result.errors,
            })
        }
        Err(e) => {
            let conn = state.db.lock().map_err(|err| err.to_string())?;
            repository::update_integration_status(&conn, &id, "error", Some(&e))?;
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_sync_status(state: State<AppState>, id: String) -> Result<Option<SyncState>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let integration = repository::get_integration(&conn, &id)?;

    integration
        .map(|i| SyncState {
            integration_id: i.id,
            status: i.status,
            last_sync: i.last_sync,
            items_synced: 0,
            items_new: 0,
            items_updated: 0,
            errors: i.error_message.map(|e| vec![e]).unwrap_or_default(),
        })
        .map(Some)
        .ok_or_else(|| format!("Integration {} not found", id))
        .map(|opt| opt)
}

#[tauri::command]
pub fn clear_integration_cache(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::clear_cache(&conn, &id)
}

#[tauri::command]
pub fn get_cached_items(
    state: State<AppState>,
    integration_id: String,
    external_type: Option<String>,
) -> Result<Vec<IntegrationCache>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_cached_items(&conn, &integration_id, external_type.as_deref())
}

#[tauri::command]
pub fn create_integration_link(
    state: State<AppState>,
    input: CreateLinkInput,
) -> Result<IntegrationLink, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_link(&conn, input)
}

#[tauri::command]
pub fn get_links_for_task(
    state: State<AppState>,
    task_id: String,
) -> Result<Vec<IntegrationLink>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_links_for_local(&conn, "task", &task_id)
}

#[tauri::command]
pub fn get_links_for_meeting(
    state: State<AppState>,
    meeting_id: String,
) -> Result<Vec<IntegrationLink>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_links_for_local(&conn, "meeting", &meeting_id)
}

#[tauri::command]
pub fn unlink_integration_item(state: State<AppState>, link_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::delete_link(&conn, &link_id)
}

#[tauri::command]
pub fn get_slack_socket_status(
    state: State<AppState>,
) -> Result<crate::integrations::slack_socket::SocketModeStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let integrations = repository::list_integrations(&conn)?;
    let slack_integration = integrations.iter().find(|i| i.integration_type == "slack");

    match slack_integration {
        Some(integration) => {
            let app_token_configured = integration.config.app_token
                .as_ref()
                .map(|t| t.starts_with("xapp-"))
                .unwrap_or(false);

            let socket_mode_enabled = integration.config.socket_mode_enabled.unwrap_or(false);

            Ok(crate::integrations::slack_socket::SocketModeStatus {
                connected: socket_mode_enabled && app_token_configured,
                app_token_configured,
                last_event_at: None,
                reconnect_count: 0,
            })
        }
        None => Ok(crate::integrations::slack_socket::SocketModeStatus {
            connected: false,
            app_token_configured: false,
            last_event_at: None,
            reconnect_count: 0,
        }),
    }
}

#[tauri::command]
pub fn detect_slack_action_items(
    text: String,
    bot_user_id: Option<String>,
) -> Vec<String> {
    crate::integrations::slack_socket::detect_action_items(
        &text,
        bot_user_id.as_deref(),
    )
    .iter()
    .map(|item| item.as_str().to_string())
    .collect()
}

#[tauri::command]
pub fn agent_integration_write(
    state: State<AppState>,
    integration_id: String,
    action_type: String,
    action_config: String,
) -> Result<IntegrationWriteResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let action = risk::ActionType::from_str(&action_type).unwrap_or(risk::ActionType::Update);

    let destination = DestinationType::External;

    let approval_id = evaluate_integration_write(
        &conn,
        &integration_id,
        action,
        destination,
        &format!("integration:{}", action_type),
        &action_config,
    )?;

    if let Some(id) = approval_id {
        return Ok(IntegrationWriteResult {
            success: false,
            queued_for_approval: true,
            approval_id: Some(id),
            result: None,
        });
    }

    Ok(IntegrationWriteResult {
        success: true,
        queued_for_approval: false,
        approval_id: None,
        result: None,
    })
}

// ─── Project Mapping Commands ────────────────────────────────────────────────

#[tauri::command]
pub async fn create_project_mapping(
    integration_id: String,
    external_key: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<mapping::IntegrationProjectMapping, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    mapping::create_mapping(&conn, &integration_id, &external_key, &project_id)
}

#[tauri::command]
pub async fn get_project_mappings(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<mapping::IntegrationProjectMapping>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    mapping::get_mappings_for_project(&conn, &project_id)
}

#[tauri::command]
pub async fn get_integration_mappings(
    integration_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<mapping::IntegrationProjectMapping>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    mapping::get_mappings_for_integration(&conn, &integration_id)
}

#[tauri::command]
pub async fn delete_project_mapping(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    mapping::delete_mapping(&conn, &id)
}

#[tauri::command]
pub async fn get_integration_cache_for_project(
    project_id: String,
    integration_type: Option<String>,
    item_type: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<IntegrationCache>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_cached_items_for_project(
        &conn,
        &project_id,
        integration_type.as_deref(),
        item_type.as_deref(),
        limit,
    )
}

#[tauri::command]
pub async fn search_cached_integration_items(
    query: String,
    project_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<IntegrationCache>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::search_integration_cache(&conn, &query, project_id.as_deref(), limit)
}
