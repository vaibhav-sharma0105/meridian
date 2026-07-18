use crate::db::repositories::prompt_templates as tpl_repo;
use crate::models::ai_settings::PromptTemplate;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPermissions {
    #[serde(default = "default_true")]
    pub read_tasks: bool,
    #[serde(default = "default_true")]
    pub read_meetings: bool,
    #[serde(default = "default_true")]
    pub read_projects: bool,
    #[serde(default)]
    pub create_task: bool,
    #[serde(default)]
    pub update_task: bool,
    #[serde(default)]
    pub delete_task: bool,
    #[serde(default)]
    pub create_meeting_note: bool,
    #[serde(default)]
    pub run_skill: bool,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

fn default_true() -> bool {
    true
}

fn default_rate_limit() -> u32 {
    100
}

impl Default for McpPermissions {
    fn default() -> Self {
        Self {
            read_tasks: true,
            read_meetings: true,
            read_projects: true,
            create_task: false,
            update_task: false,
            delete_task: false,
            create_meeting_note: false,
            run_skill: false,
            rate_limit_per_minute: 100,
        }
    }
}

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?;

    let settings: HashMap<String, String> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect();

    Ok(settings)
}

#[tauri::command]
pub async fn set_app_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_prompt_templates(state: State<'_, AppState>) -> Result<Vec<PromptTemplate>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    tpl_repo::get_all_templates(&conn)
}

#[tauri::command]
pub async fn save_prompt_template(
    template: PromptTemplate,
    state: State<'_, AppState>,
) -> Result<PromptTemplate, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    tpl_repo::save_template(&conn, &template)
}

#[tauri::command]
pub async fn get_mcp_permissions(state: State<'_, AppState>) -> Result<McpPermissions, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let json_str: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'mcp_permissions'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(s) = json_str {
        serde_json::from_str(&s).map_err(|e| e.to_string())
    } else {
        Ok(McpPermissions::default())
    }
}

#[tauri::command]
pub async fn set_mcp_permissions(
    permissions: McpPermissions,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let json_str = serde_json::to_string(&permissions).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('mcp_permissions', ?1)",
        rusqlite::params![json_str],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_mcp_permission(permissions: &McpPermissions, action: &str) -> bool {
    match action {
        "read_tasks" => permissions.read_tasks,
        "read_meetings" => permissions.read_meetings,
        "read_projects" => permissions.read_projects,
        "create_task" => permissions.create_task,
        "update_task" => permissions.update_task,
        "delete_task" => permissions.delete_task,
        "create_meeting_note" => permissions.create_meeting_note,
        "run_skill" => permissions.run_skill,
        _ => false,
    }
}

pub fn get_mcp_permissions_from_db(conn: &rusqlite::Connection) -> Result<McpPermissions, String> {
    let json_str: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'mcp_permissions'",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(s) = json_str {
        serde_json::from_str(&s).map_err(|e| e.to_string())
    } else {
        Ok(McpPermissions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_permissions_default() {
        let perms = McpPermissions::default();
        assert!(perms.read_tasks);
        assert!(perms.read_meetings);
        assert!(perms.read_projects);
        assert!(!perms.create_task);
        assert!(!perms.update_task);
        assert!(!perms.delete_task);
        assert!(!perms.create_meeting_note);
        assert!(!perms.run_skill);
        assert_eq!(perms.rate_limit_per_minute, 100);
    }

    #[test]
    fn test_check_mcp_permission_read_allowed_by_default() {
        let perms = McpPermissions::default();
        assert!(check_mcp_permission(&perms, "read_tasks"));
        assert!(check_mcp_permission(&perms, "read_meetings"));
        assert!(check_mcp_permission(&perms, "read_projects"));
    }

    #[test]
    fn test_check_mcp_permission_write_denied_by_default() {
        let perms = McpPermissions::default();
        assert!(!check_mcp_permission(&perms, "create_task"));
        assert!(!check_mcp_permission(&perms, "update_task"));
        assert!(!check_mcp_permission(&perms, "delete_task"));
        assert!(!check_mcp_permission(&perms, "create_meeting_note"));
        assert!(!check_mcp_permission(&perms, "run_skill"));
    }

    #[test]
    fn test_check_mcp_permission_unknown_action() {
        let perms = McpPermissions::default();
        assert!(!check_mcp_permission(&perms, "unknown_action"));
        assert!(!check_mcp_permission(&perms, ""));
    }

    #[test]
    fn test_check_mcp_permission_with_custom_perms() {
        let perms = McpPermissions {
            read_tasks: false,
            create_task: true,
            delete_task: true,
            ..Default::default()
        };
        assert!(!check_mcp_permission(&perms, "read_tasks"));
        assert!(check_mcp_permission(&perms, "create_task"));
        assert!(check_mcp_permission(&perms, "delete_task"));
    }

    #[test]
    fn test_mcp_permissions_serialization() {
        let perms = McpPermissions {
            create_task: true,
            run_skill: true,
            rate_limit_per_minute: 50,
            ..Default::default()
        };
        let json = serde_json::to_string(&perms).unwrap();
        let parsed: McpPermissions = serde_json::from_str(&json).unwrap();
        assert!(parsed.create_task);
        assert!(parsed.run_skill);
        assert_eq!(parsed.rate_limit_per_minute, 50);
    }

    #[test]
    fn test_mcp_permissions_deserialization_with_defaults() {
        let json = r#"{"create_task": true}"#;
        let perms: McpPermissions = serde_json::from_str(json).unwrap();
        assert!(perms.create_task);
        assert!(perms.read_tasks); // default true
        assert!(!perms.update_task); // default false
        assert_eq!(perms.rate_limit_per_minute, 100); // default
    }

    #[test]
    fn test_get_mcp_permissions_from_db_default() {
        let conn = crate::db::connection::init_test_db().unwrap();
        let perms = get_mcp_permissions_from_db(&conn).unwrap();
        assert!(perms.read_tasks);
        assert!(!perms.create_task);
    }

    #[test]
    fn test_get_mcp_permissions_from_db_custom() {
        let conn = crate::db::connection::init_test_db().unwrap();
        let custom = McpPermissions {
            create_task: true,
            run_skill: true,
            ..Default::default()
        };
        let json = serde_json::to_string(&custom).unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('mcp_permissions', ?1)",
            rusqlite::params![json],
        )
        .unwrap();

        let perms = get_mcp_permissions_from_db(&conn).unwrap();
        assert!(perms.create_task);
        assert!(perms.run_skill);
    }
}
