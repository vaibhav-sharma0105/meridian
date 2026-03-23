use crate::db::repositories::{documents as doc_repo, meetings as mtg_repo, projects as proj_repo, tasks as task_repo};
use crate::AppState;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub async fn export_project(
    project_id: String,
    format: String,
    include_docs: bool,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let (project, meetings, tasks, docs) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let project = proj_repo::get_project(&conn, &project_id)?
            .ok_or_else(|| "Project not found".to_string())?;
        let meetings = mtg_repo::get_meetings_for_project(&conn, &project_id)?;
        let tasks = task_repo::get_tasks_for_project(&conn, &project_id, &Default::default())?;
        let docs = if include_docs {
            doc_repo::get_documents_for_project(&conn, &project_id)?
        } else {
            vec![]
        };
        (project, meetings, tasks, docs)
    };

    let content = match format.as_str() {
        "json" => {
            let export = json!({
                "meridian_export_version": "1",
                "exported_at": chrono::Utc::now().to_rfc3339(),
                "app_version": "0.1.0",
                "project": project,
                "meetings": meetings,
                "tasks": tasks,
                "documents": docs,
            });
            serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?
        }
        "csv" => tasks_to_csv(&tasks, &meetings),
        "markdown" => project_to_markdown(&project, &meetings, &tasks),
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let ext = match format.as_str() {
        "json" => "json",
        "csv" => "csv",
        _ => "md",
    };

    // Save to temp location
    let filename = format!(
        "meridian_{}_{}.{}",
        project.name.replace(' ', "_").to_lowercase(),
        chrono::Local::now().format("%Y%m%d"),
        ext
    );
    let temp_path = std::env::temp_dir().join(&filename);
    std::fs::write(&temp_path, &content).map_err(|e| e.to_string())?;

    let size = content.len() as u64;
    Ok(json!({
        "file_path": temp_path.to_string_lossy(),
        "size_bytes": size,
    }))
}

#[tauri::command]
pub async fn export_all(state: State<'_, AppState>) -> Result<Value, String> {
    let projects = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        proj_repo::get_all_projects(&conn)?
    };

    let mut all_data = vec![];
    for project in &projects {
        let (meetings, tasks) = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            let meetings = mtg_repo::get_meetings_for_project(&conn, &project.id)?;
            let tasks = task_repo::get_tasks_for_project(&conn, &project.id, &Default::default())?;
            (meetings, tasks)
        };
        all_data.push(json!({
            "project": project,
            "meetings": meetings,
            "tasks": tasks,
        }));
    }

    let export = json!({
        "meridian_export_version": "1",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "app_version": "0.1.0",
        "data": all_data,
    });

    let content = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;
    let filename = format!("meridian_backup_{}.json", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let temp_path = std::env::temp_dir().join(&filename);
    std::fs::write(&temp_path, &content).map_err(|e| e.to_string())?;

    Ok(json!({
        "file_path": temp_path.to_string_lossy(),
        "size_bytes": content.len(),
    }))
}

fn tasks_to_csv(tasks: &[crate::models::task::Task], meetings: &[crate::models::meeting::Meeting]) -> String {
    let mut csv = String::from("id,title,description,assignee,assignee_confidence,due_date,due_confidence,status,tags,meeting_title,created_at,completed_at,notes\n");
    for task in tasks {
        let meeting_title = task.meeting_id.as_deref()
            .and_then(|mid| meetings.iter().find(|m| m.id == mid))
            .map(|m| m.title.as_str())
            .unwrap_or("");
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_field(&task.id),
            csv_field(&task.title),
            csv_field(task.description.as_deref().unwrap_or("")),
            csv_field(task.assignee.as_deref().unwrap_or("")),
            csv_field(&task.assignee_confidence),
            csv_field(task.due_date.as_deref().unwrap_or("")),
            csv_field(&task.due_confidence),
            csv_field(&task.status),
            csv_field(&task.tags),
            csv_field(meeting_title),
            csv_field(&task.created_at),
            csv_field(task.completed_at.as_deref().unwrap_or("")),
            csv_field(task.notes.as_deref().unwrap_or("")),
        ));
    }
    csv
}

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn project_to_markdown(
    project: &crate::models::project::Project,
    meetings: &[crate::models::meeting::Meeting],
    tasks: &[crate::models::task::Task],
) -> String {
    let mut md = format!("# Project: {}\nExported: {}\n\n", project.name, chrono::Local::now().format("%Y-%m-%d"));

    md.push_str("## Open Tasks\n\n");
    for task in tasks.iter().filter(|t| t.status == "open" || t.status == "in_progress") {
        let due = task.due_date.as_deref().map(|d| format!(" (due: {})", d)).unwrap_or_default();
        md.push_str(&format!("- [ ] {}{}\n", task.title, due));
    }

    md.push_str("\n## Completed Tasks\n\n");
    for task in tasks.iter().filter(|t| t.status == "done") {
        let completed = task.completed_at.as_deref().map(|d| format!(" (completed: {})", d)).unwrap_or_default();
        md.push_str(&format!("- [x] {}{}\n", task.title, completed));
    }

    md.push_str("\n## Meetings\n\n");
    for meeting in meetings {
        let score = meeting.health_score.map(|s| format!(" — Health: {}/100", s)).unwrap_or_default();
        md.push_str(&format!("### {}{}\n", meeting.title, score));
        if let Some(summary) = &meeting.ai_summary {
            md.push_str(&format!("**Summary:** {}\n\n", summary));
        }
    }

    md
}
