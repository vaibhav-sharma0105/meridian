pub const SQL: &str = r#"
CREATE TABLE IF NOT EXISTS document_embeddings (
  id          TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id),
  chunk_index INTEGER NOT NULL,
  chunk_text  TEXT NOT NULL,
  embedding   BLOB NOT NULL,
  model       TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_embeddings_document
  ON document_embeddings(document_id);
"#;
