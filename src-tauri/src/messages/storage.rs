use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;

use super::models::StorageStats;

pub fn calculate_storage_usage(conn: &Connection) -> Result<StorageStats, String> {
    // Get message counts
    let total_messages: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_center WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Get date range
    let oldest_message: Option<String> = conn
        .query_row(
            "SELECT MIN(created_at) FROM message_center WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    let newest_message: Option<String> = conn
        .query_row(
            "SELECT MAX(created_at) FROM message_center WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    // Collect all unique file references
    let file_refs = get_all_file_refs(conn)?;
    let total_files = file_refs.len() as i64;

    // Calculate total storage
    let storage_bytes = calculate_files_size(&file_refs)?;

    Ok(StorageStats {
        total_messages,
        total_files,
        storage_bytes,
        oldest_message,
        newest_message,
    })
}

fn get_all_file_refs(conn: &Connection) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT file_refs FROM message_center
             WHERE file_refs IS NOT NULL AND deleted_at IS NULL",
        )
        .map_err(|e| e.to_string())?;

    let mut file_refs = HashSet::new();

    let rows = stmt
        .query_map([], |row| {
            let refs_json: String = row.get(0)?;
            Ok(refs_json)
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        if let Ok(refs_json) = row {
            if let Ok(refs) = serde_json::from_str::<Vec<String>>(&refs_json) {
                for path in refs {
                    file_refs.insert(path);
                }
            }
        }
    }

    Ok(file_refs)
}

fn calculate_files_size(file_refs: &HashSet<String>) -> Result<u64, String> {
    let home = dirs_next::home_dir().ok_or("Cannot find home directory")?;
    let base_path = home.join(".meridian").join("created_files");

    let mut total_size = 0u64;

    for file_ref in file_refs {
        let full_path = if file_ref.starts_with('/') {
            Path::new(file_ref).to_path_buf()
        } else {
            base_path.join(file_ref)
        };

        if let Ok(metadata) = std::fs::metadata(&full_path) {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

pub fn get_storage_warning(stats: &StorageStats) -> Option<String> {
    let bytes_500mb = 500 * 1024 * 1024;
    let bytes_1gb = 1024 * 1024 * 1024;

    if stats.storage_bytes > bytes_1gb {
        Some(format!(
            "Message Center storage exceeds 1GB ({:.2} GB). Consider archiving old messages.",
            stats.storage_bytes as f64 / bytes_1gb as f64
        ))
    } else if stats.storage_bytes > bytes_500mb {
        Some(format!(
            "Message Center storage exceeds 500MB ({:.2} MB).",
            stats.storage_bytes as f64 / (1024.0 * 1024.0)
        ))
    } else {
        None
    }
}

pub fn format_storage_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Directory creation lives in `skills::sync::get_created_files_dir()` — the
// only producer of files under `created_files/`. This module resolves and
// measures those paths but deliberately does not create them, so there is a
// single owner of the layout.
