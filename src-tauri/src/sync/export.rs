use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::db::repositories::{
    meetings as meetings_repo, projects as projects_repo, tasks as tasks_repo,
};
use crate::team::repository as team_repo;

use super::crypto;
use super::manifest::{compute_checksum, EncryptionInfo, ExportContents, ExportManifest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_projects: bool,
    pub include_tasks: bool,
    pub include_meetings: bool,
    pub include_skills: bool,
    pub include_patterns: bool,
    pub include_team: bool,
    pub include_documents: bool,
    pub include_vectors: bool,
    pub project_ids: Option<Vec<String>>,
    pub password: Option<String>,
    pub description: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_projects: true,
            include_tasks: true,
            include_meetings: true,
            include_skills: false, // Complex types, skip for now
            include_patterns: false, // Skip for now
            include_team: true,
            include_documents: false,
            include_vectors: false,
            project_ids: None,
            password: None,
            description: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub file_path: String,
    pub file_size: u64,
    pub manifest: ExportManifest,
}

/// (step_label, current_step, total_steps) — called after each stage
/// finishes so the UI can show real progress instead of a spinner. Requires
/// `Sync` so it can be captured across the `.await` points in `export_data`'s
/// async (Qdrant) phase without breaking the command future's `Send` bound.
pub type ProgressFn<'a> = &'a (dyn Fn(&str, u32, u32) + Sync);

const EXPORT_TOTAL_STEPS: u32 = 7;

fn emit_progress(on_progress: Option<ProgressFn>, label: &str, step: u32) {
    if let Some(cb) = on_progress {
        cb(label, step, EXPORT_TOTAL_STEPS);
    }
}

