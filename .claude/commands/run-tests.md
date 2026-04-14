Run the appropriate tests for Meridian based on what changed.

Arguments: $ARGUMENTS (optional: "unit", "e2e", "rust", "all", or a specific file/pattern)

## Quick Reference

```bash
# TypeScript unit tests (Vitest)
npm run test

# E2E tests — requires Vite dev server already running on port 1420
npm run vite:dev   # Terminal 1
npm run test:e2e   # Terminal 2

# Interactive E2E UI
npm run test:e2e:ui

# Rust unit tests
npm run test:rust

# Type check only (no test runner)
npx tsc --noEmit
```

## Which tests to run for which changes

| Changed area | Run |
|---|---|
| `src/lib/tauri.ts` or any hook | `npx tsc --noEmit` + `npm run test:e2e` |
| `src/components/**` (UI) | `npm run test:e2e` |
| `src-tauri/src/commands/**` | `npm run test:rust` + `cargo check` |
| `src-tauri/src/db/repositories/**` | `npm run test:rust` |
| `src-tauri/src/connectors/**` | `cargo check` (no automated tests yet) |
| Filter logic in `useTasks.ts` | `npm run test:e2e` (task-filters.spec.ts) |
| Meeting card behaviour | `npm run test:e2e` (meeting-card.spec.ts) |
| Sidebar / navigation | `npm run test:e2e` (sidebar.spec.ts) |
| Full regression | `npm run test:rust && npx tsc --noEmit && npm run test:e2e` |

## E2E test context

- Tests run in Playwright Chromium — NOT the Tauri app
- All Tauri `invoke()` calls are mocked via `window.__TAURI_INTERNALS__`
- Zero data pollution to `~/.meridian/meridian.db`
- Mock data is in `tests/e2e/setup/tauri-mock.ts`
- The Vite server on port 1420 must be running before `npm run test:e2e`

## Common failure patterns

**All tests time out at `waitForSelector("text=Meridian")`**
→ Check that `get_app_settings` in the mock returns `{ onboarding_complete: "true" }`

**Uncaught TypeError: transformCallback is not a function**
→ The Tauri mock is missing `transformCallback`. See `tests/e2e/setup/tauri-mock.ts`.

**"command not found" at runtime (not in tests)**
→ A new `#[tauri::command]` was added but not registered in `src-tauri/src/lib.rs`.

**TypeScript error after adding filter field**
→ Check both `src/lib/tauri.ts` (TaskFilters interface) AND `src-tauri/src/models/task.rs` (Rust struct).

## After tests pass

If you added new commands or UI flows, add corresponding tests before marking work complete.
