# CLAUDE.md — Meridian Agent Context

> **Mandate for every agent:** After completing any change — feature, fix, refactor, or test — update this file and `docs/ARCHITECTURE.md` to reflect what changed. Stale documentation is worse than no documentation.

---

## What Is Meridian

Meridian is a **local-first, AI-powered meeting intelligence desktop app** built with Tauri v2. It ingests meeting transcripts (pasted text, Zoom, or Google Sheets Relay from Gmail automation), extracts structured tasks using AI, and lets users manage those tasks across projects with List/Kanban/Table views, inline editing, and an AI chat panel.

**Data lives entirely on the user's machine** — `~/.meridian/meridian.db` (SQLite). No backend server. The only outbound network calls are to the user's configured AI provider (OpenAI/Anthropic/Ollama/LiteLLM).

---

## Tech Stack — Exact Versions

| Layer | Technology | Version |
|---|---|---|
| Desktop shell | Tauri | v2.x |
| Frontend | React + TypeScript | 18.x / 5.x |
| Build | Vite | 5.x |
| Styling | Tailwind CSS | v3 |
| State | Zustand | 4.x |
| Async data | @tanstack/react-query | v5 |
| Drag & drop | @dnd-kit | 6.x |
| Backend | Rust (stable) | 1.77+ |
| Database | SQLite via rusqlite + FTS5 | — |
| Secrets | keyring crate (OS keychain) | — |
| HTTP client | reqwest (async) | — |
| Testing | Vitest (unit) + Playwright (E2E) | — |

---

## Repository Structure

```
meridian/
├── src/                          # React + TypeScript frontend
│   ├── App.tsx                   # Root: onboarding gate → AppShell
│   ├── components/
│   │   ├── layout/               # AppShell, Sidebar, MainCanvas, ContextPanel
│   │   ├── tasks/                # TaskCard, TaskListView, TaskKanbanView, TaskTableView, TaskFilters
│   │   ├── meetings/             # MeetingCard, MeetingIngest, MeetingHealthBadge
│   │   ├── ai/                   # AIChatPanel, AISettings, OutputTemplates
│   │   ├── connections/          # ConnectionsSettings (Zoom + Sheets Relay UI)
│   │   ├── documents/            # DocFolder (file upload + AI query)
│   │   ├── analytics/            # ProjectDashboard (velocity, workload charts)
│   │   ├── onboarding/           # OnboardingWizard + steps
│   │   ├── projects/             # ProjectCreate, ProjectSettings
│   │   ├── notifications/        # NotificationCenter
│   │   └── shared/               # EmptyState, UpdateBanner
│   ├── hooks/                    # useTasks, useMeetings, useSync, useAI, ...
│   ├── stores/                   # Zustand: uiStore, taskStore, projectStore, ...
│   ├── lib/
│   │   └── tauri.ts              # ★ THE ENTIRE FRONTEND API CONTRACT ★
│   └── styles/globals.css        # Design tokens, CSS vars, scrollbar, animations
│
├── src-tauri/src/                # Rust backend
│   ├── lib.rs                    # ★ ALL TAURI COMMANDS MUST BE REGISTERED HERE ★
│   ├── commands/                 # One file per domain (tasks, meetings, ai, ...)
│   ├── db/
│   │   ├── repositories/         # All SQL lives here (never in commands/)
│   │   └── migrations/           # Versioned schema files (v001–v005+)
│   ├── models/                   # Rust structs with serde (match TS interfaces)
│   ├── connectors/               # zoom.rs, sheets_relay.rs, sync.rs
│   └── ai/                       # litellm.rs, extractor.rs, embeddings.rs
│
├── tests/e2e/                    # Playwright tests
│   ├── fixtures.ts               # mockedPage fixture (injects Tauri mock)
│   └── setup/tauri-mock.ts       # window.__TAURI_INTERNALS__ mock + fixture data
│
├── CLAUDE.md                     # ← You are here
├── AGENTS.md                     # Model-agnostic pointer for any AI agent
├── docs/ARCHITECTURE.md          # Deep architecture: data flow, decisions
├── README.md                     # Full setup + user guide
└── CREDENTIALS_SETUP.md          # Zoom + Gmail OAuth credential creation
```

---

## Critical Conventions — Read Before Every Change

### 1. The Tauri Command Pipeline (most common source of bugs)

Every new backend feature follows this exact chain — missing any step silently breaks things:

```
1. Write Rust function in src-tauri/src/commands/<domain>.rs
   └── Must be: pub async fn, #[tauri::command], return Result<T, String>

2. Register in src-tauri/src/lib.rs inside .invoke_handler(tauri::generate_handler![...])
   └── FORGETTING THIS = "command not found" error at runtime, no compile warning

3. Add TypeScript wrapper in src/lib/tauri.ts
   └── Pattern: export const myCommand = (arg: Type) => invoke<ReturnType>("my_command", { arg });

4. Use from a hook or component via the tauri.ts export
   └── Never call invoke() directly in components — always go through tauri.ts
```

### 2. Client-Side Filter Fields

