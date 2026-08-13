# CLAUDE.md — Meridian Agent Context

> **Mandate for every agent:** After completing any change — feature, fix, refactor, or test — update this file and `docs/ARCHITECTURE.md` to reflect what changed. Stale documentation is worse than no documentation.

**This file = rules that change what you do.** Feature deep-dives, per-phase history, and known-gaps tables live in `docs/ARCHITECTURE.md`. Put background there, not here.

---

## What Is Meridian

Local-first, AI-powered meeting intelligence desktop app on Tauri v2. Ingests meeting transcripts (pasted text, Zoom, or Google Sheets Relay from Gmail automation), extracts structured tasks with AI, and manages them across projects via List/Kanban/Table views, inline editing, and an AI chat panel.

**All data lives on the user's machine** — `~/.meridian/meridian.db` (SQLite). No backend server. Only outbound calls are to the user's configured AI provider (OpenAI/Anthropic/Ollama/LiteLLM).

## Tech Stack (exact versions)

- **Shell/Backend**: Tauri v2.x · Rust stable 1.77+ · reqwest (async) · tokio
- **Frontend**: React 18.x + TypeScript 5.x · Vite 5.x · Tailwind CSS v3 · Zustand 4.x · @tanstack/react-query v5 · @dnd-kit 6.x
- **Data**: SQLite via rusqlite + SQLCipher (encrypted at rest) · Qdrant client (vector search) · ring crate PBKDF2 (key derivation) · keyring crate (OS keychain secrets)
- **Testing**: Vitest (unit) + Playwright (E2E)

## Repository Map

```
src/
  App.tsx                  # Root: onboarding gate → AppShell
  components/              # layout/ tasks/ meetings/ ai/ skills/ integrations/
                           # governance/ team/ sync/ activity/ messages/ role/
                           # productivity/ patterns/ suggestions/ onboarding/ shared/
  hooks/                   # useTasks, useMeetings, useSync, useAI, useSkills, ...
  stores/                  # Zustand: uiStore, taskStore, projectStore, ...
  lib/tauri.ts             # ★ THE ENTIRE FRONTEND API CONTRACT ★
  styles/globals.css       # Design tokens, CSS vars, animations
src-tauri/src/
  lib.rs                   # ★ ALL TAURI COMMANDS MUST BE REGISTERED HERE ★
  commands/                # One file per domain — no business logic, no SQL
  db/repositories/         # ALL SQL lives here
  db/migrations/           # Versioned schema files (v001–v024+)
  models/ ai/ skills/ integrations/ governance/ team/ sync/ messages/ role/
  productivity/ patterns/ suggestions/ attention/ daemon/ crypto/ audit/
  vectors/ documents/ connectors/
src-tauri/tests/           # command_contract.rs, serialization_contract.rs (boundary guards)
tests/e2e/                 # fixtures.ts (mockedPage) + setup/tauri-mock.ts
docs/ARCHITECTURE.md       # Deep architecture, feature internals, known gaps
CREDENTIALS_SETUP.md       # Zoom + Gmail + Google OAuth credential creation
```

---

## Critical Conventions

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

`src/lib/tauri.ts` is the authoritative API contract and the only place `invoke()` is called.
Guards: `src-tauri/tests/command_contract.rs` (command names) and `src-tauri/tests/serialization_contract.rs` (wire shapes). Run `npm run test:rust` before claiming frontend work is done.

### 2. Client-Side Filter Fields

`TaskFilters` fields **stripped before hitting the backend** in `src/hooks/useTasks.ts`:

```typescript
const backendFilters = {
  ...effectiveFilters,
  project_id: undefined,   // client-side: applied after fetch
  meeting_ids: undefined,  // client-side: applied after fetch
};
```

New filter field: if existing Rust SQL can't handle it, add to this strip list AND apply it in `queryFn` after the fetch. If the backend CAN handle it, add it to `TaskFilters` in `tauri.ts` + SQL in `tasks.rs`.

### 3. React Query Cache Keys

```typescript
["tasks", projectId, effectiveFilters]   // task lists
["meetings", projectId]                  // meeting lists
["projects"]                             // project list
["notifications"]                        // notification list
```

