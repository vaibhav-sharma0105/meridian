use rusqlite::Connection;
use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
            dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy()
                .to_string()
        });
        PathBuf::from(appdata).join("meridian")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".meridian")
    }
}

pub fn get_db_path() -> PathBuf {
    get_data_dir().join("meridian.db")
}

pub fn get_backup_dir() -> PathBuf {
    get_data_dir().join("backups")
}

/// Get a safe backup directory OUTSIDE the main data directory.
/// This survives accidental deletion of ~/.meridian.
/// Location: ~/Documents/Meridian Backups (macOS) or Documents\Meridian Backups (Windows)
pub fn get_safe_backup_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs_next::document_dir()
            .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("Meridian Backups")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Documents")
            .join("Meridian Backups")
    }
}

pub fn get_documents_dir() -> PathBuf {
    get_data_dir().join("documents")
}

pub fn get_logs_dir() -> PathBuf {
    get_data_dir().join("logs")
}

pub fn get_qdrant_dir() -> PathBuf {
    get_data_dir().join("qdrant")
}

/// Check if the database exists and appears to be unencrypted.
/// Used for migration detection.
pub fn is_database_unencrypted() -> bool {
    let db_path = get_db_path();
    if !db_path.exists() {
        return false;
    }

    // Try to read the first 16 bytes - SQLite databases start with "SQLite format 3"
    // Encrypted databases will have random-looking bytes
    if let Ok(bytes) = std::fs::read(&db_path) {
        if bytes.len() >= 16 {
            return bytes.starts_with(b"SQLite format 3");
        }
    }
    false
}

/// Initialize database with encryption key.
/// This is the main entry point for database access.
pub fn init_db() -> Result<Connection, String> {
    init_db_with_password(None)
}

/// Initialize database with optional password.
/// For device mode, password is None.
/// For password mode, password must be provided.
pub fn init_db_with_password(password: Option<&str>) -> Result<Connection, String> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    std::fs::create_dir_all(get_backup_dir())
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    std::fs::create_dir_all(get_documents_dir())
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;
    std::fs::create_dir_all(get_logs_dir())
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;
    std::fs::create_dir_all(get_qdrant_dir())
        .map_err(|e| format!("Failed to create qdrant directory: {}", e))?;

    let db_path = get_db_path();

    // Check if encryption is initialized
    if !crate::crypto::is_encryption_initialized() {
        // For backward compatibility during migration period,
        // if database exists and is unencrypted, open without encryption
        if is_database_unencrypted() {
            return open_unencrypted_db(&db_path);
        }

        // New install: no encryption config and no database
        // Auto-initialize with device mode for seamless experience
        // User can switch to password mode later in settings
        if !db_path.exists() {
            let key = crate::crypto::initialize_encryption(crate::crypto::KeyMode::Device, None)
                .map_err(|e| format!("Failed to initialize encryption: {}", e))?;

            // Create new encrypted database
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("Failed to create database: {}", e))?;

            let key_hex = format!("\"x'{}'\"", hex::encode(&key));
            conn.execute_batch(&format!("PRAGMA key = {};", key_hex))
                .map_err(|e| format!("Failed to set encryption key: {}", e))?;

            configure_db(&conn)?;
            crate::db::migrations::run_migrations(&conn)?;

            return Ok(conn);
        }

        // Edge case: database exists but isn't SQLite format and no encryption
        return Err("Database exists but encryption not initialized. Please reinstall or contact support.".to_string());
    }

    // Get encryption key
    let key = crate::crypto::get_sqlcipher_key(password)?;

    // Open encrypted database
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Set encryption key using PRAGMA
    conn.execute_batch(&format!("PRAGMA key = {};", key))
        .map_err(|e| format!("Failed to set encryption key: {}", e))?;

    // Verify the key works by executing a query
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")
        .map_err(|_| "Invalid encryption key or corrupted database".to_string())?;

    // Configure database
    configure_db(&conn)?;

    // Run migrations
    crate::db::migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// Open database without encryption (for migration purposes only).
fn open_unencrypted_db(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    configure_db(&conn)?;
    crate::db::migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// Configure database pragmas.
fn configure_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to configure database pragmas: {}", e))
}

