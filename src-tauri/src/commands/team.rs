use tauri::State;
use crate::AppState;
use crate::team::{
    models::{TeamMember, CreateTeamMemberInput, UpdateTeamMemberInput, AssigneeSuggestion},
    repository,
    assignee,
};
use crate::integrations::{google, slack};

#[tauri::command]
pub async fn get_team_members(state: State<'_, AppState>) -> Result<Vec<TeamMember>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_all_team_members(&conn)
}

#[tauri::command]
pub async fn get_team_member(state: State<'_, AppState>, id: String) -> Result<Option<TeamMember>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_team_member(&conn, &id)
}

#[tauri::command]
pub async fn create_team_member(
    state: State<'_, AppState>,
    input: CreateTeamMemberInput,
) -> Result<TeamMember, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_team_member(&conn, &input)
}

#[tauri::command]
pub async fn update_team_member(
    state: State<'_, AppState>,
    input: UpdateTeamMemberInput,
) -> Result<TeamMember, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::update_team_member(&conn, &input)
}

#[tauri::command]
pub async fn delete_team_member(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::delete_team_member(&conn, &id)
}

#[tauri::command]
pub async fn compute_team_workloads(state: State<'_, AppState>) -> Result<Vec<(String, f64)>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::compute_all_workload_scores(&conn)
}

#[tauri::command]
pub async fn get_assignee_suggestions(
    state: State<'_, AppState>,
    task_title: String,
    task_description: Option<String>,
    project_id: Option<String>,
) -> Result<Vec<AssigneeSuggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    assignee::get_assignee_suggestions(&conn, &task_title, task_description.as_deref(), project_id.as_deref())
}

#[tauri::command]
pub async fn record_assignee_selection(
    state: State<'_, AppState>,
    selected_name: String,
    suggestions: Vec<AssigneeSuggestion>,
    was_override: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    assignee::record_assignee_selection(&conn, &selected_name, &suggestions, was_override)
}

#[derive(serde::Serialize)]
pub struct TeamSyncResult {
    pub added: i32,
    pub updated: i32,
    pub total: i32,
}

#[tauri::command]
pub async fn sync_team_from_slack(state: State<'_, AppState>) -> Result<TeamSyncResult, String> {
    // Get Slack integration
    let access_token = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let slack_integration = crate::integrations::repository::get_integration_by_type(&conn, "slack")?
            .ok_or("No Slack integration connected")?;

        slack_integration.config.access_token
            .ok_or("No access token in Slack integration")?
    };

    // Fetch members from Slack
    let slack_members = slack::fetch_workspace_members(&access_token).await?;

    let mut added = 0;
    let mut updated = 0;

    // Upsert each member
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        for member in &slack_members {
            let input = CreateTeamMemberInput {
                name: member.name.clone(),
                email: member.email.clone(),
                avatar_url: member.avatar_url.clone(),
                source: "slack".to_string(),
                source_id: Some(member.id.clone()),
                role: None,
                expertise: None,
                metadata: None,
            };

            // Check if exists
            let existing = repository::get_team_member_by_source(&conn, "slack", &member.id)?;

            if existing.is_some() {
                updated += 1;
            } else {
                added += 1;
            }

            repository::upsert_team_member(&conn, &input)?;
        }
    }

    Ok(TeamSyncResult {
        added,
        updated,
        total: slack_members.len() as i32,
    })
}

#[tauri::command]
pub async fn sync_team_from_google(state: State<'_, AppState>) -> Result<TeamSyncResult, String> {
    let (integration_id, access_token) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let google_integration = crate::integrations::repository::get_integration_by_type(&conn, "google")?
            .ok_or("No Google integration connected")?;

        let token = google_integration.config.access_token.clone()
            .ok_or("No access token in Google integration")?;
        (google_integration.id.clone(), token)
    };

    let google_members = google::fetch_workspace_members(&access_token).await?;

    let mut added = 0;
    let mut updated = 0;

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        for member in &google_members {
            let input = CreateTeamMemberInput {
                name: member.name.clone(),
                email: member.email.clone(),
                avatar_url: member.avatar_url.clone(),
                source: "google".to_string(),
                source_id: Some(member.id.clone()),
                role: None,
                expertise: None,
                metadata: None,
            };

            let existing = repository::get_team_member_by_source(&conn, "google", &member.id)?;

            if existing.is_some() {
                updated += 1;
            } else {
                added += 1;
            }

            repository::upsert_team_member(&conn, &input)?;
        }

        let _ = crate::integrations::repository::update_integration_last_sync(&conn, &integration_id);
    }

    Ok(TeamSyncResult {
        added,
        updated,
        total: google_members.len() as i32,
    })
}
