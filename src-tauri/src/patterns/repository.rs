use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{CreateObservationInput, PatternModel, PatternObservation, UpsertPatternModelInput};

pub fn insert_observation(conn: &Connection, input: CreateObservationInput) -> Result<PatternObservation, String> {
    let id = Uuid::new_v4().to_string();
    let context_data_str = serde_json::to_string(&input.context_data)
        .map_err(|e| format!("Failed to serialize context_data: {}", e))?;

    conn.execute(
        "INSERT INTO pattern_observations (id, observation_type, entity_type, entity_id, project_id, context_data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.observation_type,
            input.entity_type,
            input.entity_id,
            input.project_id,
            context_data_str,
        ],
    )
    .map_err(|e| format!("Failed to insert observation: {}", e))?;

    get_observation_by_id(conn, &id)
}

pub fn get_observation_by_id(conn: &Connection, id: &str) -> Result<PatternObservation, String> {
    conn.query_row(
        "SELECT id, observation_type, entity_type, entity_id, project_id, context_data, created_at, processed_at
         FROM pattern_observations WHERE id = ?1",
        params![id],
        |row| {
            Ok(PatternObservation {
                id: row.get(0)?,
                observation_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                project_id: row.get(4)?,
                context_data: row.get(5)?,
                created_at: row.get(6)?,
                processed_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("Failed to get observation: {}", e))
}

pub fn get_unprocessed_observations(
    conn: &Connection,
    limit: i64,
    offset: i64,
) -> Result<Vec<PatternObservation>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, observation_type, entity_type, entity_id, project_id, context_data, created_at, processed_at
             FROM pattern_observations
             WHERE processed_at IS NULL
             ORDER BY created_at ASC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let observations = stmt
        .query_map(params![limit, offset], |row| {
            Ok(PatternObservation {
                id: row.get(0)?,
                observation_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                project_id: row.get(4)?,
                context_data: row.get(5)?,
                created_at: row.get(6)?,
                processed_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query observations: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect observations: {}", e))?;

    Ok(observations)
}

pub fn get_observations_by_type(
    conn: &Connection,
    observation_type: &str,
    project_id: Option<&str>,
    processed_only: bool,
) -> Result<Vec<PatternObservation>, String> {
    let sql = if processed_only {
        "SELECT id, observation_type, entity_type, entity_id, project_id, context_data, created_at, processed_at
         FROM pattern_observations
         WHERE observation_type = ?1 AND (?2 IS NULL OR project_id = ?2) AND processed_at IS NOT NULL
         ORDER BY created_at DESC"
    } else {
        "SELECT id, observation_type, entity_type, entity_id, project_id, context_data, created_at, processed_at
         FROM pattern_observations
         WHERE observation_type = ?1 AND (?2 IS NULL OR project_id = ?2)
         ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let observations = stmt
        .query_map(params![observation_type, project_id], |row| {
            Ok(PatternObservation {
                id: row.get(0)?,
                observation_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                project_id: row.get(4)?,
                context_data: row.get(5)?,
                created_at: row.get(6)?,
                processed_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query observations: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect observations: {}", e))?;

    Ok(observations)
}

pub fn mark_observations_processed(conn: &Connection, ids: &[String]) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "UPDATE pattern_observations SET processed_at = datetime('now') WHERE id IN ({})",
        placeholders.join(", ")
    );

    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

    conn.execute(&sql, params.as_slice())
        .map_err(|e| format!("Failed to mark observations processed: {}", e))
}

pub fn prune_old_observations(conn: &Connection, days: i64) -> Result<usize, String> {
    conn.execute(
        "DELETE FROM pattern_observations
         WHERE processed_at IS NOT NULL
         AND datetime(processed_at) < datetime('now', ?1)",
        params![format!("-{} days", days)],
    )
    .map_err(|e| format!("Failed to prune observations: {}", e))
}

pub fn upsert_pattern_model(conn: &Connection, input: UpsertPatternModelInput) -> Result<PatternModel, String> {
    let model_data_str = serde_json::to_string(&input.model_data)
        .map_err(|e| format!("Failed to serialize model_data: {}", e))?;

    let existing = get_pattern_model_by_type(conn, &input.pattern_type, input.project_id.as_deref());

    match existing {
        Ok(model) => {
            conn.execute(
                "UPDATE pattern_models
                 SET model_data = ?1, confidence = ?2, observation_count = ?3, last_updated = datetime('now')
                 WHERE id = ?4",
                params![model_data_str, input.confidence, input.observation_count, model.id],
            )
            .map_err(|e| format!("Failed to update pattern model: {}", e))?;

            get_pattern_model_by_id(conn, &model.id)
        }
        Err(_) => {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO pattern_models (id, pattern_type, project_id, model_data, confidence, observation_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    input.pattern_type,
                    input.project_id,
                    model_data_str,
                    input.confidence,
                    input.observation_count,
                ],
            )
            .map_err(|e| format!("Failed to insert pattern model: {}", e))?;

            get_pattern_model_by_id(conn, &id)
        }
    }
}

pub fn get_pattern_model_by_id(conn: &Connection, id: &str) -> Result<PatternModel, String> {
    conn.query_row(
        "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
         FROM pattern_models WHERE id = ?1",
        params![id],
        |row| {
            Ok(PatternModel {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                project_id: row.get(2)?,
                model_data: row.get(3)?,
                confidence: row.get(4)?,
                observation_count: row.get(5)?,
                last_updated: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Failed to get pattern model: {}", e))
}

pub fn get_pattern_model_by_type(
    conn: &Connection,
    pattern_type: &str,
    project_id: Option<&str>,
) -> Result<PatternModel, String> {
    if let Some(pid) = project_id {
        conn.query_row(
            "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
             FROM pattern_models WHERE pattern_type = ?1 AND project_id = ?2",
            params![pattern_type, pid],
            |row| {
                Ok(PatternModel {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    project_id: row.get(2)?,
                    model_data: row.get(3)?,
                    confidence: row.get(4)?,
                    observation_count: row.get(5)?,
                    last_updated: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get pattern model: {}", e))
    } else {
        conn.query_row(
            "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
             FROM pattern_models WHERE pattern_type = ?1 AND project_id IS NULL",
            params![pattern_type],
            |row| {
                Ok(PatternModel {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    project_id: row.get(2)?,
                    model_data: row.get(3)?,
                    confidence: row.get(4)?,
                    observation_count: row.get(5)?,
                    last_updated: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get pattern model: {}", e))
    }
}

pub fn get_pattern_models_for_project(conn: &Connection, project_id: Option<&str>) -> Result<Vec<PatternModel>, String> {
    let sql = if project_id.is_some() {
        "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
         FROM pattern_models WHERE project_id = ?1 ORDER BY pattern_type"
    } else {
        "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
         FROM pattern_models WHERE project_id IS NULL ORDER BY pattern_type"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let models = stmt
        .query_map(params![project_id], |row| {
            Ok(PatternModel {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                project_id: row.get(2)?,
                model_data: row.get(3)?,
                confidence: row.get(4)?,
                observation_count: row.get(5)?,
                last_updated: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query pattern models: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pattern models: {}", e))?;

    Ok(models)
}

pub fn get_all_pattern_models(conn: &Connection) -> Result<Vec<PatternModel>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, pattern_type, project_id, model_data, confidence, observation_count, last_updated
             FROM pattern_models ORDER BY pattern_type, project_id",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let models = stmt
        .query_map([], |row| {
            Ok(PatternModel {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                project_id: row.get(2)?,
                model_data: row.get(3)?,
                confidence: row.get(4)?,
                observation_count: row.get(5)?,
                last_updated: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query pattern models: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pattern models: {}", e))?;

    Ok(models)
}

pub fn delete_pattern_model(conn: &Connection, pattern_type: &str, project_id: Option<&str>) -> Result<bool, String> {
    let rows = if project_id.is_some() {
        conn.execute(
            "DELETE FROM pattern_models WHERE pattern_type = ?1 AND project_id = ?2",
            params![pattern_type, project_id],
        )
    } else {
        conn.execute(
            "DELETE FROM pattern_models WHERE pattern_type = ?1 AND project_id IS NULL",
            params![pattern_type],
        )
    }
    .map_err(|e| format!("Failed to delete pattern model: {}", e))?;

    Ok(rows > 0)
}

pub fn delete_all_pattern_models(conn: &Connection) -> Result<usize, String> {
    conn.execute("DELETE FROM pattern_models", [])
        .map_err(|e| format!("Failed to delete all pattern models: {}", e))
}

pub fn delete_all_observations(conn: &Connection) -> Result<usize, String> {
    conn.execute("DELETE FROM pattern_observations", [])
        .map_err(|e| format!("Failed to delete all observations: {}", e))
}

pub fn apply_pattern_decay(conn: &Connection, decay_rate: f64, inactive_days: i64) -> Result<usize, String> {
    conn.execute(
        "UPDATE pattern_models
         SET confidence = confidence * ?1, last_updated = datetime('now')
         WHERE datetime(last_updated) < datetime('now', ?2)",
        params![1.0 - decay_rate, format!("-{} days", inactive_days)],
    )
    .map_err(|e| format!("Failed to apply pattern decay: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE pattern_observations (
                id TEXT PRIMARY KEY,
                observation_type TEXT NOT NULL,
                entity_type TEXT,
                entity_id TEXT,
                project_id TEXT,
                context_data TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                processed_at TEXT
            );

            CREATE TABLE pattern_models (
                id TEXT PRIMARY KEY,
                pattern_type TEXT NOT NULL,
                project_id TEXT,
                model_data TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.0,
                observation_count INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(pattern_type, project_id)
            );
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_insert_observation() {
        let conn = setup_test_db();

        let input = CreateObservationInput {
            observation_type: "task_completion".to_string(),
            entity_type: Some("task".to_string()),
            entity_id: Some("task-123".to_string()),
            project_id: Some("proj-1".to_string()),
            context_data: json!({"task_title": "Fix bug", "task_keywords": ["fix", "bug"]}),
        };

        let obs = insert_observation(&conn, input).unwrap();

        assert_eq!(obs.observation_type, "task_completion");
        assert_eq!(obs.entity_type, Some("task".to_string()));
        assert_eq!(obs.entity_id, Some("task-123".to_string()));
        assert_eq!(obs.project_id, Some("proj-1".to_string()));
        assert!(obs.processed_at.is_none());
    }

    #[test]
    fn test_get_unprocessed_observations() {
        let conn = setup_test_db();

        // Insert multiple observations
        for i in 0..5 {
            let input = CreateObservationInput {
                observation_type: "task_completion".to_string(),
                entity_type: Some("task".to_string()),
                entity_id: Some(format!("task-{}", i)),
                project_id: Some("proj-1".to_string()),
                context_data: json!({"index": i}),
            };
            insert_observation(&conn, input).unwrap();
        }

        // Get unprocessed with pagination
        let page1 = get_unprocessed_observations(&conn, 3, 0).unwrap();
        assert_eq!(page1.len(), 3);

        let page2 = get_unprocessed_observations(&conn, 3, 3).unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_mark_observations_processed() {
        let conn = setup_test_db();

        let input = CreateObservationInput {
            observation_type: "priority_set".to_string(),
            entity_type: Some("task".to_string()),
            entity_id: Some("task-1".to_string()),
            project_id: None,
            context_data: json!({"old_priority": "low", "new_priority": "high"}),
        };
        let obs = insert_observation(&conn, input).unwrap();
        assert!(obs.processed_at.is_none());

        // Mark as processed
        let count = mark_observations_processed(&conn, &[obs.id.clone()]).unwrap();
        assert_eq!(count, 1);

        // Verify it's processed
        let updated = get_observation_by_id(&conn, &obs.id).unwrap();
        assert!(updated.processed_at.is_some());

        // Should not appear in unprocessed
        let unprocessed = get_unprocessed_observations(&conn, 10, 0).unwrap();
        assert!(unprocessed.is_empty());
    }

    #[test]
    fn test_upsert_pattern_model_create() {
        let conn = setup_test_db();

        let input = UpsertPatternModelInput {
            pattern_type: "workflow_sequence".to_string(),
            project_id: Some("proj-1".to_string()),
            model_data: json!({"sequences": []}),
            confidence: 0.5,
            observation_count: 10,
        };

        let model = upsert_pattern_model(&conn, input).unwrap();

        assert_eq!(model.pattern_type, "workflow_sequence");
        assert_eq!(model.project_id, Some("proj-1".to_string()));
        assert_eq!(model.confidence, 0.5);
        assert_eq!(model.observation_count, 10);
    }

    #[test]
    fn test_upsert_pattern_model_update() {
        let conn = setup_test_db();

        // Create initial
        let input1 = UpsertPatternModelInput {
            pattern_type: "smart_defaults".to_string(),
            project_id: None,
            model_data: json!({"priority_patterns": []}),
            confidence: 0.3,
            observation_count: 5,
        };
        let model1 = upsert_pattern_model(&conn, input1).unwrap();

        // Update with new data
        let input2 = UpsertPatternModelInput {
            pattern_type: "smart_defaults".to_string(),
            project_id: None,
            model_data: json!({"priority_patterns": [{"keyword": "bug", "priority": "high"}]}),
            confidence: 0.7,
            observation_count: 15,
        };
        let model2 = upsert_pattern_model(&conn, input2).unwrap();

        // Should be same ID (updated, not new)
        assert_eq!(model1.id, model2.id);
        assert_eq!(model2.confidence, 0.7);
        assert_eq!(model2.observation_count, 15);
    }

    #[test]
    fn test_get_pattern_models_for_project() {
        let conn = setup_test_db();

        // Create models for different projects
        upsert_pattern_model(
            &conn,
            UpsertPatternModelInput {
                pattern_type: "workflow_sequence".to_string(),
                project_id: Some("proj-1".to_string()),
                model_data: json!({}),
                confidence: 0.5,
                observation_count: 10,
            },
        )
        .unwrap();

        upsert_pattern_model(
            &conn,
            UpsertPatternModelInput {
                pattern_type: "smart_defaults".to_string(),
                project_id: Some("proj-1".to_string()),
                model_data: json!({}),
                confidence: 0.6,
                observation_count: 8,
            },
        )
        .unwrap();

        upsert_pattern_model(
            &conn,
            UpsertPatternModelInput {
                pattern_type: "workflow_sequence".to_string(),
                project_id: Some("proj-2".to_string()),
                model_data: json!({}),
                confidence: 0.4,
                observation_count: 5,
            },
        )
        .unwrap();

        // Get for proj-1
        let proj1_models = get_pattern_models_for_project(&conn, Some("proj-1")).unwrap();
        assert_eq!(proj1_models.len(), 2);

        // Get for proj-2
        let proj2_models = get_pattern_models_for_project(&conn, Some("proj-2")).unwrap();
        assert_eq!(proj2_models.len(), 1);
    }

    #[test]
    fn test_delete_pattern_model() {
        let conn = setup_test_db();

        upsert_pattern_model(
            &conn,
            UpsertPatternModelInput {
                pattern_type: "communication_style".to_string(),
                project_id: None,
                model_data: json!({"length_preference": "concise"}),
                confidence: 0.8,
                observation_count: 20,
            },
        )
        .unwrap();

        // Delete it
        let deleted = delete_pattern_model(&conn, "communication_style", None).unwrap();
        assert!(deleted);

        // Should not exist
        let result = get_pattern_model_by_type(&conn, "communication_style", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_prune_old_observations() {
        let conn = setup_test_db();

        // Insert and mark as processed with old date
        conn.execute(
            "INSERT INTO pattern_observations (id, observation_type, context_data, created_at, processed_at)
             VALUES ('old-1', 'test', '{}', datetime('now', '-100 days'), datetime('now', '-100 days'))",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO pattern_observations (id, observation_type, context_data, created_at, processed_at)
             VALUES ('new-1', 'test', '{}', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        // Prune observations older than 90 days
        let pruned = prune_old_observations(&conn, 90).unwrap();
        assert_eq!(pruned, 1);

        // Old one should be gone, new one should remain
        assert!(get_observation_by_id(&conn, "old-1").is_err());
        assert!(get_observation_by_id(&conn, "new-1").is_ok());
    }
}
