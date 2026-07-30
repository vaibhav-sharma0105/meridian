use serde::{Deserialize, Serialize};
use tauri::State;

use crate::governance::{
    approval, autonomy, models, repository, risk, undo,
};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluateActionInput {
    pub action_type: String,
    pub destination: String,
    pub content: Option<String>,
    pub integration_id: Option<String>,
    pub skill_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApprovalInput {
    pub action_type: String,
    pub action_config: String,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub risk_level: String,
    pub autonomy_mode: String,
    pub context: Option<String>,
    pub timeout_minutes: Option<i64>,
}

#[tauri::command]
pub fn evaluate_action(
    state: State<AppState>,
    input: EvaluateActionInput,
) -> Result<models::ApprovalDecision, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let action_type = risk::classify_action(&input.action_type);
    let destination = risk::classify_destination(&input.destination);
    let content_risk = input
        .content
        .as_deref()
        .map(risk::classify_content)
        .unwrap_or(risk::ContentRisk::Normal);

    let risk_score = risk::calculate_risk(action_type, destination, content_risk);

    let context = autonomy::AutonomyContext {
        integration_id: input.integration_id,
        skill_id: input.skill_id,
    };

    autonomy::AutonomyController::evaluate_action(&conn, &context, risk_score.risk_level)
}

#[tauri::command]
pub fn get_autonomy_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    autonomy::get_autonomy_setting(&conn, &key)
}

#[tauri::command]
pub fn set_autonomy_setting(
    state: State<AppState>,
    key: String,
    value: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    autonomy::set_autonomy_setting(&conn, &key, value.as_deref())
}

#[tauri::command]
pub fn get_pending_approvals(
    state: State<AppState>,
    status: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<models::PendingApproval>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::get_pending_approvals(&conn, status.as_deref(), limit)
}

#[tauri::command]
pub fn get_pending_approval(
    state: State<AppState>,
    id: String,
) -> Result<Option<models::PendingApproval>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::get_pending_approval(&conn, &id)
}

#[tauri::command]
pub fn approve_pending_action(
    state: State<AppState>,
    id: String,
) -> Result<models::PendingApproval, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::approve_action(&conn, &id, "user")
}

#[tauri::command]
pub fn reject_pending_action(
    state: State<AppState>,
    id: String,
    reason: Option<String>,
) -> Result<models::PendingApproval, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::reject_action(&conn, &id, "user", reason.as_deref())
}

#[tauri::command]
pub fn bulk_approve_actions(
    state: State<AppState>,
    ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::bulk_approve(&conn, &ids, "user")
}

#[tauri::command]
pub fn bulk_reject_actions(
    state: State<AppState>,
    ids: Vec<String>,
    reason: Option<String>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::bulk_reject(&conn, &ids, "user", reason.as_deref())
}

#[tauri::command]
pub fn get_pending_approval_count(state: State<AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::get_pending_count(&conn)
}

#[tauri::command]
pub fn create_pending_approval(
    state: State<AppState>,
    input: CreateApprovalInput,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let risk_level = models::RiskLevel::from_str(&input.risk_level)
        .ok_or_else(|| format!("Invalid risk level: {}", input.risk_level))?;
    let autonomy_mode = models::AutonomyMode::from_str(&input.autonomy_mode)
        .ok_or_else(|| format!("Invalid autonomy mode: {}", input.autonomy_mode))?;

    approval::queue_for_approval(
        &conn,
        &input.action_type,
        &input.action_config,
        input.source_type.as_deref(),
        input.source_id.as_deref(),
        risk_level,
        autonomy_mode,
        input.context.as_deref(),
        input.timeout_minutes,
    )
}

#[tauri::command]
pub fn get_action_history(
    state: State<AppState>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<models::ActionHistory>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    undo::get_action_history(&conn, entity_type.as_deref(), entity_id.as_deref(), limit)
}

#[tauri::command]
pub fn get_undoable_actions(
    state: State<AppState>,
    limit: Option<i32>,
) -> Result<Vec<models::ActionHistory>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    undo::get_undoable_actions(&conn, limit)
}

#[tauri::command]
pub fn undo_action(state: State<AppState>, action_id: String) -> Result<undo::UndoResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    undo::undo_action(&conn, &action_id)
}

#[tauri::command]
pub fn capture_action_state(
    state: State<AppState>,
    action_type: String,
    entity_type: String,
    entity_id: String,
    before_state: Option<String>,
    after_state: Option<String>,
    audit_log_id: Option<String>,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    undo::capture_action_state(
        &conn,
        &action_type,
        &entity_type,
        &entity_id,
        before_state.as_deref(),
        after_state.as_deref(),
        audit_log_id.as_deref(),
    )
}

#[tauri::command]
pub fn get_governance_metrics(
    state: State<AppState>,
    start_date: String,
    end_date: String,
    metric_type: Option<String>,
) -> Result<Vec<models::GovernanceMetrics>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_governance_metrics(&conn, &start_date, &end_date, metric_type.as_deref())
}

#[tauri::command]
pub fn create_risk_adjustment(
    state: State<AppState>,
    adjustment_type: String,
    target_type: String,
    target_id: String,
    risk_delta: i32,
    reason: Option<String>,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::create_risk_adjustment(
        &conn,
        &adjustment_type,
        &target_type,
        &target_id,
        risk_delta,
        reason.as_deref(),
    )
}

#[tauri::command]
pub fn get_risk_adjustment(
    state: State<AppState>,
    target_type: String,
    target_id: String,
) -> Result<Option<models::RiskAdjustment>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::get_risk_adjustment(&conn, &target_type, &target_id)
}

#[tauri::command]
pub fn delete_risk_adjustment(
    state: State<AppState>,
    target_type: String,
    target_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repository::delete_risk_adjustment(&conn, &target_type, &target_id)
}

#[tauri::command]
pub fn calculate_risk_level(
    action_str: String,
    destination_str: String,
    content: Option<String>,
) -> Result<String, String> {
    let level = risk::calculate_risk_level(&action_str, &destination_str, content.as_deref());
    Ok(level.as_str().to_string())
}
