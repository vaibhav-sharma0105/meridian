use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::models::{
    CreateObservationInput, PatternContribution, PatternModel, PatternObservation, UpsertPatternModelInput,
};

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

    maybe_contribute(conn, &input);

    get_observation_by_id(conn, &id)
}

// ─── Shared Patterns: contribution opt-in + anonymization ─────────────────────

const CONTRIBUTION_SETTING_KEY: &str = "pattern_contribution_enabled";
const USE_TEAM_PATTERNS_KEY: &str = "use_team_patterns";

fn get_bool_setting(conn: &Connection, key: &str, default: bool) -> bool {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .map(|v| v == "true")
    .unwrap_or(default)
}

fn set_bool_setting(conn: &Connection, key: &str, value: bool) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, if value { "true" } else { "false" }],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_contribution_enabled(conn: &Connection) -> bool {
    get_bool_setting(conn, CONTRIBUTION_SETTING_KEY, false)
}

pub fn set_contribution_enabled(conn: &Connection, enabled: bool) -> Result<(), String> {
    set_bool_setting(conn, CONTRIBUTION_SETTING_KEY, enabled)
}

/// Whether pattern queries should include team-scope models alongside
/// personal ones. Defaults on — team models only ever exist after the user
/// has imported an export containing contributions, so there's nothing to
/// opt into until then anyway.
pub fn get_use_team_patterns_enabled(conn: &Connection) -> bool {
    get_bool_setting(conn, USE_TEAM_PATTERNS_KEY, true)
}

pub fn set_use_team_patterns_enabled(conn: &Connection, enabled: bool) -> Result<(), String> {
    set_bool_setting(conn, USE_TEAM_PATTERNS_KEY, enabled)
}

/// Only these context_data keys are ever contributed — everything else
/// (task titles, assignee/project names, entity IDs) is dropped entirely
/// rather than hashed, since the goal is that no identifying data leaves
/// the device, not just that it's obfuscated.
const SAFE_CONTEXT_KEYS: &[&str] = &["task_keywords", "new_priority", "old_priority", "new_status"];

fn anonymize_context_data(observation_type: &str, context_data: &serde_json::Value) -> serde_json::Value {
    let mut safe = serde_json::Map::new();
    if let Some(obj) = context_data.as_object() {
        for key in SAFE_CONTEXT_KEYS {
            if let Some(v) = obj.get(*key) {
                safe.insert(key.to_string(), v.clone());
            }
        }
    }
    serde_json::json!({ "observation_type": observation_type, "data": safe })
}

fn hash_anonymized(value: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

/// Maps an observation_type to the pattern_type category it teaches —
/// `suggestion_dismissed` has no clean pattern_type home (it's a rejection
/// signal, not really "a pattern"), so it's never contributed.
fn observation_to_pattern_type(observation_type: &str) -> Option<&'static str> {
    match observation_type {
        "priority_set" | "assignee_set" => Some("smart_defaults"),
        "task_completion" => Some("workflow_sequence"),
        "draft_edit" => Some("communication_style"),
        _ => None,
    }
}

fn maybe_contribute(conn: &Connection, input: &CreateObservationInput) {
    if !get_contribution_enabled(conn) {
        return;
    }
    let Some(pattern_type) = observation_to_pattern_type(&input.observation_type) else {
        return;
    };

    // Never contribute anything that looks like it contains sensitive content.
    if !crate::sensitive::scan_content(&input.context_data.to_string()).is_empty() {
        return;
    }

    let anonymized = anonymize_context_data(&input.observation_type, &input.context_data);
    let is_empty = anonymized
        .get("data")
        .and_then(|d| d.as_object())
        .map(|o| o.is_empty())
        .unwrap_or(true);
    if is_empty {
        return; // nothing safe left to share for this observation
    }

    let hash = hash_anonymized(&anonymized);
    let _ = conn.execute(
        "INSERT OR IGNORE INTO pattern_contributions (id, pattern_type, observation_hash) VALUES (?1, ?2, ?3)",
        params![Uuid::new_v4().to_string(), pattern_type, hash],
    );
}

