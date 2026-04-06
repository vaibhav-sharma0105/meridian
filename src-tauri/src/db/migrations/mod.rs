pub mod v001_initial;
pub mod v002_fts;
pub mod v003_embeddings;
pub mod v004_task_priority;
pub mod v005_connectors;

pub struct Migration {
    pub version: i32,
    pub sql: &'static str,
}

pub fn get_all_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            sql: v001_initial::SQL,
        },
        Migration {
            version: 2,
            sql: v002_fts::SQL,
        },
        Migration {
            version: 3,
            sql: v003_embeddings::SQL,
        },
        Migration {
            version: 4,
            sql: v004_task_priority::SQL,
        },
        Migration {
            version: 5,
            sql: v005_connectors::SQL,
        },
    ]
}

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version    INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create schema_versions table: {}", e))?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_versions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in get_all_migrations() {
        if migration.version > current_version {
            // Backup before migration
            if let Err(e) = crate::utils::backup::backup_database() {
                eprintln!("Warning: backup before migration failed: {}", e);
            }
            conn.execute_batch(migration.sql)
                .map_err(|e| format!("Migration v{:03} failed: {}", migration.version, e))?;
            conn.execute(
                "INSERT INTO schema_versions (version) VALUES (?1)",
                rusqlite::params![migration.version],
            )
            .map_err(|e| format!("Failed to record migration version: {}", e))?;
        }
    }
    Ok(())
}
