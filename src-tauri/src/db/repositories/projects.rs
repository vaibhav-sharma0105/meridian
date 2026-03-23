use crate::models::project::{CreateProjectInput, Project, UpdateProjectInput};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn get_all_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.description, p.color, p.created_at, p.updated_at, p.archived_at,
                    COUNT(CASE WHEN t.status = 'open' OR t.status = 'in_progress' THEN 1 END) as open_task_count
             FROM projects p
             LEFT JOIN tasks t ON t.project_id = p.id
             WHERE p.archived_at IS NULL
             GROUP BY p.id
             ORDER BY p.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                archived_at: row.get(6)?,
                open_task_count: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(projects)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Option<Project>, String> {
    let result = conn.query_row(
        "SELECT id, name, description, color, created_at, updated_at, archived_at FROM projects WHERE id = ?1",
        params![id],
        |row| Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            archived_at: row.get(6)?,
            open_task_count: None,
        }),
    );

    match result {
        Ok(p) => Ok(Some(p)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_project(conn: &Connection, input: &CreateProjectInput) -> Result<Project, String> {
    let id = Uuid::new_v4().to_string();
    let color = input.color.clone().unwrap_or_else(|| "#6366f1".to_string());

    conn.execute(
        "INSERT INTO projects (id, name, description, color) VALUES (?1, ?2, ?3, ?4)",
        params![id, input.name, input.description, color],
    )
    .map_err(|e| e.to_string())?;

    get_project(conn, &id)?.ok_or_else(|| "Failed to retrieve created project".to_string())
}

pub fn update_project(conn: &Connection, input: &UpdateProjectInput) -> Result<Project, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(name) = &input.name {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, input.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(desc) = &input.description {
        conn.execute(
            "UPDATE projects SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![desc, now, input.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(color) = &input.color {
        conn.execute(
            "UPDATE projects SET color = ?1, updated_at = ?2 WHERE id = ?3",
            params![color, now, input.id],
        )
        .map_err(|e| e.to_string())?;
    }

    get_project(conn, &input.id)?.ok_or_else(|| "Project not found".to_string())
}

pub fn archive_project(conn: &Connection, id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE projects SET archived_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
