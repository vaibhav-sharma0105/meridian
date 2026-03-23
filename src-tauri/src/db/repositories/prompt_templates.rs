use crate::models::ai_settings::PromptTemplate;
use rusqlite::{params, Connection};
use uuid::Uuid;

fn row_to_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptTemplate> {
    let is_default: i64 = row.get(7)?;
    let is_builtin: i64 = row.get(8)?;
    Ok(PromptTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        system_prompt: row.get(3)?,
        user_prompt_template: row.get(4)?,
        output_format: row.get(5)?,
        created_at: row.get(6)?,
        is_default: is_default != 0,
        is_builtin: is_builtin != 0,
        updated_at: row.get(9)?,
    })
}

pub fn get_all_templates(conn: &Connection) -> Result<Vec<PromptTemplate>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, system_prompt, user_prompt_template, output_format,
                    created_at, is_default, is_builtin, updated_at
             FROM prompt_templates ORDER BY is_builtin DESC, name ASC",
        )
        .map_err(|e| e.to_string())?;

    let templates = stmt
        .query_map([], row_to_template)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(templates)
}

pub fn get_template(conn: &Connection, id: &str) -> Result<Option<PromptTemplate>, String> {
    let result = conn.query_row(
        "SELECT id, name, description, system_prompt, user_prompt_template, output_format,
                created_at, is_default, is_builtin, updated_at
         FROM prompt_templates WHERE id = ?1",
        params![id],
        row_to_template,
    );

    match result {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_template(conn: &Connection, template: &PromptTemplate) -> Result<PromptTemplate, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_templates WHERE id = ?1",
            params![template.id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if exists > 0 {
        conn.execute(
            "UPDATE prompt_templates SET name=?1, description=?2, system_prompt=?3,
             user_prompt_template=?4, output_format=?5, updated_at=?6 WHERE id=?7",
            params![
                template.name,
                template.description,
                template.system_prompt,
                template.user_prompt_template,
                template.output_format,
                now,
                template.id
            ],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let id = if template.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            template.id.clone()
        };
        conn.execute(
            "INSERT INTO prompt_templates (id, name, description, system_prompt, user_prompt_template, output_format, is_default, is_builtin)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
            params![
                id,
                template.name,
                template.description,
                template.system_prompt,
                template.user_prompt_template,
                template.output_format
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    get_template(conn, &template.id)?
        .ok_or_else(|| "Failed to retrieve saved template".to_string())
}
