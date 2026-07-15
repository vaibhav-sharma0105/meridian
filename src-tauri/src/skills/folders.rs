use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFolder {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub files: Vec<SkillFileEntry>,
    pub has_executables: bool,
    pub created_at: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_executable: bool,
    pub size: u64,
    pub children: Option<Vec<SkillFileEntry>>,
}

fn skills_dir() -> PathBuf {
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".meridian").join("skills")
}

fn is_executable_extension(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("sh" | "bash" | "zsh") => true,
        Some("ps1" | "bat" | "cmd") => true,
        Some("py" | "js" | "ts") => true,
        Some("rb" | "pl") => true,
        _ => false,
    }
}

fn build_file_tree(dir: &Path, base: &Path) -> Result<Vec<SkillFileEntry>, String> {
    let mut entries = Vec::new();

    let read_dir = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

    let mut items: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .collect();
    items.sort_by_key(|e| e.file_name());

    for entry in items {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        let relative_path = path.strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            let children = build_file_tree(&path, base)?;
            entries.push(SkillFileEntry {
                name,
                path: relative_path,
                is_directory: true,
                is_executable: false,
                size: 0,
                children: Some(children),
            });
        } else {
            let metadata = fs::metadata(&path).ok();
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            let is_executable = is_executable_extension(&path);

            entries.push(SkillFileEntry {
                name,
                path: relative_path,
                is_directory: false,
                is_executable,
                size,
                children: None,
            });
        }
    }

    Ok(entries)
}

pub fn ensure_skills_dir() -> Result<PathBuf, String> {
    let dir = skills_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create skills directory: {}", e))?;
    }
    Ok(dir)
}

pub fn list_skill_folders() -> Result<Vec<SkillFolder>, String> {
    let dir = ensure_skills_dir()?;
    let mut folders = Vec::new();

    let read_dir = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read skills directory: {}", e))?;

    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let files = build_file_tree(&path, &path)?;
        let has_executables = has_any_executable(&files);

        let metadata = fs::metadata(&path).ok();
        let created_at = metadata
            .and_then(|m| m.created().ok())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        let description = read_folder_description(&path);

        folders.push(SkillFolder {
            name,
            path: path.to_string_lossy().to_string(),
            description,
            files,
            has_executables,
            created_at,
            enabled: true, // Will be updated by list_skill_folders_with_state
        });
    }

    folders.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(folders)
}

/// List skill folders with enabled state from database
pub fn list_skill_folders_with_state(conn: &rusqlite::Connection) -> Result<Vec<SkillFolder>, String> {
    let mut folders = list_skill_folders()?;

    // Get enabled states from app_settings
    for folder in &mut folders {
        let key = format!("skill_folder_enabled_{}", folder.name);
        let enabled: bool = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                rusqlite::params![key],
                |row| {
                    let val: String = row.get(0)?;
                    Ok(val == "true")
                },
            )
            .unwrap_or(true); // Default to enabled
        folder.enabled = enabled;
    }

    Ok(folders)
}

/// Toggle folder skill enabled state
pub fn toggle_folder_skill_enabled(
    conn: &rusqlite::Connection,
    folder_name: &str,
    enabled: bool,
) -> Result<bool, String> {
    let key = format!("skill_folder_enabled_{}", folder_name);
    let value = if enabled { "true" } else { "false" };

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Failed to update folder enabled state: {}", e))?;

    Ok(enabled)
}

fn has_any_executable(files: &[SkillFileEntry]) -> bool {
    for f in files {
        if f.is_executable {
            return true;
        }
        if let Some(children) = &f.children {
            if has_any_executable(children) {
                return true;
            }
        }
    }
    false
}

