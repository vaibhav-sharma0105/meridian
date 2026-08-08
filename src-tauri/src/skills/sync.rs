use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportableSkill {
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub size_bytes: u64,
    pub has_scripts: bool,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSyncInfo {
    pub skill_id: String,
    pub sync_source: Option<String>,
    pub sync_path: Option<String>,
    pub sync_commit: Option<String>,
    pub last_sync_check: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable { remote_commit: String },
    LocalModified,
    Conflict { local_hash: String, remote_commit: String },
    NotSynced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStrategy {
    KeepLocal,
    UseRemote,
    Manual,
}

pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn get_skill_sync_info(conn: &Connection, skill_id: &str) -> Result<Option<SkillSyncInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, sync_source, sync_path, sync_commit, last_sync_check, content_hash
             FROM skills WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row([skill_id], |row| {
        Ok(SkillSyncInfo {
            skill_id: row.get(0)?,
            sync_source: row.get(1)?,
            sync_path: row.get(2)?,
            sync_commit: row.get(3)?,
            last_sync_check: row.get(4)?,
            content_hash: row.get(5)?,
        })
    });

    match result {
        Ok(info) => Ok(Some(info)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn update_skill_sync_info(
    conn: &Connection,
    skill_id: &str,
    sync_source: Option<&str>,
    sync_path: Option<&str>,
    sync_commit: Option<&str>,
    content_hash: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE skills SET sync_source = ?2, sync_path = ?3, sync_commit = ?4,
         content_hash = ?5, last_sync_check = ?6, updated_at = ?6
         WHERE id = ?1",
        params![skill_id, sync_source, sync_path, sync_commit, content_hash, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_name_conflict(conn: &Connection, name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM skills WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn get_meridian_skills_dir() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("Could not determine home directory")?;
    let skills_dir = home.join(".meridian").join("skills");
    std::fs::create_dir_all(&skills_dir)
        .map_err(|e| format!("Failed to create skills directory: {}", e))?;
    Ok(skills_dir)
}

pub fn get_created_files_dir() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("Could not determine home directory")?;
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let files_dir = home.join(".meridian").join("created_files").join(date);
    std::fs::create_dir_all(&files_dir)
        .map_err(|e| format!("Failed to create files directory: {}", e))?;
    Ok(files_dir)
}

pub fn save_skill_locally(
    name: &str,
    content: &str,
    scripts: Option<&[(String, String)]>,
) -> Result<PathBuf, String> {
    let skills_dir = get_meridian_skills_dir()?;
    let skill_dir = skills_dir.join(name);
    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    // Write main skill.md file
    let skill_path = skill_dir.join("skill.md");
    std::fs::write(&skill_path, content)
        .map_err(|e| format!("Failed to write skill file: {}", e))?;

    // Write any script files
    if let Some(scripts) = scripts {
        for (script_name, script_content) in scripts {
            let script_path = skill_dir.join(script_name);
            std::fs::write(&script_path, script_content)
                .map_err(|e| format!("Failed to write script {}: {}", script_name, e))?;
        }
    }

    Ok(skill_dir)
}

pub fn read_skill_content(skill_id: &str, name: &str) -> Result<Option<String>, String> {
    let skills_dir = get_meridian_skills_dir()?;
    let skill_path = skills_dir.join(name).join("skill.md");

    if skill_path.exists() {
        let content = std::fs::read_to_string(&skill_path)
            .map_err(|e| format!("Failed to read skill file: {}", e))?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

pub fn delete_skill_locally(name: &str) -> Result<(), String> {
    let skills_dir = get_meridian_skills_dir()?;
    let skill_dir = skills_dir.join(name);

    if skill_dir.exists() {
        std::fs::remove_dir_all(&skill_dir)
            .map_err(|e| format!("Failed to delete skill directory: {}", e))?;
    }
    Ok(())
}

// GitHub API types for skill discovery
#[derive(Debug, Deserialize)]
pub struct GitHubTreeItem {
    pub path: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub size: Option<u64>,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubTree {
    pub sha: String,
    pub tree: Vec<GitHubTreeItem>,
    pub truncated: bool,
}

#[derive(Debug, Deserialize)]
pub struct GitHubContent {
    pub content: Option<String>,
    pub encoding: Option<String>,
    pub size: u64,
    pub sha: String,
}

pub async fn list_importable_skills_from_repo(
    access_token: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<ImportableSkill>, String> {
    let client = reqwest::Client::new();
    let mut skills = Vec::new();

    // Check for .claude/skills/ and .agents/skills/ directories
    for skill_dir in &[".claude/skills", ".agents/skills"] {
        let tree_url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/HEAD?recursive=1",
            owner, repo
        );

        let response = client
            .get(&tree_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Meridian-Desktop")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch repo tree: {}", e))?;

        if !response.status().is_success() {
            continue;
        }

        let tree: GitHubTree = response.json().await.map_err(|e| e.to_string())?;

        // Find skill directories within the skill_dir path
        let skill_paths: std::collections::HashSet<String> = tree
            .tree
            .iter()
            .filter(|item| item.path.starts_with(skill_dir) && item.path.contains("/skill.md"))
            .map(|item| {
                // Extract the skill directory path
                let parts: Vec<&str> = item.path.rsplitn(2, '/').collect();
                if parts.len() == 2 {
                    parts[1].to_string()
                } else {
                    item.path.clone()
                }
            })
            .collect();

        for skill_path in skill_paths {
            // Get skill name from path
            let name = skill_path
                .rsplit('/')
                .next()
                .unwrap_or(&skill_path)
                .to_string();

            // Count files and check for scripts
            let mut size_bytes: u64 = 0;
            let mut file_count = 0;
            let mut has_scripts = false;

            for item in &tree.tree {
                if item.path.starts_with(&skill_path) && item.item_type == "blob" {
                    file_count += 1;
                    size_bytes += item.size.unwrap_or(0);

                    // Check for script files
                    if item.path.ends_with(".py")
                        || item.path.ends_with(".sh")
                        || item.path.ends_with(".js")
                        || item.path.ends_with(".ts")
                    {
                        has_scripts = true;
                    }
                }
            }

            skills.push(ImportableSkill {
                path: skill_path,
                name,
                description: None, // Would need to fetch skill.md content to get this
                size_bytes,
                has_scripts,
                file_count,
            });
        }
    }

    Ok(skills)
}

pub async fn fetch_skill_content_from_repo(
    access_token: &str,
    owner: &str,
    repo: &str,
    skill_path: &str,
) -> Result<(String, Vec<(String, String)>), String> {
    let client = reqwest::Client::new();

    // Fetch skill.md content
    let skill_md_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}/skill.md",
        owner, repo, skill_path
    );

    let response = client
        .get(&skill_md_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Meridian-Desktop")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch skill.md: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch skill.md: {}", response.status()));
    }

    let content_info: GitHubContent = response.json().await.map_err(|e| e.to_string())?;

    let skill_content = if let Some(content) = content_info.content {
        // GitHub returns base64 encoded content
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            content.replace('\n', ""),
        )
        .map_err(|e| format!("Failed to decode skill content: {}", e))?;
        String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8 in skill: {}", e))?
    } else {
        return Err("No content in skill.md".to_string());
    };

    // Fetch additional script files
    let tree_url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/HEAD?recursive=1",
        owner, repo
    );

    let tree_response = client
        .get(&tree_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Meridian-Desktop")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tree: {}", e))?;

    let tree: GitHubTree = tree_response.json().await.map_err(|e| e.to_string())?;

    let mut scripts = Vec::new();
    for item in &tree.tree {
        if item.path.starts_with(skill_path)
            && item.item_type == "blob"
            && !item.path.ends_with("/skill.md")
        {
            let file_url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                owner, repo, item.path
            );

            let file_response = client
                .get(&file_url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "Meridian-Desktop")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch {}: {}", item.path, e))?;

            if file_response.status().is_success() {
                let file_info: GitHubContent =
                    file_response.json().await.map_err(|e| e.to_string())?;

                if let Some(content) = file_info.content {
                    let decoded = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        content.replace('\n', ""),
                    )
                    .ok();

                    if let Some(bytes) = decoded {
                        if let Ok(text) = String::from_utf8(bytes) {
                            let filename = item.path.rsplit('/').next().unwrap_or(&item.path);
                            scripts.push((filename.to_string(), text));
                        }
                    }
                }
            }
        }
    }

    Ok((skill_content, scripts))
}

