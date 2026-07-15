use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;

use crate::patterns::models::CreateObservationInput;
use crate::patterns::repository as patterns_repo;
use crate::skills::{
    self, approval, cron as skill_cron, repository as skills_repo, CreateSkillInput,
    CreateSkillRunInput, Skill, SkillFilters, SkillRun, SkillStats, UpdateSkillInput,
};
use crate::AppState;

#[tauri::command]
pub async fn create_skill(
    input: CreateSkillInput,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let skill = skills_repo::create_skill(&conn, &input)?;

    // Compute next_run_at for scheduled skills
    if skill.trigger_type == "schedule" {
        if let Some(trigger_config) = skill.get_trigger_config() {
            if let Some(ref cron_expr) = trigger_config.cron {
                let timezone = trigger_config.timezone.as_deref();
                if let Ok(next_run) = skill_cron::compute_next_run(cron_expr, timezone) {
                    let _ = skills_repo::update_next_run_at(&conn, &skill.id, &next_run);
                }
            }
        }
    }

    // Return the updated skill
    skills_repo::get_skill(&conn, &skill.id)
}

#[tauri::command]
pub async fn get_skill(id: String, state: State<'_, AppState>) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::get_skill(&conn, &id)
}

#[tauri::command]
pub async fn list_skills(
    shared: Option<bool>,
    category: Option<String>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<Skill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let filters = SkillFilters {
        enabled,
        shared,
        category,
        trigger_type: None,
        search: None,
    };
    skills_repo::list_skills(&conn, &filters)
}

#[tauri::command]
pub async fn update_skill(
    input: UpdateSkillInput,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::update_skill(&conn, &input)
}

#[tauri::command]
pub async fn delete_skill(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::delete_skill(&conn, &id)
}

#[tauri::command]
pub async fn toggle_skill_enabled(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Update only the enabled field
    let update = UpdateSkillInput {
        id: id.clone(),
        name: None,
        description: None,
        trigger_type: None,
        trigger_config: None,
        context_config: None,
        action_config: None,
        approval_mode: None,
        enabled: Some(enabled),
        shared: None,
        category: None,
        icon: None,
        tags: None,
    };

    let skill = skills_repo::update_skill(&conn, &update)?;

    // Recompute next_run_at when enabling a scheduled skill
    if enabled && skill.trigger_type == "schedule" {
        if let Some(trigger_config) = skill.get_trigger_config() {
            if let Some(ref cron_expr) = trigger_config.cron {
                let timezone = trigger_config.timezone.as_deref();
                if let Ok(next_run) = skill_cron::compute_next_run(cron_expr, timezone) {
                    let _ = skills_repo::update_next_run_at(&conn, &skill.id, &next_run);
                }
            }
        }
    }

    // Record pattern observation when disabling
    if !enabled {
        let _ = patterns_repo::insert_observation(
            &conn,
            CreateObservationInput {
                observation_type: "skill_disable".to_string(),
                entity_type: Some("skill".to_string()),
                entity_id: Some(id.clone()),
                project_id: None,
                context_data: json!({
                    "skill_name": skill.name,
                    "trigger_type": skill.trigger_type,
                }),
            },
        );
    }

    skills_repo::get_skill(&conn, &skill.id)
}

