use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{
    CreateIntegrationInput, CreateLinkInput, Integration, IntegrationCache, IntegrationConfig,
    IntegrationLink, IntegrationPermissions, UpdateIntegrationInput,
};

pub fn list_integrations(conn: &Connection) -> Result<Vec<Integration>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, name, config, permissions, autonomy_mode, status,
                    last_sync, sync_interval_minutes, webhook_token, error_message,
                    created_at, updated_at
             FROM integrations
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let config_str: String = row.get(3)?;
            let permissions_str: Option<String> = row.get(4)?;
            Ok(Integration {
                id: row.get(0)?,
                integration_type: row.get(1)?,
                name: row.get(2)?,
                config: serde_json::from_str(&config_str).unwrap_or_default(),
                permissions: permissions_str
                    .and_then(|s| serde_json::from_str(&s).ok()),
                autonomy_mode: row.get(5)?,
                status: row.get(6)?,
                last_sync: row.get(7)?,
                sync_interval_minutes: row.get(8)?,
                webhook_token: row.get(9)?,
                error_message: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_integration(conn: &Connection, id: &str) -> Result<Option<Integration>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, name, config, permissions, autonomy_mode, status,
                    last_sync, sync_interval_minutes, webhook_token, error_message,
                    created_at, updated_at
             FROM integrations WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([id], |row| {
        let config_str: String = row.get(3)?;
        let permissions_str: Option<String> = row.get(4)?;
        Ok(Integration {
            id: row.get(0)?,
            integration_type: row.get(1)?,
            name: row.get(2)?,
            config: serde_json::from_str(&config_str).unwrap_or_default(),
            permissions: permissions_str.and_then(|s| serde_json::from_str(&s).ok()),
            autonomy_mode: row.get(5)?,
            status: row.get(6)?,
            last_sync: row.get(7)?,
            sync_interval_minutes: row.get(8)?,
            webhook_token: row.get(9)?,
            error_message: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    });

    match result {
        Ok(integration) => Ok(Some(integration)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_integration_by_type(conn: &Connection, integration_type: &str) -> Result<Option<Integration>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, type, name, config, permissions, autonomy_mode, status,
                    last_sync, sync_interval_minutes, webhook_token, error_message,
                    created_at, updated_at
             FROM integrations WHERE type = ?1 LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([integration_type], |row| {
        let config_str: String = row.get(3)?;
        let permissions_str: Option<String> = row.get(4)?;
        Ok(Integration {
            id: row.get(0)?,
            integration_type: row.get(1)?,
            name: row.get(2)?,
            config: serde_json::from_str(&config_str).unwrap_or_default(),
            permissions: permissions_str.and_then(|s| serde_json::from_str(&s).ok()),
            autonomy_mode: row.get(5)?,
            status: row.get(6)?,
            last_sync: row.get(7)?,
            sync_interval_minutes: row.get(8)?,
            webhook_token: row.get(9)?,
            error_message: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    });

    match result {
        Ok(integration) => Ok(Some(integration)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_integration(conn: &Connection, input: CreateIntegrationInput) -> Result<Integration, String> {
    let id = Uuid::new_v4().to_string();
    let webhook_token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let config_json = serde_json::to_string(&input.config).map_err(|e| e.to_string())?;
    let permissions_json = input
        .permissions
        .as_ref()
        .map(|p| serde_json::to_string(p))
        .transpose()
        .map_err(|e| e.to_string())?;
    let autonomy_mode = input.autonomy_mode.unwrap_or_else(|| "manual".to_string());
    let sync_interval = input.sync_interval_minutes.unwrap_or(15);

    conn.execute(
        "INSERT INTO integrations (id, type, name, config, permissions, autonomy_mode,
                                   status, sync_interval_minutes, webhook_token, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'disconnected', ?7, ?8, ?9, ?9)",
        params![
            id,
            input.integration_type,
            input.name,
            config_json,
            permissions_json,
            autonomy_mode,
            sync_interval,
            webhook_token,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_integration(conn, &id)?.ok_or_else(|| "Failed to retrieve created integration".to_string())
}

pub fn update_integration(conn: &Connection, input: UpdateIntegrationInput) -> Result<Integration, String> {
    let existing = get_integration(conn, &input.id)?
        .ok_or_else(|| format!("Integration {} not found", input.id))?;

    let name = input.name.unwrap_or(existing.name);
    let config = input.config.unwrap_or(existing.config);
    let permissions = input.permissions.or(existing.permissions);
    let autonomy_mode = input.autonomy_mode.unwrap_or(existing.autonomy_mode);
    let status = input.status.unwrap_or(existing.status);
    let sync_interval = input.sync_interval_minutes.unwrap_or(existing.sync_interval_minutes);
    let error_message = input.error_message.or(existing.error_message);
    let now = chrono::Utc::now().to_rfc3339();

    let config_json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    let permissions_json = permissions
        .as_ref()
        .map(|p| serde_json::to_string(p))
        .transpose()
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE integrations
         SET name = ?2, config = ?3, permissions = ?4, autonomy_mode = ?5,
             status = ?6, sync_interval_minutes = ?7, error_message = ?8, updated_at = ?9
         WHERE id = ?1",
        params![
            input.id,
            name,
            config_json,
            permissions_json,
            autonomy_mode,
            status,
            sync_interval,
            error_message,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_integration(conn, &input.id)?.ok_or_else(|| "Failed to retrieve updated integration".to_string())
}

pub fn update_integration_status(
    conn: &Connection,
    id: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE integrations SET status = ?2, error_message = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, status, error_message, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_integration_last_sync(conn: &Connection, id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE integrations SET last_sync = ?2, status = 'connected', error_message = NULL, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_integration(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM integrations WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Integration Cache operations

pub fn upsert_cache_item(
    conn: &Connection,
    integration_id: &str,
    external_type: &str,
    external_id: &str,
    external_url: Option<&str>,
    data: &serde_json::Value,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let data_json = serde_json::to_string(data).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO integration_cache (id, integration_id, external_type, external_id, external_url, data, synced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(integration_id, external_type, external_id)
         DO UPDATE SET data = ?6, external_url = ?5, synced_at = ?7",
        params![id, integration_id, external_type, external_id, external_url, data_json, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

pub fn get_cached_items(
    conn: &Connection,
    integration_id: &str,
    external_type: Option<&str>,
) -> Result<Vec<IntegrationCache>, String> {
    let query = if external_type.is_some() {
        "SELECT id, integration_id, external_type, external_id, external_url, data, synced_at
         FROM integration_cache WHERE integration_id = ?1 AND external_type = ?2
         ORDER BY synced_at DESC"
    } else {
        "SELECT id, integration_id, external_type, external_id, external_url, data, synced_at
         FROM integration_cache WHERE integration_id = ?1
         ORDER BY synced_at DESC"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let rows = if let Some(et) = external_type {
        stmt.query_map(params![integration_id, et], map_cache_row)
    } else {
        stmt.query_map(params![integration_id], map_cache_row)
    }
    .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn map_cache_row(row: &rusqlite::Row) -> rusqlite::Result<IntegrationCache> {
    let data_str: String = row.get(5)?;
    Ok(IntegrationCache {
        id: row.get(0)?,
        integration_id: row.get(1)?,
        external_type: row.get(2)?,
        external_id: row.get(3)?,
        external_url: row.get(4)?,
        data: serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null),
        synced_at: row.get(6)?,
    })
}

pub fn clear_cache(conn: &Connection, integration_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM integration_cache WHERE integration_id = ?1",
        [integration_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// Integration Links operations

pub fn create_link(conn: &Connection, input: CreateLinkInput) -> Result<IntegrationLink, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let sync_enabled = input.sync_enabled.unwrap_or(true);

    conn.execute(
        "INSERT INTO integration_links (id, integration_id, local_type, local_id,
                                        external_type, external_id, external_url, sync_enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.integration_id,
            input.local_type,
            input.local_id,
            input.external_type,
            input.external_id,
            input.external_url,
            sync_enabled,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_link(conn, &id)?.ok_or_else(|| "Failed to retrieve created link".to_string())
}

pub fn get_link(conn: &Connection, id: &str) -> Result<Option<IntegrationLink>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, integration_id, local_type, local_id, external_type,
                    external_id, external_url, sync_enabled, created_at
             FROM integration_links WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([id], map_link_row);

    match result {
        Ok(link) => Ok(Some(link)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_links_for_local(
    conn: &Connection,
    local_type: &str,
    local_id: &str,
) -> Result<Vec<IntegrationLink>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, integration_id, local_type, local_id, external_type,
                    external_id, external_url, sync_enabled, created_at
             FROM integration_links WHERE local_type = ?1 AND local_id = ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![local_type, local_id], map_link_row)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_links_for_external(
    conn: &Connection,
    integration_id: &str,
    external_type: &str,
    external_id: &str,
) -> Result<Vec<IntegrationLink>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, integration_id, local_type, local_id, external_type,
                    external_id, external_url, sync_enabled, created_at
             FROM integration_links
             WHERE integration_id = ?1 AND external_type = ?2 AND external_id = ?3",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![integration_id, external_type, external_id], map_link_row)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn map_link_row(row: &rusqlite::Row) -> rusqlite::Result<IntegrationLink> {
    Ok(IntegrationLink {
        id: row.get(0)?,
        integration_id: row.get(1)?,
        local_type: row.get(2)?,
        local_id: row.get(3)?,
        external_type: row.get(4)?,
        external_id: row.get(5)?,
        external_url: row.get(6)?,
        sync_enabled: row.get(7)?,
        created_at: row.get(8)?,
    })
}

pub fn delete_link(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM integration_links WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_links_for_local(
    conn: &Connection,
    local_type: &str,
    local_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM integration_links WHERE local_type = ?1 AND local_id = ?2",
        params![local_type, local_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::init_test_db;

    fn setup_test_db() -> Connection {
        init_test_db().expect("Failed to create test db")
    }

    #[test]
    fn test_create_and_get_integration() {
        let conn = setup_test_db();

        let input = CreateIntegrationInput {
            integration_type: "github".to_string(),
            name: "My GitHub".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: Some("supervised".to_string()),
            sync_interval_minutes: Some(15),
        };

        let integration = create_integration(&conn, input).expect("Failed to create integration");
        assert_eq!(integration.name, "My GitHub");
        assert_eq!(integration.integration_type, "github");
        assert_eq!(integration.status, "disconnected");

        let fetched = get_integration(&conn, &integration.id)
            .expect("Failed to get integration")
            .expect("Integration not found");
        assert_eq!(fetched.id, integration.id);
        assert_eq!(fetched.name, "My GitHub");
    }

    #[test]
    fn test_list_integrations() {
        let conn = setup_test_db();

        let input1 = CreateIntegrationInput {
            integration_type: "github".to_string(),
            name: "GitHub".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let input2 = CreateIntegrationInput {
            integration_type: "jira".to_string(),
            name: "Jira".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };

        create_integration(&conn, input1).expect("Failed to create integration 1");
        create_integration(&conn, input2).expect("Failed to create integration 2");

        let list = list_integrations(&conn).expect("Failed to list integrations");
        assert!(list.len() >= 2);
    }

    #[test]
    fn test_update_integration() {
        let conn = setup_test_db();

        let input = CreateIntegrationInput {
            integration_type: "slack".to_string(),
            name: "Slack".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let integration = create_integration(&conn, input).expect("Failed to create integration");

        let update = UpdateIntegrationInput {
            id: integration.id.clone(),
            name: Some("Updated Slack".to_string()),
            status: Some("connected".to_string()),
            ..Default::default()
        };
        let updated = update_integration(&conn, update).expect("Failed to update integration");
        assert_eq!(updated.name, "Updated Slack");
        assert_eq!(updated.status, "connected");
    }

    #[test]
    fn test_delete_integration() {
        let conn = setup_test_db();

        let input = CreateIntegrationInput {
            integration_type: "github".to_string(),
            name: "To Delete".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let integration = create_integration(&conn, input).expect("Failed to create integration");

        delete_integration(&conn, &integration.id).expect("Failed to delete integration");

        let fetched = get_integration(&conn, &integration.id).expect("Failed to query");
        assert!(fetched.is_none());
    }

    #[test]
    fn test_create_and_get_link() {
        let conn = setup_test_db();

        // First create an integration
        let int_input = CreateIntegrationInput {
            integration_type: "github".to_string(),
            name: "GitHub".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let integration = create_integration(&conn, int_input).expect("Failed to create integration");

        let link_input = CreateLinkInput {
            integration_id: integration.id.clone(),
            local_type: "task".to_string(),
            local_id: "task-123".to_string(),
            external_type: "issue".to_string(),
            external_id: "456".to_string(),
            external_url: Some("https://github.com/test/repo/issues/456".to_string()),
            sync_enabled: Some(true),
        };

        let link = create_link(&conn, link_input).expect("Failed to create link");
        assert_eq!(link.local_type, "task");
        assert_eq!(link.external_type, "issue");

        let links = get_links_for_local(&conn, "task", "task-123").expect("Failed to get links");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].id, link.id);
    }

    #[test]
    fn test_delete_link() {
        let conn = setup_test_db();

        let int_input = CreateIntegrationInput {
            integration_type: "jira".to_string(),
            name: "Jira".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let integration = create_integration(&conn, int_input).expect("Failed to create integration");

        let link_input = CreateLinkInput {
            integration_id: integration.id.clone(),
            local_type: "task".to_string(),
            local_id: "task-789".to_string(),
            external_type: "jira_issue".to_string(),
            external_id: "PROJ-123".to_string(),
            external_url: None,
            sync_enabled: None,
        };

        let link = create_link(&conn, link_input).expect("Failed to create link");
        delete_link(&conn, &link.id).expect("Failed to delete link");

        let fetched = get_link(&conn, &link.id).expect("Failed to query");
        assert!(fetched.is_none());
    }

    #[test]
    fn test_cache_operations() {
        let conn = setup_test_db();

        let int_input = CreateIntegrationInput {
            integration_type: "github".to_string(),
            name: "GitHub Cache Test".to_string(),
            config: IntegrationConfig::default(),
            permissions: None,
            autonomy_mode: None,
            sync_interval_minutes: None,
        };
        let integration = create_integration(&conn, int_input).expect("Failed to create integration");

        // Upsert cache item
        let data = serde_json::json!({"title": "Test Issue", "number": 1});
        upsert_cache_item(
            &conn,
            &integration.id,
            "issue",
            "1",
            Some("https://github.com/test/repo/issues/1"),
            &data,
        )
        .expect("Failed to upsert cache item");

        // Get cached items
        let items = get_cached_items(&conn, &integration.id, Some("issue"))
            .expect("Failed to get cached items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].external_id, "1");

        // Clear cache
        clear_cache(&conn, &integration.id).expect("Failed to clear cache");
        let items_after = get_cached_items(&conn, &integration.id, None)
            .expect("Failed to get cached items");
        assert_eq!(items_after.len(), 0);
    }
}
