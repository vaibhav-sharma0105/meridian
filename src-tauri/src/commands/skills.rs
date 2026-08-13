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
        autonomy_mode: None,
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

#[tauri::command]
pub async fn create_skill_from_conversation(
    description: String,
    conversation_context: Option<Vec<Value>>,
    source_conversation_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let client = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills::get_ai_client(&conn)?
    };

    // Generate skill from description using LLM
    let extracted = skills::chat_extract::generate_skill_from_chat(
        &client,
        &description,
        conversation_context.as_deref(),
    ).await?;

    // Convert to CreateSkillInput
    let input = skills::chat_extract::convert_to_skill_input(
        &extracted,
        source_conversation_id.as_deref(),
    );

    // Create the skill
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    skills_repo::create_skill(&conn, &input)
}

#[tauri::command]
pub async fn detect_skill_pattern(
    conversation_history: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<skills::chat_extract::PatternDetection, String> {
    let client = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills::get_ai_client(&conn)?
    };

    skills::chat_extract::detect_pattern(&client, &conversation_history).await
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

// ─── Skill Sync (Phase 9) ───────────────────────────────────────────────────

#[tauri::command]
pub async fn list_importable_skills(
    integration_id: String,
    owner: String,
    repo: String,
    state: State<'_, AppState>,
) -> Result<Vec<skills::sync::ImportableSkill>, String> {
    let access_token = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let integration = crate::integrations::repository::get_integration(&conn, &integration_id)?
            .ok_or("Integration not found")?;

        if integration.integration_type != "github" {
            return Err("Only GitHub integrations support skill sync".to_string());
        }

        integration.config.access_token
            .ok_or("No access token for this integration")?
    };

    skills::sync::list_importable_skills_from_repo(&access_token, &owner, &repo).await
}

#[tauri::command]
pub async fn import_skill_from_repo(
    integration_id: String,
    skill_path: String,
    local_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Skill, String> {
    let (access_token, owner, repo) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let integration = crate::integrations::repository::get_integration(&conn, &integration_id)?
            .ok_or("Integration not found")?;

        if integration.integration_type != "github" {
            return Err("Only GitHub integrations support skill sync".to_string());
        }

        let token = integration.config.access_token
            .ok_or("No access token")?;

        // Parse owner/repo from skill_path or config
        // Expected format: "owner/repo/.claude/skills/skill-name"
        let parts: Vec<&str> = skill_path.splitn(3, '/').collect();
        if parts.len() < 3 {
            return Err("Invalid skill path format".to_string());
        }

        (token, parts[0].to_string(), parts[1].to_string())
    };

    // Extract just the skill directory path
    let skill_dir = skill_path.splitn(3, '/').nth(2)
        .ok_or("Invalid skill path")?;

    // Fetch content from GitHub
    let (content, scripts) = skills::sync::fetch_skill_content_from_repo(
        &access_token, &owner, &repo, skill_dir
    ).await?;

    // Determine local name
    let name = local_name.unwrap_or_else(|| {
        skill_dir.rsplit('/').next().unwrap_or("imported-skill").to_string()
    });

    // Security: Validate skill name to prevent path traversal
    skills::sync::validate_skill_name(&name)?;

    // Check for name conflict
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        if skills::sync::check_name_conflict(&conn, &name)? {
            return Err(format!("A skill named '{}' already exists", name));
        }
    }

    // Security: Validate script names
    for (script_name, _) in &scripts {
        skills::sync::validate_script_name(script_name)?;
    }

    // Save locally
    let scripts_vec: Vec<(String, String)> = scripts;
    skills::sync::save_skill_locally(&name, &content, Some(&scripts_vec))?;

    // Compute content hash
    let content_hash = skills::sync::compute_content_hash(&content);

    // Get remote commit
    let remote_commit = skills::sync::get_repo_head_commit(&access_token, &owner, &repo).await?;

    // Create skill in database
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let input = CreateSkillInput {
        name: name.clone(),
        description: Some(format!("Imported from {}/{}", owner, repo)),
        trigger_type: "manual".to_string(),
        trigger_config: None,
        context_config: None,
        action_config: None,
        approval_mode: Some("notify".to_string()),
        category: Some("imported".to_string()),
        icon: Some("📥".to_string()),
        tags: Some(vec!["synced".to_string()]),
        is_builtin: false,
        shared: false,
    };

    let skill = skills_repo::create_skill(&conn, &input)?;

    // Update sync info
    skills::sync::update_skill_sync_info(
        &conn,
        &skill.id,
        Some(&format!("github:{}/{}", owner, repo)),
        Some(skill_dir),
        Some(&remote_commit),
        Some(&content_hash),
    )?;

    skills_repo::get_skill(&conn, &skill.id)
}

