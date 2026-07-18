use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{
    CreateSkillInput, CreateSkillRunInput, Skill, SkillFilters, SkillRun, SkillStats,
    UpdateSkillInput,
};

// ─── Skill CRUD ──────────────────────────────────────────────────────────────

pub fn create_skill(conn: &Connection, input: &CreateSkillInput) -> Result<Skill, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let trigger_config = input
        .trigger_config
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());
    let context_config = input
        .context_config
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());
    let action_config = input
        .action_config
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());
    let tags = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".to_string()));
    let approval_mode = input.approval_mode.clone().unwrap_or_else(|| "notify".to_string());

    conn.execute(
        "INSERT INTO skills (id, name, description, trigger_type, trigger_config, context_config,
         action_config, approval_mode, category, icon, tags, is_builtin, shared, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            id,
            input.name,
            input.description,
            input.trigger_type,
            trigger_config,
            context_config,
            action_config,
            approval_mode,
            input.category,
            input.icon,
            tags,
            input.is_builtin as i32,
            input.shared as i32,
            now,
            now,
        ],
    )
    .map_err(|e| format!("Failed to create skill: {}", e))?;

    get_skill(conn, &id)
}

pub fn get_skill(conn: &Connection, id: &str) -> Result<Skill, String> {
    conn.query_row(
        "SELECT id, name, description, trigger_type, trigger_config, context_config, action_config,
         approval_mode, enabled, shared, owner_id, category, icon, tags, next_run_at, cloned_from_id,
         is_builtin, created_at, updated_at
         FROM skills WHERE id = ?1",
        params![id],
        |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_config: row.get(4)?,
                context_config: row.get(5)?,
                action_config: row.get(6)?,
                approval_mode: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                shared: row.get::<_, i32>(9)? != 0,
                owner_id: row.get(10)?,
                category: row.get(11)?,
                icon: row.get(12)?,
                tags: row.get(13)?,
                next_run_at: row.get(14)?,
                cloned_from_id: row.get(15)?,
                is_builtin: row.get::<_, i32>(16)? != 0,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        },
    )
    .map_err(|e| format!("Skill not found: {}", e))
}

pub fn list_skills(conn: &Connection, filters: &SkillFilters) -> Result<Vec<Skill>, String> {
    let mut sql = String::from(
        "SELECT id, name, description, trigger_type, trigger_config, context_config, action_config,
         approval_mode, enabled, shared, owner_id, category, icon, tags, next_run_at, cloned_from_id,
         is_builtin, created_at, updated_at FROM skills WHERE 1=1",
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(enabled) = filters.enabled {
        sql.push_str(" AND enabled = ?");
        params_vec.push(Box::new(enabled as i32));
    }

    if let Some(shared) = filters.shared {
        sql.push_str(" AND shared = ?");
        params_vec.push(Box::new(shared as i32));
    }

    if let Some(ref category) = filters.category {
        sql.push_str(" AND category = ?");
        params_vec.push(Box::new(category.clone()));
    }

    if let Some(ref trigger_type) = filters.trigger_type {
        sql.push_str(" AND trigger_type = ?");
        params_vec.push(Box::new(trigger_type.clone()));
    }

    if let Some(ref search) = filters.search {
        sql.push_str(" AND (name LIKE ? OR description LIKE ?)");
        let pattern = format!("%{}%", search);
        params_vec.push(Box::new(pattern.clone()));
        params_vec.push(Box::new(pattern));
    }

    sql.push_str(" ORDER BY created_at DESC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let skills = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_config: row.get(4)?,
                context_config: row.get(5)?,
                action_config: row.get(6)?,
                approval_mode: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                shared: row.get::<_, i32>(9)? != 0,
                owner_id: row.get(10)?,
                category: row.get(11)?,
                icon: row.get(12)?,
                tags: row.get(13)?,
                next_run_at: row.get(14)?,
                cloned_from_id: row.get(15)?,
                is_builtin: row.get::<_, i32>(16)? != 0,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

pub fn update_skill(conn: &Connection, input: &UpdateSkillInput) -> Result<Skill, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];
    let mut idx = 2;

    macro_rules! add_field {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                sets.push(format!("{} = ?{}", $field, idx));
                params_vec.push(Box::new(v.clone()));
                idx += 1;
            }
        };
    }

    add_field!("name", &input.name);
    add_field!("description", &input.description);
    add_field!("trigger_type", &input.trigger_type);
    add_field!("approval_mode", &input.approval_mode);
    add_field!("category", &input.category);
    add_field!("icon", &input.icon);

    if let Some(enabled) = input.enabled {
        sets.push(format!("enabled = ?{}", idx));
        params_vec.push(Box::new(enabled as i32));
        idx += 1;
    }

    if let Some(shared) = input.shared {
        sets.push(format!("shared = ?{}", idx));
        params_vec.push(Box::new(shared as i32));
        idx += 1;
    }

    if let Some(ref trigger_config) = input.trigger_config {
        sets.push(format!("trigger_config = ?{}", idx));
        let json = serde_json::to_string(trigger_config).unwrap_or_default();
        params_vec.push(Box::new(json));
        idx += 1;
    }

    if let Some(ref context_config) = input.context_config {
        sets.push(format!("context_config = ?{}", idx));
        let json = serde_json::to_string(context_config).unwrap_or_default();
        params_vec.push(Box::new(json));
        idx += 1;
    }

    if let Some(ref action_config) = input.action_config {
        sets.push(format!("action_config = ?{}", idx));
        let json = serde_json::to_string(action_config).unwrap_or_default();
        params_vec.push(Box::new(json));
        idx += 1;
    }

    if let Some(ref tags) = input.tags {
        sets.push(format!("tags = ?{}", idx));
        let json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        params_vec.push(Box::new(json));
        idx += 1;
    }

    params_vec.push(Box::new(input.id.clone()));
    let sql = format!("UPDATE skills SET {} WHERE id = ?{}", sets.join(", "), idx);

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params_refs.as_slice())
        .map_err(|e| format!("Failed to update skill: {}", e))?;

    get_skill(conn, &input.id)
}

