pub const SQL: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  description  TEXT,
  color        TEXT NOT NULL DEFAULT '#6366f1',
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
  archived_at  TEXT
);

CREATE TABLE IF NOT EXISTS meetings (
  id               TEXT PRIMARY KEY,
  project_id       TEXT NOT NULL REFERENCES projects(id),
  title            TEXT NOT NULL,
  platform         TEXT NOT NULL DEFAULT 'manual',
  raw_transcript   TEXT,
  ai_summary       TEXT,
  health_score     INTEGER,
  health_breakdown TEXT,
  attendees        TEXT,
  meeting_at       TEXT,
  ingested_at      TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tasks (
  id                    TEXT PRIMARY KEY,
  project_id            TEXT NOT NULL REFERENCES projects(id),
  meeting_id            TEXT REFERENCES meetings(id),
  title                 TEXT NOT NULL,
  description           TEXT,
  assignee              TEXT,
  assignee_confidence   TEXT NOT NULL DEFAULT 'unassigned',
  assignee_source_quote TEXT,
  due_date              TEXT,
  due_confidence        TEXT NOT NULL DEFAULT 'none',
  due_source_quote      TEXT,
  status                TEXT NOT NULL DEFAULT 'open',
  tags                  TEXT NOT NULL DEFAULT '[]',
  kanban_column         TEXT NOT NULL DEFAULT 'open',
  kanban_order          INTEGER NOT NULL DEFAULT 0,
  is_duplicate          INTEGER NOT NULL DEFAULT 0,
  duplicate_of_id       TEXT REFERENCES tasks(id),
  notes                 TEXT,
  created_at            TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at            TEXT NOT NULL DEFAULT (datetime('now')),
  completed_at          TEXT
);

CREATE TABLE IF NOT EXISTS documents (
  id               TEXT PRIMARY KEY,
  project_id       TEXT NOT NULL REFERENCES projects(id),
  filename         TEXT NOT NULL,
  file_path        TEXT NOT NULL,
  file_type        TEXT NOT NULL,
  source_url       TEXT,
  content_text     TEXT,
  chunks           TEXT,
  embeddings_ready INTEGER NOT NULL DEFAULT 0,
  embedding_model  TEXT,
  file_size_bytes  INTEGER,
  uploaded_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ai_settings (
  id                 TEXT PRIMARY KEY,
  label              TEXT NOT NULL,
  provider           TEXT NOT NULL,
  base_url           TEXT,
  model_id           TEXT,
  ollama_base_url    TEXT NOT NULL DEFAULT 'http://localhost:11434',
  ollama_model       TEXT NOT NULL DEFAULT 'nomic-embed-text',
  embedding_provider TEXT NOT NULL DEFAULT 'ollama',
  is_active          INTEGER NOT NULL DEFAULT 0,
  created_at         TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS prompt_templates (
  id                   TEXT PRIMARY KEY,
  name                 TEXT NOT NULL,
  description          TEXT,
  system_prompt        TEXT NOT NULL,
  user_prompt_template TEXT NOT NULL,
  output_format        TEXT NOT NULL DEFAULT 'markdown',
  is_default           INTEGER NOT NULL DEFAULT 0,
  is_builtin           INTEGER NOT NULL DEFAULT 1,
  created_at           TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notifications (
  id         TEXT PRIMARY KEY,
  type       TEXT NOT NULL,
  title      TEXT NOT NULL,
  body       TEXT NOT NULL,
  task_id    TEXT REFERENCES tasks(id),
  project_id TEXT REFERENCES projects(id),
  is_read    INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS chat_history (
  id          TEXT PRIMARY KEY,
  project_id  TEXT REFERENCES projects(id),
  meeting_id  TEXT REFERENCES meetings(id),
  role        TEXT NOT NULL,
  content     TEXT NOT NULL,
  template_id TEXT REFERENCES prompt_templates(id),
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (key, value) VALUES
  ('theme', 'system'),
  ('language', 'en'),
  ('onboarding_complete', 'false'),
  ('notification_email_digest', 'true'),
  ('notification_desktop', 'true'),
  ('email_digest_address', ''),
  ('task_due_warning_days', '2'),
  ('default_task_view', 'list');

INSERT OR IGNORE INTO prompt_templates (id, name, description, system_prompt, user_prompt_template, output_format, is_default, is_builtin) VALUES

('tpl_2x2', '2x2 Leadership Update',
'Slide-ready executive status update in 4 quadrants',
'You are a chief of staff preparing a concise executive update. Format your response as a 2x2 grid with exactly these four quadrants: "Accomplishments" (what was completed), "In Progress" (active work), "Blockers" (what is stuck and needs leadership attention), "Next Steps" (committed actions with owners and dates). Use bullet points. Be specific. Maximum 3 bullets per quadrant. Use the project context provided.',
'Project: {{project_name}}
Open tasks: {{open_tasks}}
Completed tasks (recent): {{completed_tasks}}
Recent meetings: {{recent_meetings}}

Generate a 2x2 leadership update for this project.',
'markdown', 1, 1),

('tpl_jira', 'Jira Feature Request',
'Structured Jira ticket from meeting discussion',
'You are a senior product manager writing Jira tickets. Output a structured ticket with these exact fields: Summary (one line), Type (Bug/Feature/Task/Story), Priority (Critical/High/Medium/Low), Description (problem statement), Acceptance Criteria (numbered list of testable criteria), Labels (comma separated), Story Points (estimate: 1/2/3/5/8/13). Use the context provided. Be precise and actionable.',
'Project: {{project_name}}
Context from meeting: {{meeting_context}}
Relevant tasks: {{related_tasks}}
Project documents: {{doc_context}}

Generate a Jira ticket for the main feature or issue discussed.',
'jira', 1, 1),

('tpl_agenda', 'Next Meeting Agenda',
'Auto-generated agenda from open tasks and project status',
'You are an executive assistant preparing a meeting agenda. Generate a structured agenda with: (1) Quick wins to celebrate from completed tasks, (2) Blockers requiring group decision, (3) Tasks overdue or at risk, (4) Open items needing assignment or date commitment, (5) AOB. Each item should have a suggested time allocation. Total meeting should not exceed 45 minutes unless context demands it.',
'Project: {{project_name}}
Open tasks: {{open_tasks}}
Overdue tasks: {{overdue_tasks}}
Blockers: {{blockers}}
Last meeting summary: {{last_meeting_summary}}

Generate the agenda for the next meeting on this project.',
'markdown', 1, 1),

('tpl_status', 'Project Status Report',
'Comprehensive open vs done status for stakeholders',
'You are a project manager writing a stakeholder status report. Include: Executive Summary (2 sentences), Overall Status (Green/Amber/Red with reason), Completed This Period, In Progress, Upcoming, Risks and Mitigations, Team Workload summary. Be honest about delays. Flag anything Red clearly.',
'Project: {{project_name}}
All tasks: {{all_tasks}}
Analytics: {{analytics}}
Follow-through rate: {{follow_through_rate}}
Velocity: {{velocity}}

Generate a complete project status report.',
'markdown', 1, 1),

('tpl_freeform', 'Free-form Prompt',
'Ask anything about this project',
'You are a knowledgeable project intelligence assistant with full context of this project''s meetings, tasks, documents, and history. Answer the user''s question accurately and concisely using the provided context. If the answer requires information not in the context, say so clearly.',
'Project: {{project_name}}
Project context: {{full_context}}

User question: {{user_question}}',
'markdown', 1, 1);
"#;
