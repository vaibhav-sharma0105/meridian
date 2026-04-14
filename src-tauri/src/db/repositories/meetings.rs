use crate::models::meeting::{CreateMeetingInput, Meeting};
use rusqlite::{params, Connection};
use uuid::Uuid;

fn row_to_meeting(row: &rusqlite::Row<'_>) -> rusqlite::Result<Meeting> {
    let ai_summary: Option<String> = row.get(5)?;
    let ingested_at: String = row.get(10)?;
    Ok(Meeting {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        platform: row.get(3)?,
        raw_transcript: row.get(4)?,
        ai_summary: ai_summary.clone(),
        summary: ai_summary,
        decisions: row.get(5 + 1)?,   // decisions is col 6 with new schema... actually recompute
        health_score: row.get(7)?,
        health_breakdown: row.get(8)?,
        attendees: row.get(9)?,
        duration_minutes: row.get(10 + 1)?,
        meeting_at: row.get(12)?,
        ingested_at: ingested_at.clone(),
        created_at: ingested_at,
        updated_at: row.get(13)?,
    })
}

// Explicit select list with new columns in correct order
const MEETING_COLS: &str = "id, project_id, title, platform, raw_transcript, ai_summary,
    decisions, health_score, health_breakdown, attendees, duration_minutes, ingested_at,
    meeting_at, updated_at";

fn row_to_meeting_v2(row: &rusqlite::Row<'_>) -> rusqlite::Result<Meeting> {
    let ai_summary: Option<String> = row.get(5)?;
    let ingested_at: String = row.get(11)?;
    Ok(Meeting {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        platform: row.get(3)?,
        raw_transcript: row.get(4)?,
        ai_summary: ai_summary.clone(),
        summary: ai_summary,
        decisions: row.get(6)?,
        health_score: row.get(7)?,
        health_breakdown: row.get(8)?,
        attendees: row.get(9)?,
        duration_minutes: row.get(10)?,
        meeting_at: row.get(12)?,
        ingested_at: ingested_at.clone(),
        created_at: ingested_at,
        updated_at: row.get(13)?,
    })
}

pub fn get_meetings_for_project(conn: &Connection, project_id: &str) -> Result<Vec<Meeting>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM meetings WHERE project_id = ?1
             ORDER BY COALESCE(meeting_at, ingested_at) DESC",
            MEETING_COLS
        ))
        .map_err(|e| e.to_string())?;

    let meetings = stmt
        .query_map(params![project_id], row_to_meeting_v2)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(meetings)
}

pub fn get_meeting(conn: &Connection, id: &str) -> Result<Option<Meeting>, String> {
    let result = conn.query_row(
        &format!("SELECT {} FROM meetings WHERE id = ?1", MEETING_COLS),
        params![id],
        row_to_meeting_v2,
    );

    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_meeting(conn: &Connection, input: &CreateMeetingInput) -> Result<Meeting, String> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO meetings (id, project_id, title, platform, raw_transcript, attendees,
            duration_minutes, meeting_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.project_id,
            input.title,
            input.platform,
            input.raw_transcript,
            input.attendees,
            input.duration_minutes,
            input.meeting_at
        ],
    )
    .map_err(|e| e.to_string())?;

    get_meeting(conn, &id)?.ok_or_else(|| "Failed to retrieve created meeting".to_string())
}

pub fn update_meeting_summary(
    conn: &Connection,
    id: &str,
    summary: &str,
    decisions: Option<&str>,
    health_score: i32,
    health_breakdown: &str,
    attendees: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE meetings SET ai_summary = ?1, decisions = ?2, health_score = ?3,
            health_breakdown = ?4, attendees = ?5, updated_at = ?6 WHERE id = ?7",
        params![summary, decisions, health_score, health_breakdown, attendees, now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn rename_meeting(conn: &Connection, id: &str, title: &str) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let rows = conn
        .execute(
            "UPDATE meetings SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("Meeting {} not found", id));
    }
    Ok(())
}

/// Move a meeting to a different project.
/// Only updates the meeting row — task migration is handled separately.
pub fn move_meeting_project(
    conn: &Connection,
    meeting_id: &str,
    new_project_id: &str,
) -> Result<(), String> {
    let rows = conn
        .execute(
            "UPDATE meetings SET project_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![new_project_id, meeting_id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err("Meeting not found".to_string());
    }
    Ok(())
}

pub fn soft_delete_meeting(conn: &Connection, id: &str) -> Result<(), String> {
    // Disassociate tasks from this meeting before deleting
    conn.execute(
        "UPDATE tasks SET meeting_id = NULL WHERE meeting_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    // Hard delete the meeting row
    conn.execute("DELETE FROM meetings WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
