use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{AttentionFilters, AttentionItem};

pub fn upsert_attention_item(
    conn: &Connection,
    source_type: &str,
    source_id: &str,
    severity: &str,
    category: &str,
    reason_text: Option<&str>,
    matched_skill_id: Option<&str>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO attention_items (id, source_type, source_id, severity, category, reason_text, matched_skill_id, computed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(source_type, source_id, category) DO UPDATE SET
            severity = excluded.severity,
            reason_text = excluded.reason_text,
            matched_skill_id = excluded.matched_skill_id,
            computed_at = excluded.computed_at,
            dismissed_at = NULL",
        params![id, source_type, source_id, severity, category, reason_text, matched_skill_id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

pub fn list_attention_items(
    conn: &Connection,
    filters: &AttentionFilters,
) -> Result<Vec<AttentionItem>, String> {
    let include_dismissed = filters.include_dismissed.unwrap_or(false);

    let mut sql = String::from(
        "SELECT id, source_type, source_id, severity, category, reason_text, matched_skill_id, computed_at, dismissed_at
         FROM attention_items WHERE 1=1",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if !include_dismissed {
        sql.push_str(" AND dismissed_at IS NULL");
    }
    if let Some(ref s) = filters.severity {
        sql.push_str(" AND severity = ?");
        params_vec.push(Box::new(s.clone()));
    }
    if let Some(ref s) = filters.source_type {
        sql.push_str(" AND source_type = ?");
        params_vec.push(Box::new(s.clone()));
    }
    if let Some(ref c) = filters.category {
        sql.push_str(" AND category = ?");
        params_vec.push(Box::new(c.clone()));
    }

    sql.push_str(" ORDER BY CASE severity WHEN 'critical' THEN 0 WHEN 'warning' THEN 1 ELSE 2 END, computed_at DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_refs), |row| {
            Ok(AttentionItem {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_id: row.get(2)?,
                severity: row.get(3)?,
                category: row.get(4)?,
                reason_text: row.get(5)?,
                matched_skill_id: row.get(6)?,
                computed_at: row.get(7)?,
                dismissed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn dismiss_attention_item(conn: &Connection, id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE attention_items SET dismissed_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_attention_items(conn: &Connection, source_type: Option<&str>) -> Result<u64, String> {
    let count = if let Some(st) = source_type {
        conn.execute(
            "DELETE FROM attention_items WHERE source_type = ?1",
            [st],
        )
    } else {
        conn.execute("DELETE FROM attention_items", [])
    }
    .map_err(|e| e.to_string())?;
    Ok(count as u64)
}

pub fn get_attention_count(conn: &Connection) -> Result<(u32, u32), String> {
    let critical: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM attention_items WHERE severity = 'critical' AND dismissed_at IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let warning: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM attention_items WHERE severity = 'warning' AND dismissed_at IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok((critical, warning))
}
