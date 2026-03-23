use crate::models::ai_settings::AiSettings;
use rusqlite::{params, Connection};
use uuid::Uuid;

fn row_to_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiSettings> {
    let is_active: i64 = row.get(8)?;
    Ok(AiSettings {
        id: row.get(0)?,
        label: row.get(1)?,
        provider: row.get(2)?,
        base_url: row.get(3)?,
        model_id: row.get(4)?,
        ollama_base_url: row.get(5)?,
        ollama_model: row.get(6)?,
        embedding_provider: row.get(7)?,
        is_active: is_active != 0,
        created_at: row.get(9)?,
    })
}

pub fn get_active_settings(conn: &Connection) -> Result<Option<AiSettings>, String> {
    let result = conn.query_row(
        "SELECT id, label, provider, base_url, model_id, ollama_base_url, ollama_model,
                embedding_provider, is_active, created_at
         FROM ai_settings WHERE is_active = 1 LIMIT 1",
        [],
        row_to_settings,
    );

    match result {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_all_settings(conn: &Connection) -> Result<Vec<AiSettings>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, label, provider, base_url, model_id, ollama_base_url, ollama_model,
                    embedding_provider, is_active, created_at
             FROM ai_settings ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_map([], row_to_settings)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

pub fn save_settings(
    conn: &Connection,
    id: &str,
    label: &str,
    provider: &str,
    base_url: Option<&str>,
    model_id: Option<&str>,
    ollama_base_url: &str,
    ollama_model: &str,
    embedding_provider: &str,
) -> Result<AiSettings, String> {
    // Deactivate all others
    conn.execute("UPDATE ai_settings SET is_active = 0", [])
        .map_err(|e| e.to_string())?;

    // Check if this ID already exists
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ai_settings WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if exists > 0 {
        conn.execute(
            "UPDATE ai_settings SET label=?1, provider=?2, base_url=?3, model_id=?4,
             ollama_base_url=?5, ollama_model=?6, embedding_provider=?7, is_active=1 WHERE id=?8",
            params![label, provider, base_url, model_id, ollama_base_url, ollama_model, embedding_provider, id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO ai_settings (id, label, provider, base_url, model_id, ollama_base_url, ollama_model, embedding_provider, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
            params![id, label, provider, base_url, model_id, ollama_base_url, ollama_model, embedding_provider],
        )
        .map_err(|e| e.to_string())?;
    }

    get_active_settings(conn)?.ok_or_else(|| "Failed to retrieve saved settings".to_string())
}
