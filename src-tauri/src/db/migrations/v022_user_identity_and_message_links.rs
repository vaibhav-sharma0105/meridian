pub const SQL: &str = r#"
-- User identity: needed to distinguish "my items" from "team items" when
-- ordering My Activity by role. Phase 10 specified role-based ordering but
-- never defined who "me" is; these columns supply that.
ALTER TABLE user_profile ADD COLUMN display_name TEXT;
ALTER TABLE user_profile ADD COLUMN user_email TEXT;
ALTER TABLE user_profile ADD COLUMN user_aliases TEXT;  -- JSON array of extra names/handles to match against task.assignee

-- Deep link from a notification to the Message Center entry holding the full
-- result. Without this a "View full result" link has no target.
ALTER TABLE notifications ADD COLUMN message_id TEXT REFERENCES message_center(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_notifications_message ON notifications(message_id)
    WHERE message_id IS NOT NULL;
"#;