#[tauri::command]
pub async fn run_skill_manually(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<SkillRun, String> {
    // Get all data needed with the lock
    let (run_id, skill, context, action_config, ai_client) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        // Create a run for manual trigger
        let run = skills_repo::create_skill_run(
            &conn,
            &CreateSkillRunInput {
                skill_id: skill_id.clone(),
                trigger_type: "manual".to_string(),
                trigger_context: None,
            },
        )?;

        let skill = skills_repo::get_skill(&conn, &skill_id)?;
        skills_repo::update_run_status(&conn, &run.id, "running")?;

        let context = skills::build_context(&conn, &skill)?;
        let action_config = skill.get_action_config().unwrap_or_default();
        let ai_client = skills::get_ai_client(&conn)?;

        (run.id, skill, context, action_config, ai_client)
    };

    // Execute AI call asynchronously (without holding the lock)
    let needs_approval = skills::executor::check_needs_approval(&skill, &action_config);
    let action_type = action_config.action_type.as_deref().unwrap_or("summarize");
    let start = std::time::Instant::now();

    let result = match action_type {
        "summarize" => skills::execute_summarize_ai(&ai_client, &context, &action_config).await,
        "draft_message" => skills::execute_draft_ai(&ai_client, &context, &action_config).await,
        "create_tasks" => skills::execute_create_tasks_ai(&ai_client, &context, &action_config).await,
        "analyze" => skills::execute_analyze_ai(&ai_client, &context, &action_config).await,
        "custom" => {
            let ctx_config = skill.get_context_config().unwrap_or_default();
            skills::execute_custom_ai(&ai_client, &context, &action_config, &ctx_config).await
        }
        _ => Err(format!("Unknown action type: {}", action_type)),
    }?;

    let duration_ms = start.elapsed().as_millis() as i64;
    let exec_result = skills::ExecutionResult {
        output: result.0,
        duration_ms,
        pending_changes: result.1,
        needs_approval,
    };

    // Complete the run with new lock
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills::complete_skill_run(&conn, &run_id, &exec_result)?;

        // Record manual trigger observation for pattern learning
        let _ = patterns_repo::insert_observation(
            &conn,
            CreateObservationInput {
                observation_type: "skill_manual_trigger".to_string(),
                entity_type: Some("skill".to_string()),
                entity_id: Some(skill_id.clone()),
                project_id: skill.get_context_config().and_then(|c| c.project_id),
                context_data: json!({
                    "skill_name": skill.name,
                    "action_type": skill.get_action_config().and_then(|a| a.action_type).unwrap_or_default(),
                }),
            },
        );

        skills_repo::get_skill_run(&conn, &run_id)
    }
}

#[tauri::command]
pub async fn test_run_skill(skill_id: String, state: State<'_, AppState>) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the skill
    let skill = skills_repo::get_skill(&conn, &skill_id)?;

    // Build context only (no execution, no side effects)
    let context = skills::build_context(&conn, &skill)?;

    Ok(serde_json::json!({
        "skill_id": skill_id,
        "skill_name": skill.name,
        "context": context,
        "context_tasks_count": context.tasks.len(),
        "context_meetings_count": context.meetings.len(),
        "context_truncated": context.truncated,
        "action_type": skill.get_action_config().and_then(|c| c.action_type),
        "approval_mode": skill.approval_mode,
    }))
}

#[tauri::command]
pub async fn get_skill_runs(
    skill_id: String,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<SkillRun>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::list_skill_runs(
        &conn,
        &skill_id,
        status.as_deref(),
        limit.unwrap_or(20),
        offset.unwrap_or(0),
    )
}

#[tauri::command]
pub async fn get_skill_run(id: String, state: State<'_, AppState>) -> Result<SkillRun, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::get_skill_run(&conn, &id)
}

#[tauri::command]
pub async fn approve_skill_run(
    run_id: String,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::approve_skill_run(&conn, &run_id, project_id.as_deref())
}

#[tauri::command]
pub async fn reject_skill_run(
    run_id: String,
    reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    approval::reject_skill_run(&conn, &run_id, reason.as_deref())
}

