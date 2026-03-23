use crate::AppState;
use rusqlite::params;
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn import_project(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;

    let data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    // Check version
    let version = data["meridian_export_version"].as_str().unwrap_or("0");
    let version_num: u32 = version.parse().unwrap_or(0);
    if version_num > 1 {
        return Err(format!(
            "This export was created with a newer version of Meridian — some fields may not be available"
        ));
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut projects_imported = 0i64;
    let mut meetings_imported = 0i64;
    let mut tasks_imported = 0i64;

    // Handle single project export
    if let Some(project) = data.get("project") {
        import_single_project(&conn, project, &data, &mut projects_imported, &mut meetings_imported, &mut tasks_imported)?;
    }

    // Handle multi-project (export_all) format
    if let Some(all_data) = data["data"].as_array() {
        for entry in all_data {
            if let Some(project) = entry.get("project") {
                import_single_project(&conn, project, entry, &mut projects_imported, &mut meetings_imported, &mut tasks_imported)?;
            }
        }
    }

    Ok(serde_json::json!({
        "projects_imported": projects_imported,
        "meetings_imported": meetings_imported,
        "tasks_imported": tasks_imported,
    }))
}

fn import_single_project(
    conn: &rusqlite::Connection,
    project: &Value,
    data: &Value,
    projects_imported: &mut i64,
    meetings_imported: &mut i64,
    tasks_imported: &mut i64,
) -> Result<(), String> {
    let old_project_id = project["id"].as_str().unwrap_or("").to_string();
    let new_project_id = Uuid::new_v4().to_string();

    let mut name = project["name"].as_str().unwrap_or("Imported Project").to_string();
    // Check for duplicate name
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        name = format!("{} (imported)", name);
    }

    conn.execute(
        "INSERT INTO projects (id, name, description, color) VALUES (?1, ?2, ?3, ?4)",
        params![
            new_project_id,
            name,
            project["description"].as_str(),
            project["color"].as_str().unwrap_or("#6366f1"),
        ],
    )
    .map_err(|e| e.to_string())?;
    *projects_imported += 1;

    // Import meetings with ID mapping
    let mut meeting_id_map = std::collections::HashMap::new();
    if let Some(meetings) = data["meetings"].as_array() {
        for meeting in meetings {
            if meeting["project_id"].as_str() == Some(&old_project_id) || meetings.len() > 0 {
                let old_id = meeting["id"].as_str().unwrap_or("").to_string();
                let new_id = Uuid::new_v4().to_string();
                meeting_id_map.insert(old_id.clone(), new_id.clone());

                conn.execute(
                    "INSERT INTO meetings (id, project_id, title, platform, ai_summary, health_score, meeting_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        new_id,
                        new_project_id,
                        meeting["title"].as_str().unwrap_or("Imported Meeting"),
                        meeting["platform"].as_str().unwrap_or("manual"),
                        meeting["ai_summary"].as_str(),
                        meeting["health_score"].as_i64(),
                        meeting["meeting_at"].as_str(),
                    ],
                )
                .map_err(|e| e.to_string())?;
                *meetings_imported += 1;
            }
        }
    }

    // Import tasks
    if let Some(tasks) = data["tasks"].as_array() {
        for task in tasks {
            let new_id = Uuid::new_v4().to_string();
            let old_meeting_id = task["meeting_id"].as_str();
            let new_meeting_id = old_meeting_id.and_then(|mid| meeting_id_map.get(mid)).cloned();

            conn.execute(
                "INSERT INTO tasks (id, project_id, meeting_id, title, description, assignee,
                 assignee_confidence, due_date, due_confidence, status, tags, notes)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    new_id,
                    new_project_id,
                    new_meeting_id,
                    task["title"].as_str().unwrap_or("Imported Task"),
                    task["description"].as_str(),
                    task["assignee"].as_str(),
                    task["assignee_confidence"].as_str().unwrap_or("unassigned"),
                    task["due_date"].as_str(),
                    task["due_confidence"].as_str().unwrap_or("none"),
                    task["status"].as_str().unwrap_or("open"),
                    task["tags"].as_str().unwrap_or("[]"),
                    task["notes"].as_str(),
                ],
            )
            .map_err(|e| e.to_string())?;
            *tasks_imported += 1;
        }
    }

    Ok(())
}
