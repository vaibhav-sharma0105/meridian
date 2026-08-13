use chrono::{Duration, Utc};
use rusqlite::{params, Connection};

use super::models::CleanupStats;

pub fn cleanup_expired_messages(conn: &Connection) -> Result<CleanupStats, String> {
    let now = Utc::now();
    let hard_delete_cutoff = (now - Duration::days(30)).to_rfc3339();

    // Get user retention preference
    let retention_days: Option<i64> = conn
        .query_row(
            "SELECT CASE message_retention
                WHEN '90d' THEN 90
                WHEN '1y' THEN 365
                ELSE NULL
             END FROM user_profile WHERE id = 'default'",
            [],
            |row| row.get(0),
        )
        .ok();

    let mut soft_deleted = 0i64;
    let mut hard_deleted = 0i64;
    let mut files_removed = 0i64;

    // 1. Hard-delete messages that were soft-deleted more than 30 days ago
    let messages_to_hard_delete: Vec<(String, Option<String>)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, file_refs FROM message_center
                 WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            )
            .map_err(|e| e.to_string())?;

        let results: Vec<(String, Option<String>)> = stmt
            .query_map(params![hard_delete_cutoff], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        results
    };

    for (id, file_refs_json) in &messages_to_hard_delete {
        // Track files for potential cleanup
        if let Some(refs_json) = file_refs_json {
            if let Ok(refs) = serde_json::from_str::<Vec<String>>(refs_json) {
                for file_path in refs {
                    if cleanup_orphaned_file(conn, &file_path)? {
                        files_removed += 1;
                    }
                }
            }
        }

        conn.execute("DELETE FROM message_center WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        hard_deleted += 1;
    }

    // 2. Soft-delete messages older than retention (if not 'forever')
    if let Some(days) = retention_days {
        let soft_delete_cutoff = (now - Duration::days(days)).to_rfc3339();
        soft_deleted = conn
            .execute(
                "UPDATE message_center
                 SET deleted_at = datetime('now'), updated_at = datetime('now')
                 WHERE deleted_at IS NULL AND created_at < ?1",
                params![soft_delete_cutoff],
            )
            .map_err(|e| e.to_string())? as i64;
    }

    Ok(CleanupStats {
        soft_deleted,
        hard_deleted,
        files_removed,
    })
}

fn cleanup_orphaned_file(conn: &Connection, file_path: &str) -> Result<bool, String> {
    // Check if any other message references this file
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_center
             WHERE file_refs LIKE ?1 AND deleted_at IS NULL",
            params![format!("%{}%", file_path)],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count == 0 {
        // No other references, safe to delete
        let full_path = if file_path.starts_with('/') {
            file_path.to_string()
        } else {
            let home = dirs_next::home_dir().ok_or("Cannot find home directory")?;
            home.join(".meridian")
                .join("created_files")
                .join(file_path)
                .to_string_lossy()
                .to_string()
        };

        if std::path::Path::new(&full_path).exists() {
            std::fs::remove_file(&full_path).map_err(|e| e.to_string())?;
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn get_soft_deleted_messages(
    conn: &Connection,
    limit: i64,
) -> Result<Vec<super::models::Message>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, message_type, title, content, source_id, source_type,
                    auto_pinned, pinned_reason, file_refs, ai_visible_until, deleted_at,
                    created_at, updated_at
             FROM message_center
             WHERE deleted_at IS NOT NULL
             ORDER BY deleted_at DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let results: Vec<super::models::Message> = stmt
        .query_map(params![limit], |row| {
            let file_refs_str: Option<String> = row.get(9)?;
            let file_refs =
                file_refs_str.and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok());

            Ok(super::models::Message {
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

    Ok(results)
}
