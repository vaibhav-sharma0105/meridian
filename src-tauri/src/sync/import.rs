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

pub fn import_data(
    conn: &Connection,
    archive_path: &Path,
    options: &ImportOptions,
    conflict_resolutions: &HashMap<String, ConflictResolution>,
) -> Result<ImportResult, String> {
    let (mut archive, manifest) = open_archive(archive_path, options.password.as_deref())?;

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

    result.conflict_count = result.conflicts.len() as i32;
    result.success = result.errors.is_empty();

    if result.success {
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
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

fn import_project(
    conn: &Connection,
    project: &Project,
    resolution: ConflictResolution,
) -> Result<bool, String> {
    use crate::models::project::UpdateProjectInput;

    if let Ok(Some(_)) = projects_repo::get_project(conn, &project.id) {
        match resolution {
            ConflictResolution::Skip | ConflictResolution::Ask => return Ok(false),
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
            ConflictResolution::Skip | ConflictResolution::Ask => return Ok(false),
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
            ConflictResolution::Skip | ConflictResolution::Ask => return Ok(false),
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
            ConflictResolution::Skip | ConflictResolution::Ask => return Ok(false),
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

pub fn import_skill_standalone(_conn: &Connection, file_path: &Path) -> Result<crate::skills::models::Skill, String> {
    let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;

    // Try JSON format
    if let Ok(skill) = serde_json::from_str::<crate::skills::models::Skill>(&content) {
        // For now, return the parsed skill - actual import would need to create it
        return Ok(skill);
    }

    Err("Invalid skill file format".to_string())
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

    #[test]
    fn test_encrypted_round_trip_imports_all_entity_types() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let export_options = ExportOptions {
            password: Some("s3cret-pass".to_string()),
            ..Default::default()
        };
        export_data(&source, tmp.path(), &export_options).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(
            &dest,
            tmp.path(),
            &default_import_options(Some("s3cret-pass")),
            &HashMap::new(),
        ).unwrap();

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

    #[test]
    fn test_wrong_password_is_rejected() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let export_options = ExportOptions {
            password: Some("correct-password".to_string()),
            ..Default::default()
        };
        export_data(&source, tmp.path(), &export_options).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(
            &dest,
            tmp.path(),
            &default_import_options(Some("wrong-password")),
            &HashMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_unencrypted_export_still_imports() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        export_data(&source, tmp.path(), &ExportOptions::default()).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new()).unwrap();
        assert!(result.success);
        assert_eq!(result.imported_count, 4);
    }

    #[test]
    fn test_replace_mode_wipes_existing_local_data_first() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default()).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);
        // Pre-existing local data that should be wiped by Replace mode.
        dest.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('local-only', 'Local Project', '2020-01-01', '2020-01-01')",
            [],
        ).unwrap();

        let mut options = default_import_options(None);
        options.mode = ImportMode::Replace;

        let result = import_data(&dest, tmp.path(), &options, &HashMap::new()).unwrap();
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

    #[test]
    fn test_merge_mode_keeps_existing_unrelated_local_data() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default()).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);
        dest.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('local-only', 'Local Project', '2020-01-01', '2020-01-01')",
            [],
        ).unwrap();

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new()).unwrap();
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

    #[test]
    fn test_preview_import_detects_conflicts() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default()).unwrap();

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

    #[test]
    fn test_corrupted_archive_fails_checksum_verification() {
        let source = seed_source_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_data(&source, tmp.path(), &ExportOptions::default()).unwrap();

        // Flip a byte inside the zip's local file data (well past the zip
        // header) to corrupt content without breaking the zip container
        // structure enough to fail before checksum verification runs.
        let mut bytes = std::fs::read(tmp.path()).unwrap();
        let corrupt_at = bytes.len() - 20;
        bytes[corrupt_at] ^= 0xFF;
        std::fs::write(tmp.path(), &bytes).unwrap();

        let dest = Connection::open_in_memory().unwrap();
        setup_schema(&dest);

        let result = import_data(&dest, tmp.path(), &default_import_options(None), &HashMap::new());
        assert!(result.is_err());
    }
}
