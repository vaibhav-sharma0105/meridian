use crate::db::repositories::{meetings as repo, projects as proj_repo, tasks as task_repo};
use crate::models::meeting::{CreateMeetingInput, Meeting};
use crate::AppState;
use serde_json::{json, Value};
use tauri::State;

/// Core ingest logic — shared by the manual ingest command and the auto-import pipeline.
///
/// `enforce_min_words`: pass `true` for user-entered transcripts (manual ingest),
/// `false` for machine-generated content like Zoom AI summaries.
pub async fn ingest_meeting_core(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: &State<'_, AppState>,
) -> Result<Value, String> {
    ingest_meeting_core_inner(project_id, title, platform, raw_transcript, attendees, duration_minutes, meeting_at, state, true).await
}

pub async fn ingest_meeting_core_from_connector(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: &State<'_, AppState>,
) -> Result<Value, String> {
    ingest_meeting_core_inner(project_id, title, platform, raw_transcript, attendees, duration_minutes, meeting_at, state, false).await
}

async fn ingest_meeting_core_inner(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: &State<'_, AppState>,
    enforce_min_words: bool,
) -> Result<Value, String> {
    // Validate transcript length (skipped for auto-imported connector content)
    if enforce_min_words {
        let word_count = raw_transcript.split_whitespace().count();
        if word_count < 50 {
            return Err("Transcript too short — paste the full meeting text (minimum 50 words)".to_string());
        }
    }

    let (meeting, project_info) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        // Create meeting record
        let input = CreateMeetingInput {
            project_id: project_id.clone(),
            title: title.clone(),
            platform: platform.clone(),
            raw_transcript: raw_transcript.clone(),
            attendees: attendees.clone(),
            duration_minutes,
            meeting_at,
        };
        let meeting = repo::create_meeting(&conn, &input)?;

        // Get project name for context
        let project = proj_repo::get_project(&conn, &project_id)?
            .ok_or_else(|| "Project not found".to_string())?;

        // Get existing open tasks for duplicate detection
        let existing_tasks = task_repo::get_open_tasks_for_project(&conn, &project_id)?;
        let existing_titles: Vec<String> = existing_tasks.iter().map(|t| t.title.clone()).collect();

        // Get all projects for cross-project task routing
        let all_projects = proj_repo::get_all_projects(&conn)?;
        let all_project_names: Vec<String> = all_projects.iter().map(|p| p.name.clone()).collect();

        (meeting, (project, existing_titles, all_project_names, all_projects))
    };

    let (project, existing_titles, all_project_names, all_projects) = project_info;

    // Get AI settings + API key in one DB lock (no Keychain access)
    let (ai_settings, api_key) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let settings = crate::db::repositories::ai_settings::get_active_settings(&conn)?
            .ok_or_else(|| {
                "No AI provider configured — please set up your AI provider in Settings"
                    .to_string()
            })?;
        let key = crate::commands::ai::get_api_key_from_db(&conn, &settings.label);
        (settings, key)
    };
    let ai_settings = ai_settings;

    let litellm = crate::commands::ai::get_litellm_client_pub(&ai_settings, &api_key);

    // Extract tasks
    let extraction = crate::ai::extractor::extract_tasks(
        &litellm,
        &raw_transcript,
        &project.name,
        &existing_titles,
        &all_project_names,
    )
    .await?;

    // Calculate health score
    let wc = crate::utils::health_score::count_words(&raw_transcript);
    let had_agenda = crate::utils::health_score::detect_agenda(&raw_transcript);
    let health = crate::utils::health_score::calculate_health_score(
        had_agenda,
        extraction.health.decisions_count,
        extraction.health.tasks_count,
        extraction.health.attendees_count,
        0.0,
        wc,
    );

    let health_json = serde_json::to_string(&health).unwrap_or_default();
    let attendees_str = if let Some(att) = &attendees {
        att.clone()
    } else {
        extraction.attendees.join(", ")
    };
    let decisions_str = if extraction.decisions.is_empty() {
        None
    } else {
        Some(extraction.decisions.join("\n"))
    };

    // Update meeting with summary and health
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        repo::update_meeting_summary(
            &conn,
            &meeting.id,
            &extraction.summary,
            decisions_str.as_deref(),
            health.total,
            &health_json,
            &attendees_str,
        )?;
    }

    // Insert extracted tasks
    let mut inserted_tasks = vec![];
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        for extracted in &extraction.tasks {
            // Route to a different project if the AI identified one
            let target_project_id = if let Some(task_project_name) = &extracted.project {
                let matched = all_projects.iter().find(|p| {
                    p.name.to_lowercase() == task_project_name.to_lowercase()
                        && p.id != project_id
                });
                matched.map(|p| p.id.clone()).unwrap_or_else(|| project_id.clone())
            } else {
                project_id.clone()
            };

            let input = crate::models::task::CreateTaskInput {
                project_id: target_project_id,
                meeting_id: Some(meeting.id.clone()),
                title: extracted.title.clone(),
                description: extracted.description.clone(),
                assignee: extracted.assignee.clone(),
                assignee_confidence: Some(extracted.assignee_confidence.clone()),
                assignee_source_quote: extracted.assignee_source_quote.clone(),
                due_date: extracted.due_date.clone(),
                due_confidence: Some(extracted.due_confidence.clone()),
                due_source_quote: extracted.due_source_quote.clone(),
                priority: extracted.priority.clone(),
                confidence_score: extracted.confidence_score,
                tags: Some(extracted.tags.clone()),
                kanban_column: None,
                notes: extracted.notes.clone(),
                is_duplicate: None,
                duplicate_of_id: None,
            };
            match task_repo::create_task(&conn, &input) {
                Ok(task) => inserted_tasks.push(task),
                Err(e) => eprintln!("Failed to insert task '{}': {}", extracted.title, e),
            }
        }
    }

    // Get updated meeting
    let updated_meeting = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        repo::get_meeting(&conn, &meeting.id)?
    };

    Ok(json!({
        "meeting": updated_meeting,
        "tasks": inserted_tasks,
    }))
}