pub fn delete_skill(conn: &Connection, id: &str) -> Result<(), String> {
    let is_builtin: bool = conn
        .query_row(
            "SELECT is_builtin FROM skills WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    if is_builtin {
        return Err("Cannot delete built-in skills. Use 'Reset defaults' to restore them.".to_string());
    }

    conn.execute("DELETE FROM skills WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete skill: {}", e))?;
    Ok(())
}

pub fn get_due_scheduled_skills(conn: &Connection) -> Result<Vec<Skill>, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, trigger_type, trigger_config, context_config, action_config,
             approval_mode, enabled, shared, owner_id, category, icon, tags, next_run_at, cloned_from_id,
             is_builtin, created_at, updated_at
             FROM skills
             WHERE trigger_type = 'schedule' AND enabled = 1 AND next_run_at <= ?1",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map(params![now], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_config: row.get(4)?,
                context_config: row.get(5)?,
                action_config: row.get(6)?,
                approval_mode: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                shared: row.get::<_, i32>(9)? != 0,
                owner_id: row.get(10)?,
                category: row.get(11)?,
                icon: row.get(12)?,
                tags: row.get(13)?,
                next_run_at: row.get(14)?,
                cloned_from_id: row.get(15)?,
                is_builtin: row.get::<_, i32>(16)? != 0,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

pub fn get_skills_for_event(conn: &Connection, event_type: &str) -> Result<Vec<Skill>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, trigger_type, trigger_config, context_config, action_config,
             approval_mode, enabled, shared, owner_id, category, icon, tags, next_run_at, cloned_from_id,
             is_builtin, created_at, updated_at
             FROM skills
             WHERE trigger_type = 'event' AND enabled = 1",
        )
        .map_err(|e| e.to_string())?;

    let all_skills = stmt
        .query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_config: row.get(4)?,
                context_config: row.get(5)?,
                action_config: row.get(6)?,
                approval_mode: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                shared: row.get::<_, i32>(9)? != 0,
                owner_id: row.get(10)?,
                category: row.get(11)?,
                icon: row.get(12)?,
                tags: row.get(13)?,
                next_run_at: row.get(14)?,
                cloned_from_id: row.get(15)?,
                is_builtin: row.get::<_, i32>(16)? != 0,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Filter by event_type in trigger_config
    let matching = all_skills
        .into_iter()
        .filter(|s| {
            s.get_trigger_config()
                .and_then(|c| c.event_type)
                .map(|et| et == event_type)
                .unwrap_or(false)
        })
        .collect();

    Ok(matching)
}

pub fn update_next_run_at(conn: &Connection, skill_id: &str, next_run_at: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE skills SET next_run_at = ?1 WHERE id = ?2",
        params![next_run_at, skill_id],
    )
    .map_err(|e| format!("Failed to update next_run_at: {}", e))?;
    Ok(())
}

// ─── Skill Runs ──────────────────────────────────────────────────────────────

pub fn create_skill_run(conn: &Connection, input: &CreateSkillRunInput) -> Result<SkillRun, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let trigger_context = input
        .trigger_context
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());

    conn.execute(
        "INSERT INTO skill_runs (id, skill_id, status, trigger_type, trigger_context, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5)",
        params![id, input.skill_id, input.trigger_type, trigger_context, now],
    )
    .map_err(|e| format!("Failed to create skill run: {}", e))?;

    get_skill_run(conn, &id)
}

