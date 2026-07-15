use rusqlite::{params, Connection};
use serde::Deserialize;

use super::models::{ActionConfig, ContextConfig, CreateSkillInput, TriggerConfig};
use super::repository as skills_repo;

#[derive(Debug, Deserialize)]
struct BuiltinTemplate {
    name: String,
    description: String,
    trigger_type: String,
    trigger_config: TriggerConfig,
    context_config: Option<ContextConfig>,
    action_config: ActionConfig,
    approval_mode: String,
    category: Option<String>,
    icon: Option<String>,
    tags: Option<Vec<String>>,
}

const TEMPLATES_JSON: &str = include_str!("../../resources/builtin-skills/templates.json");

fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| format!("Failed to set setting: {}", e))?;
    Ok(())
}

pub fn load_builtin_skills(conn: &Connection) -> Result<Vec<String>, String> {
    let already_initialized = get_setting(conn, "builtin_skills_initialized")
        .unwrap_or_default();

    if already_initialized == "true" {
        return Ok(vec![]);
    }

    let templates: Vec<BuiltinTemplate> = serde_json::from_str(TEMPLATES_JSON)
        .map_err(|e| format!("Failed to parse builtin skill templates: {}", e))?;

    let mut created_ids = Vec::new();

    for template in &templates {
        let input = CreateSkillInput {
            name: template.name.clone(),
            description: Some(template.description.clone()),
            trigger_type: template.trigger_type.clone(),
            trigger_config: Some(template.trigger_config.clone()),
            context_config: template.context_config.clone(),
            action_config: Some(template.action_config.clone()),
            approval_mode: Some(template.approval_mode.clone()),
            category: template.category.clone(),
            icon: template.icon.clone(),
            tags: template.tags.clone(),
            is_builtin: true,
            shared: false, // Built-in skills are not shared by default
        };

        match skills_repo::create_skill(conn, &input) {
            Ok(skill) => created_ids.push(skill.id),
            Err(e) => eprintln!("Failed to create builtin skill '{}': {}", template.name, e),
        }
    }

    set_setting(conn, "builtin_skills_initialized", "true")?;

    Ok(created_ids)
}

pub fn reset_builtin_skills(conn: &Connection) -> Result<Vec<String>, String> {
    conn.execute("DELETE FROM skills WHERE is_builtin = 1", [])
        .map_err(|e| format!("Failed to delete builtin skills: {}", e))?;
    set_setting(conn, "builtin_skills_initialized", "false")?;
    load_builtin_skills(conn)
}
