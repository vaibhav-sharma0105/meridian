# AGENTS.md — Meridian Agent Orientation

This file is the entry point for any AI agent (Claude Code, Cursor, GitHub Copilot, Gemini Code Assist, or any other tool) working on this repository.

---

## Start Here

| Document | What it contains |
|---|---|
| [`CLAUDE.md`](CLAUDE.md) | Full agent context: architecture, conventions, design system, gotchas, testing, dev preferences |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Deep data flow, schema, component patterns, technical debt |
| [`src/lib/tauri.ts`](src/lib/tauri.ts) | **The frontend API contract** — every Tauri IPC command and TypeScript model |
| [`README.md`](README.md) | Setup instructions, user guide, connectors (Zoom, Sheets Relay) |
| [`CREDENTIALS_SETUP.md`](CREDENTIALS_SETUP.md) | Zoom OAuth and Gmail OAuth credential creation |

---

## What Is This Project

Meridian is a **local-first desktop app** (Tauri v2 = Rust backend + React frontend) that:
1. Ingests meeting transcripts (paste, Zoom sync, Google Sheets Relay via Gmail automation)
2. Extracts structured tasks using AI (LiteLLM / OpenAI / Anthropic / Ollama)
3. Manages tasks across projects with List, Kanban, and Table views
4. Stores everything locally in SQLite — no cloud, no backend server

---

## Three Things Every Agent Must Know

### 1. The Tauri boundary
`src/lib/tauri.ts` is the only place `invoke()` is called. Every Rust command has a typed TypeScript wrapper there. When you add a backend function, you must **also** register it in `src-tauri/src/lib.rs` — there is no compile error if you forget, only a silent runtime failure.

### 2. Client-side filter fields
`TaskFilters` has fields (`meeting_ids`, `project_id`) that are stripped before hitting the backend. They are applied in `useTasks.ts` after the fetch. Before adding a new filter field, read `CLAUDE.md` → "Client-Side Filter Fields".

### 3. The onboarding gate in tests
Playwright E2E tests mock `window.__TAURI_INTERNALS__`. The mock must return `{ onboarding_complete: "true" }` for `get_app_settings` — otherwise the app shows the onboarding screen and tests time out. See `tests/e2e/setup/tauri-mock.ts`.

---

## Agent Skills (Claude Code)

If you are Claude Code, these slash commands are available:

| Command | When to use |
|---|---|
| `/add-tauri-command` | Adding a new Rust IPC command end-to-end |
| `/add-task-filter` | Adding a new filter field to the task filter bar |
| `/run-tests` | Running the right test suite for what changed |
| `/fix-ui` | Making a UI change following the design system |
| `/update-docs` | Updating docs after any change |

---

## Maintenance Mandate

**After every change, update the relevant documentation files.** This applies to all agents, not just Claude Code. Specifically:

- New Tauri command → update `CLAUDE.md` conventions + `tauri-mock.ts`
- Schema change → update `docs/ARCHITECTURE.md` schema section
- New UI pattern → update `CLAUDE.md` design system section
- New user-facing feature → update `README.md` features section

Stale documentation is worse than no documentation — it actively misleads future agents.