/// Everything that touches `conn` — must finish (and drop the borrow) before
/// `export_data` hits its first `.await`, since `rusqlite::Connection` isn't
/// `Sync` and can't be held across a suspension point in a command future
/// that Tauri requires to be `Send`.
///
/// `#[tauri::command]` call sites should call this directly (inside the
/// `state.db.lock()` scope) rather than going through `export_data`, then
/// drop the lock and call `finish_export` — passing `conn`/a `MutexGuard`
/// into ANY function that returns a future spanning an `.await`, even one
/// that only borrows it for a moment before its own first await, still
/// makes the *caller's* future non-`Send` (the borrow must stay valid for
/// the callee future's whole lifetime as far as the borrow checker is
/// concerned). `export_data` below is fine for non-command callers (tests)
/// where the `Send` bound doesn't apply.
pub fn build_local_entries(
    conn: &Connection,
    options: &ExportOptions,
    on_progress: Option<ProgressFn>,
) -> Result<(ZipWriter<Cursor<Vec<u8>>>, ExportContents, Vec<(String, Vec<u8>)>), String> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);
    let zip_options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let mut contents = ExportContents::default();

    // Entries hashed for the integrity checksum, in write order.
    let mut checksum_entries: Vec<(String, Vec<u8>)> = Vec::new();

    // Export projects
    if options.include_projects {
        let projects = match &options.project_ids {
            Some(ids) => {
                let mut filtered = Vec::new();
                for id in ids {
                    if let Ok(Some(p)) = projects_repo::get_project(conn, id) {
                        filtered.push(p);
                    }
                }
                filtered
            }
            None => projects_repo::get_all_projects(conn).unwrap_or_default(),
        };
        contents.project_count = projects.len() as i32;
        contents.projects = !projects.is_empty();

        let json = serde_json::to_string_pretty(&projects).map_err(|e| e.to_string())?;
        zip.start_file("data/projects.json", zip_options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        checksum_entries.push(("data/projects.json".to_string(), json.into_bytes()));
    }
    emit_progress(on_progress, "Projects", 1);

    // Export tasks
    if options.include_tasks {
        let tasks = match &options.project_ids {
            Some(ids) => {
                let mut all_tasks = Vec::new();
                for id in ids {
                    if let Ok(project_tasks) = tasks_repo::get_tasks_for_project(conn, id, &Default::default()) {
                        all_tasks.extend(project_tasks);
                    }
                }
                all_tasks
            }
            None => tasks_repo::get_all_tasks(conn, &Default::default()).unwrap_or_default(),
        };
        contents.task_count = tasks.len() as i32;
        contents.tasks = !tasks.is_empty();

        let json = serde_json::to_string_pretty(&tasks).map_err(|e| e.to_string())?;
        zip.start_file("data/tasks.json", zip_options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        checksum_entries.push(("data/tasks.json".to_string(), json.into_bytes()));
    }
    emit_progress(on_progress, "Tasks", 2);

    // Export meetings
    if options.include_meetings {
        let meetings = match &options.project_ids {
            Some(ids) => {
                let mut all_meetings = Vec::new();
                for id in ids {
                    if let Ok(project_meetings) = meetings_repo::get_meetings_for_project(conn, id, false) {
                        all_meetings.extend(project_meetings);
                    }
                }
                all_meetings
            }
            None => {
                let projects = projects_repo::get_all_projects(conn).unwrap_or_default();
                let mut all_meetings = Vec::new();
                for p in &projects {
                    if let Ok(m) = meetings_repo::get_meetings_for_project(conn, &p.id, false) {
                        all_meetings.extend(m);
                    }
                }
                all_meetings
            }
        };
        contents.meeting_count = meetings.len() as i32;
        contents.meetings = !meetings.is_empty();

        let json = serde_json::to_string_pretty(&meetings).map_err(|e| e.to_string())?;
        zip.start_file("data/meetings.json", zip_options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        checksum_entries.push(("data/meetings.json".to_string(), json.into_bytes()));
    }
    emit_progress(on_progress, "Meetings", 3);

    // Export team members
    if options.include_team {
        let team = team_repo::get_all_team_members(conn).unwrap_or_default();
        contents.team_member_count = team.len() as i32;
        contents.team_members = !team.is_empty();

        let json = serde_json::to_string_pretty(&team).map_err(|e| e.to_string())?;
        zip.start_file("data/team_members.json", zip_options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        checksum_entries.push(("data/team_members.json".to_string(), json.into_bytes()));
    }
    emit_progress(on_progress, "Team Members", 4);

    // Export anonymized pattern contributions (Shared Patterns). Unlike
    // skills, this is safe to export unconditionally — anonymization
    // already happened at observation time (see patterns/repository.rs), so
    // there's no raw personal data in pattern_contributions to worry about.
    if options.include_patterns {
        let contributions = crate::patterns::repository::get_all_pattern_contributions(conn).unwrap_or_default();
        contents.pattern_count = contributions.len() as i32;
        contents.patterns = !contributions.is_empty();

        let json = serde_json::to_string_pretty(&contributions).map_err(|e| e.to_string())?;
        zip.start_file("data/pattern_contributions.json", zip_options)
            .map_err(|e| e.to_string())?;
        zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        checksum_entries.push(("data/pattern_contributions.json".to_string(), json.into_bytes()));
    }

    // Skills aren't wired up yet - mark as not included
    contents.skills = false;
    contents.skill_count = 0;

    Ok((zip, contents, checksum_entries))
}

/// Convenience wrapper combining `build_local_entries` + `finish_export` for
/// callers that don't need to worry about the `Send`/`conn` split above
/// (tests, or anything not called through a `#[tauri::command]`).
pub async fn export_data(
    conn: &Connection,
    output_path: &Path,
    options: &ExportOptions,
    on_progress: Option<ProgressFn<'_>>,
) -> Result<ExportResult, String> {
    let (zip, contents, checksum_entries) = build_local_entries(conn, options, on_progress)?;
    finish_export(zip, contents, checksum_entries, output_path, options, on_progress).await
}

/// The async remainder of export_data: Qdrant vector snapshots, manifest,
/// checksum, encryption, and writing to disk. Takes no `conn` — see
/// `build_local_entries`'s doc comment for why that split exists.
pub async fn finish_export(
    mut zip: ZipWriter<Cursor<Vec<u8>>>,
    mut contents: ExportContents,
    mut checksum_entries: Vec<(String, Vec<u8>)>,
    output_path: &Path,
    options: &ExportOptions,
    on_progress: Option<ProgressFn<'_>>,
) -> Result<ExportResult, String> {
    let zip_options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // Export vector embeddings via Qdrant's native snapshot mechanism.
    // Snapshot files are binary blobs, so — unlike the JSON entities above —
    // they're not part of the sha256 integrity check; the zip format's own
    // per-entry CRC32 covers corruption detection for these.
    if options.include_vectors {
        let qdrant = crate::vectors::qdrant::QdrantClient::new(None);
        if qdrant.is_available().await {
            if let Ok(collections) = qdrant.list_collections().await {
                let mut exported_any = false;
                for collection in &collections {
                    if let Ok(snapshot_bytes) = qdrant.export_snapshot(collection).await {
                        zip.start_file(
                            format!("vectors/qdrant_snapshot/{}.snapshot", collection),
                            zip_options,
                        )
                        .map_err(|e| e.to_string())?;
                        zip.write_all(&snapshot_bytes).map_err(|e| e.to_string())?;
                        exported_any = true;
                    }
                }
                contents.vectors = exported_any;
            }
        }
        // Qdrant unreachable or empty — export proceeds without vectors
        // rather than failing the whole export over an optional extra.
    }
    emit_progress(on_progress, "Vectors", 5);

    // Encryption is applied to the whole archive after it's built (see below),
    // so the salt actually used lives in the archive's outer header, not here.
    let encryption_info = options.password.as_ref().map(|_| EncryptionInfo {
        algorithm: "AES-256-GCM".to_string(),
        kdf: "PBKDF2-SHA256".to_string(),
        iterations: 100_000,
        salt: "stored in archive header".to_string(),
    });

    let mut manifest = ExportManifest::new(contents.clone(), encryption_info);
    if let Some(desc) = &options.description {
        manifest = manifest.with_description(desc);
    }

    let manifest_json = manifest.to_json()?;
    zip.start_file("manifest.json", zip_options)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| e.to_string())?;
    checksum_entries.push(("manifest.json".to_string(), manifest_json.into_bytes()));

    let checksum_refs: Vec<(&str, &[u8])> = checksum_entries
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    let checksum = compute_checksum(&checksum_refs);
    zip.start_file("checksum.sha256", zip_options)
        .map_err(|e| e.to_string())?;
    zip.write_all(checksum.as_bytes()).map_err(|e| e.to_string())?;
    emit_progress(on_progress, "Finalizing archive", 6);

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    let zip_bytes = cursor.into_inner();

    let output_bytes = match &options.password {
        Some(password) if !password.is_empty() => crypto::encrypt(&zip_bytes, password)?,
        _ => zip_bytes,
    };

    std::fs::write(output_path, &output_bytes).map_err(|e| e.to_string())?;
    emit_progress(on_progress, "Writing to disk", 7);

    let file_size = output_bytes.len() as u64;

    Ok(ExportResult {
        file_path: output_path.to_string_lossy().to_string(),
        file_size,
        manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT,
                color TEXT NOT NULL DEFAULT '#6366f1',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')), archived_at TEXT
            );
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY, project_id TEXT NOT NULL, meeting_id TEXT,
                title TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'open',
                is_duplicate INTEGER NOT NULL DEFAULT 0, archived_at TEXT
            );
            CREATE TABLE meetings (id TEXT PRIMARY KEY, project_id TEXT NOT NULL, title TEXT NOT NULL, platform TEXT NOT NULL DEFAULT 'manual');
            CREATE TABLE team_members (id TEXT PRIMARY KEY, name TEXT NOT NULL, source TEXT NOT NULL, source_id TEXT, UNIQUE(source, source_id));"
        ).unwrap();
        conn
    }

    #[tokio::test]
    async fn test_export_reports_progress_for_each_step_in_order() {
        let conn = setup_test_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let steps: std::sync::Mutex<Vec<(String, u32, u32)>> = std::sync::Mutex::new(Vec::new());
        let callback = |label: &str, current: u32, total: u32| {
            steps.lock().unwrap().push((label.to_string(), current, total));
        };

        export_data(&conn, tmp.path(), &ExportOptions::default(), Some(&callback)).await.unwrap();

        let recorded = steps.into_inner().unwrap();
        assert_eq!(
            recorded.iter().map(|(label, _, _)| label.as_str()).collect::<Vec<_>>(),
            vec!["Projects", "Tasks", "Meetings", "Team Members", "Vectors", "Finalizing archive", "Writing to disk"]
        );
        // current should increase 1..=total monotonically, total constant
        for (i, (_, current, total)) in recorded.iter().enumerate() {
            assert_eq!(*current, (i + 1) as u32);
            assert_eq!(*total, EXPORT_TOTAL_STEPS);
        }
    }

    #[tokio::test]
    async fn test_export_with_vectors_degrades_gracefully_when_qdrant_unavailable() {
        // No Qdrant server is running in this test environment — export
        // should still succeed, just without a vectors/ entry in the archive,
        // rather than failing the whole export over an optional extra.
        let conn = setup_test_db();
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let options = ExportOptions {
            include_vectors: true,
            ..ExportOptions::default()
        };

        let result = export_data(&conn, tmp.path(), &options, None).await.unwrap();
        assert!(!result.manifest.contents.vectors);
    }
}
