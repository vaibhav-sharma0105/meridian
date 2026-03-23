use crate::models::task::{CreateTaskInput, PartialTaskUpdate, Task, TaskFilters, UpdateTaskInput};
use rusqlite::{params, Connection};
use uuid::Uuid;

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let is_dup: i64 = row.get(16)?;
    Ok(Task {
        id: row.get(0)?,
        project_id: row.get(1)?,
        meeting_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        assignee: row.get(5)?,
        assignee_confidence: row.get(6)?,
        assignee_source_quote: row.get(7)?,
        due_date: row.get(8)?,
        due_confidence: row.get(9)?,
        due_source_quote: row.get(10)?,
        status: row.get(11)?,
        priority: row.get::<_, Option<String>>(12)?.unwrap_or_else(|| "medium".to_string()),
        confidence_score: row.get(13)?,
        tags: row.get(14)?,
        kanban_column: row.get(15)?,
        kanban_order: row.get(16 + 1)?,  // shift for new columns
        notes: row.get(18)?,
        is_duplicate: is_dup != 0,
        duplicate_of_id: row.get(20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
        completed_at: row.get(23)?,
    })
}

fn row_to_task_v2(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let is_dup: i64 = row.get(18)?;
    Ok(Task {
        id: row.get(0)?,
        project_id: row.get(1)?,
        meeting_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        assignee: row.get(5)?,
        assignee_confidence: row.get(6)?,
        assignee_source_quote: row.get(7)?,
        due_date: row.get(8)?,
        due_confidence: row.get(9)?,
        due_source_quote: row.get(10)?,
        status: row.get(11)?,
        priority: row.get::<_, Option<String>>(12)?.unwrap_or_else(|| "medium".to_string()),
        confidence_score: row.get(13)?,
        tags: row.get(14)?,
        kanban_column: row.get(15)?,
        kanban_order: row.get(16)?,
        notes: row.get(17)?,
        is_duplicate: is_dup != 0,
        duplicate_of_id: row.get(19)?,
        created_at: row.get(20)?,
        updated_at: row.get(21)?,
        completed_at: row.get(22)?,
    })
}

const TASK_COLUMNS: &str = "id, project_id, meeting_id, title, description, assignee,
    assignee_confidence, assignee_source_quote, due_date, due_confidence, due_source_quote,
    status, priority, confidence_score, tags, kanban_column, kanban_order, notes, is_duplicate,
    duplicate_of_id, created_at, updated_at, completed_at";

pub fn get_tasks_for_project(
    conn: &Connection,
    project_id: &str,
    filters: &TaskFilters,
) -> Result<Vec<Task>, String> {
    let mut conditions = vec!["project_id = ?1".to_string()];
    let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(project_id.to_string())];
    let mut param_idx = 2;

    if let Some(assignee) = &filters.assignee {
        conditions.push(format!("assignee = ?{}", param_idx));
        bind_values.push(Box::new(assignee.clone()));
        param_idx += 1;
    }

    if let Some(status) = &filters.status {
        conditions.push(format!("status = ?{}", param_idx));
        bind_values.push(Box::new(status.clone()));
        param_idx += 1;
    }

    if let Some(from) = &filters.date_from {
        conditions.push(format!("due_date >= ?{}", param_idx));
        bind_values.push(Box::new(from.clone()));
        param_idx += 1;
    }

    if let Some(to) = &filters.date_to {
        conditions.push(format!("due_date <= ?{}", param_idx));
        bind_values.push(Box::new(to.clone()));
        param_idx += 1;
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT {} FROM tasks WHERE {} ORDER BY \
         CASE WHEN due_date < date('now') AND status != 'done' THEN 0 ELSE 1 END, \
         kanban_order ASC, created_at DESC",
        TASK_COLUMNS, where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_slice: Vec<&dyn rusqlite::ToSql> =
        bind_values.iter().map(|v| v.as_ref()).collect();

    let tasks = stmt
        .query_map(params_slice.as_slice(), row_to_task_v2)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Apply FTS search if query provided
    if let Some(query) = &filters.search_query {
        if !query.is_empty() {
            let filtered: Vec<Task> = tasks
                .into_iter()
                .filter(|t| {
                    t.title.to_lowercase().contains(&query.to_lowercase())
                        || t.description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                })
                .collect();
            return Ok(filtered);
        }
    }

    Ok(tasks)
}