`TaskFilters` has fields that are **stripped before hitting the backend** in `useTasks.ts`:

```typescript
// src/hooks/useTasks.ts
const backendFilters = {
  ...effectiveFilters,
  project_id: undefined,   // client-side: applied after fetch
  meeting_ids: undefined,  // client-side: applied after fetch
};
```

When adding a new filter field: if it cannot be handled by the existing Rust SQL, add it to this strip list AND apply it in the `queryFn` after the fetch. If the backend CAN handle it, just add it to `TaskFilters` in `tauri.ts` and add SQL in `tasks.rs`.

### 3. React Query Cache Keys

All queries use this key pattern:
```typescript
["tasks", projectId, effectiveFilters]   // task lists
["meetings", projectId]                  // meeting lists
["projects"]                             // project list
["notifications"]                        // notification list
```

When mutating data, always invalidate or `setQueryData` the correct key:
```typescript
// Instant UI update (no refetch):
qc.setQueryData<Type[]>(["meetings", projectId], old => old?.map(...));

// Eventual consistency (schedules refetch):
qc.invalidateQueries({ queryKey: ["meetings", projectId] });
```

Use `setQueryData` for mutations where the new value is known immediately (rename, status change). Use `invalidateQueries` for complex mutations where the server may return derived data.

### 4. Onboarding Gate

`App.tsx` calls `getAppSettings()` on mount. If `settings["onboarding_complete"] !== "true"`, it shows `OnboardingWizard` instead of `AppShell`. **In Playwright tests**, the Tauri mock must return:
```javascript
get_app_settings: { onboarding_complete: "true", theme: "light", language: "en" }
```
Without this, tests time out waiting for the sidebar that never renders.

### 5. Tauri v2 Mock for Tests

`window.__TAURI_INTERNALS__` in Playwright tests requires **both** `invoke` and `transformCallback`:
```javascript
window.__TAURI_INTERNALS__ = {
  invoke: async (cmd, args) => { ... },
  transformCallback: (callback, once) => { /* returns numeric ID */ ++callbackId },
  convertFileSrc: (path) => path,
  metadata: { currentWindow: { label: 'main' } },
};
```
Missing `transformCallback` → `@tauri-apps/api` event listeners crash → React never mounts → all tests time out.

### 6. Database Migrations

New schema changes go in a new migration file `src-tauri/src/db/migrations/v00N_description.rs`. The migration runner in `db/connection.rs` applies them in order. Never modify existing migration files — always add a new one.

---

## Design System

