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

pub fn get_documents_dir() -> PathBuf {
    get_data_dir().join("documents")
}

pub fn get_logs_dir() -> PathBuf {
    get_data_dir().join("logs")
}

pub fn init_db() -> Result<Connection, String> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    std::fs::create_dir_all(get_backup_dir())
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    std::fs::create_dir_all(get_documents_dir())
        .map_err(|e| format!("Failed to create documents directory: {}", e))?;
    std::fs::create_dir_all(get_logs_dir())
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;

    let db_path = get_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable WAL mode for concurrent access
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to configure database pragmas: {}", e))?;

    // Run migrations
    crate::db::migrations::run_migrations(&conn)?;

    Ok(conn)
}
