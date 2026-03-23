use crate::models::ai_settings::AppNotification;
use rusqlite::{params, Connection};
use uuid::Uuid;

fn row_to_notification(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppNotification> {
    let is_read: i64 = row.get(6)?;
    Ok(AppNotification {
        id: row.get(0)?,
        notification_type: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        task_id: row.get(4)?,
        project_id: row.get(5)?,
        is_read: is_read != 0,
        created_at: row.get(7)?,
    })
}

pub fn get_notifications(conn: &Connection) -> Result<Vec<AppNotification>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, title, body, task_id, project_id, is_read, created_at
             FROM notifications ORDER BY created_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let notifs = stmt
        .query_map([], row_to_notification)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(notifs)
}

pub fn create_notification(
    conn: &Connection,
    notification_type: &str,
    title: &str,
    body: &str,
    task_id: Option<&str>,
    project_id: Option<&str>,
) -> Result<AppNotification, String> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO notifications (id, type, title, body, task_id, project_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, notification_type, title, body, task_id, project_id],
    )
    .map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT id, type, title, body, task_id, project_id, is_read, created_at
         FROM notifications WHERE id = ?1",
        params![id],
        row_to_notification,
    )
    .map_err(|e| e.to_string())?;

    Ok(result)
}

pub fn mark_read(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE notifications SET is_read = 1 WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn mark_all_read(conn: &Connection) -> Result<(), String> {
    conn.execute("UPDATE notifications SET is_read = 1", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
