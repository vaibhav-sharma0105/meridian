use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

use crate::db::repositories::{projects as projects_repo, tasks as tasks_repo};
use crate::models::meeting::Meeting;
use crate::models::project::Project;
use crate::models::task::Task;
use crate::team::models::TeamMember;
use crate::team::repository as team_repo;

use super::crypto;
use super::manifest::{compute_checksum, ExportManifest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub mode: ImportMode,
    pub password: Option<String>,
    pub conflict_resolution: ConflictResolution,
    pub create_backup: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ImportMode {
    Merge,
    Replace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    Skip,
    Overwrite,
    Ask,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Merge,
            password: None,
            conflict_resolution: ConflictResolution::Ask,
            create_backup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub local_name: String,
    pub import_name: String,
    pub local_updated: Option<String>,
    pub import_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub conflict_count: i32,
    pub errors: Vec<String>,
    pub conflicts: Vec<ImportConflict>,
    pub backup_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub manifest: ExportManifest,
    pub conflicts: Vec<ImportConflict>,
    pub new_items: HashMap<String, i32>,
}

/// Fixed order data files are hashed in at export time — must match
/// `export_data`'s checksum entries exactly.
const CHECKSUM_ORDER: &[&str] = &[
    "data/projects.json",
    "data/tasks.json",
    "data/meetings.json",
    "data/team_members.json",
    "data/pattern_contributions.json",
    "manifest.json",
];

/// Reads the archive, transparently decrypting it if it's a Meridian
/// encrypted archive, verifies checksum.sha256 (if present), and returns the
/// parsed zip plus its manifest.
fn open_archive(
    archive_path: &Path,
    password: Option<&str>,
) -> Result<(ZipArchive<Cursor<Vec<u8>>>, ExportManifest), String> {
    let raw = std::fs::read(archive_path).map_err(|e| e.to_string())?;

    let zip_bytes = if crypto::is_encrypted(&raw) {
        let pwd = password.ok_or_else(|| "This archive is password-protected".to_string())?;
        crypto::decrypt(&raw, pwd)?
    } else {
        raw
    };

    let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;

    let manifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| e.to_string())?;
        let mut contents = String::new();
        manifest_file
            .read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        ExportManifest::from_json(&contents)?
    };

    if !manifest.is_compatible() {
        return Err(format!(
            "Incompatible export format version: {}",
            manifest.format_version
        ));
    }

    verify_checksum(&mut archive)?;

    Ok((archive, manifest))
}

fn verify_checksum(archive: &mut ZipArchive<Cursor<Vec<u8>>>) -> Result<(), String> {
    let stored = match archive.by_name("checksum.sha256") {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s).map_err(|e| e.to_string())?;
            s
        }
        Err(_) => return Ok(()), // older archives without a checksum entry
    };

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for name in CHECKSUM_ORDER {
        if let Ok(mut f) = archive.by_name(name) {
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            entries.push((name.to_string(), bytes));
        }
    }

    let refs: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|(n, b)| (n.as_str(), b.as_slice()))
        .collect();
    let computed = compute_checksum(&refs);

    if computed.trim() != stored.trim() {
        return Err(
            "Archive checksum verification failed — the file may be corrupted or tampered with"
                .to_string(),
        );
    }

    Ok(())
}

fn read_json_entry<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    name: &str,
    included: bool,
) -> Result<Option<Vec<T>>, String> {
    if !included {
        return Ok(None);
    }
    match archive.by_name(name) {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s).map_err(|e| e.to_string())?;
            let items: Vec<T> = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            Ok(Some(items))
        }
        Err(_) => Ok(None),
    }
}

