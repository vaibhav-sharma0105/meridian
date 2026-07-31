use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationProjectMapping {
    pub id: String,
    pub integration_id: String,
    pub external_key: String,
    pub project_id: String,
    pub created_at: String,
}

pub fn create_mapping(
    conn: &Connection,
    integration_id: &str,
    external_key: &str,
    project_id: &str,
) -> Result<IntegrationProjectMapping, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO integration_project_mapping (id, integration_id, external_key, project_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(integration_id, external_key) DO UPDATE SET project_id = excluded.project_id",
        params![id, integration_id, external_key, project_id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(IntegrationProjectMapping {
        id,
        integration_id: integration_id.to_string(),
        external_key: external_key.to_string(),
        project_id: project_id.to_string(),
        created_at: now,
    })
}

pub fn get_mappings_for_project(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<IntegrationProjectMapping>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, integration_id, external_key, project_id, created_at
             FROM integration_project_mapping WHERE project_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([project_id], |row| {
            Ok(IntegrationProjectMapping {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                external_key: row.get(2)?,
                project_id: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_mappings_for_integration(
    conn: &Connection,
    integration_id: &str,
) -> Result<Vec<IntegrationProjectMapping>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, integration_id, external_key, project_id, created_at
             FROM integration_project_mapping WHERE integration_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([integration_id], |row| {
            Ok(IntegrationProjectMapping {
                id: row.get(0)?,
                integration_id: row.get(1)?,
                external_key: row.get(2)?,
                project_id: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn delete_mapping(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM integration_project_mapping WHERE id = ?1",
        [id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
