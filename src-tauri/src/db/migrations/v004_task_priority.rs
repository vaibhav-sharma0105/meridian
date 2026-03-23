pub const SQL: &str = r#"
-- Add priority and confidence_score columns to tasks table
ALTER TABLE tasks ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium';
ALTER TABLE tasks ADD COLUMN confidence_score REAL;

-- Add decisions and duration_minutes to meetings
ALTER TABLE meetings ADD COLUMN decisions TEXT;
ALTER TABLE meetings ADD COLUMN duration_minutes INTEGER;

-- Add title column to documents (display name)
ALTER TABLE documents ADD COLUMN title TEXT;
"#;
