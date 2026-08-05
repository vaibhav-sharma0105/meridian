pub mod v001_initial;
pub mod v002_fts;
pub mod v003_embeddings;
pub mod v004_task_priority;
pub mod v005_connectors;
pub mod v006_archive;
pub mod v007_audit_log;
pub mod v008_daemon_jobs;
pub mod v009_pattern_learning;
pub mod v010_proactive_agent;
pub mod v011_subtasks;
pub mod v012_skills;
pub mod v013_skill_builtin_flag;
pub mod v014_integrations;
pub mod v015_governance;
pub mod v016_team_sync;
pub mod v017_expertise_learning;
pub mod v018_integration_visibility;
pub mod v019_phase8_completion;

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
        Migration {
            version: 6,
            sql: v006_archive::SQL,
        },
        Migration {
            version: 7,
            sql: v007_audit_log::SQL,
        },
        Migration {
            version: 8,
            sql: v008_daemon_jobs::V008_DAEMON_JOBS,
        },
        Migration {
            version: 9,
            sql: v009_pattern_learning::SQL,
        },
        Migration {
            version: 10,
            sql: v010_proactive_agent::SQL,
        },
        Migration {
            version: 11,
            sql: v011_subtasks::SQL,
        },
        Migration {
            version: 12,
            sql: v012_skills::SQL,
        },
        Migration {
            version: 13,
            sql: v013_skill_builtin_flag::SQL,
        },
        Migration {
            version: 14,
            sql: v014_integrations::SQL,
        },
        Migration {
            version: 15,
            sql: v015_governance::SQL,
        },
        Migration {
            version: 16,
            sql: v016_team_sync::SQL,
        },
        Migration {
            version: 17,
            sql: v017_expertise_learning::SQL,
        },
        Migration {
            version: 18,
            sql: v018_integration_visibility::SQL,
        },
        Migration {
            version: 19,
            sql: v019_phase8_completion::SQL,
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

            // Post-migration hooks for specific versions
            if migration.version == 15 {
                run_v015_column_additions(conn)?;
            }
            if migration.version == 16 {
                v016_team_sync::run_post_migration(conn)?;
            }

            conn.execute(
                "INSERT INTO schema_versions (version) VALUES (?1)",
                rusqlite::params![migration.version],
            )
            .map_err(|e| format!("Failed to record migration version: {}", e))?;
        }
    }
    Ok(())
}

fn column_exists(conn: &rusqlite::Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        }) {
            for row in rows.flatten() {
                if row == column {
                    return true;
                }
            }
        }
    }
    false
}

pub fn add_column_if_not_exists(conn: &rusqlite::Connection, table: &str, column: &str, col_type: &str) -> Result<(), String> {
    if !column_exists(conn, table, column) {
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
        conn.execute(&sql, [])
            .map_err(|e| format!("Failed to add column {}.{}: {}", table, column, e))?;
    }
    Ok(())
}

fn run_v015_column_additions(conn: &rusqlite::Connection) -> Result<(), String> {
    // Add columns to audit_log
    add_column_if_not_exists(conn, "audit_log", "risk_level", "TEXT")?;
    add_column_if_not_exists(conn, "audit_log", "autonomy_mode", "TEXT")?;
    add_column_if_not_exists(conn, "audit_log", "autonomy_source", "TEXT")?;
    add_column_if_not_exists(conn, "audit_log", "approval_id", "TEXT")?;
    add_column_if_not_exists(conn, "audit_log", "undo_action_id", "TEXT")?;

    // Add autonomy_mode to integrations table
    add_column_if_not_exists(conn, "integrations", "autonomy_mode", "TEXT")?;

    // Add autonomy_mode to skills table
    add_column_if_not_exists(conn, "skills", "autonomy_mode", "TEXT")?;

    Ok(())
}
