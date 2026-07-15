pub const SQL: &str = r#"
ALTER TABLE skills ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0;
"#;