pub fn get_skill_run(conn: &Connection, id: &str) -> Result<SkillRun, String> {
    conn.query_row(
        "SELECT id, skill_id, status, trigger_type, trigger_context, output, error, pending_changes,
         started_at, completed_at, duration_ms, approval_decision, approval_reason, created_at
         FROM skill_runs WHERE id = ?1",
        params![id],
        |row| {
            Ok(SkillRun {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                status: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_context: row.get(4)?,
                output: row.get(5)?,
                error: row.get(6)?,
                pending_changes: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                duration_ms: row.get(10)?,
                approval_decision: row.get(11)?,
                approval_reason: row.get(12)?,
                created_at: row.get(13)?,
            })
        },
    )
    .map_err(|e| format!("Skill run not found: {}", e))
}

pub fn list_skill_runs(
    conn: &Connection,
    skill_id: &str,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<SkillRun>, String> {
    let mut sql = String::from(
        "SELECT id, skill_id, status, trigger_type, trigger_context, output, error, pending_changes,
         started_at, completed_at, duration_ms, approval_decision, approval_reason, created_at
         FROM skill_runs WHERE skill_id = ?1",
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(skill_id.to_string())];

    if let Some(s) = status {
        sql.push_str(" AND status = ?2");
        params_vec.push(Box::new(s.to_string()));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    params_vec.push(Box::new(limit));
    params_vec.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let runs = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(SkillRun {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                status: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_context: row.get(4)?,
                output: row.get(5)?,
                error: row.get(6)?,
                pending_changes: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                duration_ms: row.get(10)?,
                approval_decision: row.get(11)?,
                approval_reason: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(runs)
}

pub fn update_run_status(conn: &Connection, run_id: &str, status: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();

    if status == "running" {
        conn.execute(
            "UPDATE skill_runs SET status = ?1, started_at = ?2 WHERE id = ?3",
            params![status, now, run_id],
        )
        .map_err(|e| format!("Failed to update run status: {}", e))?;
    } else if status == "completed" || status == "failed" || status == "cancelled" {
        conn.execute(
            "UPDATE skill_runs SET status = ?1, completed_at = ?2 WHERE id = ?3",
            params![status, now, run_id],
        )
        .map_err(|e| format!("Failed to update run status: {}", e))?;
    } else {
        conn.execute(
            "UPDATE skill_runs SET status = ?1 WHERE id = ?2",
            params![status, run_id],
        )
        .map_err(|e| format!("Failed to update run status: {}", e))?;
    }

    Ok(())
}

pub fn set_run_output(conn: &Connection, run_id: &str, output: &str, duration_ms: i64) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE skill_runs SET output = ?1, duration_ms = ?2, completed_at = ?3, status = 'completed' WHERE id = ?4",
        params![output, duration_ms, now, run_id],
    )
    .map_err(|e| format!("Failed to set run output: {}", e))?;
    Ok(())
}

pub fn set_run_error(conn: &Connection, run_id: &str, error: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE skill_runs SET error = ?1, completed_at = ?2, status = 'failed' WHERE id = ?3",
        params![error, now, run_id],
    )
    .map_err(|e| format!("Failed to set run error: {}", e))?;
    Ok(())
}

pub fn set_pending_changes(conn: &Connection, run_id: &str, changes: &serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string(changes).unwrap_or_default();
    conn.execute(
        "UPDATE skill_runs SET pending_changes = ?1, status = 'approval_pending' WHERE id = ?2",
        params![json, run_id],
    )
    .map_err(|e| format!("Failed to set pending changes: {}", e))?;
    Ok(())
}

pub fn set_approval_decision(
    conn: &Connection,
    run_id: &str,
    decision: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let new_status = if decision == "approved" { "completed" } else { "cancelled" };

    conn.execute(
        "UPDATE skill_runs SET approval_decision = ?1, approval_reason = ?2, status = ?3, completed_at = ?4 WHERE id = ?5",
        params![decision, reason, new_status, now, run_id],
    )
    .map_err(|e| format!("Failed to set approval decision: {}", e))?;
    Ok(())
}

pub fn prune_old_runs(conn: &Connection, days: i32) -> Result<i64, String> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.to_rfc3339();

    let deleted = conn
        .execute(
            "DELETE FROM skill_runs WHERE created_at < ?1",
            params![cutoff_str],
        )
        .map_err(|e| format!("Failed to prune old runs: {}", e))?;

    Ok(deleted as i64)
}