pub fn get_open_tasks_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Task>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM tasks WHERE project_id = ?1 AND status IN ('open', 'in_progress')",
            TASK_COLUMNS
        ))
        .map_err(|e| e.to_string())?;

    let tasks = stmt
        .query_map(params![project_id], row_to_task_v2)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tasks)
}

pub fn create_task(conn: &Connection, input: &CreateTaskInput) -> Result<Task, String> {
    let id = Uuid::new_v4().to_string();
    let tags_json = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|| "[]".to_string());
    let is_dup = input.is_duplicate.unwrap_or(false) as i64;
    let priority = input.priority.as_deref().unwrap_or("medium");
    let kanban_col = input.kanban_column.as_deref().unwrap_or("open");

    conn.execute(
        "INSERT INTO tasks (id, project_id, meeting_id, title, description, assignee,
            assignee_confidence, assignee_source_quote, due_date, due_confidence,
            due_source_quote, priority, confidence_score, tags, kanban_column, notes,
            is_duplicate, duplicate_of_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            id,
            input.project_id,
            input.meeting_id,
            input.title,
            input.description,
            input.assignee,
            input.assignee_confidence.as_deref().unwrap_or("unassigned"),
            input.assignee_source_quote,
            input.due_date,
            input.due_confidence.as_deref().unwrap_or("none"),
            input.due_source_quote,
            priority,
            input.confidence_score,
            tags_json,
            kanban_col,
            input.notes,
            is_dup,
            input.duplicate_of_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    get_task(conn, &id)
}

pub fn get_task(conn: &Connection, id: &str) -> Result<Task, String> {
    conn.query_row(
        &format!("SELECT {} FROM tasks WHERE id = ?1", TASK_COLUMNS),
        params![id],
        row_to_task_v2,
    )
    .map_err(|e| e.to_string())
}

pub fn update_task(conn: &Connection, input: &UpdateTaskInput) -> Result<Task, String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut bind: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
    let mut idx = 2;

    macro_rules! add_field {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                sets.push(format!("{} = ?{}", $field, idx));
                bind.push(Box::new(v.clone()));
                idx += 1;
            }
        };
    }

    add_field!("title", &input.title);
    add_field!("description", &input.description);
    add_field!("assignee", &input.assignee);
    add_field!("assignee_confidence", &input.assignee_confidence);
    add_field!("due_date", &input.due_date);
    add_field!("due_confidence", &input.due_confidence);
    add_field!("priority", &input.priority);
    add_field!("notes", &input.notes);

    if let Some(status) = &input.status {
        sets.push(format!("status = ?{}", idx));
        bind.push(Box::new(status.clone()));
        idx += 1;
        if status == "done" {
            sets.push(format!("completed_at = ?{}", idx));
            bind.push(Box::new(now.clone()));
            idx += 1;
        }
    }

    if let Some(tags) = &input.tags {
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        sets.push(format!("tags = ?{}", idx));
        bind.push(Box::new(tags_json));
        idx += 1;
    }

    if let Some(col) = &input.kanban_column {
        sets.push(format!("kanban_column = ?{}", idx));
        bind.push(Box::new(col.clone()));
        idx += 1;
    }

    if let Some(order) = &input.kanban_order {
        sets.push(format!("kanban_order = ?{}", idx));
        bind.push(Box::new(*order));
        idx += 1;
    }

    bind.push(Box::new(input.id.clone()));
    let sql = format!(
        "UPDATE tasks SET {} WHERE id = ?{}",
        sets.join(", "),
        idx
    );

    let params_slice: Vec<&dyn rusqlite::ToSql> = bind.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params_slice.as_slice())
        .map_err(|e| e.to_string())?;

    get_task(conn, &input.id)
}

pub fn bulk_update_tasks(
    conn: &Connection,
    task_ids: &[String],
    updates: &PartialTaskUpdate,
) -> Result<(), String> {
    for id in task_ids {
        let input = UpdateTaskInput {
            id: id.clone(),
            title: None,
            description: None,
            assignee: updates.assignee.clone(),
            assignee_confidence: None,
            due_date: updates.due_date.clone(),
            due_confidence: None,
            status: updates.status.clone(),
            priority: None,
            tags: updates.tags.clone(),
            kanban_column: updates.kanban_column.clone(),
            kanban_order: None,
            notes: None,
        };
        update_task(conn, &input)?;
    }
    Ok(())
}

pub fn reorder_task(
    conn: &Connection,
    task_id: &str,
    new_column: &str,
    new_order: i64,
) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE tasks SET kanban_column = ?1, kanban_order = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
        params![new_column, new_order, new_column, now, task_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_task(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