#[tauri::command]
pub async fn ingest_meeting(
    project_id: String,
    title: String,
    platform: String,
    raw_transcript: String,
    attendees: Option<String>,
    duration_minutes: Option<i32>,
    meeting_at: Option<String>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    ingest_meeting_core(project_id, title, platform, raw_transcript, attendees, duration_minutes, meeting_at, &state).await
}

#[tauri::command]
pub async fn ingest_meeting_from_file(
    project_id: String,
    file_path: String,
    title: Option<String>,
    platform: Option<String>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let path = std::path::Path::new(&file_path);

    // Parse the file using the shared file parser
    let parsed = crate::utils::file_parser::parse_file(path).await?;
    let raw_transcript = parsed.content_text;

    // Derive a title from the filename if none provided
    let resolved_title = title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Meeting")
            .to_string()
    });
    let resolved_platform = platform.unwrap_or_else(|| "other".to_string());

    // Delegate to the core ingest logic
    ingest_meeting_core(
        project_id,
        resolved_title,
        resolved_platform,
        raw_transcript,
        None,
        None,
        None,
        &state,
    )
    .await
}

#[tauri::command]
pub async fn get_meetings_for_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Meeting>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_meetings_for_project(&conn, &project_id)
}

#[tauri::command]
pub async fn get_meeting(id: String, state: State<'_, AppState>) -> Result<Option<Meeting>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::get_meeting(&conn, &id)
}

#[tauri::command]
pub async fn delete_meeting(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::soft_delete_meeting(&conn, &id)
}

#[tauri::command]
pub async fn rename_meeting(
    id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Meeting title cannot be empty".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::rename_meeting(&conn, &id, title.trim())
}

/// Return the number of open/in-progress tasks that would move with a meeting.
/// Used to populate the confirmation dialog before the user commits to the move.
#[tauri::command]
pub async fn count_moveable_tasks(
    meeting_id: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let meeting = repo::get_meeting(&conn, &meeting_id)?
        .ok_or_else(|| "Meeting not found".to_string())?;
    task_repo::count_moveable_tasks(&conn, &meeting_id, &meeting.project_id)
}

/// Move a meeting and its eligible tasks to a different project atomically.
///
/// Rules:
/// - Only tasks still in the meeting's current project are considered
/// - Only open/in_progress tasks move; completed/cancelled tasks stay
/// - Tasks already manually reassigned to other projects are not touched
///
/// Returns `{ old_project_id, new_project_id, tasks_moved }`.
#[tauri::command]
pub async fn move_meeting_to_project(
    meeting_id: String,
    new_project_id: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Fetch the meeting so we know its current project
    let meeting = repo::get_meeting(&conn, &meeting_id)?
        .ok_or_else(|| "Meeting not found".to_string())?;

    let old_project_id = meeting.project_id.clone();

    // Guard: moving to the same project is a no-op / user error
    if old_project_id == new_project_id {
        return Err("Meeting is already in that project".to_string());
    }

    // Guard: target project must exist and not be archived
    let target = proj_repo::get_project(&conn, &new_project_id)?
        .ok_or_else(|| "Target project not found".to_string())?;
    if target.archived_at.is_some() {
        return Err(format!(
            "'{}' is archived — unarchive it first before moving meetings into it",
            target.name
        ));
    }

    // Move the meeting row
    repo::move_meeting_project(&conn, &meeting_id, &new_project_id)?;

    // Move eligible tasks
    let tasks_moved =
        task_repo::move_tasks_for_meeting(&conn, &meeting_id, &old_project_id, &new_project_id)?;

    Ok(json!({
        "old_project_id": old_project_id,
        "new_project_id": new_project_id,
        "tasks_moved": tasks_moved,
    }))
}
