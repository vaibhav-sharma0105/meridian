use crate::db::connection::{get_backup_dir, get_db_path};
use std::path::Path;

pub fn backup_database() -> Result<String, String> {
    let db_path = get_db_path();
    let backup_dir = get_backup_dir();

    if !db_path.exists() {
        return Ok("No database to backup yet".to_string());
    }

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("meridian_{}.db", timestamp));

    std::fs::copy(&db_path, &backup_path)
        .map_err(|e| format!("Failed to backup database: {}", e))?;

    cleanup_old_backups(&backup_dir, 10)?;

    Ok(backup_path.to_string_lossy().to_string())
}

fn cleanup_old_backups(backup_dir: &Path, keep: usize) -> Result<(), String> {
    let mut backups: Vec<_> = std::fs::read_dir(backup_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("meridian_")
        })
        .collect();

    // Sort by modified time, newest first
    backups.sort_by_key(|e| {
        std::cmp::Reverse(
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        )
    });

    // Delete the ones beyond keep count
    for entry in backups.iter().skip(keep) {
        let _ = std::fs::remove_file(entry.path());
    }

    Ok(())
}

use std::cmp::Reverse;
