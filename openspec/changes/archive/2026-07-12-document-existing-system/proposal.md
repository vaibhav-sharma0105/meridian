## Why

Meridian has grown into a full-featured meeting intelligence app with tasks, meetings, documents, AI chat, and integrations — but this functionality exists only in code. Without formal specifications, AI agents cannot understand the system's contracts, behaviors, and boundaries. This backfill creates the foundational specs that all future development will reference, enabling agentic development at scale.

## What Changes

- Create comprehensive specifications for all existing Meridian features
- Document data models, API contracts, UI components, and behaviors
- Establish the spec structure that future features will follow
- No code changes — this is documentation of what already exists

## Capabilities

### New Capabilities

- `task-management`: Core task CRUD, filtering, views (List/Kanban/Table), bulk actions, inline editing
- `meeting-management`: Meeting import, transcript handling, AI summary extraction, meeting-to-task linking
- `document-management`: Document upload (PDF/DOCX/PPTX/TXT/MD/CSV/VTT/SRT), chunking, FTS search, embeddings
- `ai-chat`: Multi-provider AI chat (OpenAI/Anthropic/Gemini/Groq/LiteLLM/Ollama), context-aware responses, prompt templates
- `project-management`: Project CRUD, project-scoped views, cross-project navigation
- `integration-zoom`: Zoom OAuth, meeting sync, transcript/summary fetching
- `integration-sheets-relay`: Google Sheets polling for Gmail automation, incremental sync
- `integration-mcp-server`: Read-only MCP server exposing tasks/meetings/projects to external agents
- `notification-system`: In-app notifications, pending imports, notification center UI

### Modified Capabilities

(None — this change documents existing features without modifying requirements)

## Impact

- **New files**: `openspec/specs/<capability>/spec.md` for each capability listed above
- **Documentation**: Establishes authoritative source of truth for system behavior
- **Agent enablement**: Enables AI agents to understand and work with the codebase
- **No runtime changes**: Zero code modifications