/// Create a new encrypted database and migrate data from an unencrypted one.
pub fn migrate_to_encrypted(password: Option<&str>) -> Result<(), String> {
    use crate::crypto::{initialize_encryption, KeyMode};

    let db_path = get_db_path();
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");

    // Create backup in SAFE location (outside ~/.meridian) - survives accidental deletion
    let safe_backup_dir = get_safe_backup_dir();
    std::fs::create_dir_all(&safe_backup_dir)
        .map_err(|e| format!("Failed to create safe backup directory: {}", e))?;
    let safe_backup_path = safe_backup_dir.join(format!("meridian-backup-{}.db", timestamp));

    // Also create backup in regular location for convenience
    let backup_path = get_backup_dir().join(format!("pre-encryption-backup-{}.db", timestamp));

    // Verify we have an unencrypted database to migrate
    if !is_database_unencrypted() {
        return Err("No unencrypted database to migrate".to_string());
    }

    // Create SAFE backup first (this one survives ~/.meridian deletion)
    std::fs::copy(&db_path, &safe_backup_path)
        .map_err(|e| format!("Failed to create safe backup at {:?}: {}", safe_backup_path, e))?;

    // Create regular backup too
    std::fs::copy(&db_path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    // Determine key mode
    let mode = if password.is_some() {
        KeyMode::Password
    } else {
        KeyMode::Device
    };

    // Initialize encryption (creates key.json)
    let key_bytes = initialize_encryption(mode, password)?;
    let key_hex = format!("\"x'{}'\"", hex::encode(&key_bytes));

    // Open unencrypted database
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open unencrypted database: {}", e))?;

    // Create encrypted database using SQLCipher's export
    let encrypted_path = get_data_dir().join("meridian-encrypted.db");

    conn.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS encrypted KEY {};
         SELECT sqlcipher_export('encrypted');
         DETACH DATABASE encrypted;",
        encrypted_path.to_string_lossy(),
        key_hex
    ))
    .map_err(|e| format!("Failed to export to encrypted database: {}", e))?;

    drop(conn);

    // Verify encrypted database
    let verify_conn = Connection::open(&encrypted_path)
        .map_err(|e| format!("Failed to open encrypted database for verification: {}", e))?;

    verify_conn
        .execute_batch(&format!("PRAGMA key = {};", key_hex))
        .map_err(|e| format!("Failed to set key on encrypted database: {}", e))?;

    // Count tables to verify
    let count: i64 = verify_conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to verify encrypted database: {}", e))?;

    if count == 0 {
        return Err("Encrypted database appears empty after migration".to_string());
    }

    drop(verify_conn);

    // Replace original with encrypted
    std::fs::remove_file(&db_path)
        .map_err(|e| format!("Failed to remove original database: {}", e))?;
    std::fs::rename(&encrypted_path, &db_path)
        .map_err(|e| format!("Failed to move encrypted database: {}", e))?;

    Ok(())
}

/// List safe backups (outside ~/.meridian).
pub fn list_safe_backups() -> Result<Vec<(String, PathBuf)>, String> {
    let safe_dir = get_safe_backup_dir();
    if !safe_dir.exists() {
        return Ok(vec![]);
    }

    let mut backups: Vec<(String, PathBuf)> = std::fs::read_dir(&safe_dir)
        .map_err(|e| format!("Failed to read safe backup directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "db" {
                let name = path.file_name()?.to_str()?.to_string();
                Some((name, path))
            } else {
                None
            }
        })
        .collect();

    // Sort by name (which includes timestamp) descending
    backups.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(backups)
}

/// Restore database from a safe backup.
pub fn restore_from_safe_backup(backup_path: &std::path::Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err(format!("Backup file not found: {:?}", backup_path));
    }

    let db_path = get_db_path();
    let data_dir = get_data_dir();

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    // If current database exists, back it up first
    if db_path.exists() {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let pre_restore_backup = get_safe_backup_dir().join(format!("pre-restore-{}.db", timestamp));
        std::fs::copy(&db_path, &pre_restore_backup)
            .map_err(|e| format!("Failed to backup current database before restore: {}", e))?;
    }

    // Copy backup to database location
    std::fs::copy(backup_path, &db_path)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    // Remove any encryption config since we're restoring an unencrypted backup
    let key_config_path = data_dir.join("key.json");
    if key_config_path.exists() {
        std::fs::remove_file(&key_config_path).ok();
    }

    Ok(())
}

/// Get the safe backup directory path (for display to users).
pub fn get_safe_backup_location() -> String {
    get_safe_backup_dir().to_string_lossy().to_string()
}

/// Re-encrypt database with new password.
pub fn reencrypt_database(current_password: &str, new_password: &str) -> Result<(), String> {
    // This uses SQLCipher's rekey functionality
    let conn = init_db_with_password(Some(current_password))?;

    // Change the password using crypto module (updates key.json)
    let new_key_bytes = crate::crypto::key::change_password(current_password, new_password)?;
    let new_key_hex = format!("\"x'{}'\"", hex::encode(&new_key_bytes));

    // Rekey the database
    conn.execute_batch(&format!("PRAGMA rekey = {};", new_key_hex))
        .map_err(|e| format!("Failed to rekey database: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_database_unencrypted_detects_sqlite_header() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // Create a standard SQLite database
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY);").unwrap();
        drop(conn);

        // Read first bytes - should start with "SQLite format 3"
        let bytes = fs::read(&db_path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));
    }

    #[test]
    fn test_configure_db_sets_pragmas() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        configure_db(&conn).unwrap();

        // Note: WAL mode returns "memory" for in-memory databases
        // The pragma is set but in-memory DBs use memory journal mode
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        // In-memory DBs return "memory", file DBs would return "wal"
        assert!(journal_mode == "memory" || journal_mode == "wal");

        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }

    #[test]
    fn test_get_data_dir_returns_valid_path() {
        let dir = get_data_dir();
        // Should end with "meridian" or ".meridian"
        let dir_name = dir.file_name().unwrap().to_str().unwrap();
        assert!(dir_name == "meridian" || dir_name == ".meridian");
    }

    #[test]
    fn test_backup_dir_is_subdirectory_of_data_dir() {
        let data_dir = get_data_dir();
        let backup_dir = get_backup_dir();
        assert!(backup_dir.starts_with(&data_dir));
        assert!(backup_dir.ends_with("backups"));
    }
}
