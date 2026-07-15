use rusqlite::{params, Connection};
use uuid::Uuid;

use super::models::{CreateDraftInput, DraftMessage, UpdateDraftInput};

pub fn create_draft(conn: &Connection, input: CreateDraftInput) -> Result<DraftMessage, String> {
    let id = Uuid::new_v4().to_string();
    let ai_signature = input.ai_signature.unwrap_or(true);

    conn.execute(
        "INSERT INTO draft_messages (id, task_id, channel, recipient, subject, body, ai_signature)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.task_id,
            input.channel,
            input.recipient,
            input.subject,
            input.body,
            ai_signature,
        ],
    )
    .map_err(|e| format!("Failed to create draft: {}", e))?;

    get_draft_by_id(conn, &id)
}

pub fn get_draft_by_id(conn: &Connection, id: &str) -> Result<DraftMessage, String> {
    conn.query_row(
        "SELECT id, task_id, channel, recipient, subject, body, ai_signature, status, sensitive_warnings, created_at, updated_at, sent_at
         FROM draft_messages WHERE id = ?1",
        params![id],
        |row| {
            let ai_sig: i32 = row.get(6)?;
            Ok(DraftMessage {
                id: row.get(0)?,
                task_id: row.get(1)?,
                channel: row.get(2)?,
                recipient: row.get(3)?,
                subject: row.get(4)?,
                body: row.get(5)?,
                ai_signature: ai_sig != 0,
                status: row.get(7)?,
                sensitive_warnings: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                sent_at: row.get(11)?,
            })
        },
    )
    .map_err(|e| format!("Draft not found: {}", e))
}

pub fn get_drafts_for_task(conn: &Connection, task_id: &str) -> Result<Vec<DraftMessage>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, channel, recipient, subject, body, ai_signature, status, sensitive_warnings, created_at, updated_at, sent_at
         FROM draft_messages WHERE task_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![task_id], |row| {
        let ai_sig: i32 = row.get(6)?;
        Ok(DraftMessage {
            id: row.get(0)?,
            task_id: row.get(1)?,
            channel: row.get(2)?,
            recipient: row.get(3)?,
            subject: row.get(4)?,
            body: row.get(5)?,
            ai_signature: ai_sig != 0,
            status: row.get(7)?,
            sensitive_warnings: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            sent_at: row.get(11)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn update_draft(conn: &Connection, id: &str, input: UpdateDraftInput) -> Result<DraftMessage, String> {
    let mut updates = vec![];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    let mut idx = 1;

    if let Some(recipient) = &input.recipient {
        updates.push(format!("recipient = ?{}", idx));
        params_vec.push(Box::new(recipient.clone()));
        idx += 1;
    }

    if let Some(subject) = &input.subject {
        updates.push(format!("subject = ?{}", idx));
        params_vec.push(Box::new(subject.clone()));
        idx += 1;
    }

    if let Some(body) = &input.body {
        updates.push(format!("body = ?{}", idx));
        params_vec.push(Box::new(body.clone()));
        idx += 1;
    }

    if let Some(ai_sig) = input.ai_signature {
        updates.push(format!("ai_signature = ?{}", idx));
        params_vec.push(Box::new(if ai_sig { 1i32 } else { 0i32 }));
        idx += 1;
    }

    if let Some(warnings) = &input.sensitive_warnings {
        updates.push(format!("sensitive_warnings = ?{}", idx));
        params_vec.push(Box::new(warnings.clone()));
        idx += 1;
    }

    if let Some(status) = &input.status {
        updates.push(format!("status = ?{}", idx));
        params_vec.push(Box::new(status.clone()));
        idx += 1;
    }

    if updates.is_empty() {
        return get_draft_by_id(conn, id);
    }

    updates.push(format!("updated_at = datetime('now')"));
    let sql = format!(
        "UPDATE draft_messages SET {} WHERE id = ?{}",
        updates.join(", "),
        idx
    );
    params_vec.push(Box::new(id.to_string()));

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params_slice.as_slice())
        .map_err(|e| format!("Failed to update draft: {}", e))?;

    get_draft_by_id(conn, id)
}

pub fn delete_draft(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM draft_messages WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete draft: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE draft_messages (
                id TEXT PRIMARY KEY,
                task_id TEXT,
                channel TEXT NOT NULL,
                recipient TEXT,
                subject TEXT,
                body TEXT NOT NULL,
                ai_signature INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'draft',
                sensitive_warnings TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                sent_at TEXT
            )"
        ).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_draft() {
        let conn = setup_test_db();
        let input = CreateDraftInput {
            task_id: Some("task-1".to_string()),
            channel: "email".to_string(),
            recipient: Some("alice@example.com".to_string()),
            subject: Some("Follow up".to_string()),
            body: "Hi Alice, following up on our meeting.".to_string(),
            ai_signature: Some(true),
        };

        let draft = create_draft(&conn, input).unwrap();
        assert_eq!(draft.channel, "email");
        assert_eq!(draft.recipient, Some("alice@example.com".to_string()));
        assert!(draft.ai_signature);
        assert_eq!(draft.status, "draft");
    }

    #[test]
    fn test_get_drafts_for_task() {
        let conn = setup_test_db();

        create_draft(&conn, CreateDraftInput {
            task_id: Some("task-1".to_string()),
            channel: "email".to_string(),
            recipient: None,
            subject: None,
            body: "Draft 1".to_string(),
            ai_signature: None,
        }).unwrap();

        create_draft(&conn, CreateDraftInput {
            task_id: Some("task-1".to_string()),
            channel: "slack".to_string(),
            recipient: None,
            subject: None,
            body: "Draft 2".to_string(),
            ai_signature: None,
        }).unwrap();

        let drafts = get_drafts_for_task(&conn, "task-1").unwrap();
        assert_eq!(drafts.len(), 2);
    }

    #[test]
    fn test_update_draft() {
        let conn = setup_test_db();
        let draft = create_draft(&conn, CreateDraftInput {
            task_id: None,
            channel: "email".to_string(),
            recipient: None,
            subject: None,
            body: "Original".to_string(),
            ai_signature: Some(true),
        }).unwrap();

        let updated = update_draft(&conn, &draft.id, UpdateDraftInput {
            recipient: None,
            subject: Some("New subject".to_string()),
            body: Some("Updated body".to_string()),
            ai_signature: Some(false),
            sensitive_warnings: None,
            status: None,
        }).unwrap();

        assert_eq!(updated.subject, Some("New subject".to_string()));
        assert_eq!(updated.body, "Updated body");
        assert!(!updated.ai_signature);
    }

    #[test]
    fn test_delete_draft() {
        let conn = setup_test_db();
        let draft = create_draft(&conn, CreateDraftInput {
            task_id: None,
            channel: "clipboard".to_string(),
            recipient: None,
            subject: None,
            body: "Test".to_string(),
            ai_signature: None,
        }).unwrap();

        delete_draft(&conn, &draft.id).unwrap();
        assert!(get_draft_by_id(&conn, &draft.id).is_err());
    }
}
