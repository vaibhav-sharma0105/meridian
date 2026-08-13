pub const SQL: &str = r#"
-- Opt-in archival of old created_files date directories into per-day zips.
-- Off by default: archiving rewrites where files live on disk, so it should
-- never happen without the user asking for it.
ALTER TABLE user_profile ADD COLUMN archive_old_files INTEGER DEFAULT 0;
ALTER TABLE user_profile ADD COLUMN archive_after_days INTEGER DEFAULT 90;
"#;
