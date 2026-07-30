pub const SQL: &str = r#"
-- Tracks keyword occurrence counts that haven't yet crossed the promotion
-- threshold into team_members.expertise. Kept separate from `metadata`
-- (which is overwritten by workspace sync) so expertise learning survives
-- Slack/Google roster resyncs.
ALTER TABLE team_members ADD COLUMN expertise_pending TEXT;
"#;
