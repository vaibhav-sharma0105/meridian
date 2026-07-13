use crate::db::connection::{
    get_backup_dir, get_db_path, get_safe_backup_dir, get_safe_backup_location,
    is_database_unencrypted, list_safe_backups, migrate_to_encrypted,
    restore_from_safe_backup,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub needs_migration: bool,
    pub database_exists: bool,
    pub is_encrypted: bool,
    pub backup_exists: bool,
    pub backup_path: Option<String>,
    pub database_size_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub backup_path: String,
    pub safe_backup_path: String,
    pub tables_migrated: i64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,
    pub size_mb: f64,
    pub created_at: String,
    pub age_days: i64,
}

#[tauri::command]
pub async fn get_migration_status() -> Result<MigrationStatus, String> {
    let db_path = get_db_path();
    let database_exists = db_path.exists();

    let is_unencrypted = is_database_unencrypted();
    let needs_migration = database_exists && is_unencrypted;

    let database_size_mb = if database_exists {
        fs::metadata(&db_path)
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Check for existing backup
    let backup_dir = get_backup_dir();
    let backup_exists = if backup_dir.exists() {
        fs::read_dir(&backup_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_name().to_string_lossy().starts_with("pre-encryption-backup"))
            })
            .unwrap_or(false)
    } else {
        false
    };

    let backup_path = if backup_exists {
        fs::read_dir(&backup_dir)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().starts_with("pre-encryption-backup"))
                    .map(|e| e.path().to_string_lossy().to_string())
            })
    } else {
        None
    };

    Ok(MigrationStatus {
        needs_migration,
        database_exists,
        is_encrypted: database_exists && !is_unencrypted,
        backup_exists,
        backup_path,
        database_size_mb,
    })
}

#[tauri::command]
pub async fn migrate_database(password: Option<String>) -> Result<MigrationResult, String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let backup_path = get_backup_dir().join(format!("pre-encryption-backup-{}.db", timestamp));
    let safe_backup_path = get_safe_backup_dir().join(format!("meridian-backup-{}.db", timestamp));

    match migrate_to_encrypted(password.as_deref()) {
        Ok(()) => {
            // Count tables in the new encrypted database to report
            let tables_migrated = count_tables_in_db()?;

            Ok(MigrationResult {
                success: true,
                backup_path: backup_path.to_string_lossy().to_string(),
                safe_backup_path: safe_backup_path.to_string_lossy().to_string(),
                tables_migrated,
                error: None,
            })
        }
        Err(e) => Ok(MigrationResult {
            success: false,
            backup_path: backup_path.to_string_lossy().to_string(),
            safe_backup_path: safe_backup_path.to_string_lossy().to_string(),
            tables_migrated: 0,
            error: Some(e),
        }),
    }
}

fn count_tables_in_db() -> Result<i64, String> {
    let db_path = get_db_path();
    if !db_path.exists() {
        return Ok(0);
    }

    // Try to open with encryption key
    let conn = crate::db::connection::init_db()?;
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count tables: {}", e))?;

    Ok(count)
}

#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let now = chrono::Utc::now();
    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir).map_err(|e| format!("Failed to read backup dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        if !filename.ends_with(".db") {
            continue;
        }

        let metadata = fs::metadata(&path).map_err(|e| format!("Failed to get metadata: {}", e))?;
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string()
            })
            .unwrap_or_else(|_| "unknown".to_string());

        let age_days = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| {
                let created = chrono::DateTime::<chrono::Utc>::from(t);
                (now - created).num_days()
            })
            .unwrap_or(0);

        backups.push(BackupInfo {
            path: path.to_string_lossy().to_string(),
            size_mb,
            created_at,
            age_days,
        });
    }

    // Sort by age (newest first)
    backups.sort_by(|a, b| a.age_days.cmp(&b.age_days));

    Ok(backups)
}

#[tauri::command]
pub async fn cleanup_old_backups(max_age_days: i64) -> Result<usize, String> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(0);
    }

    let now = chrono::Utc::now();
    let mut deleted = 0;

    for entry in fs::read_dir(&backup_dir).map_err(|e| format!("Failed to read backup dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let age_days = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| {
                let created = chrono::DateTime::<chrono::Utc>::from(t);
                (now - created).num_days()
            })
            .unwrap_or(0);

        if age_days > max_age_days {
            if fs::remove_file(&path).is_ok() {
                deleted += 1;
            }
        }
    }

    Ok(deleted)
}

#[tauri::command]
pub async fn restore_from_backup(backup_path: String) -> Result<(), String> {
    let backup = std::path::PathBuf::from(&backup_path);
    if !backup.exists() {
        return Err("Backup file not found".to_string());
    }

    let db_path = get_db_path();

    // Create a backup of current database first
    if db_path.exists() {
        let current_backup = get_backup_dir().join(format!(
            "pre-restore-backup-{}.db",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ));
        fs::copy(&db_path, &current_backup)
            .map_err(|e| format!("Failed to backup current database: {}", e))?;
    }

    // Remove encryption config since we're restoring unencrypted backup
    let key_config_path = crate::db::connection::get_data_dir().join("key.json");
    if key_config_path.exists() {
        fs::remove_file(&key_config_path)
            .map_err(|e| format!("Failed to remove encryption config: {}", e))?;
    }

    // Restore the backup
    fs::copy(&backup, &db_path).map_err(|e| format!("Failed to restore backup: {}", e))?;

    Ok(())
}

/// Get the safe backup directory location (for display to users).
#[tauri::command]
pub async fn get_safe_backup_dir_path() -> Result<String, String> {
    Ok(get_safe_backup_location())
}

/// List backups in the safe backup directory (survives ~/.meridian deletion).
#[tauri::command]
pub async fn list_safe_backups_cmd() -> Result<Vec<BackupInfo>, String> {
    let backups = list_safe_backups()?;
    let now = chrono::Utc::now();

    let backup_infos: Vec<BackupInfo> = backups
        .into_iter()
        .filter_map(|(name, path)| {
            let metadata = fs::metadata(&path).ok()?;
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

            let created_at = metadata
                .created()
                .or_else(|_| metadata.modified())
                .map(|t| {
                    chrono::DateTime::<chrono::Utc>::from(t)
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string()
                })
                .unwrap_or_else(|_| "unknown".to_string());

            let age_days = metadata
                .created()
                .or_else(|_| metadata.modified())
                .map(|t| {
                    let created = chrono::DateTime::<chrono::Utc>::from(t);
                    (now - created).num_days()
                })
                .unwrap_or(0);

            Some(BackupInfo {
                path: path.to_string_lossy().to_string(),
                size_mb,
                created_at,
                age_days,
            })
        })
        .collect();

    Ok(backup_infos)
}

/// Restore from a safe backup (outside ~/.meridian).
#[tauri::command]
pub async fn restore_safe_backup(backup_path: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(&backup_path);
    restore_from_safe_backup(&path)
}
