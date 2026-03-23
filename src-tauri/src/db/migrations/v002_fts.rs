pub const SQL: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS meetings_fts USING fts5(
  title, ai_summary, raw_transcript,
  content='meetings', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
  title, description, notes, assignee,
  content='tasks', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
  filename, content_text,
  content='documents', content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS meetings_ai AFTER INSERT ON meetings BEGIN
  INSERT INTO meetings_fts(rowid, title, ai_summary, raw_transcript)
  VALUES (new.rowid, new.title, new.ai_summary, new.raw_transcript);
END;

CREATE TRIGGER IF NOT EXISTS meetings_ad AFTER DELETE ON meetings BEGIN
  INSERT INTO meetings_fts(meetings_fts, rowid, title, ai_summary, raw_transcript)
  VALUES ('delete', old.rowid, old.title, old.ai_summary, old.raw_transcript);
END;

CREATE TRIGGER IF NOT EXISTS meetings_au AFTER UPDATE ON meetings BEGIN
  INSERT INTO meetings_fts(meetings_fts, rowid, title, ai_summary, raw_transcript)
  VALUES ('delete', old.rowid, old.title, old.ai_summary, old.raw_transcript);
  INSERT INTO meetings_fts(rowid, title, ai_summary, raw_transcript)
  VALUES (new.rowid, new.title, new.ai_summary, new.raw_transcript);
END;

CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
  INSERT INTO tasks_fts(rowid, title, description, notes, assignee)
  VALUES (new.rowid, new.title, new.description, new.notes, new.assignee);
END;

CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description, notes, assignee)
  VALUES ('delete', old.rowid, old.title, old.description, old.notes, old.assignee);
END;

CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description, notes, assignee)
  VALUES ('delete', old.rowid, old.title, old.description, old.notes, old.assignee);
  INSERT INTO tasks_fts(rowid, title, description, notes, assignee)
  VALUES (new.rowid, new.title, new.description, new.notes, new.assignee);
END;

CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
  INSERT INTO documents_fts(rowid, filename, content_text)
  VALUES (new.rowid, new.filename, new.content_text);
END;

CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, filename, content_text)
  VALUES ('delete', old.rowid, old.filename, old.content_text);
END;

CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, filename, content_text)
  VALUES ('delete', old.rowid, old.filename, old.content_text);
  INSERT INTO documents_fts(rowid, filename, content_text)
  VALUES (new.rowid, new.filename, new.content_text);
END;
"#;
