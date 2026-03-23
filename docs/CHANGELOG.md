# Changelog

All notable changes to Meridian are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- Multi-language support: English, Hindi (hi), Gujarati (gu) via react-i18next
- Command palette (Ctrl/Cmd+K) with fuzzy search across navigation, actions, and AI commands
- Kanban board view with drag-and-drop via @dnd-kit
- Analytics dashboard: velocity chart, workload heatmap, follow-through rate, health score history
- Document semantic search via Ollama embeddings (optional; falls back to FTS5 keyword search)
- Output templates: 2×2 update, Jira ticket, next agenda, status report
- Desktop notifications for overdue tasks
- Database backup command (`backup_database`)
- Import / export (JSON format) for projects and all associated data
- Per-provider AI settings with OS keychain storage for API keys
- Onboarding wizard (4 steps): welcome → AI setup → first project → first transcript
- Prompt template editor in Settings

### Fixed
- `SearchResult` missing `document_title` and `content` fields
- `extract_tasks_from_transcript` not forwarding `priority` and `confidence_score` to DB
- Documents repository SELECT missing `title` column after v004 migration
- `upload_text` command added for pasting raw text as a document

---

## [0.1.0] — 2026-03-22

### Added
- Initial release of Meridian
- Local-first SQLite storage with WAL mode and integer-versioned migrations (v001–v004)
- Meeting ingestion: transcript → AI extraction → tasks + health score
- Task management: list, kanban, and table views with filters
- Document ingestion: PDF, DOCX, PPTX, XLSX, TXT, MD, URL
- AI chat panel with project context and conversation history
- Meeting health scoring (0–100) with breakdown
- Duplicate task detection based on existing open tasks
- Assignee and due-date confidence levels (`committed`, `inferred`, `none/unassigned`)
- FTS5 full-text search across documents
- Dark / light / system theme via Tailwind `class` strategy
- Tauri v2 with OS keychain integration via `keyring` crate
- LiteLLM-compatible HTTP gateway for OpenAI, Anthropic, Gemini, Groq, Ollama
- Notifications centre with read / unread state
- Integration guides for Zoom, Google Meet, and Microsoft Teams

### Database schema (v001–v004)
- **v001**: Core tables — `projects`, `meetings`, `tasks`, `documents`, `document_embeddings`, `ai_settings`, `app_settings`, `schema_versions`
- **v002**: `chat_history`, `notifications`, `prompt_templates` tables; FTS5 virtual table `documents_fts`
- **v003**: `health_breakdown` column on `meetings`; `kanban_column`, `kanban_order`, `updated_at`, `completed_at` on `tasks`; `open_task_count` view on `projects`
- **v004**: `priority`, `confidence_score` on `tasks`; `decisions`, `duration_minutes` on `meetings`; `title` on `documents`