fn task_exists(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

fn meeting_exists(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM meetings WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn preview_import(
    conn: &Connection,
    archive_path: &Path,
    options: &ImportOptions,
) -> Result<ImportPreview, String> {
    let (mut archive, manifest) = open_archive(archive_path, options.password.as_deref())?;

    let mut conflicts = Vec::new();
    let mut new_items: HashMap<String, i32> = HashMap::new();

    if let Some(projects) =
        read_json_entry::<Project>(&mut archive, "data/projects.json", manifest.contents.projects)?
    {
        let mut new_count = 0;
        for project in &projects {
            if let Ok(Some(existing)) = projects_repo::get_project(conn, &project.id) {
                conflicts.push(ImportConflict {
                    entity_type: "project".to_string(),
                    entity_id: project.id.clone(),
                    local_name: existing.name,
                    import_name: project.name.clone(),
                    local_updated: Some(existing.updated_at),
                    import_updated: Some(project.updated_at.clone()),
                });
            } else {
                new_count += 1;
            }
        }
        new_items.insert("projects".to_string(), new_count);
    }

    if let Some(tasks) =
        read_json_entry::<Task>(&mut archive, "data/tasks.json", manifest.contents.tasks)?
    {
        let mut new_count = 0;
        for task in &tasks {
            if task_exists(conn, &task.id)? {
                let existing = tasks_repo::get_task(conn, &task.id)?;
                conflicts.push(ImportConflict {
                    entity_type: "task".to_string(),
                    entity_id: task.id.clone(),
                    local_name: existing.title,
                    import_name: task.title.clone(),
                    local_updated: Some(existing.updated_at),
                    import_updated: Some(task.updated_at.clone()),
                });
            } else {
                new_count += 1;
            }
        }
        new_items.insert("tasks".to_string(), new_count);
    }

    if let Some(meetings) =
        read_json_entry::<Meeting>(&mut archive, "data/meetings.json", manifest.contents.meetings)?
    {
        let mut new_count = 0;
        for meeting in &meetings {
            if meeting_exists(conn, &meeting.id)? {
                let existing_title: String = conn
                    .query_row(
                        "SELECT title FROM meetings WHERE id = ?1",
                        [&meeting.id],
                        |row| row.get(0),
                    )
                    .unwrap_or_default();
                let existing_updated: String = conn
                    .query_row(
                        "SELECT updated_at FROM meetings WHERE id = ?1",
                        [&meeting.id],
                        |row| row.get(0),
                    )
                    .unwrap_or_default();
                conflicts.push(ImportConflict {
                    entity_type: "meeting".to_string(),
                    entity_id: meeting.id.clone(),
                    local_name: existing_title,
                    import_name: meeting.title.clone(),
                    local_updated: Some(existing_updated),
                    import_updated: Some(meeting.updated_at.clone()),
                });
            } else {
                new_count += 1;
            }
        }
        new_items.insert("meetings".to_string(), new_count);
    }

    if let Some(members) = read_json_entry::<TeamMember>(
        &mut archive,
        "data/team_members.json",
        manifest.contents.team_members,
    )? {
        let mut new_count = 0;
        for member in &members {
            if let Ok(Some(existing)) = team_repo::get_team_member(conn, &member.id) {
                conflicts.push(ImportConflict {
                    entity_type: "team_member".to_string(),
                    entity_id: member.id.clone(),
                    local_name: existing.name,
                    import_name: member.name.clone(),
                    local_updated: existing.last_synced_at,
                    import_updated: member.last_synced_at.clone(),
                });
            } else {
                new_count += 1;
            }
        }
        new_items.insert("team_members".to_string(), new_count);
    }

    Ok(ImportPreview {
        manifest,
        conflicts,
        new_items,
    })
}

const IMPORT_TOTAL_STEPS: u32 = 8;

fn read_vector_snapshots(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    included: bool,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    if !included {
        return Ok(Vec::new());
    }

    let names: Vec<String> = archive
        .file_names()
        .filter(|n| n.starts_with("vectors/qdrant_snapshot/") && n.ends_with(".snapshot"))
        .map(|s| s.to_string())
        .collect();

    let mut result = Vec::new();
    for name in names {
        let collection = name
            .trim_start_matches("vectors/qdrant_snapshot/")
            .trim_end_matches(".snapshot")
            .to_string();
        let mut bytes = Vec::new();
        archive
            .by_name(&name)
            .map_err(|e| e.to_string())?
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        result.push((collection, bytes));
    }
    Ok(result)
}

/// Applies everything that touches `conn`: parses the archive, runs the SQL
/// transaction, and pulls out any vector snapshot bytes (without recovering
/// them into Qdrant yet — that's async and `conn` can't cross an `.await` in
/// a `#[tauri::command]`'s future, same reasoning as
/// `sync::export::build_local_entries`'s doc comment). Call sites inside a
/// Tauri command should call this directly (inside the DB lock scope), then
/// drop the lock and call `finish_import` for the Qdrant step.
pub fn apply_local_import(
    conn: &Connection,
    archive_path: &Path,
    options: &ImportOptions,
    conflict_resolutions: &HashMap<String, ConflictResolution>,
    on_progress: Option<super::export::ProgressFn>,
) -> Result<(ImportResult, Vec<(String, Vec<u8>)>), String> {
    let mut step_count = 0u32;
    let mut report = |label: &str| {
        step_count += 1;
        if let Some(cb) = on_progress {
            cb(label, step_count, IMPORT_TOTAL_STEPS);
        }
    };

    let (mut archive, manifest) = open_archive(archive_path, options.password.as_deref())?;
    report("Reading archive");

    // Read every payload up front — easier than holding archive borrows
    // across the transaction below.
    let projects =
        read_json_entry::<Project>(&mut archive, "data/projects.json", manifest.contents.projects)?;
    let tasks = read_json_entry::<Task>(&mut archive, "data/tasks.json", manifest.contents.tasks)?;
    let meetings =
        read_json_entry::<Meeting>(&mut archive, "data/meetings.json", manifest.contents.meetings)?;
    let team_members = read_json_entry::<TeamMember>(
        &mut archive,
        "data/team_members.json",
        manifest.contents.team_members,
    )?;
    let pattern_contributions = read_json_entry::<crate::patterns::models::PatternContribution>(
        &mut archive,
        "data/pattern_contributions.json",
        manifest.contents.patterns,
    )?;
    let vector_snapshots = read_vector_snapshots(&mut archive, manifest.contents.vectors)?;

    let mut result = ImportResult {
        success: true,
        imported_count: 0,
        skipped_count: 0,
        conflict_count: 0,
        errors: Vec::new(),
        conflicts: Vec::new(),
        backup_path: None,
    };

    if options.create_backup {
        match crate::utils::backup::backup_database() {
            Ok(path) => result.backup_path = Some(path),
            Err(e) => return Err(format!("Failed to create pre-import backup: {}", e)),
        }
    }
    report("Backing up current data");

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    if options.mode == ImportMode::Replace {
        // Delete children before parents to respect foreign keys.
        if tasks.is_some() || meetings.is_some() || projects.is_some() {
            tx.execute("DELETE FROM tasks", [])
                .map_err(|e| e.to_string())?;
        }
        if meetings.is_some() || projects.is_some() {
            tx.execute("DELETE FROM meetings", [])
                .map_err(|e| e.to_string())?;
        }
        if projects.is_some() {
            tx.execute("DELETE FROM projects", [])
                .map_err(|e| e.to_string())?;
        }
        if team_members.is_some() {
            tx.execute("DELETE FROM team_members", [])
                .map_err(|e| e.to_string())?;
        }
    }

    if let Some(projects) = &projects {
        for project in projects {
            let resolution = conflict_resolutions
                .get(&format!("project:{}", project.id))
                .copied()
                .unwrap_or(options.conflict_resolution);

            match import_project(&tx, project, resolution) {
                Ok(true) => result.imported_count += 1,
                Ok(false) => result.skipped_count += 1,
                Err(e) => result.errors.push(e),
            }
        }
    }
    report("Projects");

    if let Some(tasks) = &tasks {
        for task in tasks {
            let resolution = conflict_resolutions
                .get(&format!("task:{}", task.id))
                .copied()
                .unwrap_or(options.conflict_resolution);

            match import_task(&tx, task, resolution) {
                Ok(true) => result.imported_count += 1,
                Ok(false) => result.skipped_count += 1,
                Err(e) => result.errors.push(e),
            }
        }
    }
    report("Tasks");

    if let Some(meetings) = &meetings {
        for meeting in meetings {
            let resolution = conflict_resolutions
                .get(&format!("meeting:{}", meeting.id))
                .copied()
                .unwrap_or(options.conflict_resolution);

            match import_meeting(&tx, meeting, resolution) {
                Ok(true) => result.imported_count += 1,
                Ok(false) => result.skipped_count += 1,
                Err(e) => result.errors.push(e),
            }
        }
    }
    report("Meetings");

    if let Some(team_members) = &team_members {
        for member in team_members {
            let resolution = conflict_resolutions
                .get(&format!("team_member:{}", member.id))
                .copied()
                .unwrap_or(options.conflict_resolution);

            match import_team_member(&tx, member, resolution) {
                Ok(true) => result.imported_count += 1,
                Ok(false) => result.skipped_count += 1,
                Err(e) => result.errors.push(e),
            }
        }
    }

    report("Team Members");

    if let Some(contributions) = &pattern_contributions {
        match crate::patterns::repository::merge_team_contributions(&tx, contributions) {
            Ok(merged) => result.imported_count += merged,
            Err(e) => result.errors.push(format!("Failed to merge team patterns: {}", e)),
        }
    }

    result.conflict_count = result.conflicts.len() as i32;
    result.success = result.errors.is_empty();

    if result.success {
        tx.commit().map_err(|e| e.to_string())?;
        Ok((result, vector_snapshots))
    } else {
        // Dropping `tx` without committing triggers an automatic ROLLBACK.
        let backup_note = match &result.backup_path {
            Some(p) => format!(" A backup was saved to {} before this import ran.", p),
            None => String::new(),
        };
        Err(format!(
            "Import failed with {} error(s); all changes were rolled back.{} Errors: {}",
            result.errors.len(),
            backup_note,
            result.errors.join("; ")
        ))
    }
}

/// Recovers Qdrant collections from the snapshot bytes `apply_local_import`
/// pulled out of the archive — async, and deliberately takes no `conn`
/// (SQL data is already committed by this point; vector recovery runs
/// after, on a best-effort basis).
pub async fn finish_import(
    mut result: ImportResult,
    vector_snapshots: Vec<(String, Vec<u8>)>,
    on_progress: Option<super::export::ProgressFn<'_>>,
) -> ImportResult {
    if !vector_snapshots.is_empty() {
        let qdrant = crate::vectors::qdrant::QdrantClient::new(None);
        if qdrant.is_available().await {
            for (collection, bytes) in &vector_snapshots {
                if let Err(e) = qdrant.import_snapshot(collection, bytes).await {
                    result
                        .errors
                        .push(format!("Failed to restore vectors for '{}': {}", collection, e));
                }
            }
        } else {
            result.errors.push(
                "Vector snapshots were in the archive but Qdrant isn't running — skipped restoring them"
                    .to_string(),
            );
        }
    }
    if let Some(cb) = on_progress {
        cb("Vectors", 7, IMPORT_TOTAL_STEPS);
    }
    if let Some(cb) = on_progress {
        cb("Done", 8, IMPORT_TOTAL_STEPS);
    }
    result
}

/// Convenience wrapper combining `apply_local_import` + `finish_import` for
/// callers that don't need the `Send`/`conn` split (tests, or anything not
/// called through a `#[tauri::command]`).
pub async fn import_data(
    conn: &Connection,
    archive_path: &Path,
    options: &ImportOptions,
    conflict_resolutions: &HashMap<String, ConflictResolution>,
    on_progress: Option<super::export::ProgressFn<'_>>,
) -> Result<ImportResult, String> {
    let (result, vector_snapshots) =
        apply_local_import(conn, archive_path, options, conflict_resolutions, on_progress)?;
    Ok(finish_import(result, vector_snapshots, on_progress).await)
}

fn import_project(
    conn: &Connection,
    project: &Project,
    resolution: ConflictResolution,
) -> Result<bool, String> {
    use crate::models::project::UpdateProjectInput;

    if let Ok(Some(_)) = projects_repo::get_project(conn, &project.id) {
        match resolution {
            ConflictResolution::Skip => return Ok(false),
            ConflictResolution::Ask => {
                return Err(format!(
                    "Project {} conflicts with an existing project and was never resolved to skip/overwrite",
                    project.id
                ))
            }
            ConflictResolution::Overwrite => {
                let input = UpdateProjectInput {
                    id: project.id.clone(),
                    name: Some(project.name.clone()),
                    description: project.description.clone(),
                    color: Some(project.color.clone()),
                };
                projects_repo::update_project(conn, &input)?;
            }
        }
    } else {
        conn.execute(
            "INSERT INTO projects (id, name, description, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                project.id,
                project.name,
                project.description,
                project.color,
                project.created_at,
                project.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

fn import_task(conn: &Connection, task: &Task, resolution: ConflictResolution) -> Result<bool, String> {
    if task_exists(conn, &task.id)? {
        match resolution {
            ConflictResolution::Skip => return Ok(false),
            ConflictResolution::Ask => {
                return Err(format!(
                    "Task {} conflicts with an existing task and was never resolved to skip/overwrite",
                    task.id
                ))
            }
            ConflictResolution::Overwrite => {
                conn.execute(
                    "UPDATE tasks SET title = ?1, description = ?2, status = ?3, priority = ?4,
                     assignee = ?5, due_date = ?6, tags = ?7, updated_at = ?8 WHERE id = ?9",
                    rusqlite::params![
                        task.title,
                        task.description,
                        task.status,
                        task.priority,
                        task.assignee,
                        task.due_date,
                        task.tags,
                        task.updated_at,
                        task.id
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    } else {
        conn.execute(
            "INSERT INTO tasks (id, project_id, meeting_id, title, description, status, priority,
             assignee, due_date, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                task.id,
                task.project_id,
                task.meeting_id,
                task.title,
                task.description,
                task.status,
                task.priority,
                task.assignee,
                task.due_date,
                task.tags,
                task.created_at,
                task.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

fn import_meeting(
    conn: &Connection,
    meeting: &Meeting,
    resolution: ConflictResolution,
) -> Result<bool, String> {
    if meeting_exists(conn, &meeting.id)? {
        match resolution {
            ConflictResolution::Skip => return Ok(false),
            ConflictResolution::Ask => {
                return Err(format!(
                    "Meeting {} conflicts with an existing meeting and was never resolved to skip/overwrite",
                    meeting.id
                ))
            }
            ConflictResolution::Overwrite => {
                conn.execute(
                    "UPDATE meetings SET project_id = ?1, title = ?2, platform = ?3, raw_transcript = ?4,
                     ai_summary = ?5, decisions = ?6, health_score = ?7, health_breakdown = ?8,
                     attendees = ?9, duration_minutes = ?10, meeting_at = ?11, updated_at = ?12
                     WHERE id = ?13",
                    rusqlite::params![
                        meeting.project_id,
                        meeting.title,
                        meeting.platform,
                        meeting.raw_transcript,
                        meeting.ai_summary,
                        meeting.decisions,
                        meeting.health_score,
                        meeting.health_breakdown,
                        meeting.attendees,
                        meeting.duration_minutes,
                        meeting.meeting_at,
                        meeting.updated_at,
                        meeting.id
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    } else {
        conn.execute(
            "INSERT INTO meetings (id, project_id, title, platform, raw_transcript, ai_summary,
             decisions, health_score, health_breakdown, attendees, duration_minutes, ingested_at,
             meeting_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                meeting.id,
                meeting.project_id,
                meeting.title,
                meeting.platform,
                meeting.raw_transcript,
                meeting.ai_summary,
                meeting.decisions,
                meeting.health_score,
                meeting.health_breakdown,
                meeting.attendees,
                meeting.duration_minutes,
                meeting.ingested_at,
                meeting.meeting_at,
                meeting.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

fn import_team_member(
    conn: &Connection,
    member: &TeamMember,
    resolution: ConflictResolution,
) -> Result<bool, String> {
    if let Ok(Some(_)) = team_repo::get_team_member(conn, &member.id) {
        match resolution {
            ConflictResolution::Skip => return Ok(false),
            ConflictResolution::Ask => {
                return Err(format!(
                    "Team member {} conflicts with an existing team member and was never resolved to skip/overwrite",
                    member.id
                ))
            }
            ConflictResolution::Overwrite => {
                let input = crate::team::models::UpdateTeamMemberInput {
                    id: member.id.clone(),
                    name: Some(member.name.clone()),
                    email: member.email.clone(),
                    avatar_url: member.avatar_url.clone(),
                    role: Some(member.role.clone()),
                    expertise: member.expertise.clone(),
                    metadata: member.metadata.clone(),
                };
                team_repo::update_team_member(conn, &input)?;
            }
        }
    } else {
        let input = crate::team::models::CreateTeamMemberInput {
            name: member.name.clone(),
            email: member.email.clone(),
            avatar_url: member.avatar_url.clone(),
            source: member.source.clone(),
            source_id: member.source_id.clone(),
            role: Some(member.role.clone()),
            expertise: member.expertise.clone(),
            metadata: member.metadata.clone(),
        };
        team_repo::create_team_member(conn, &input)?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::super::export::{export_data, ExportOptions};
    use super::*;

    fn setup_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT NOT NULL DEFAULT '#6366f1',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                archived_at TEXT
            );
            CREATE TABLE meetings (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                platform TEXT NOT NULL DEFAULT 'manual',
                raw_transcript TEXT,
                ai_summary TEXT,
                decisions TEXT,
                health_score INTEGER,
                health_breakdown TEXT,
                attendees TEXT,
                duration_minutes INTEGER,
                meeting_at TEXT,
                ingested_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                archived_at TEXT
            );
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                meeting_id TEXT,
                parent_task_id TEXT,
                title TEXT NOT NULL,
                description TEXT,
                assignee TEXT,
                assignee_confidence TEXT NOT NULL DEFAULT 'unassigned',
                assignee_source_quote TEXT,
                due_date TEXT,
                due_confidence TEXT NOT NULL DEFAULT 'none',
                due_source_quote TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                priority TEXT NOT NULL DEFAULT 'medium',
                confidence_score REAL,
                tags TEXT NOT NULL DEFAULT '[]',
                kanban_column TEXT NOT NULL DEFAULT 'open',
                kanban_order INTEGER NOT NULL DEFAULT 0,
                is_duplicate INTEGER NOT NULL DEFAULT 0,
                duplicate_of_id TEXT,
                notes TEXT,
                plan_complexity TEXT,
                plan_data TEXT,
                plan_generated_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT,
                archived_at TEXT
            );
            CREATE TABLE team_members (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                avatar_url TEXT,
                source TEXT NOT NULL,
                source_id TEXT,
                role TEXT DEFAULT 'member',
                expertise TEXT,
                workload_score REAL,
                metadata TEXT,
                last_synced_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(source, source_id)
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
            );"
        ).unwrap();
    }

    fn seed_source_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        setup_schema(&conn);
        conn.execute(
            "INSERT INTO projects (id, name, description, color, created_at, updated_at) VALUES
             ('p1', 'Project One', 'desc', '#000000', '2026-01-01', '2026-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO meetings (id, project_id, title, platform, raw_transcript, ingested_at, updated_at)
             VALUES ('m1', 'p1', 'Kickoff', 'manual', 'transcript text', '2026-01-01', '2026-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, meeting_id, title, status, priority, tags, created_at, updated_at)
             VALUES ('t1', 'p1', 'm1', 'Do the thing', 'open', 'high', '[]', '2026-01-01', '2026-01-01')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO team_members (id, name, source, role, created_at) VALUES
             ('tm1', 'Alice', 'manual', 'member', '2026-01-01')",
            [],
        ).unwrap();
        conn
    }

    fn default_import_options(password: Option<&str>) -> ImportOptions {
        ImportOptions {
            mode: ImportMode::Merge,
            password: password.map(|p| p.to_string()),
            conflict_resolution: ConflictResolution::Overwrite,
            create_backup: false, // avoid touching the real ~/.meridian in tests
        }
    }

    #[tokio::test]
    async fn test_encrypted_round_trip_imports_all_entity_types() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let export_options = ExportOptions {
            password: Some("s3cret-pass".to_string()),
            ..Default::default()
        };
        export_data(&source, tmp.path(), &export_options, None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(
            &dest,
            tmp.path(),
            &default_import_options(Some("s3cret-pass")),
            &HashMap::new(),
            None,
        ).await.unwrap();

        assert!(result.success, "errors: {:?}", result.errors);
        assert_eq!(result.imported_count, 4); // project + task + meeting + team member

        let project_count: i32 = dest.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        let task_count: i32 = dest.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0)).unwrap();
        let meeting_count: i32 = dest.query_row("SELECT COUNT(*) FROM meetings", [], |r| r.get(0)).unwrap();
        let member_count: i32 = dest.query_row("SELECT COUNT(*) FROM team_members", [], |r| r.get(0)).unwrap();
        assert_eq!(project_count, 1);
        assert_eq!(task_count, 1);
        assert_eq!(meeting_count, 1);
        assert_eq!(member_count, 1);

        let title: String = dest
            .query_row("SELECT title FROM meetings WHERE id = 'm1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Kickoff");
    }

    #[tokio::test]
    async fn test_wrong_password_is_rejected() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let export_options = ExportOptions {
            password: Some("correct-password".to_string()),
            ..Default::default()
        };
        export_data(&source, tmp.path(), &export_options, None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(
            &dest,
            tmp.path(),
            &default_import_options(Some("wrong-password")),
            &HashMap::new(),
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unencrypted_export_still_imports() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new(), None).await.unwrap();
        assert!(result.success);
        assert_eq!(result.imported_count, 4);
    }

    #[tokio::test]
    async fn test_replace_mode_wipes_existing_local_data_first() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);
        // Pre-existing local data that should be wiped by Replace mode.
        dest.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('local-only', 'Local Project', '2020-01-01', '2020-01-01')",
            [],
        ).unwrap();

        let mut options = default_import_options(None);
        options.mode = ImportMode::Replace;

        let result = import_data(&dest, tmp.path(), &options, &HashMap::new(), None).await.unwrap();
        assert!(result.success, "errors: {:?}", result.errors);

        let local_still_present: i32 = dest
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE id = 'local-only'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(local_still_present, 0, "Replace mode should wipe pre-existing local projects");

        let imported_present: i32 = dest
            .query_row("SELECT COUNT(*) FROM projects WHERE id = 'p1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(imported_present, 1);
    }

    #[tokio::test]
    async fn test_merge_mode_keeps_existing_unrelated_local_data() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);
        dest.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('local-only', 'Local Project', '2020-01-01', '2020-01-01')",
            [],
        ).unwrap();

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new(), None).await.unwrap();
        assert!(result.success);

        let local_still_present: i32 = dest
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE id = 'local-only'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(local_still_present, 1, "Merge mode must not touch unrelated local data");
    }

    #[tokio::test]
    async fn test_preview_import_detects_conflicts() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        // Destination already has the same project id -> should be reported as a conflict.
        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);
        dest.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('p1', 'Old Name', '2020-01-01', '2020-01-01')",
            [],
        ).unwrap();

        let preview = preview_import(&dest, tmp.path(), &default_import_options(None)).unwrap();
        assert!(preview.conflicts.iter().any(|c| c.entity_type == "project" && c.entity_id == "p1"));
    }

    #[tokio::test]
    async fn test_corrupted_archive_fails_checksum_verification() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        // Flip a byte inside the zip's local file data (well past the zip
        // header) to corrupt content without breaking the zip container
        // structure enough to fail before checksum verification runs.
        let mut bytes = std::fs::read(tmp.path()).unwrap();
        let corrupt_at = bytes.len() - 20;
        bytes[corrupt_at] ^= 0xFF;
        std::fs::write(tmp.path(), &bytes).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new(), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_reports_progress_including_backup_and_commit() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default(), None).await.unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let steps: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let callback = |label: &str, _current: u32, _total: u32| {
            steps.lock().unwrap().push(label.to_string());
        };

        import_data(
            &dest,
            tmp.path(),
            &default_import_options(None),
            &HashMap::new(),
            Some(&callback),
        ).await.unwrap();

        let recorded = steps.into_inner().unwrap();
        assert_eq!(
            recorded,
            vec!["Reading archive", "Backing up current data", "Projects", "Tasks", "Meetings", "Team Members", "Vectors", "Done"]
        );
    }

    #[tokio::test]
    async fn test_shared_patterns_round_trip_through_export_and_import() {
        let source = seed_source_db();
        source.execute(
            "INSERT INTO pattern_contributions (id, pattern_type, observation_hash) VALUES
             ('pc1', 'smart_defaults', 'hash-a'), ('pc2', 'smart_defaults', 'hash-b')",
            [],
        ).unwrap();

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let export_options = ExportOptions { include_patterns: true, ..ExportOptions::default() };
        let export_result = export_data(&source, tmp.path(), &export_options, None).await.unwrap();
        assert!(export_result.manifest.contents.patterns);
        assert_eq!(export_result.manifest.contents.pattern_count, 2);

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new(), None)
            .await
            .unwrap();
        assert!(result.success, "errors: {:?}", result.errors);

        let team_model = crate::patterns::repository::get_team_pattern_model_by_type(&dest, "smart_defaults").unwrap();
        assert_eq!(team_model.scope, "team");
        assert_eq!(team_model.contributor_count, 2);
    }
}
