pub const SQL: &str = r#"
-- Add linking_workflow field to integrations
ALTER TABLE integrations ADD COLUMN linking_workflow TEXT DEFAULT 'lazy';

-- Add index for cache staleness queries
CREATE INDEX IF NOT EXISTS idx_cache_staleness
    ON integration_cache(integration_id, synced_at DESC) WHERE archived_at IS NULL;
"#;
