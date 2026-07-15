use crate::patterns::models::{
    CommunicationStyleModelData, CreateObservationInput, PatternModel, SmartDefaults,
    SmartDefaultsModelData, WorkflowSequenceModelData, WorkflowSuggestion,
};
use crate::patterns::repository as repo;
use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PatternSummary {
    pub pattern_type: String,
    pub confidence: f64,
    pub observation_count: i64,
    pub last_updated: String,
}

#[tauri::command]
pub async fn get_pattern_summaries(
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<PatternSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let models = repo::get_pattern_models_for_project(&conn, project_id.as_deref())?;

    Ok(models
        .into_iter()
        .map(|m| PatternSummary {
            pattern_type: m.pattern_type,
            confidence: m.confidence,
            observation_count: m.observation_count,
            last_updated: m.last_updated,
        })
        .collect())
}

#[tauri::command]
pub async fn get_pattern_model(
    pattern_type: String,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<PatternModel, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_pattern_model_by_type(&conn, &pattern_type, project_id.as_deref())
}

#[tauri::command]
pub async fn get_workflow_suggestions(
    completed_task_id: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<WorkflowSuggestion>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let model = match repo::get_pattern_model_by_type(&conn, "workflow_sequence", Some(&project_id)) {
        Ok(m) => m,
        Err(_) => return Ok(vec![]),
    };

    if model.confidence < 0.5 {
        return Ok(vec![]);
    }

    let model_data: WorkflowSequenceModelData = serde_json::from_str(&model.model_data)
        .map_err(|e| format!("Failed to parse workflow model: {}", e))?;

    let task = crate::db::repositories::tasks::get_task(&conn, &completed_task_id)?;
    let task_keywords: Vec<String> = task
        .title
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|s| s.to_string())
        .collect();

    let mut suggestions = vec![];
    for seq in &model_data.sequences {
        if model_data.negative_sequences.contains(&format!("{}→{}", seq.trigger_action, seq.follow_action)) {
            continue;
        }

        let trigger_keywords: Vec<&str> = seq.trigger_action.split_whitespace().collect();
        let matches = trigger_keywords.iter().any(|k| task_keywords.contains(&k.to_lowercase()));

        if matches && seq.occurrence_count >= 3 {
            let confidence = (seq.occurrence_count as f64 / 10.0).min(1.0) * model.confidence;
            if confidence >= 0.5 {
                suggestions.push(WorkflowSuggestion {
                    trigger_task_id: completed_task_id.clone(),
                    suggested_action: seq.follow_action.clone(),
                    confidence,
                    sequence_id: format!("{}→{}", seq.trigger_action, seq.follow_action),
                });
            }
        }
    }

    Ok(suggestions)
}

