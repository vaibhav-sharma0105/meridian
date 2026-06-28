pub const SQL: &str = "
ALTER TABLE tasks ADD COLUMN archived_at TEXT;
ALTER TABLE meetings ADD COLUMN archived_at TEXT;
";
