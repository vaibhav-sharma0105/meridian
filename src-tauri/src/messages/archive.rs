use chrono::{Duration, NaiveDate, Utc};
use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Result of an archival pass.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ArchiveStats {
    pub dirs_archived: i64,
    pub files_archived: i64,
    pub bytes_before: u64,
}

/// Archival settings read from `user_profile`.
pub struct ArchiveSettings {
    pub enabled: bool,
    pub after_days: i64,
}

pub fn get_settings(conn: &Connection) -> ArchiveSettings {
    conn.query_row(
        "SELECT COALESCE(archive_old_files, 0), COALESCE(archive_after_days, 90)
         FROM user_profile WHERE id = 'default'",
        [],
        |row| {
            Ok(ArchiveSettings {
                enabled: row.get::<_, i64>(0)? != 0,
                after_days: row.get::<_, i64>(1)?,
            })
        },
    )
    .unwrap_or(ArchiveSettings {
        enabled: false,
        after_days: 90,
    })
}

fn created_files_root() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(".meridian").join("created_files"))
}

/// `created_files/` is partitioned into `YYYY-MM-DD` directories by
/// `skills::sync::get_created_files_dir()`, so a whole day archives as a unit.
/// Anything that is not a parseable date directory is left alone.
fn is_archivable_day(name: &str, cutoff: NaiveDate) -> bool {
    match NaiveDate::parse_from_str(name, "%Y-%m-%d") {
        Ok(date) => date < cutoff,
        Err(_) => false,
    }
}

/// Zips one date directory into `archive/{date}.zip` and removes the original.
/// Files stay accessible inside the archive, which is what "archive" means here
/// — this is not a deletion path.
fn archive_day(dir: &Path, archive_root: &Path, date: &str) -> Result<(i64, u64), String> {
    std::fs::create_dir_all(archive_root)
        .map_err(|e| format!("Failed to create archive dir: {}", e))?;

    let zip_path = archive_root.join(format!("{}.zip", date));
    if zip_path.exists() {
        return Err(format!("Archive already exists: {}", zip_path.display()));
    }

    let zip_file =
        File::create(&zip_path).map_err(|e| format!("Failed to create archive: {}", e))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut count = 0i64;
    let mut bytes = 0u64;

    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let mut contents = Vec::new();
        File::open(&path)
            .and_then(|mut f| f.read_to_end(&mut contents))
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        zip.start_file(name, options)
            .map_err(|e| format!("Failed to add file to archive: {}", e))?;
        zip.write_all(&contents)
            .map_err(|e| format!("Failed to write archive entry: {}", e))?;

        bytes += contents.len() as u64;
        count += 1;
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize archive: {}", e))?;

    // Only remove the originals once the archive is safely closed.
    std::fs::remove_dir_all(dir)
        .map_err(|e| format!("Failed to remove archived dir {}: {}", dir.display(), e))?;

    Ok((count, bytes))
}

/// Archives `created_files/` day directories older than the configured window.
/// A no-op unless the user has explicitly enabled it.
pub fn archive_old_files(conn: &Connection) -> Result<ArchiveStats, String> {
    let settings = get_settings(conn);
    if !settings.enabled {
        return Ok(ArchiveStats::default());
    }

    let root = created_files_root()?;
    if !root.exists() {
        return Ok(ArchiveStats::default());
    }

    let cutoff = (Utc::now() - Duration::days(settings.after_days))
        .date_naive();
    let archive_root = root.join("archive");
    let mut stats = ArchiveStats::default();

    let entries =
        std::fs::read_dir(&root).map_err(|e| format!("Failed to read created_files: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if name == "archive" || !is_archivable_day(&name, cutoff) {
            continue;
        }

        match archive_day(&path, &archive_root, &name) {
            Ok((count, bytes)) => {
                stats.dirs_archived += 1;
                stats.files_archived += count;
                stats.bytes_before += bytes;
            }
            // One bad day directory should not abort the whole pass.
            Err(e) => eprintln!("Archive skipped for {}: {}", name, e),
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_date_dirs_are_archivable() {
        let cutoff = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        assert!(is_archivable_day("2026-01-15", cutoff));
        assert!(!is_archivable_day("2026-08-15", cutoff));
        // Non-date directories are never touched.
        assert!(!is_archivable_day("archive", cutoff));
        assert!(!is_archivable_day("skills", cutoff));
        assert!(!is_archivable_day("", cutoff));
    }

    #[test]
    fn test_cutoff_boundary_is_exclusive() {
        let cutoff = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        // A directory exactly at the cutoff is still inside the window.
        assert!(!is_archivable_day("2026-06-01", cutoff));
        assert!(is_archivable_day("2026-05-31", cutoff));
    }

    #[test]
    fn test_disabled_by_default_is_a_noop() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE user_profile (id TEXT PRIMARY KEY, archive_old_files INTEGER DEFAULT 0, archive_after_days INTEGER DEFAULT 90);
             INSERT INTO user_profile (id) VALUES ('default');",
        )
        .unwrap();

        let settings = get_settings(&conn);
        assert!(!settings.enabled);
        assert_eq!(settings.after_days, 90);
        assert_eq!(archive_old_files(&conn).unwrap(), ArchiveStats::default());
    }

    #[test]
    fn test_settings_are_read_when_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE user_profile (id TEXT PRIMARY KEY, archive_old_files INTEGER DEFAULT 0, archive_after_days INTEGER DEFAULT 90);
             INSERT INTO user_profile (id, archive_old_files, archive_after_days) VALUES ('default', 1, 30);",
        )
        .unwrap();

        let settings = get_settings(&conn);
        assert!(settings.enabled);
        assert_eq!(settings.after_days, 30);
    }

    #[test]
    fn test_archive_day_zips_and_removes_original() {
        let tmp = std::env::temp_dir().join(format!("meridian_arch_{}", uuid::Uuid::new_v4()));
        let day = tmp.join("2026-01-01");
        std::fs::create_dir_all(&day).unwrap();
        std::fs::write(day.join("a.txt"), b"hello").unwrap();
        std::fs::write(day.join("b.txt"), b"world").unwrap();

        let archive_root = tmp.join("archive");
        let (count, bytes) = archive_day(&day, &archive_root, "2026-01-01").unwrap();

        assert_eq!(count, 2);
        assert_eq!(bytes, 10);
        assert!(archive_root.join("2026-01-01.zip").exists());
        assert!(!day.exists(), "original dir should be removed after archiving");

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_archive_refuses_to_overwrite_existing_archive() {
        let tmp = std::env::temp_dir().join(format!("meridian_arch_{}", uuid::Uuid::new_v4()));
        let day = tmp.join("2026-01-02");
        std::fs::create_dir_all(&day).unwrap();
        std::fs::write(day.join("a.txt"), b"data").unwrap();

        let archive_root = tmp.join("archive");
        std::fs::create_dir_all(&archive_root).unwrap();
        std::fs::write(archive_root.join("2026-01-02.zip"), b"pre-existing").unwrap();

        let result = archive_day(&day, &archive_root, "2026-01-02");
        assert!(result.is_err(), "must not clobber an existing archive");
        assert!(day.exists(), "originals must survive a failed archive");

        std::fs::remove_dir_all(&tmp).ok();
    }
}