#[tauri::command]
pub async fn clone_skill(
    skill_id: String,
    new_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get original skill
    let original = skills_repo::get_skill(&conn, &skill_id)?;

    // Create clone with new name
    let cloned_name = new_name.unwrap_or_else(|| format!("{} (Copy)", original.name));

    // Parse configs from the original skill
    let trigger_config = original.get_trigger_config();
    let context_config = original.get_context_config();
    let action_config = original.get_action_config();
    let tags = original.get_tags();
    let tags = if tags.is_empty() { None } else { Some(tags) };

    let input = CreateSkillInput {
        name: cloned_name,
        description: original.description.clone(),
        trigger_type: original.trigger_type.clone(),
        trigger_config,
        context_config,
        action_config,
        approval_mode: Some(original.approval_mode.clone()),
        category: original.category.clone(),
        icon: original.icon.clone(),
        tags,
        is_builtin: false,
        shared: false, // Cloned skills start unshared
    };

    skills_repo::create_skill(&conn, &input)
}

#[tauri::command]
pub async fn export_skill(skill_id: String, state: State<'_, AppState>) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let skill = skills_repo::get_skill(&conn, &skill_id)?;

    // Export skill as JSON (excluding internal fields)
    Ok(serde_json::json!({
        "name": skill.name,
        "description": skill.description,
        "trigger_type": skill.trigger_type,
        "trigger_config": skill.trigger_config,
        "context_config": skill.context_config,
        "action_config": skill.action_config,
        "approval_mode": skill.approval_mode,
        "category": skill.category,
        "icon": skill.icon,
        "tags": skill.tags,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0",
    }))
}

#[tauri::command]
pub async fn export_skill_to_directory(
    skill_md_content: String,
    skill_name: String,
) -> Result<String, String> {
    let handle = tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "macos")]
        let folder_path = {
            let output = std::process::Command::new("osascript")
                .args([
                    "-e",
                    "POSIX path of (choose folder with prompt \"Choose location to export skill\")",
                ])
                .output()
                .map_err(|e| format!("Failed to open folder picker: {}", e))?;

            if !output.status.success() {
                return Err("Export cancelled".to_string());
            }
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                return Err("Export cancelled".to_string());
            }
            PathBuf::from(path)
        };
        #[cfg(not(target_os = "macos"))]
        let folder_path = {
            rfd::FileDialog::new()
                .set_title("Choose location to export skill")
                .pick_folder()
                .ok_or_else(|| "Export cancelled".to_string())?
        };

        let slug = skill_name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
            .trim_matches('-')
            .to_string();

        let skill_dir = folder_path.join(&slug);
        std::fs::create_dir_all(&skill_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        let skill_file = skill_dir.join("skill.md");
        std::fs::write(&skill_file, &skill_md_content)
            .map_err(|e| format!("Failed to write skill.md: {}", e))?;

        Ok(skill_dir.to_string_lossy().to_string())
    });
    handle.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_skill(
    skill_json: Value,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Validate required fields
    let name = skill_json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required field: name")?;

    let trigger_type = skill_json
        .get("trigger_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing required field: trigger_type")?;

    // Parse configs from JSON
    let trigger_config = skill_json
        .get("trigger_config")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let context_config = skill_json
        .get("context_config")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let action_config = skill_json
        .get("action_config")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let tags = skill_json
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let input = CreateSkillInput {
        name: name.to_string(),
        description: skill_json
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from),
        trigger_type: trigger_type.to_string(),
        trigger_config,
        context_config,
        action_config,
        approval_mode: skill_json
            .get("approval_mode")
            .and_then(|v| v.as_str())
            .map(String::from),
        category: skill_json
            .get("category")
            .and_then(|v| v.as_str())
            .map(String::from),
        icon: skill_json
            .get("icon")
            .and_then(|v| v.as_str())
            .map(String::from),
        tags,
        is_builtin: false,
        shared: skill_json
            .get("shared")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    };

    skills_repo::create_skill(&conn, &input)
}

#[tauri::command]
pub async fn get_skill_stats(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<SkillStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::get_skill_stats(&conn, &skill_id)
}