pub async fn get_repo_head_commit(
    access_token: &str,
    owner: &str,
    repo: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/commits/HEAD", owner, repo);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Meridian-Desktop")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch HEAD commit: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch HEAD: {}", response.status()));
    }

    #[derive(Deserialize)]
    struct CommitResponse {
        sha: String,
    }

    let commit: CommitResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(commit.sha)
}

pub fn check_update_status(
    sync_info: &SkillSyncInfo,
    remote_commit: &str,
    current_content_hash: &str,
) -> UpdateStatus {
    if sync_info.sync_source.is_none() {
        return UpdateStatus::NotSynced;
    }

    let stored_hash = sync_info.content_hash.as_deref().unwrap_or("");
    let stored_commit = sync_info.sync_commit.as_deref().unwrap_or("");

    let local_modified = stored_hash != current_content_hash;
    let remote_updated = stored_commit != remote_commit;

    match (local_modified, remote_updated) {
        (false, false) => UpdateStatus::UpToDate,
        (false, true) => UpdateStatus::UpdateAvailable {
            remote_commit: remote_commit.to_string(),
        },
        (true, false) => UpdateStatus::LocalModified,
        (true, true) => UpdateStatus::Conflict {
            local_hash: current_content_hash.to_string(),
            remote_commit: remote_commit.to_string(),
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub action_taken: String,
    pub new_commit: Option<String>,
    pub new_content_hash: Option<String>,
    pub trust_revoked: bool,
}

pub async fn sync_skill(
    conn: &Connection,
    skill_id: &str,
    skill_name: &str,
    access_token: &str,
    strategy: SyncStrategy,
) -> Result<SyncResult, String> {
    let sync_info = get_skill_sync_info(conn, skill_id)?
        .ok_or("Skill has no sync info")?;

    let sync_source = sync_info.sync_source.as_ref()
        .ok_or("Skill is not synced from a remote source")?;

    if !sync_source.starts_with("github:") {
        return Err("Only GitHub sync sources are supported".to_string());
    }

    let repo_path = &sync_source[7..];
    let parts: Vec<&str> = repo_path.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid sync source format".to_string());
    }
    let (owner, repo) = (parts[0], parts[1]);

    let skill_path = sync_info.sync_path.as_ref()
        .ok_or("Skill has no sync path")?;

    // Get current local content
    let local_content = read_skill_content(skill_id, skill_name)?
        .unwrap_or_default();
    let local_hash = compute_content_hash(&local_content);

    // Get remote state
    let remote_commit = get_repo_head_commit(access_token, owner, repo).await?;
    let status = check_update_status(&sync_info, &remote_commit, &local_hash);

    match status {
        UpdateStatus::UpToDate => {
            Ok(SyncResult {
                success: true,
                action_taken: "already_up_to_date".to_string(),
                new_commit: None,
                new_content_hash: None,
                trust_revoked: false,
            })
        }
        UpdateStatus::NotSynced => {
            Err("Skill is not configured for sync".to_string())
        }
        UpdateStatus::UpdateAvailable { .. } => {
            // Fetch and apply remote content
            let (content, scripts) = fetch_skill_content_from_repo(
                access_token, owner, repo, skill_path
            ).await?;

            let scripts_slice: Vec<(String, String)> = scripts;
            save_skill_locally(skill_name, &content, Some(&scripts_slice))?;

            let new_hash = compute_content_hash(&content);
            update_skill_sync_info(conn, skill_id, None, None, Some(&remote_commit), Some(&new_hash))?;

            // Revoke trust since content changed
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE skills SET trust_state = 'untrusted', trust_granted_at = NULL, updated_at = ?2 WHERE id = ?1",
                params![skill_id, now],
            ).map_err(|e| e.to_string())?;

            Ok(SyncResult {
                success: true,
                action_taken: "updated_from_remote".to_string(),
                new_commit: Some(remote_commit),
                new_content_hash: Some(new_hash),
                trust_revoked: true,
            })
        }
        UpdateStatus::LocalModified => {
            match strategy {
                SyncStrategy::KeepLocal => {
                    // Update hash to current local, keep local changes
                    update_skill_sync_info(conn, skill_id, None, None, None, Some(&local_hash))?;
                    Ok(SyncResult {
                        success: true,
                        action_taken: "kept_local".to_string(),
                        new_commit: None,
                        new_content_hash: Some(local_hash),
                        trust_revoked: false,
                    })
                }
                SyncStrategy::UseRemote => {
                    // Overwrite with remote
                    let (content, scripts) = fetch_skill_content_from_repo(
                        access_token, owner, repo, skill_path
                    ).await?;

                    let scripts_slice: Vec<(String, String)> = scripts;
                    save_skill_locally(skill_name, &content, Some(&scripts_slice))?;

                    let new_hash = compute_content_hash(&content);
                    update_skill_sync_info(conn, skill_id, None, None, Some(&remote_commit), Some(&new_hash))?;

                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "UPDATE skills SET trust_state = 'untrusted', trust_granted_at = NULL, updated_at = ?2 WHERE id = ?1",
                        params![skill_id, now],
                    ).map_err(|e| e.to_string())?;

                    Ok(SyncResult {
                        success: true,
                        action_taken: "overwrote_with_remote".to_string(),
                        new_commit: Some(remote_commit),
                        new_content_hash: Some(new_hash),
                        trust_revoked: true,
                    })
                }
                SyncStrategy::Manual => {
                    Ok(SyncResult {
                        success: false,
                        action_taken: "manual_resolution_required".to_string(),
                        new_commit: None,
                        new_content_hash: None,
                        trust_revoked: false,
                    })
                }
            }
        }
        UpdateStatus::Conflict { .. } => {
            match strategy {
                SyncStrategy::KeepLocal => {
                    update_skill_sync_info(conn, skill_id, None, None, Some(&remote_commit), Some(&local_hash))?;
                    Ok(SyncResult {
                        success: true,
                        action_taken: "kept_local_resolved_conflict".to_string(),
                        new_commit: Some(remote_commit),
                        new_content_hash: Some(local_hash),
                        trust_revoked: false,
                    })
                }
                SyncStrategy::UseRemote => {
                    let (content, scripts) = fetch_skill_content_from_repo(
                        access_token, owner, repo, skill_path
                    ).await?;

                    let scripts_slice: Vec<(String, String)> = scripts;
                    save_skill_locally(skill_name, &content, Some(&scripts_slice))?;

                    let new_hash = compute_content_hash(&content);
                    update_skill_sync_info(conn, skill_id, None, None, Some(&remote_commit), Some(&new_hash))?;

                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "UPDATE skills SET trust_state = 'untrusted', trust_granted_at = NULL, updated_at = ?2 WHERE id = ?1",
                        params![skill_id, now],
                    ).map_err(|e| e.to_string())?;

                    Ok(SyncResult {
                        success: true,
                        action_taken: "used_remote_resolved_conflict".to_string(),
                        new_commit: Some(remote_commit),
                        new_content_hash: Some(new_hash),
                        trust_revoked: true,
                    })
                }
                SyncStrategy::Manual => {
                    Ok(SyncResult {
                        success: false,
                        action_taken: "conflict_requires_manual_resolution".to_string(),
                        new_commit: None,
                        new_content_hash: None,
                        trust_revoked: false,
                    })
                }
            }
        }
    }
}

