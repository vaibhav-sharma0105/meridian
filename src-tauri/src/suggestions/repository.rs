use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{CreateSuggestionInput, Suggestion};

pub fn create_suggestion(conn: &Connection, input: CreateSuggestionInput) -> Result<Suggestion, String> {
    let id = Uuid::new_v4().to_string();
    let severity = input.severity.unwrap_or_else(|| "info".to_string());

    conn.execute(
        "INSERT INTO suggestions (id, type, title, description, reasoning, action_config, severity, project_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.suggestion_type,
            input.title,
            input.description,
            input.reasoning,
            input.action_config,
            severity,
            input.project_id,
        ],
    )
    .map_err(|e| format!("Failed to create suggestion: {}", e))?;

    get_suggestion_by_id(conn, &id)
}

pub fn get_suggestion_by_id(conn: &Connection, id: &str) -> Result<Suggestion, String> {
    conn.query_row(
        "SELECT id, type, title, description, reasoning, action_config, severity, status, project_id, created_at, acted_at
         FROM suggestions WHERE id = ?1",
        params![id],
        |row| {
            Ok(Suggestion {
                id: row.get(0)?,
                suggestion_type: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                reasoning: row.get(4)?,
                action_config: row.get(5)?,
                severity: row.get(6)?,
                status: row.get(7)?,
                project_id: row.get(8)?,
                created_at: row.get(9)?,
                acted_at: row.get(10)?,
            })
        },
    )
    .map_err(|e| format!("Suggestion not found: {}", e))
}

pub fn get_pending_suggestions(conn: &Connection, project_id: Option<&str>) -> Result<Vec<Suggestion>, String> {
    let sql = match project_id {
        Some(_) => {
            "SELECT id, type, title, description, reasoning, action_config, severity, status, project_id, created_at, acted_at
             FROM suggestions WHERE status = 'pending' AND project_id = ?1
             ORDER BY CASE severity WHEN 'critical' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END, created_at DESC"
        }
        None => {
            "SELECT id, type, title, description, reasoning, action_config, severity, status, project_id, created_at, acted_at
             FROM suggestions WHERE status = 'pending'
             ORDER BY CASE severity WHEN 'critical' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END, created_at DESC"
        }
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(pid) = project_id {
        stmt.query_map(params![pid], map_suggestion_row)
    } else {
        stmt.query_map([], map_suggestion_row)
    }
    .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn update_suggestion_status(conn: &Connection, id: &str, status: &str) -> Result<(), String> {
    let acted_at = if status == "accepted" || status == "dismissed" {
        Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    conn.execute(
        "UPDATE suggestions SET status = ?1, acted_at = ?2 WHERE id = ?3",
        params![status, acted_at, id],
    )
    .map_err(|e| format!("Failed to update suggestion status: {}", e))?;

    Ok(())
}

pub fn get_suggestions_count_today(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM suggestions WHERE date(created_at) = date('now')",
        [],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to count today's suggestions: {}", e))
}

pub fn delete_old_suggestions(conn: &Connection, days: i32) -> Result<usize, String> {
    conn.execute(
        "DELETE FROM suggestions WHERE created_at < datetime('now', ?1)",
        params![format!("-{} days", days)],
    )
    .map_err(|e| format!("Failed to delete old suggestions: {}", e))
}

fn map_suggestion_row(row: &rusqlite::Row) -> rusqlite::Result<Suggestion> {
    Ok(Suggestion {
        id: row.get(0)?,
        suggestion_type: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        reasoning: row.get(4)?,
        action_config: row.get(5)?,
        severity: row.get(6)?,
        status: row.get(7)?,
        project_id: row.get(8)?,
        created_at: row.get(9)?,
        acted_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE suggestions (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                reasoning TEXT,
                action_config TEXT,
                severity TEXT NOT NULL DEFAULT 'info',
                status TEXT NOT NULL DEFAULT 'pending',
                project_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                acted_at TEXT
            )"
        ).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_suggestion() {
        let conn = setup_test_db();
        let input = CreateSuggestionInput {
            suggestion_type: "overdue_task".to_string(),
            title: "Task is overdue".to_string(),
            description: Some("Task 'Review PR' is 2 days overdue".to_string()),
            reasoning: Some("Task due date was 2 days ago".to_string()),
            action_config: Some(r#"{"task_id": "123"}"#.to_string()),
            severity: Some("warning".to_string()),
            project_id: Some("proj-1".to_string()),
        };

        let suggestion = create_suggestion(&conn, input).unwrap();
        assert_eq!(suggestion.suggestion_type, "overdue_task");
        assert_eq!(suggestion.title, "Task is overdue");
        assert_eq!(suggestion.severity, "warning");
        assert_eq!(suggestion.status, "pending");
    }

    #[test]
    fn test_get_pending_suggestions() {
        let conn = setup_test_db();

        create_suggestion(&conn, CreateSuggestionInput {
            suggestion_type: "stale_task".to_string(),
            title: "Stale task".to_string(),
            description: None,
            reasoning: None,
            action_config: None,
            severity: Some("info".to_string()),
            project_id: Some("proj-1".to_string()),
        }).unwrap();

        create_suggestion(&conn, CreateSuggestionInput {
            suggestion_type: "overdue_task".to_string(),
            title: "Overdue task".to_string(),
            description: None,
            reasoning: None,
            action_config: None,
            severity: Some("critical".to_string()),
            project_id: Some("proj-1".to_string()),
        }).unwrap();

        let pending = get_pending_suggestions(&conn, Some("proj-1")).unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].severity, "critical");
    }

    #[test]
    fn test_update_suggestion_status() {
        let conn = setup_test_db();
        let suggestion = create_suggestion(&conn, CreateSuggestionInput {
            suggestion_type: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            reasoning: None,
            action_config: None,
            severity: None,
            project_id: None,
        }).unwrap();

        update_suggestion_status(&conn, &suggestion.id, "accepted").unwrap();
        let updated = get_suggestion_by_id(&conn, &suggestion.id).unwrap();
        assert_eq!(updated.status, "accepted");
        assert!(updated.acted_at.is_some());
    }

    #[test]
    fn test_get_suggestions_count_today() {
        let conn = setup_test_db();

        create_suggestion(&conn, CreateSuggestionInput {
            suggestion_type: "test".to_string(),
            title: "Test 1".to_string(),
            description: None,
            reasoning: None,
            action_config: None,
            severity: None,
            project_id: None,
        }).unwrap();

        create_suggestion(&conn, CreateSuggestionInput {
            suggestion_type: "test".to_string(),
            title: "Test 2".to_string(),
            description: None,
            reasoning: None,
            action_config: None,
            severity: None,
            project_id: None,
        }).unwrap();

        let count = get_suggestions_count_today(&conn).unwrap();
        assert_eq!(count, 2);
    }
}
