use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::*;

pub fn create_message(conn: &Connection, input: CreateMessageInput) -> Result<Message, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let file_refs_json = input
        .file_refs
        .as_ref()
        .map(|refs| serde_json::to_string(refs).unwrap_or_default());

    conn.execute(
        "INSERT INTO message_center (
            id, project_id, message_type, title, content, source_id, source_type,
            auto_pinned, pinned_reason, file_refs, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            id,
            input.project_id,
            input.message_type,
            input.title,
            input.content,
            input.source_id,
            input.source_type,
            input.auto_pinned.unwrap_or(false) as i32,
            input.pinned_reason,
            file_refs_json,
            now,
            now,
        ],
    )
    .map_err(|e| format!("Failed to create message: {}", e))?;

    get_message(conn, &id)
}

pub fn get_message(conn: &Connection, id: &str) -> Result<Message, String> {
    conn.query_row(
        "SELECT id, project_id, message_type, title, content, source_id, source_type,
                auto_pinned, pinned_reason, file_refs, ai_visible_until, deleted_at,
                created_at, updated_at
         FROM message_center WHERE id = ?1",
        params![id],
        |row| {
            let file_refs_str: Option<String> = row.get(9)?;
            let file_refs = file_refs_str
                .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok());

            Ok(Message {
                id: row.get(0)?,
                project_id: row.get(1)?,
                message_type: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                source_id: row.get(5)?,
                source_type: row.get(6)?,
                auto_pinned: row.get::<_, i32>(7)? != 0,
                pinned_reason: row.get(8)?,
                file_refs,
                ai_visible_until: row.get(10)?,
                deleted_at: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        },
    )
    .map_err(|e| format!("Message not found: {}", e))
}

pub fn get_messages(
    conn: &Connection,
    filters: &MessageFilters,
    page: i64,
    per_page: i64,
) -> Result<PaginatedMessages, String> {
    let mut conditions = vec![];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if !filters.include_deleted.unwrap_or(false) {
        conditions.push("deleted_at IS NULL".to_string());
    }

    if let Some(ref project_id) = filters.project_id {
        conditions.push(format!("project_id = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(project_id.clone()));
    }

    if let Some(ref message_type) = filters.message_type {
        conditions.push(format!("message_type = ?{}", params_vec.len() + 1));
        params_vec.push(Box::new(message_type.clone()));
    }

    if let Some(ref search) = filters.search {
        conditions.push(format!(
            "(title LIKE ?{} OR content LIKE ?{})",
            params_vec.len() + 1,
            params_vec.len() + 2
        ));
        let search_pattern = format!("%{}%", search);
        params_vec.push(Box::new(search_pattern.clone()));
        params_vec.push(Box::new(search_pattern));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Get total count
    let count_sql = format!("SELECT COUNT(*) FROM message_center {}", where_clause);
    let total: i64 = {
        let mut stmt = conn.prepare(&count_sql).map_err(|e| e.to_string())?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(params_refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    // Get paginated results
    let offset = (page - 1) * per_page;
    let query_sql = format!(
        "SELECT id, project_id, message_type, title, content, source_id, source_type,
                auto_pinned, pinned_reason, file_refs, ai_visible_until, deleted_at,
                created_at, updated_at
         FROM message_center {}
         ORDER BY created_at DESC
         LIMIT ?{} OFFSET ?{}",
        where_clause,
        params_vec.len() + 1,
        params_vec.len() + 2
    );

    params_vec.push(Box::new(per_page));
    params_vec.push(Box::new(offset));

    let mut stmt = conn.prepare(&query_sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let messages = stmt
        .query_map(params_refs.as_slice(), |row| {
            let file_refs_str: Option<String> = row.get(9)?;
            let file_refs = file_refs_str
                .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok());

            Ok(Message {
                id: row.get(0)?,
                project_id: row.get(1)?,
                message_type: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                source_id: row.get(5)?,
                source_type: row.get(6)?,
                auto_pinned: row.get::<_, i32>(7)? != 0,
                pinned_reason: row.get(8)?,
                file_refs,
                ai_visible_until: row.get(10)?,
                deleted_at: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(PaginatedMessages {
        messages,
        total,
        page,
        per_page,
    })
}

pub fn get_messages_for_ai_context(
    conn: &Connection,
    project_id: Option<&str>,
    ai_context_days: i64,
) -> Result<Vec<Message>, String> {
    let cutoff = Utc::now() - chrono::Duration::days(ai_context_days);
    let cutoff_str = cutoff.to_rfc3339();

    let mut sql = String::from(
        "SELECT id, project_id, message_type, title, content, source_id, source_type,
                auto_pinned, pinned_reason, file_refs, ai_visible_until, deleted_at,
                created_at, updated_at
         FROM message_center
         WHERE deleted_at IS NULL
           AND created_at > ?1
           AND (ai_visible_until IS NULL OR ai_visible_until > datetime('now'))",
    );

    if project_id.is_some() {
        sql.push_str(" AND project_id = ?2");
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT 50");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let results: Vec<Message> = if let Some(pid) = project_id {
        stmt.query_map(params![cutoff_str, pid], map_message_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![cutoff_str], map_message_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(results)
}

fn map_message_row(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    let file_refs_str: Option<String> = row.get(9)?;
    let file_refs = file_refs_str.and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok());

    Ok(Message {
        id: row.get(0)?,
        project_id: row.get(1)?,
        message_type: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        source_id: row.get(5)?,
        source_type: row.get(6)?,
        auto_pinned: row.get::<_, i32>(7)? != 0,
        pinned_reason: row.get(8)?,
        file_refs,
        ai_visible_until: row.get(10)?,
        deleted_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

pub fn soft_delete_message(conn: &Connection, id: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE message_center SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| format!("Failed to delete message: {}", e))?;
    Ok(())
}

pub fn restore_message(conn: &Connection, id: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE message_center SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| format!("Failed to restore message: {}", e))?;
    Ok(())
}

pub fn pin_from_source(
    conn: &Connection,
    source_type: &str,
    source_id: &str,
    title: &str,
    content: Option<&str>,
    project_id: Option<&str>,
) -> Result<Message, String> {
    create_message(
        conn,
        CreateMessageInput {
            project_id: project_id.map(String::from),
            message_type: "pinned_chat".to_string(),
            title: title.to_string(),
            content: content.map(String::from),
            source_id: Some(source_id.to_string()),
            source_type: Some(source_type.to_string()),
            auto_pinned: Some(false),
            pinned_reason: Some("user_pinned".to_string()),
            file_refs: None,
        },
    )
}