pub fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Skill name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Skill name cannot contain path separators or '..'".to_string());
    }
    if name.starts_with('.') {
        return Err("Skill name cannot start with '.'".to_string());
    }
    if name.len() > 100 {
        return Err("Skill name too long (max 100 characters)".to_string());
    }
    Ok(())
}

pub fn validate_script_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Script name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Script name too long (max 255 chars)".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Script name cannot contain path separators or '..'".to_string());
    }
    if name.starts_with('.') && name != ".env" {
        return Err("Script name cannot start with '.' (except .env)".to_string());
    }
    // Reject Windows drive letters (e.g., "C:")
    if name.len() >= 2 && name.chars().nth(1) == Some(':') {
        return Err("Script name cannot contain drive letter".to_string());
    }
    // Reject Windows reserved names
    let base_name = name.split('.').next().unwrap_or(name).to_uppercase();
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.contains(&base_name.as_str()) {
        return Err("Script name uses reserved Windows name".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("test content");
        let hash2 = compute_content_hash("test content");
        let hash3 = compute_content_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_check_update_status() {
        let sync_info = SkillSyncInfo {
            skill_id: "test".to_string(),
            sync_source: Some("github:owner/repo".to_string()),
            sync_path: Some(".claude/skills/test".to_string()),
            sync_commit: Some("abc123".to_string()),
            last_sync_check: None,
            content_hash: Some("hash1".to_string()),
        };

        // Up to date
        assert_eq!(
            check_update_status(&sync_info, "abc123", "hash1"),
            UpdateStatus::UpToDate
        );

        // Remote updated
        assert!(matches!(
            check_update_status(&sync_info, "def456", "hash1"),
            UpdateStatus::UpdateAvailable { .. }
        ));

        // Local modified
        assert_eq!(
            check_update_status(&sync_info, "abc123", "hash2"),
            UpdateStatus::LocalModified
        );

        // Conflict
        assert!(matches!(
            check_update_status(&sync_info, "def456", "hash2"),
            UpdateStatus::Conflict { .. }
        ));
    }

    #[test]
    fn test_validate_skill_name() {
        // Valid names
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("skill_123").is_ok());
        assert!(validate_skill_name("CamelCaseSkill").is_ok());

        // Invalid: empty
        assert!(validate_skill_name("").is_err());

        // Invalid: path separators
        assert!(validate_skill_name("../escape").is_err());
        assert!(validate_skill_name("path/to/skill").is_err());
        assert!(validate_skill_name("path\\to\\skill").is_err());

        // Invalid: starts with dot
        assert!(validate_skill_name(".hidden").is_err());

        // Invalid: too long
        let long_name = "a".repeat(101);
        assert!(validate_skill_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_script_name() {
        // Valid names
        assert!(validate_script_name("script.sh").is_ok());
        assert!(validate_script_name("run.py").is_ok());
        assert!(validate_script_name(".env").is_ok()); // Exception for .env

        // Invalid: empty
        assert!(validate_script_name("").is_err());

        // Invalid: path separators
        assert!(validate_script_name("../escape.sh").is_err());
        assert!(validate_script_name("path/script.sh").is_err());
        assert!(validate_script_name("path\\script.sh").is_err());

        // Invalid: starts with dot (except .env)
        assert!(validate_script_name(".hidden").is_err());
        assert!(validate_script_name(".bashrc").is_err());

        // Invalid: Windows drive letters
        assert!(validate_script_name("C:script.bat").is_err());

        // Invalid: Windows reserved names
        assert!(validate_script_name("CON").is_err());
        assert!(validate_script_name("PRN.txt").is_err());
        assert!(validate_script_name("NUL.sh").is_err());
        assert!(validate_script_name("COM1").is_err());
        assert!(validate_script_name("LPT1.bat").is_err());

        // Valid: similar but not reserved
        assert!(validate_script_name("CONX").is_ok());
        assert!(validate_script_name("my-nul").is_ok());
    }
}
