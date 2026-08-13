pub const SQL: &str = r#"
-- Persist the two ProductivitySettings fields the command already accepted but
-- silently dropped: there were no columns for them, so a user toggling either
-- one saw it revert on reload.
ALTER TABLE user_profile ADD COLUMN show_suggestions INTEGER DEFAULT 1;
ALTER TABLE user_profile ADD COLUMN data_retention_days INTEGER DEFAULT 365;
"#;