### Colors
- **Primary accent**: `indigo-500` (#6366f1) — use ONLY for truly interactive/important elements (active state, CTA buttons, selected rings)
- **Background**: `white` / `zinc-900` (canvas), `#111113` (sidebar dark)
- **Borders**: `zinc-100` / `zinc-800` (subtle), `zinc-200` / `zinc-700` (hover)
- **Text hierarchy**: `zinc-900` (titles), `zinc-500` (body/description), `zinc-400` (metadata/labels)
- **Priority borders**: `red-500` critical, `orange-400` high, `yellow-400` medium, `zinc-300` low

### Typography
- Font: `Inter` at `13–13.5px` base, `letter-spacing: -0.01em`
- Title weight: `font-semibold` (600)
- Description: `text-[12px] text-zinc-500 line-clamp-2`
- Metadata: `text-[11px] text-zinc-400` with `·` dot separators

### Component Patterns
- **Cards**: `border-l-[3px]` priority color, subtle border (`zinc-100/zinc-800`), hover → `zinc-50/zinc-800` (NOT transparent — avoid opacity tricks that look disabled)
- **Active filter state**: Replace select with `ActiveChip` component (colored pill with inline `×`)
- **Tabs**: Underline style (`border-b-2 border-indigo-500` on active, `border-transparent` inactive)
- **Popovers/dropdowns**: `absolute top-full mt-1`, `shadow-xl`, `animate-fade-in`, close on outside click via `useEffect` + `mousedown`
- **Custom checkboxes**: `sr-only` native input + styled div, `Check` icon from lucide-react

### Spacing
- Card padding: `px-3 py-2.5`
- Section headers: `px-4`
- Filter bar: `px-4 py-2`
- Gap between metadata items: dot-separated (not gap-based)

---

## Data Flow Summary

```
User action (click/type)
  → Zustand store update (taskStore.setFilters / uiStore.setSelectedTask)
  → React Query hook (useTasks/useMeetings) re-runs query
  → useTasks strips client-only filter fields
  → invoke("get_tasks_for_project", { projectId, filters })
  → Rust: commands/tasks.rs → db/repositories/tasks.rs (SQL)
  → SQLite → Vec<Task>
  → React Query cache updated
  → Component re-renders
```

For writes (create/update/delete):
```
Component calls api.updateTask(input)
  → invoke("update_task", { input })
  → Rust: optimistic mutation in onMutate (React Query)
  → commands/tasks.rs → repositories/tasks.rs
  → qc.setQueryData (immediate) OR qc.invalidateQueries (eventual)
```

---

## Sync Architecture (Zoom + Sheets Relay)

```
useSync() → syncConnections() → invoke("sync_connections")
  → sync.rs: sync_zoom() + sync_sheets_relay()
  → For each meeting/row: upsert_pending_import() [INSERT OR IGNORE]
  → Dedup by: external_meeting_id (Zoom) / source_email_id (Sheets)
  → SyncResult { new_imports, skipped_duplicates, errors }
  → useSync.ts: toast for new imports + duplicates skipped
```

Sheets Relay special handling: JSON blobs in cells are detected by `extract_embedded_json()` in `sheets_relay.rs`. The `source_subject` column always wins as meeting title (strips "Meeting assets for " prefix and " are ready!" suffix).

---

## Testing

### Unit Tests
```bash
npm run test           # Vitest — runs src/**/*.test.ts files
npm run test:rust       # Cargo test — runs src-tauri/src/**/*_test.rs
```

### E2E Tests (Playwright)
```bash
# Terminal 1 — must be running first:
npm run vite:dev       # Vite dev server on localhost:1420

# Terminal 2:
npm run test:e2e       # 39 tests, ~4 seconds
npm run test:e2e:ui    # Interactive Playwright UI
```

E2E tests run in Playwright's Chromium (not the Tauri app) — **zero data pollution to SQLite**. All Tauri calls are mocked. Mock data lives in `tests/e2e/setup/tauri-mock.ts`.

---

## Running the App

```bash
npm run dev            # Full Tauri app (Rust + React, hot reload)
npm run vite:dev       # React only (no Rust, port 1420)
npm run build          # Production binary
```

Credentials for Zoom OAuth must be set as env vars before `npm run dev`:
```bash
export ZOOM_CLIENT_ID=your_id
export ZOOM_CLIENT_SECRET=your_secret
npm run dev
```

---

## Observed Development Preferences

These preferences were captured from actual development sessions and should guide agent behavior:

1. **Ask before acting on ambiguous tasks** — ask 2 questions at a time, wait for answers before proceeding. Never assume.
2. **No speculative abstractions** — don't add helpers, utilities, or patterns "for future use". Solve exactly the problem at hand.
3. **No cosmetic additions** — don't add comments, docstrings, type annotations, or error handling to code you didn't change.
4. **Minimal scope** — a bug fix doesn't need surrounding cleanup. A feature doesn't need extra configurability.
5. **Verify before recommending** — if you reference a function, file, or flag, confirm it exists. Don't recommend stale patterns.
6. **Fix root causes, not symptoms** — identify the actual bug before writing a fix. Don't retry the same failing approach.
7. **Confirm destructive actions** — always ask before deleting files, force-pushing, or modifying shared infrastructure.
8. **UI changes require browser verification** — after any frontend change, check the result in context. Don't claim "done" based on code review alone.
9. **Progressive disclosure in UI** — less critical information should be hidden or de-emphasized. Important information (title, status, priority) must always be visible.
10. **Human attention psychology** — design decisions should direct user attention toward what matters. Accent color (indigo) reserved for truly important/actionable elements only.
11. **indigo accent sparingly** — one clear primary action per screen. Supporting actions use zinc/muted tones.
12. **Hover states must look interactive**, not disabled — avoid transparent overlays; use solid `zinc-50 / zinc-800` backgrounds.
13. **`setQueryData` for instant updates** — after a successful mutation, patch the cache immediately. Don't rely on `invalidateQueries` alone for user-facing updates.

---

## When You Finish a Change

Update the following before marking work complete:

1. **This file (`CLAUDE.md`)** — if you added a new pattern, convention, or gotcha
2. **`docs/ARCHITECTURE.md`** — if data flow, schema, or component structure changed
3. **`tests/e2e/setup/tauri-mock.ts`** — if you added new Tauri commands, add mock responses
4. **`src/lib/tauri.ts`** — the living API contract; keep it the authoritative source
5. **Playwright tests** — add/update tests for new UI flows

---

## Known Gotchas

| Gotcha | Details |
|---|---|
| Missing command registration | New `#[tauri::command]` must be added to `lib.rs` invoke_handler. No compile error — only a runtime "command not found". |
| `height: "50%"` in flex | Don't use inline percentage height in flex children — use `h-1/2 flex-shrink-0` Tailwind classes instead. |
| Onboarding gate in tests | Mock must return `onboarding_complete: "true"` in `get_app_settings` response. |
| Tauri v2 `transformCallback` | The mock for `window.__TAURI_INTERNALS__` MUST include `transformCallback`. Without it, React never mounts. |
| Stale closure in onBlur | Input `onBlur` captures stale state when `onKeyDown` (Escape) triggers unmount. Use a `cancelingRef` guard. |
| `getByText` strict mode | Playwright's `.or()` locator fails if both branches match. Use `.first()` or target one specific element. |
| Client filter fields | `meeting_ids` and `project_id` in `TaskFilters` are client-only — strip them in `useTasks.ts` before the `invoke` call. |
| `INSERT OR IGNORE` dedup | `upsert_pending_import` silently skips duplicates (returns `false`). Track in `SyncResult.skipped_duplicates`. |