#[tauri::command]
pub async fn check_skill_updates(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<skills::sync::UpdateStatus, String> {
    let sync_info = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills::sync::get_skill_sync_info(&conn, &skill_id)?
            .ok_or("Skill not found")?
    };

    if sync_info.sync_source.is_none() {
        return Ok(skills::sync::UpdateStatus::NotSynced);
    }

    // Parse sync_source to get owner/repo
    let source = sync_info.sync_source.as_ref().unwrap();
    if !source.starts_with("github:") {
        return Err("Unsupported sync source".to_string());
    }

    let repo_path = &source[7..]; // Remove "github:" prefix
    let parts: Vec<&str> = repo_path.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid sync source format".to_string());
    }

    // Get access token from GitHub integration
    let access_token = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let integrations = crate::integrations::repository::list_integrations(&conn)?;
        integrations.iter()
            .find(|i| i.integration_type == "github" && i.status == "connected")
            .and_then(|i| i.config.access_token.clone())
            .ok_or("No connected GitHub integration")?
    };

    // Get remote commit
    let remote_commit = skills::sync::get_repo_head_commit(&access_token, parts[0], parts[1]).await?;

    // Get current content hash
    let skill = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills_repo::get_skill(&conn, &skill_id)?
    };

    let current_hash = if let Some(content) = skills::sync::read_skill_content(&skill_id, &skill.name)? {
        skills::sync::compute_content_hash(&content)
    } else {
        sync_info.content_hash.clone().unwrap_or_default()
    };

    Ok(skills::sync::check_update_status(&sync_info, &remote_commit, &current_hash))
}

#[tauri::command]
pub async fn sync_skill(
    skill_id: String,
    strategy: String,
    state: State<'_, AppState>,
) -> Result<skills::sync::SyncResult, String> {
    // Get skill info and token first, then drop the lock
    let (skill_name, sync_source, sync_path, sync_commit, content_hash, access_token) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let skill = skills_repo::get_skill(&conn, &skill_id)?;

        // Get access token from GitHub integration
        let integrations = crate::integrations::repository::list_integrations(&conn)?;
        let token = integrations.iter()
            .find(|i| i.integration_type == "github" && i.status == "connected")
            .and_then(|i| i.config.access_token.clone())
            .ok_or("No connected GitHub integration")?;

        (
            skill.name,
            skill.sync_source,
            skill.sync_path,
            skill.sync_commit,
            skill.content_hash,
            token,
        )
    };

    let sync_source = sync_source.ok_or("Skill is not synced from a remote source")?;
    if !sync_source.starts_with("github:") {
        return Err("Only GitHub sync sources are supported".to_string());
    }

    let repo_path = &sync_source[7..];
    let parts: Vec<&str> = repo_path.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid sync source format".to_string());
    }
    let (owner, repo) = (parts[0], parts[1]);

    let skill_path = sync_path.ok_or("Skill has no sync path")?;

    // Get current local content
    let local_content = skills::sync::read_skill_content(&skill_id, &skill_name)?
        .unwrap_or_default();
    let local_hash = skills::sync::compute_content_hash(&local_content);

    // Build sync info
    let sync_info = skills::sync::SkillSyncInfo {
        skill_id: skill_id.clone(),
        sync_source: Some(sync_source.clone()),
        sync_path: Some(skill_path.clone()),
        sync_commit,
        last_sync_check: None,
        content_hash,
    };

    // Get remote state
    let remote_commit = skills::sync::get_repo_head_commit(&access_token, owner, repo).await?;
    let status = skills::sync::check_update_status(&sync_info, &remote_commit, &local_hash);

    let sync_strategy = match strategy.as_str() {
        "keep_local" => skills::sync::SyncStrategy::KeepLocal,
        "use_remote" => skills::sync::SyncStrategy::UseRemote,
        _ => skills::sync::SyncStrategy::Manual,
    };

    // Handle each status
    match status {
        skills::sync::UpdateStatus::UpToDate => {
            Ok(skills::sync::SyncResult {
                success: true,
                action_taken: "already_up_to_date".to_string(),
                new_commit: None,
                new_content_hash: None,
                trust_revoked: false,
            })
        }
        skills::sync::UpdateStatus::NotSynced => {
            Err("Skill is not configured for sync".to_string())
        }
        skills::sync::UpdateStatus::UpdateAvailable { .. } |
        skills::sync::UpdateStatus::LocalModified |
        skills::sync::UpdateStatus::Conflict { .. } => {
            match sync_strategy {
                skills::sync::SyncStrategy::Manual => {
                    Ok(skills::sync::SyncResult {
                        success: false,
                        action_taken: "manual_resolution_required".to_string(),
                        new_commit: None,
                        new_content_hash: None,
                        trust_revoked: false,
                    })
                }
                skills::sync::SyncStrategy::KeepLocal => {
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    skills::sync::update_skill_sync_info(&conn, &skill_id, None, None, Some(&remote_commit), Some(&local_hash))?;
                    Ok(skills::sync::SyncResult {
                        success: true,
                        action_taken: "kept_local".to_string(),
                        new_commit: Some(remote_commit),
                        new_content_hash: Some(local_hash),
                        trust_revoked: false,
                    })
                }
                skills::sync::SyncStrategy::UseRemote => {
                    // Fetch remote content
                    let (content, scripts) = skills::sync::fetch_skill_content_from_repo(
                        &access_token, owner, repo, &skill_path
                    ).await?;

                    // Validate names before saving to prevent path traversal
                    skills::sync::validate_skill_name(&skill_name)?;
                    for (script_name, _) in &scripts {
                        skills::sync::validate_script_name(script_name)?;
                    }

                    let scripts_slice: Vec<(String, String)> = scripts;
                    skills::sync::save_skill_locally(&skill_name, &content, Some(&scripts_slice))?;

                    let new_hash = skills::sync::compute_content_hash(&content);

                    // Update DB
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    skills::sync::update_skill_sync_info(&conn, &skill_id, None, None, Some(&remote_commit), Some(&new_hash))?;

                    // Revoke trust
                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "UPDATE skills SET trust_state = 'untrusted', trust_granted_at = NULL, updated_at = ?2 WHERE id = ?1",
                        rusqlite::params![skill_id, now],
                    ).map_err(|e| e.to_string())?;

                    Ok(skills::sync::SyncResult {
                        success: true,
                        action_taken: "updated_from_remote".to_string(),
                        new_commit: Some(remote_commit),
                        new_content_hash: Some(new_hash),
                        trust_revoked: true,
                    })
                }
            }
        }
    }
}

