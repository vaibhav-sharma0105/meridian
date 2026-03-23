# Database Migrations

Meridian uses integer-versioned migrations stored in `src-tauri/src/db/migrations/`. Each version is applied exactly once; the current version is tracked in the `schema_versions` table.

## How migrations work

On startup, `db::migrations::run_migrations()`:

1. Ensures the `schema_versions` table exists.
2. Reads the highest version already applied.
3. Applies every migration whose version number is greater than the current version, in order.
4. Records the new version in `schema_versions`.

Migrations run inside an implicit SQLite transaction (WAL mode). If any statement fails, the migration is not recorded and the database is left in its prior state.

---

## Migration versions

### v001 — Core schema

Creates the foundational tables:

| Table | Purpose |
|-------|---------|
| `projects` | Top-level workspaces |
| `meetings` | Ingested transcripts and AI outputs |
| `tasks` | Extracted and manually created action items |
| `documents` | Uploaded files and web pages |
| `document_embeddings` | Per-chunk vector embeddings for semantic search |
| `ai_settings` | AI provider configuration (API keys in OS keychain) |
| `app_settings` | Key-value store for UI preferences |
| `schema_versions` | Migration tracking |

### v002 — Conversation and notification layer

| Table | Purpose |
|-------|---------|
| `chat_history` | AI conversation messages, scoped to project or meeting |
| `notifications` | In-app notification queue |
| `prompt_templates` | User-defined and built-in output templates |
| `documents_fts` | FTS5 virtual table over `documents.content_text` |

### v003 — Kanban and meeting health

- `meetings`: adds `health_breakdown TEXT`
- `tasks`: adds `kanban_column TEXT NOT NULL DEFAULT 'open'`, `kanban_order INTEGER NOT NULL DEFAULT 0`, `updated_at TEXT`, `completed_at TEXT`
- Creates `project_task_counts` view that exposes `open_task_count` per project

### v004 — Task priority, document titles, meeting details

- `tasks`: adds `priority TEXT NOT NULL DEFAULT 'medium'`, `confidence_score REAL`
- `meetings`: adds `decisions TEXT`, `duration_minutes INTEGER`
- `documents`: adds `title TEXT`

---

## Adding a new migration

1. Create `src-tauri/src/db/migrations/v005_your_name.rs`:
   ```rust
   pub const SQL: &str = r#"
   ALTER TABLE tasks ADD COLUMN my_new_field TEXT;
   "#;
   ```

2. Register it in `src-tauri/src/db/migrations/mod.rs`:
   ```rust
   mod v005_your_name;

   pub fn run_migrations(conn: &Connection) -> Result<(), String> {
       // ... existing migrations ...
       run_if_needed(conn, 5, v005_your_name::SQL)?;
       Ok(())
   }
   ```

3. Update any affected Rust model structs and repository functions.
4. Update the TypeScript interface in `src/lib/tauri.ts`.

---

## Manual migration (disaster recovery)

If the app fails to start due to a migration error:

1. Back up the database: copy `{data_dir}/meridian.db` to a safe location.
2. Open the database with any SQLite client.
3. Check `SELECT * FROM schema_versions ORDER BY version DESC LIMIT 1;` to see the applied version.
4. Apply the failing SQL manually.
5. Insert `INSERT INTO schema_versions (version, applied_at) VALUES (N, datetime('now'));`.
6. Restart Meridian.
