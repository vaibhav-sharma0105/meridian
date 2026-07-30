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

pub fn export_data(
    conn: &Connection,
    output_path: &Path,
    options: &ExportOptions,
) -> Result<ExportResult, String> {
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

    // Skills and patterns are complex - mark as not included for now
    contents.skills = false;
    contents.patterns = false;
    contents.skill_count = 0;
    contents.pattern_count = 0;

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

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    let zip_bytes = cursor.into_inner();

    let output_bytes = match &options.password {
        Some(password) if !password.is_empty() => crypto::encrypt(&zip_bytes, password)?,
        _ => zip_bytes,
    };

    std::fs::write(output_path, &output_bytes).map_err(|e| e.to_string())?;

    let file_size = output_bytes.len() as u64;

    Ok(ExportResult {
        file_path: output_path.to_string_lossy().to_string(),
        file_size,
        manifest,
    })
}

pub fn export_skill_standalone(
    conn: &Connection,
    skill_id: &str,
    output_path: &Path,
) -> Result<String, String> {
    use crate::skills::repository as skills_repo;

    let skill = skills_repo::get_skill(conn, skill_id)?;
    let skill_json = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
    std::fs::write(output_path, &skill_json).map_err(|e| e.to_string())?;
    Ok(output_path.to_string_lossy().to_string())
}