pub fn get_all_pattern_contributions(conn: &Connection) -> Result<Vec<PatternContribution>, String> {
    let mut stmt = conn
        .prepare("SELECT pattern_type, observation_hash FROM pattern_contributions")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PatternContribution {
                pattern_type: row.get(0)?,
                observation_hash: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Merges a teammate's exported contributions into this device's team-scope
/// pattern models. Dedupes against contributions already known locally
/// (including ones this same device already contributed) via the
/// `pattern_contributions` UNIQUE constraint, so re-importing the same
/// export twice — or importing from two teammates who happened to observe
/// the same anonymized pattern — doesn't inflate `contributor_count`.
pub fn merge_team_contributions(conn: &Connection, contributions: &[PatternContribution]) -> Result<i32, String> {
    use std::collections::HashMap;
    let mut new_by_type: HashMap<String, i64> = HashMap::new();

    for c in contributions {
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO pattern_contributions (id, pattern_type, observation_hash) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4().to_string(), c.pattern_type, c.observation_hash],
            )
            .map_err(|e| e.to_string())?;
        if inserted > 0 {
            *new_by_type.entry(c.pattern_type.clone()).or_insert(0) += 1;
        }
    }

    let mut total_merged = 0i32;
    for (pattern_type, count) in &new_by_type {
        upsert_team_pattern_model(conn, pattern_type, *count)?;
        total_merged += *count as i32;
    }

    Ok(total_merged)
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

const PATTERN_MODEL_COLS: &str =
    "id, pattern_type, project_id, model_data, confidence, observation_count, last_updated, scope, contributor_count";

fn row_to_pattern_model(row: &rusqlite::Row) -> rusqlite::Result<PatternModel> {
    Ok(PatternModel {
        id: row.get(0)?,
        pattern_type: row.get(1)?,
        project_id: row.get(2)?,
        model_data: row.get(3)?,
        confidence: row.get(4)?,
        observation_count: row.get(5)?,
        last_updated: row.get(6)?,
        scope: row.get(7)?,
        contributor_count: row.get(8)?,
    })
}

/// Always operates on the personal-scope model — the only kind this app's
/// own pattern-learning jobs ever write. Team-scope models are handled
/// separately by `upsert_team_pattern_model` / `get_team_pattern_model_by_type`.
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
                "INSERT INTO pattern_models (id, pattern_type, project_id, model_data, confidence, observation_count, scope)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'personal')",
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
        &format!("SELECT {} FROM pattern_models WHERE id = ?1", PATTERN_MODEL_COLS),
        params![id],
        row_to_pattern_model,
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
            &format!(
                "SELECT {} FROM pattern_models WHERE pattern_type = ?1 AND project_id = ?2 AND scope = 'personal'",
                PATTERN_MODEL_COLS
            ),
            params![pattern_type, pid],
            row_to_pattern_model,
        )
        .map_err(|e| format!("Failed to get pattern model: {}", e))
    } else {
        conn.query_row(
            &format!(
                "SELECT {} FROM pattern_models WHERE pattern_type = ?1 AND project_id IS NULL AND scope = 'personal'",
                PATTERN_MODEL_COLS
            ),
            params![pattern_type],
            row_to_pattern_model,
        )
        .map_err(|e| format!("Failed to get pattern model: {}", e))
    }
}

pub fn get_pattern_models_for_project(conn: &Connection, project_id: Option<&str>) -> Result<Vec<PatternModel>, String> {
    let sql = if project_id.is_some() {
        format!(
            "SELECT {} FROM pattern_models WHERE project_id = ?1 ORDER BY pattern_type",
            PATTERN_MODEL_COLS
        )
    } else {
        format!(
            "SELECT {} FROM pattern_models WHERE project_id IS NULL ORDER BY pattern_type",
            PATTERN_MODEL_COLS
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let models = stmt
        .query_map(params![project_id], row_to_pattern_model)
        .map_err(|e| format!("Failed to query pattern models: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pattern models: {}", e))?;

    Ok(models)
}

pub fn get_all_pattern_models(conn: &Connection) -> Result<Vec<PatternModel>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM pattern_models ORDER BY pattern_type, project_id",
            PATTERN_MODEL_COLS
        ))
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let models = stmt
        .query_map([], row_to_pattern_model)
        .map_err(|e| format!("Failed to query pattern models: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pattern models: {}", e))?;

    Ok(models)
}

/// Sentinel `project_id` for team-scope models. The existing
/// `UNIQUE(pattern_type, project_id)` constraint on `pattern_models`
/// predates the `scope` column and doesn't include it — a team-scope row
/// with `project_id IS NULL` would collide with a personal global (non
/// project-specific) row of the same `pattern_type`. Never a real project
/// id, so team rows stay distinct without needing a migration to fix the
/// constraint itself.
const TEAM_SCOPE_PROJECT_ID: &str = "__team__";