- `qc.setQueryData(key, updater)` — mutations where the new value is known immediately (rename, status change). Instant UI, no refetch. **Use this for user-facing updates.**
- `qc.invalidateQueries({ queryKey })` — complex mutations where the server returns derived data.

### 4. Playwright / Tauri Mock Requirements

- **Onboarding gate**: `App.tsx` shows `OnboardingWizard` unless `settings["onboarding_complete"] === "true"`. Mock must return `get_app_settings: { onboarding_complete: "true", theme: "light", language: "en" }` or tests time out waiting for a sidebar that never renders.
- **`window.__TAURI_INTERNALS__`** must include `invoke`, `transformCallback` (returns a numeric ID), `convertFileSrc`, and `metadata: { currentWindow: { label: 'main' } }`. Missing `transformCallback` → `@tauri-apps/api` event listeners crash → React never mounts → all tests time out.

### 5. Database Migrations

New schema goes in a NEW file `src-tauri/src/db/migrations/v00N_description.rs`, registered in the runner. **Never modify an existing migration file.**

### 6. Dual Retention Model (Message Center)

Two independent windows — do not conflate them:
- `user_profile.ai_context_days` (default 30) — how far back the AI can see. Enforced by `messages::get_messages_for_ai_context()`, consumed by `chat_with_project`. **Changing it changes real prompt content — it is not cosmetic.**
- `user_profile.message_retention` (default `forever`) — how long content stays browsable/searchable in the UI.

Content keeps existing long after the AI stops referencing it. Both are user-editable in the Message Center settings panel.

### 7. Role-Based Ordering Needs an Identity

`get_attention_items` reorders My Activity by the user's confirmed role (`role/ordering.rs`). The rules need to know who "me" is — that comes from `user_profile.display_name` / `user_email` / `user_aliases` (v022), matched against the free-text `tasks.assignee`.

**Ordering degrades silently by design**: unset identity, an unconfirmed role, or role `pm` all fall back to the repository's severity + recency order. If ordering "doesn't work", check identity before debugging the sort. Severity always outranks the role rule — a critical item never sorts below a warning.

### 8. Message Center Notifications Are Two Halves

`RoutingDecision::MessageCenterWithNotification` means **both** a `message_center` row and a notification linked to it via `notifications.message_id` (v022). Use `create_notification_for_message()`, not `create_notification_full()`, or the "View full result" link has no target. `should_create_message()` only covers the message half — it does not create the notification.

### 9. `conn`/`Send` Split (touching `sync/export.rs` or `sync/import.rs`)

`rusqlite::Connection` isn't `Sync`, but a `#[tauri::command]`'s future must be `Send`. Passing `conn` / a `MutexGuard<Connection>` into *anything* returning a future that spans an `.await` makes the **caller's** future non-`Send` too. Therefore: `build_local_entries`/`apply_local_import` are plain **sync** fns doing all `conn` work and returning owned data; `finish_export`/`finish_import` are **async** and take no `conn`. `commands/sync.rs` runs the sync half inside the `state.db.lock()` block, drops the lock, then awaits the async half. `ProgressFn` callbacks need an explicit `+ Sync` bound for the same reason.

---

## Design System