#[tauri::command]
pub async fn dismiss_workflow_suggestion(
    sequence_id: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let _ = repo::insert_observation(
        &conn,
        CreateObservationInput {
            observation_type: "suggestion_dismissed".to_string(),
            entity_type: Some("workflow_sequence".to_string()),
            entity_id: Some(sequence_id.clone()),
            project_id: Some(project_id),
            context_data: json!({
                "sequence_id": sequence_id,
                "suggestion_type": "workflow"
            }),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn get_smart_defaults(
    task_title: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<SmartDefaults, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let model = match repo::get_pattern_model_by_type(&conn, "smart_defaults", Some(&project_id)) {
        Ok(m) if m.confidence >= 0.5 => m,
        _ => {
            return Ok(SmartDefaults {
                suggested_priority: None,
                priority_confidence: 0.0,
                suggested_assignee: None,
                assignee_confidence: 0.0,
                source: "none".to_string(),
            })
        }
    };

    let model_data: SmartDefaultsModelData = serde_json::from_str(&model.model_data)
        .map_err(|e| format!("Failed to parse smart defaults model: {}", e))?;

    let title_lower = task_title.to_lowercase();
    let keywords: Vec<&str> = title_lower.split_whitespace().collect();

    let mut priority_match: Option<(String, i64)> = None;
    for pattern in &model_data.priority_patterns {
        if keywords.iter().any(|k| k.contains(&pattern.keyword.to_lowercase())) {
            if priority_match.is_none() || pattern.occurrence_count > priority_match.as_ref().unwrap().1 {
                priority_match = Some((pattern.priority.clone(), pattern.occurrence_count));
            }
        }
    }

    let mut assignee_match: Option<(String, i64)> = None;
    for pattern in &model_data.assignee_patterns {
        if keywords.iter().any(|k| k.contains(&pattern.keyword.to_lowercase())) {
            if assignee_match.is_none() || pattern.occurrence_count > assignee_match.as_ref().unwrap().1 {
                assignee_match = Some((pattern.assignee.clone(), pattern.occurrence_count));
            }
        }
    }

    let project_default = model_data.project_defaults.get(&project_id);
    let (suggested_priority, priority_confidence, source) = match priority_match {
        Some((p, count)) => (Some(p), (count as f64 / 10.0).min(1.0) * model.confidence, "keyword"),
        None => match project_default.and_then(|d| d.default_priority.clone()) {
            Some(p) => (Some(p), model.confidence * 0.7, "project"),
            None => (None, 0.0, "none"),
        },
    };

    let (suggested_assignee, assignee_confidence) = match assignee_match {
        Some((a, count)) => (Some(a), (count as f64 / 10.0).min(1.0) * model.confidence),
        None => match project_default.and_then(|d| d.default_assignee.clone()) {
            Some(a) => (Some(a), model.confidence * 0.7),
            None => (None, 0.0),
        },
    };

    Ok(SmartDefaults {
        suggested_priority,
        priority_confidence,
        suggested_assignee,
        assignee_confidence,
        source: source.to_string(),
    })
}

#[tauri::command]
pub async fn get_communication_style(
    context: Option<String>,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<CommunicationStyleModelData>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let pattern_type = match context.as_deref() {
        Some("task_followup") => "communication_style_task_followup",
        Some("meeting_summary") => "communication_style_meeting_summary",
        _ => "communication_style",
    };

    let model = match repo::get_pattern_model_by_type(&conn, pattern_type, project_id.as_deref()) {
        Ok(m) if m.confidence >= 0.6 => m,
        _ => return Ok(None),
    };

    let model_data: CommunicationStyleModelData = serde_json::from_str(&model.model_data)
        .map_err(|e| format!("Failed to parse communication style model: {}", e))?;

    Ok(Some(model_data))
}

#[tauri::command]
pub async fn record_draft_edit(
    original_text: String,
    edited_text: String,
    context_type: Option<String>,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let original_len = original_text.len() as f64;
    let edited_len = edited_text.len() as f64;
    let length_delta = if original_len > 0.0 {
        (edited_len - original_len) / original_len
    } else {
        0.0
    };

    let _ = repo::insert_observation(
        &conn,
        CreateObservationInput {
            observation_type: "draft_edit".to_string(),
            entity_type: Some("ai_draft".to_string()),
            entity_id: None,
            project_id,
            context_data: json!({
                "original_text": original_text,
                "edited_text": edited_text,
                "length_delta": length_delta,
                "context_type": context_type.unwrap_or_else(|| "general".to_string())
            }),
        },
    );

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct LearningExport {
    pub version: String,
    pub exported_at: String,
    pub pattern_models: Vec<PatternModel>,
}

#[tauri::command]
pub async fn export_learning_data(state: State<'_, AppState>) -> Result<LearningExport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let models = repo::get_all_pattern_models(&conn)?;

    Ok(LearningExport {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        pattern_models: models,
    })
}

#[derive(Debug, Deserialize)]
pub struct LearningImport {
    pub version: String,
    pub pattern_models: Vec<PatternModel>,
}

#[tauri::command]
pub async fn import_learning_data(
    data: LearningImport,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    if data.version != "1.0" {
        return Err(format!("Unsupported import version: {}", data.version));
    }

    let mut imported = 0;
    for model in data.pattern_models {
        let model_data: serde_json::Value = serde_json::from_str(&model.model_data)
            .map_err(|e| format!("Invalid model_data JSON: {}", e))?;

        let _ = repo::upsert_pattern_model(
            &conn,
            crate::patterns::models::UpsertPatternModelInput {
                pattern_type: model.pattern_type,
                project_id: model.project_id,
                model_data,
                confidence: model.confidence,
                observation_count: model.observation_count,
            },
        )?;
        imported += 1;
    }

    Ok(imported)
}

#[tauri::command]
pub async fn reset_pattern_category(
    pattern_type: String,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::delete_pattern_model(&conn, &pattern_type, project_id.as_deref())
}

#[tauri::command]
pub async fn reset_all_learning(state: State<'_, AppState>) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let models_deleted = repo::delete_all_pattern_models(&conn)?;
    let _ = repo::delete_all_observations(&conn)?;
    Ok(models_deleted)
}