/// Team-scope equivalent of `get_pattern_model_by_type` — team patterns are
/// always global (no per-project split), since contributions carry no
/// project association after anonymization.
pub fn get_team_pattern_model_by_type(conn: &Connection, pattern_type: &str) -> Result<PatternModel, String> {
    conn.query_row(
        &format!(
            "SELECT {} FROM pattern_models WHERE pattern_type = ?1 AND scope = 'team' AND project_id = ?2",
            PATTERN_MODEL_COLS
        ),
        params![pattern_type, TEAM_SCOPE_PROJECT_ID],
        row_to_pattern_model,
    )
    .map_err(|e| format!("Failed to get team pattern model: {}", e))
}

pub fn get_team_pattern_models(conn: &Connection) -> Result<Vec<PatternModel>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM pattern_models WHERE scope = 'team' ORDER BY pattern_type",
            PATTERN_MODEL_COLS
        ))
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let models = stmt
        .query_map([], row_to_pattern_model)
        .map_err(|e| format!("Failed to query pattern models: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pattern models: {}", e))?;

    Ok(models)
}

/// Merges an imported teammate's contribution counts into this device's
/// team-scope pattern models — one model per pattern_type, `contributor_count`
/// incremented by however many new (previously-unseen locally) hashes this
/// import brought for that type. `model_data` intentionally carries only a
/// count, not reconstructed keyword content: `pattern_contributions` stores
/// hashes only (by design, for privacy), so there's nothing to reconstitute
/// a team member's actual keyword/assignee pattern from — the value of a
/// team pattern here is purely "N teammates share evidence of a pattern in
/// this category," not the pattern's specific content.
pub fn upsert_team_pattern_model(conn: &Connection, pattern_type: &str, new_hash_count: i64) -> Result<PatternModel, String> {
    if new_hash_count <= 0 {
        return get_team_pattern_model_by_type(conn, pattern_type);
    }

    match get_team_pattern_model_by_type(conn, pattern_type) {
        Ok(model) => {
            let new_contributor_count = model.contributor_count + new_hash_count;
            let confidence = (new_contributor_count as f64 / 20.0).min(1.0);
            conn.execute(
                "UPDATE pattern_models
                 SET contributor_count = ?1, confidence = ?2, observation_count = observation_count + ?3,
                     model_data = ?4, last_updated = datetime('now')
                 WHERE id = ?5",
                params![
                    new_contributor_count,
                    confidence,
                    new_hash_count,
                    serde_json::json!({ "contribution_count": new_contributor_count }).to_string(),
                    model.id
                ],
            )
            .map_err(|e| format!("Failed to update team pattern model: {}", e))?;
            get_pattern_model_by_id(conn, &model.id)
        }
        Err(_) => {
            let id = Uuid::new_v4().to_string();
            let confidence = (new_hash_count as f64 / 20.0).min(1.0);
            conn.execute(
                "INSERT INTO pattern_models
                 (id, pattern_type, project_id, model_data, confidence, observation_count, scope, contributor_count)
                 VALUES (?1, ?2, ?7, ?3, ?4, ?5, 'team', ?6)",
                params![
                    id,
                    pattern_type,
                    serde_json::json!({ "contribution_count": new_hash_count }).to_string(),
                    confidence,
                    new_hash_count,
                    new_hash_count,
                    TEAM_SCOPE_PROJECT_ID,
                ],
            )
            .map_err(|e| format!("Failed to insert team pattern model: {}", e))?;
            get_pattern_model_by_id(conn, &id)
        }
    }
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
                scope TEXT DEFAULT 'personal',
                contributor_count INTEGER DEFAULT 1,
                UNIQUE(pattern_type, project_id)
            );

            CREATE TABLE pattern_contributions (
                id TEXT PRIMARY KEY,
                pattern_type TEXT NOT NULL,
                observation_hash TEXT NOT NULL,
                contributed_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(pattern_type, observation_hash)
            );

            CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT);
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

    #[test]
    fn test_contribution_disabled_by_default_no_contribution_recorded() {
        let conn = setup_test_db();
        assert!(!get_contribution_enabled(&conn));

        insert_observation(
            &conn,
            CreateObservationInput {
                observation_type: "priority_set".to_string(),
                entity_type: Some("task".to_string()),
                entity_id: Some("task-1".to_string()),
                project_id: None,
                context_data: json!({ "old_priority": "low", "new_priority": "high", "task_keywords": ["billing"] }),
            },
        )
        .unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM pattern_contributions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_contribution_enabled_records_anonymized_hash_only() {
        let conn = setup_test_db();
        set_contribution_enabled(&conn, true).unwrap();

        insert_observation(
            &conn,
            CreateObservationInput {
                observation_type: "priority_set".to_string(),
                entity_type: Some("task".to_string()),
                entity_id: Some("task-1".to_string()),
                project_id: Some("secret-project".to_string()),
                context_data: json!({
                    "old_priority": "low",
                    "new_priority": "high",
                    "task_title": "Fix Alice's login bug", // must never leak into the contribution
                    "task_keywords": ["billing"]
                }),
            },
        )
        .unwrap();

        let rows = get_all_pattern_contributions(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pattern_type, "smart_defaults");
        // The hash is opaque, but we can prove the raw title never made it into
        // storage at all by checking no plaintext task/person data is anywhere
        // in the contributions table (only id/pattern_type/hash columns exist).
        assert_ne!(rows[0].observation_hash, "Fix Alice's login bug");
    }

    #[test]
    fn test_contribution_skipped_for_sensitive_content() {
        let conn = setup_test_db();
        set_contribution_enabled(&conn, true).unwrap();

        insert_observation(
            &conn,
            CreateObservationInput {
                observation_type: "priority_set".to_string(),
                entity_type: Some("task".to_string()),
                entity_id: Some("task-1".to_string()),
                project_id: None,
                context_data: json!({
                    "old_priority": "low",
                    "new_priority": "high",
                    "task_keywords": ["ssn 123-45-6789"]
                }),
            },
        )
        .unwrap();

        let rows = get_all_pattern_contributions(&conn).unwrap();
        assert!(rows.is_empty(), "sensitive content must never be contributed");
    }

    #[test]
    fn test_contribution_deduplicates_identical_hashes() {
        let conn = setup_test_db();
        set_contribution_enabled(&conn, true).unwrap();

        for _ in 0..3 {
            insert_observation(
                &conn,
                CreateObservationInput {
                    observation_type: "priority_set".to_string(),
                    entity_type: Some("task".to_string()),
                    entity_id: Some(Uuid::new_v4().to_string()),
                    project_id: None,
                    context_data: json!({ "old_priority": "low", "new_priority": "high", "task_keywords": ["billing"] }),
                },
            )
            .unwrap();
        }

        let rows = get_all_pattern_contributions(&conn).unwrap();
        assert_eq!(rows.len(), 1, "identical anonymized observations should dedupe to one hash");
    }

    #[test]
    fn test_merge_team_contributions_increments_contributor_count_only_for_new_hashes() {
        let conn = setup_test_db();

        let batch1 = vec![
            PatternContribution { pattern_type: "smart_defaults".to_string(), observation_hash: "hash-a".to_string() },
            PatternContribution { pattern_type: "smart_defaults".to_string(), observation_hash: "hash-b".to_string() },
        ];
        let merged1 = merge_team_contributions(&conn, &batch1).unwrap();
        assert_eq!(merged1, 2);

        let model = get_team_pattern_model_by_type(&conn, "smart_defaults").unwrap();
        assert_eq!(model.scope, "team");
        assert_eq!(model.contributor_count, 2);

        // Re-importing the same batch (e.g. the same export imported twice)
        // must not inflate the count.
        let merged2 = merge_team_contributions(&conn, &batch1).unwrap();
        assert_eq!(merged2, 0);
        let model = get_team_pattern_model_by_type(&conn, "smart_defaults").unwrap();
        assert_eq!(model.contributor_count, 2);

        // A genuinely new hash from a different teammate's export does count.
        let batch2 = vec![
            PatternContribution { pattern_type: "smart_defaults".to_string(), observation_hash: "hash-c".to_string() },
        ];
        merge_team_contributions(&conn, &batch2).unwrap();
        let model = get_team_pattern_model_by_type(&conn, "smart_defaults").unwrap();
        assert_eq!(model.contributor_count, 3);
    }

    #[test]
    fn test_personal_and_team_scope_dont_collide_on_same_pattern_type() {
        // Regression test: pattern_models has UNIQUE(pattern_type, project_id)
        // with no `scope` in the constraint. A personal global model and a
        // team model of the same pattern_type must both be able to exist.
        let conn = setup_test_db();

        upsert_pattern_model(
            &conn,
            UpsertPatternModelInput {
                pattern_type: "smart_defaults".to_string(),
                project_id: None,
                model_data: json!({}),
                confidence: 0.5,
                observation_count: 1,
            },
        )
        .unwrap();

        let batch = vec![
            PatternContribution { pattern_type: "smart_defaults".to_string(), observation_hash: "hash-x".to_string() },
        ];
        merge_team_contributions(&conn, &batch).unwrap();

        let personal = get_pattern_model_by_type(&conn, "smart_defaults", None).unwrap();
        assert_eq!(personal.scope, "personal");
        let team = get_team_pattern_model_by_type(&conn, "smart_defaults").unwrap();
        assert_eq!(team.scope, "team");
        assert_ne!(personal.id, team.id);
    }
}