**Colors**
- Primary accent `indigo-500` (#6366f1) — ONLY for truly interactive/important elements (active state, CTA buttons, selected rings)
- Background: `white` / `zinc-900` (canvas), `#111113` (sidebar dark)
- Borders: `zinc-100`/`zinc-800` (subtle), `zinc-200`/`zinc-700` (hover)
- Text: `zinc-900` (titles), `zinc-500` (body/description), `zinc-400` (metadata/labels)
- Priority borders: `red-500` critical, `orange-400` high, `yellow-400` medium, `zinc-300` low

**Typography** — `Inter` at 13–13.5px base, `letter-spacing: -0.01em`; titles `font-semibold` (600); description `text-[12px] text-zinc-500 line-clamp-2`; metadata `text-[11px] text-zinc-400` with `·` dot separators.

**Component patterns**
- Cards: `border-l-[3px]` priority color, subtle border, hover → `zinc-50`/`zinc-800` (NOT transparent — opacity tricks look disabled)
- Active filter state: `ActiveChip` component (colored pill with inline `×`) replaces the select
- Tabs: underline (`border-b-2 border-indigo-500` active, `border-transparent` inactive)
- Popovers/dropdowns: `absolute top-full mt-1`, `shadow-xl`, `animate-fade-in`, outside-click close via `useEffect` + `mousedown`
- Custom checkboxes: `sr-only` native input + styled div, `Check` icon from lucide-react

**Spacing** — card padding `px-3 py-2.5`; section headers `px-4`; filter bar `px-4 py-2`; metadata dot-separated, not gap-based.

---

## Commands

```bash
npm run dev            # Full Tauri app (Rust + React, hot reload)
npm run vite:dev       # React only (no Rust, port 1420)
npm run build          # Production binary

npm run test           # Vitest — src/**/*.test.ts
npm run test:rust      # Cargo test — includes the boundary contract guards
npm run test:e2e       # Playwright, 149 tests in 16 files — REQUIRES `npm run vite:dev` running first
npm run test:e2e:ui    # Interactive Playwright UI
```

⚠️ **Never leave `npm run vite:dev` running alongside `npm run dev`.** `vite.config.ts` sets `port: 1420, strictPort: true`, and `npm run dev` runs `npm run vite:dev` as its `beforeDevCommand`. Whichever starts second cannot bind and **exits**, so the Tauri window loads `devUrl` (`localhost:1420`) against a dead or foreign server and renders a **blank white screen** with no error in the app. Kill the standalone Vite (`lsof -ti:1420 | xargs kill`) before `npm run dev`, and after running E2E tests.

Zoom OAuth needs env vars before `npm run dev`:
```bash
export ZOOM_CLIENT_ID=your_id
export ZOOM_CLIENT_SECRET=your_secret
```

E2E tests run in Playwright's Chromium (not the Tauri app) — **zero data pollution to SQLite**. All Tauri calls mocked from `tests/e2e/setup/tauri-mock.ts`.

---

## Development Preferences

1. **Ask before acting on ambiguous tasks** — 2 questions at a time, wait for answers. Never assume.
2. **No speculative abstractions** — no helpers/utilities/patterns "for future use".
3. **No cosmetic additions** — no comments, docstrings, type annotations, or error handling on code you didn't change.
4. **Minimal scope** — a bug fix doesn't need surrounding cleanup; a feature doesn't need extra configurability.
5. **Verify before recommending** — confirm any function/file/flag you reference actually exists.
6. **Fix root causes, not symptoms** — identify the real bug; don't retry a failing approach.
7. **Confirm destructive actions** — ask before deleting files, force-pushing, or touching shared infrastructure.
8. **UI changes require browser verification** — never claim "done" from code review alone.
9. **Progressive disclosure in UI** — de-emphasize/hide less critical info; title, status, priority always visible.
10. **Human attention psychology** — design should direct attention to what matters.
11. **indigo accent sparingly** — one clear primary action per screen; supporting actions use zinc/muted.
12. **Hover states must look interactive**, not disabled — solid `zinc-50`/`zinc-800`, no transparent overlays.
13. **`setQueryData` for instant updates** — patch cache after a successful mutation; don't rely on `invalidateQueries` alone.

---

## When You Finish a Change

1. **`CLAUDE.md`** — if you added a pattern, convention, or gotcha
2. **`docs/ARCHITECTURE.md`** — if data flow, schema, or component structure changed
3. **`tests/e2e/setup/tauri-mock.ts`** — mock responses for any new Tauri commands
4. **`src/lib/tauri.ts`** — keep the API contract authoritative
5. **Playwright tests** — add/update for new UI flows

---

## Known Gotchas

| Gotcha | Details |
|---|---|
| Missing command registration | New `#[tauri::command]` must be added to `lib.rs` invoke_handler. No compile error — only a runtime "command not found". **Guarded by `src-tauri/tests/command_contract.rs`**, which fails `cargo test` if any `invoke("name")` in `tauri.ts` has no matching handler. Run `npm run test:rust` before claiming frontend work is done. |
| Command name/param drift | `invoke()` takes a plain string, so a wrong **command name** or wrong **argument names** typecheck cleanly and fail only at runtime — and UI code often swallows the error into an empty render. The contract test catches wrong names; argument shapes are **not** covered, so always read the Rust signature (`state` is not a JS arg; `Option<T>` params are optional; a struct param like `settings: ProductivitySettings` must be sent as `{ settings: {...} }`, not flattened). |
| Rust enums crossing the boundary | Serde defaults to **external** tagging: `enum E { A { x } }` serializes as `{"A":{"x":1}}`, and a unit variant as the bare string `"A"` — **not** the `{"type":"A"}` discriminated union TS code expects. Every `status.type === "..."` check then silently evaluates false. Add `#[serde(tag = "type")]` to any enum returned from a command. Pinned by `src-tauri/tests/serialization_contract.rs`. |
| Mocks that encode assumptions | `tests/e2e/setup/tauri-mock.ts` returns whatever you write, so a mock built from the *TypeScript* type will make tests pass against a shape the Rust backend never emits — the tests then validate the bug. Build mocks from the **Rust** struct/enum, and pin the wire shape in `serialization_contract.rs` so both sides are checked against one source of truth. |
| `height: "50%"` in flex | Don't use inline percentage height in flex children — use `h-1/2 flex-shrink-0` Tailwind classes instead. |
| Onboarding gate in tests | Mock must return `onboarding_complete: "true"` in `get_app_settings` response. |
| `meridian-mcp` not in `cargo test` | The MCP crate is a separate binary and is **not** built by a workspace-root `cargo test`. It silently rotted out of sync with `CreateTaskInput` for an unknown period. Run `cargo check -p meridian-mcp` after touching shared models. |
| Blank white screen in the Tauri window | Almost always a port-1420 collision, not a code bug: `strictPort: true` means a stray `npm run vite:dev` makes `npm run dev`'s own Vite exit. Check `lsof -ti:1420` before debugging the frontend. A genuinely broken frontend shows the onboarding wizard or a console error, not a blank page. |
| Tauri v2 `transformCallback` | The mock for `window.__TAURI_INTERNALS__` MUST include `transformCallback`. Without it, React never mounts. |
| Stale closure in onBlur | Input `onBlur` captures stale state when `onKeyDown` (Escape) triggers unmount. Use a `cancelingRef` guard. |
| `getByText` strict mode | Playwright's `.or()` locator fails if both branches match. Use `.first()` or target one specific element. |
| Client filter fields | `meeting_ids` and `project_id` in `TaskFilters` are client-only — strip them in `useTasks.ts` before the `invoke` call. |
| `INSERT OR IGNORE` dedup | `upsert_pending_import` silently skips duplicates (returns `false`). Track in `SyncResult.skipped_duplicates`. |
| Encrypted DB auto-init | New installs auto-initialize device-mode encryption. Existing unencrypted DBs continue working (backward compatible). |
| Qdrant not embedded | Qdrant runs as external service (localhost:6334). Check `is_available()` before operations. |
| Audit log performance | Always query with filters and pagination. Unfiltered queries on large logs are slow. |
| macOS folder picker | `@tauri-apps/plugin-dialog` `open({ directory: true })` and `rfd::FileDialog::pick_folder()` don't work reliably on macOS (NSOpenPanel sheet issue). Use `osascript -e "choose folder"` via `pick_folder_dialog` command instead. |

---

## Where To Read More (`docs/ARCHITECTURE.md`)

| Topic | Section |
|---|---|
| Layout, state layers, filter pipeline, API contract | Frontend Architecture |
| Command/repository layers, schema, AI pipeline, sync | Backend Architecture |
| Secrets, SQLCipher key modes, audit logging | Security Model |
| Embeddings, chunking, hybrid RRF search, embedding worker | Embedding Architecture |
| Pattern learning, proactive agent, drafts, sensitive scan | Pattern Learning / Proactive Agent |
| Skills: triggers, actions, approval modes, run lifecycle, folder packages, YAML+MD `skill.md` format | Skills & Automation |
| GitHub/Jira/Slack/Google OAuth, sync jobs, MCP permissions | External Integrations |
| Autonomy modes/inheritance, risk classification, approvals, undo | Governance & Autonomy |
| Team roster, assignee intelligence, expertise learning, export/import, shared patterns | Team & Sync |
| My Activity, attention items, integration browser, cache retention | Integration Visibility |
| Message Center producers, role inference/drift, productivity | Message Center, Role & Productivity |
| Per-feature "Known Gaps / Future Work" tables | end of each feature section |

Specs: canonical requirements live in `openspec/specs/` — read those, not archived change folders under `openspec/changes/archive/`.
