//! Guards the migration runner itself.
//!
//! Nothing else in the test suite executes migration SQL, so a syntactically
//! invalid or ordering-dependent migration compiles fine, passes every unit
//! test, and only fails at app startup — where the symptom is a blank window
//! rather than an error.

use rusqlite::Connection;

fn fresh_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    meridian_lib::db::migrations::run_migrations(&conn).expect("migrations must apply cleanly");
    conn
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql).expect("prepare table_info");
    let names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query table_info")
        .filter_map(|r| r.ok())
        .collect();
    names.iter().any(|n| n == column)
}

#[test]
fn all_migrations_apply_to_a_fresh_database() {
    let conn = fresh_db();
    let applied: i32 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_versions", [], |r| r.get(0))
        .expect("read schema_versions");
    let expected = meridian_lib::db::migrations::get_all_migrations()
        .iter()
        .map(|m| m.version)
        .max()
        .unwrap();
    assert_eq!(
        applied, expected,
        "every registered migration should be recorded as applied"
    );
}

#[test]
fn migrations_are_idempotent_across_restarts() {
    // The app re-runs the runner on every launch; a second pass must be a no-op
    // rather than an error.
    let conn = Connection::open_in_memory().expect("open in-memory db");
    meridian_lib::db::migrations::run_migrations(&conn).expect("first run");
    meridian_lib::db::migrations::run_migrations(&conn).expect("second run must be a no-op");
}

#[test]
fn v022_adds_user_identity_columns() {
    let conn = fresh_db();
    assert!(column_exists(&conn, "user_profile", "display_name"));
    assert!(column_exists(&conn, "user_profile", "user_email"));
    assert!(column_exists(&conn, "user_profile", "user_aliases"));
}

#[test]
fn v022_adds_notification_message_link() {
    let conn = fresh_db();
    assert!(column_exists(&conn, "notifications", "message_id"));
}

#[test]
fn notification_message_link_is_usable_with_foreign_keys_on() {
    // SQLite restricts ADD COLUMN with a REFERENCES clause; verify the column
    // actually works rather than just existing.
    let conn = fresh_db();
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable foreign keys");

    conn.execute(
        "INSERT INTO message_center (id, message_type, title, created_at, updated_at)
         VALUES ('m1', 'digest', 'Test digest', datetime('now'), datetime('now'))",
        [],
    )
    .expect("insert message");

    conn.execute(
        "INSERT INTO notifications (id, type, title, body, message_id)
         VALUES ('n1', 'digest', 'Test', 'View full digest', 'm1')",
        [],
    )
    .expect("insert notification linked to message");

    let linked: Option<String> = conn
        .query_row(
            "SELECT message_id FROM notifications WHERE id = 'n1'",
            [],
            |r| r.get(0),
        )
        .expect("read back link");
    assert_eq!(linked.as_deref(), Some("m1"));
}

#[test]
fn upgrading_an_existing_v21_database_applies_v22() {
    // The blank-screen scenario: a database already at v21 from a previous
    // launch, upgraded in place. Distinct from the fresh-install path above.
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version    INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .expect("create schema_versions");

    for migration in meridian_lib::db::migrations::get_all_migrations() {
        if migration.version > 21 {
            break;
        }
        conn.execute_batch(migration.sql)
            .unwrap_or_else(|e| panic!("v{:03} failed: {}", migration.version, e));
        conn.execute(
            "INSERT INTO schema_versions (version) VALUES (?1)",
            [migration.version],
        )
        .expect("record version");
    }

    meridian_lib::db::migrations::run_migrations(&conn)
        .expect("upgrading from v21 must succeed");

    assert!(column_exists(&conn, "user_profile", "display_name"));
    assert!(column_exists(&conn, "notifications", "message_id"));
}

#[test]
fn v023_adds_file_archival_columns() {
    let conn = fresh_db();
    assert!(column_exists(&conn, "user_profile", "archive_old_files"));
    assert!(column_exists(&conn, "user_profile", "archive_after_days"));
}

#[test]
fn v024_persists_productivity_settings() {
    // These two were accepted by update_productivity_settings but silently
    // dropped for lack of columns — a toggle reverted on reload.
    let conn = fresh_db();
    assert!(column_exists(&conn, "user_profile", "show_suggestions"));
    assert!(column_exists(&conn, "user_profile", "data_retention_days"));
}

#[test]
fn productivity_settings_round_trip() {
    let conn = fresh_db();
    conn.execute(
        "INSERT INTO user_profile (id, created_at, updated_at)
         VALUES ('default', datetime('now'), datetime('now'))",
        [],
    )
    .expect("seed profile");

    conn.execute(
        "UPDATE user_profile SET show_suggestions = 0, data_retention_days = 30
         WHERE id = 'default'",
        [],
    )
    .expect("update settings");

    let (show, days): (i64, i64) = conn
        .query_row(
            "SELECT show_suggestions, data_retention_days FROM user_profile WHERE id = 'default'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("read back");
    assert_eq!(show, 0, "show_suggestions must persist");
    assert_eq!(days, 30, "data_retention_days must persist");
}