pub fn get_skill_stats(conn: &Connection, skill_id: &str) -> Result<SkillStats, String> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM skill_runs WHERE skill_id = ?1",
            params![skill_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM skill_runs WHERE skill_id = ?1 AND status = 'completed'",
            params![skill_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM skill_runs WHERE skill_id = ?1 AND status = 'failed'",
            params![skill_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let success_rate = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let avg_duration: Option<f64> = conn
        .query_row(
            "SELECT AVG(duration_ms) FROM skill_runs WHERE skill_id = ?1 AND duration_ms IS NOT NULL",
            params![skill_id],
            |row| row.get(0),
        )
        .ok();

    let last_run: Option<String> = conn
        .query_row(
            "SELECT created_at FROM skill_runs WHERE skill_id = ?1 ORDER BY created_at DESC LIMIT 1",
            params![skill_id],
            |row| row.get(0),
        )
        .ok();

    Ok(SkillStats {
        total_runs: total,
        completed_runs: completed,
        failed_runs: failed,
        success_rate,
        avg_duration_ms: avg_duration,
        last_run_at: last_run,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                trigger_type TEXT NOT NULL,
                trigger_config TEXT,
                context_config TEXT,
                action_config TEXT,
                approval_mode TEXT NOT NULL DEFAULT 'notify',
                enabled INTEGER NOT NULL DEFAULT 1,
                shared INTEGER NOT NULL DEFAULT 0,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                owner_id TEXT,
                category TEXT,
                icon TEXT,
                tags TEXT,
                next_run_at TEXT,
                cloned_from_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE skill_runs (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                trigger_type TEXT NOT NULL,
                trigger_context TEXT,
                output TEXT,
                error TEXT,
                pending_changes TEXT,
                started_at TEXT,
                completed_at TEXT,
                duration_ms INTEGER,
                approval_decision TEXT,
                approval_reason TEXT,
                created_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_skill() {
        let conn = setup_test_db();
        let input = CreateSkillInput {
            name: "Test Skill".to_string(),
            description: Some("A test skill".to_string()),
            trigger_type: "manual".to_string(),
            trigger_config: None,
            context_config: None,
            action_config: None,
            approval_mode: None,
            category: Some("custom".to_string()),
            icon: Some("⚡".to_string()),
            tags: Some(vec!["test".to_string()]),
            is_builtin: false,
            shared: false,
        };

        let skill = create_skill(&conn, &input).unwrap();
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.trigger_type, "manual");
        assert!(skill.enabled);

        let fetched = get_skill(&conn, &skill.id).unwrap();
        assert_eq!(fetched.id, skill.id);
    }

    #[test]
    fn test_list_skills_with_filters() {
        let conn = setup_test_db();

        create_skill(&conn, &CreateSkillInput {
            name: "Skill 1".to_string(),
            description: None,
            trigger_type: "schedule".to_string(),
            trigger_config: None,
            context_config: None,
            action_config: None,
            approval_mode: None,
            category: Some("productivity".to_string()),
            icon: None,
            tags: None,
            is_builtin: false,
            shared: false,
        }).unwrap();

        create_skill(&conn, &CreateSkillInput {
            name: "Skill 2".to_string(),
            description: None,
            trigger_type: "manual".to_string(),
            trigger_config: None,
            context_config: None,
            action_config: None,
            approval_mode: None,
            category: Some("custom".to_string()),
            icon: None,
            tags: None,
            is_builtin: false,
            shared: false,
        }).unwrap();

        let all = list_skills(&conn, &SkillFilters::default()).unwrap();
        assert_eq!(all.len(), 2);

        let filtered = list_skills(&conn, &SkillFilters {
            category: Some("productivity".to_string()),
            ..Default::default()
        }).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Skill 1");
    }

    #[test]
    fn test_skill_run_lifecycle() {
        let conn = setup_test_db();

        let skill = create_skill(&conn, &CreateSkillInput {
            name: "Test".to_string(),
            description: None,
            trigger_type: "manual".to_string(),
            trigger_config: None,
            context_config: None,
            action_config: None,
            approval_mode: None,
            category: None,
            icon: None,
            tags: None,
            is_builtin: false,
            shared: false,
        }).unwrap();

        let run = create_skill_run(&conn, &CreateSkillRunInput {
            skill_id: skill.id.clone(),
            trigger_type: "manual".to_string(),
            trigger_context: None,
        }).unwrap();

        assert_eq!(run.status, "pending");

        update_run_status(&conn, &run.id, "running").unwrap();
        let updated = get_skill_run(&conn, &run.id).unwrap();
        assert_eq!(updated.status, "running");
        assert!(updated.started_at.is_some());

        set_run_output(&conn, &run.id, "Output text", 1500).unwrap();
        let final_run = get_skill_run(&conn, &run.id).unwrap();
        assert_eq!(final_run.status, "completed");
        assert_eq!(final_run.duration_ms, Some(1500));
    }
}