#[tauri::command]
pub async fn grant_skill_trust(
    skill_id: String,
    network_mode: String,
    allowlist: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Validate network_mode
    if !["none", "allowlist", "full"].contains(&network_mode.as_str()) {
        return Err(format!("Invalid network_mode: {}. Must be 'none', 'allowlist', or 'full'", network_mode));
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let allowlist_json = allowlist.map(|list| serde_json::to_string(&list).unwrap_or_default());

    conn.execute(
        "UPDATE skills SET trust_state = 'trusted', trust_granted_at = ?2, network_mode = ?3, network_allowlist = ?4, updated_at = ?2 WHERE id = ?1",
        rusqlite::params![skill_id, now, network_mode, allowlist_json],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn revoke_skill_trust(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE skills SET trust_state = 'revoked', updated_at = ?2 WHERE id = ?1",
        rusqlite::params![skill_id, now],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_skill_trust_state(
    skill_id: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let result: (Option<String>, Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT trust_state, trust_granted_at, network_mode, network_allowlist FROM skills WHERE id = ?1",
            [&skill_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "trust_state": result.0.unwrap_or_else(|| "untrusted".to_string()),
        "trust_granted_at": result.1,
        "network_mode": result.2.unwrap_or_else(|| "none".to_string()),
        "network_allowlist": result.3,
    }))
}

#[tauri::command]
pub async fn execute_skill_sandboxed(
    skill_id: String,
    script_name: String,
    inputs: Value,
    run_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<skills::sandbox::SandboxExecutionResult, String> {
    // Security: Validate script name to prevent path traversal
    skills::sync::validate_script_name(&script_name)?;

    // Get skill and verify trust
    let skill = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        skills_repo::get_skill(&conn, &skill_id)?
    };

    // Check trust state from skill model
    let trust = skill.trust_state.as_deref().unwrap_or("untrusted");
    if trust != "trusted" {
        return Err(format!(
            "Skill '{}' is not trusted. Grant trust before executing scripts.",
            skill.name
        ));
    }

    // Validate skill name for path safety
    skills::sync::validate_skill_name(&skill.name)?;

    // Get skill path and verify it's within the skills directory
    let skills_dir = skills::sync::get_meridian_skills_dir()?;
    let skill_path = skills_dir.join(&skill.name);

    // Security: Verify the path is actually within skills_dir (prevent path traversal)
    let canonical_skills_dir = skills_dir.canonicalize()
        .map_err(|e| format!("Failed to canonicalize skills dir: {}", e))?;
    let canonical_skill_path = skill_path.canonicalize()
        .map_err(|_| format!("Skill directory not found: {}", skill_path.display()))?;

    if !canonical_skill_path.starts_with(&canonical_skills_dir) {
        return Err("Invalid skill path: path traversal detected".to_string());
    }

    // Build sandbox config from skill settings
    let net_mode = skill.network_mode.as_deref().unwrap_or("none");
    let sandbox_network = match net_mode {
        "none" => skills::sandbox::NetworkMode::None,
        "allowlist" => {
            // Note: Allowlist enforcement depends on backend capabilities
            // Docker doesn't natively support allowlists, so we fall back to None for safety
            let backend = skills::sandbox::detect_backend();
            match backend {
                skills::sandbox::SandboxBackend::Docker => {
                    // Docker can't enforce allowlist, use None for safety
                    skills::sandbox::NetworkMode::None
                }
                _ => {
                    let hosts: Vec<String> = skill.network_allowlist
                        .as_ref()
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or_default();
                    if hosts.is_empty() {
                        skills::sandbox::NetworkMode::None
                    } else {
                        skills::sandbox::NetworkMode::Allowlist(hosts)
                    }
                }
            }
        }
        "full" => skills::sandbox::NetworkMode::Full,
        _ => skills::sandbox::NetworkMode::None,
    };

    let config = skills::sandbox::SandboxConfig {
        timeout_secs: 60,
        memory_mb: 512,
        network_mode: sandbox_network,
    };

    // Execute in sandbox
    let result = skills::sandbox::execute_in_sandbox(
        &canonical_skill_path,
        &script_name,
        &inputs,
        &config,
    ).await?;

    // Record execution in database
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        // Store output files in skill_outputs table with run_id
        for file in &result.output_files {
            let _ = conn.execute(
                "INSERT INTO skill_outputs (id, skill_id, skill_run_id, file_name, file_path, file_size, mime_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    skill_id,
                    run_id,
                    file.name,
                    file.path.to_string_lossy().to_string(),
                    file.size as i64,
                    file.mime_type,
                    now,
                ],
            );
        }

        // Surface produced files in the Message Center so they are browsable and
        // so orphan cleanup has references to track. Paths are stored relative to
        // `created_files/` where possible — absolute paths would not survive a
        // restore onto a machine with a different home directory.
        if !result.output_files.is_empty() {
            let created_files_root = dirs_next::home_dir()
                .map(|h| h.join(".meridian").join("created_files"));

            let file_refs: Vec<String> = result
                .output_files
                .iter()
                .map(|f| {
                    created_files_root
                        .as_ref()
                        .and_then(|root| f.path.strip_prefix(root).ok())
                        .map(|rel| rel.to_string_lossy().to_string())
                        .unwrap_or_else(|| f.path.to_string_lossy().to_string())
                })
                .collect();

            let succeeded = result.exit_code == 0;
            let output_preview = if result.stdout.trim().is_empty() {
                None
            } else {
                Some(result.stdout.clone())
            };

            let _ = crate::messages::repository::create_message(
                &conn,
                crate::messages::models::CreateMessageInput {
                    project_id: None,
                    message_type: "skill_result".to_string(),
                    title: format!(
                        "{} — {} file{}{}",
                        skill.name,
                        file_refs.len(),
                        if file_refs.len() == 1 { "" } else { "s" },
                        if succeeded { "" } else { " (failed)" }
                    ),
                    content: output_preview,
                    source_id: run_id.clone(),
                    source_type: Some("skill_run".to_string()),
                    // Generated files are always worth keeping — this mirrors the
                    // AiChat/has_files auto-pin rule in messages::routing.
                    auto_pinned: Some(true),
                    pinned_reason: Some("file_attachment".to_string()),
                    file_refs: Some(file_refs),
                },
            );
        }
    }

    Ok(result)
}