fn read_folder_description(path: &Path) -> Option<String> {
    let readme = path.join("README.md");
    if readme.exists() {
        if let Ok(content) = fs::read_to_string(&readme) {
            return content.lines().next().map(|l| l.trim_start_matches('#').trim().to_string());
        }
    }
    let skill_md = path.join("skill.md");
    if skill_md.exists() {
        if let Ok(content) = fs::read_to_string(&skill_md) {
            for line in content.lines() {
                if line.starts_with("description:") {
                    return Some(line.trim_start_matches("description:").trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

pub fn install_skill_folder(source_path: &str) -> Result<SkillFolder, String> {
    let source = PathBuf::from(source_path);
    if !source.is_dir() {
        return Err("Source path is not a directory".to_string());
    }

    validate_skill_folder(&source)?;

    let folder_name = source.file_name()
        .ok_or("Invalid source path")?
        .to_string_lossy()
        .to_string();

    let dest = ensure_skills_dir()?.join(&folder_name);
    if dest.exists() {
        return Err(format!("Skill folder '{}' already exists", folder_name));
    }

    copy_dir_recursive(&source, &dest)?;

    let files = build_file_tree(&dest, &dest)?;
    let has_executables = has_any_executable(&files);
    let description = read_folder_description(&dest);

    Ok(SkillFolder {
        name: folder_name,
        path: dest.to_string_lossy().to_string(),
        description,
        files,
        has_executables,
        created_at: chrono::Utc::now().to_rfc3339(),
        enabled: true,
    })
}

fn validate_skill_folder(path: &Path) -> Result<(), String> {
    let skill_md = path.join("skill.md");
    if !skill_md.exists() {
        return Err(
            "Invalid skill folder: missing 'skill.md'. A valid skill package must contain a \
             skill.md file with YAML frontmatter (name, trigger, action) and markdown body \
             (# Instructions section at minimum)."
                .to_string(),
        );
    }

    let content = fs::read_to_string(&skill_md)
        .map_err(|e| format!("Failed to read skill.md: {}", e))?;

    if !content.starts_with("---") {
        return Err(
            "Invalid skill.md: missing YAML frontmatter. File must start with '---' \
             followed by YAML metadata (name, trigger, action) and end with '---'."
                .to_string(),
        );
    }

    let end_marker = content[3..].find("---");
    let frontmatter_str = match end_marker {
        Some(pos) => &content[3..3 + pos],
        None => {
            return Err(
                "Invalid skill.md: YAML frontmatter not closed. Expected a second '---' \
                 line after the metadata block."
                    .to_string(),
            );
        }
    };

    if !frontmatter_str.contains("name:") {
        return Err(
            "Invalid skill.md: missing required 'name' field in frontmatter.".to_string()
        );
    }

    if !frontmatter_str.contains("description:") {
        return Err(
            "Invalid skill.md: missing required 'description' field in frontmatter.".to_string()
        );
    }

    Ok(())
}

pub fn delete_skill_folder(folder_name: &str) -> Result<(), String> {
    let dir = skills_dir().join(folder_name);
    if !dir.exists() {
        return Err(format!("Skill folder '{}' not found", folder_name));
    }
    if !dir.starts_with(skills_dir()) {
        return Err("Invalid folder path".to_string());
    }
    fs::remove_dir_all(&dir)
        .map_err(|e| format!("Failed to delete skill folder: {}", e))?;
    Ok(())
}

pub fn get_skill_folder(folder_name: &str) -> Result<SkillFolder, String> {
    let dir = skills_dir().join(folder_name);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("Skill folder '{}' not found", folder_name));
    }

    let files = build_file_tree(&dir, &dir)?;
    let has_executables = has_any_executable(&files);
    let description = read_folder_description(&dir);

    let metadata = fs::metadata(&dir).ok();
    let created_at = metadata
        .and_then(|m| m.created().ok())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    Ok(SkillFolder {
        name: folder_name.to_string(),
        path: dir.to_string_lossy().to_string(),
        description,
        files,
        has_executables,
        created_at,
        enabled: true,
    })
}

pub fn read_skill_file(folder_name: &str, file_path: &str) -> Result<String, String> {
    let dir = skills_dir().join(folder_name);
    let full_path = dir.join(file_path);

    if !full_path.starts_with(&dir) {
        return Err("Invalid file path (path traversal attempt)".to_string());
    }
    if !full_path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create directory {:?}: {}", dest, e))?;

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read {:?}: {}", src, e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {:?}: {}", src_path, e))?;
        }
    }
    Ok(())
}

pub fn execute_skill_script(
    folder_name: &str,
    script_path: &str,
) -> Result<String, String> {
    let dir = skills_dir().join(folder_name);
    let full_path = dir.join(script_path);

    if !full_path.starts_with(&dir) {
        return Err("Invalid script path (path traversal attempt)".to_string());
    }
    if !full_path.exists() {
        return Err(format!("Script not found: {}", script_path));
    }
    if !is_executable_extension(&full_path) {
        return Err("File is not a recognized executable script".to_string());
    }

    let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (cmd, args) = match ext {
        "py" => ("python3", vec![full_path.to_string_lossy().to_string()]),
        "js" => ("node", vec![full_path.to_string_lossy().to_string()]),
        "ts" => ("npx", vec!["tsx".to_string(), full_path.to_string_lossy().to_string()]),
        "sh" | "bash" => ("bash", vec![full_path.to_string_lossy().to_string()]),
        "zsh" => ("zsh", vec![full_path.to_string_lossy().to_string()]),
        "rb" => ("ruby", vec![full_path.to_string_lossy().to_string()]),
        "pl" => ("perl", vec![full_path.to_string_lossy().to_string()]),
        #[cfg(target_os = "windows")]
        "ps1" => ("powershell", vec!["-File".to_string(), full_path.to_string_lossy().to_string()]),
        #[cfg(target_os = "windows")]
        "bat" | "cmd" => ("cmd", vec!["/c".to_string(), full_path.to_string_lossy().to_string()]),
        _ => return Err(format!("Unsupported script type: .{}", ext)),
    };

    let output = std::process::Command::new(cmd)
        .args(&args)
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Script failed (exit code {:?}): {}", output.status.code(), stderr))
    }
}