#[tauri::command]
pub async fn record_skill_output_edit(
    skill_id: String,
    run_id: String,
    original_output: String,
    edited_output: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let _ = patterns_repo::insert_observation(
        &conn,
        CreateObservationInput {
            observation_type: "skill_output_edit".to_string(),
            entity_type: Some("skill_run".to_string()),
            entity_id: Some(run_id),
            project_id: None,
            context_data: json!({
                "skill_id": skill_id,
                "original_length": original_output.len(),
                "edited_length": edited_output.len(),
                "length_delta": edited_output.len() as i64 - original_output.len() as i64,
            }),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn initialize_builtin_skills(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::builtin::load_builtin_skills(&conn)
}

#[tauri::command]
pub async fn reset_builtin_skills(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::builtin::reset_builtin_skills(&conn)
}

#[tauri::command]
pub async fn extract_skill_from_chat(
    description: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let client = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills::get_ai_client(&conn)?
    };

    let messages = vec![
        json!({"role": "system", "content":
            "You are a skill extraction assistant. Given a natural language description of an automation, \
             extract a structured skill definition. Return ONLY valid JSON with these fields:\n\
             {\n\
               \"name\": \"short-kebab-case-name\",\n\
               \"description\": \"One-line description\",\n\
               \"trigger_type\": \"schedule\" | \"event\" | \"manual\",\n\
               \"trigger_config\": { \"cron\": \"0 9 * * 1\" } | { \"event_type\": \"task_completed\" } | {},\n\
               \"action_type\": \"summarize\" | \"draft_message\" | \"create_tasks\" | \"analyze\" | \"custom\",\n\
               \"system_prompt\": \"Instructions for the AI when executing this skill\",\n\
               \"approval_mode\": \"auto\" | \"notify\" | \"approve_first\"\n\
             }\n\n\
             Infer the best trigger and action from context. For schedules, produce valid cron expressions. \
             Output ONLY the JSON object, no markdown."}),
        json!({"role": "user", "content": description}),
    ];

    let response = client.chat_completion(messages, Some(500)).await?;

    let parsed: Value = serde_json::from_str(&response)
        .or_else(|_| {
            let trimmed = response.trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            serde_json::from_str(trimmed)
        })
        .map_err(|e| format!("Failed to parse skill extraction: {}", e))?;

    Ok(parsed)
}

// ─── Folder-based Skills ────────────────────────────────────────────────────

#[tauri::command]
pub async fn pick_folder_dialog() -> Result<Option<String>, String> {
    let handle = tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("osascript")
                .args([
                    "-e",
                    "POSIX path of (choose folder with prompt \"Select skill folder to install\")",
                ])
                .output()
                .map_err(|e| format!("Failed to open folder picker: {}", e))?;

            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(path))
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let folder = rfd::FileDialog::new()
                .set_title("Select skill folder to install")
                .pick_folder();
            Ok(folder.map(|p| p.to_string_lossy().to_string()))
        }
    });
    handle.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_skill_folders(state: State<'_, AppState>) -> Result<Vec<skills::SkillFolder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::folders::list_skill_folders_with_state(&conn)
}

#[tauri::command]
pub async fn get_skill_folder(folder_name: String) -> Result<skills::SkillFolder, String> {
    skills::folders::get_skill_folder(&folder_name)
}

#[tauri::command]
pub async fn toggle_folder_skill_enabled(
    folder_name: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills::folders::toggle_folder_skill_enabled(&conn, &folder_name, enabled)
}

#[tauri::command]
pub async fn install_skill_folder(source_path: String) -> Result<skills::SkillFolder, String> {
    skills::folders::install_skill_folder(&source_path)
}

#[tauri::command]
pub async fn delete_skill_folder(folder_name: String) -> Result<(), String> {
    skills::folders::delete_skill_folder(&folder_name)
}

#[tauri::command]
pub async fn read_skill_file(folder_name: String, file_path: String) -> Result<String, String> {
    skills::folders::read_skill_file(&folder_name, &file_path)
}

#[tauri::command]
pub async fn execute_skill_script(
    folder_name: String,
    script_path: String,
) -> Result<String, String> {
    skills::folders::execute_skill_script(&folder_name, &script_path)
}
