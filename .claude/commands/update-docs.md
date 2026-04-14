Update Meridian's agent documentation after a code change.

Arguments: $ARGUMENTS (optional: brief description of what changed)

This command should be run after EVERY meaningful change — feature, fix, refactor, schema change, or new pattern.
Stale documentation misleads future agents more than no documentation.

## What to update and when

### Always check CLAUDE.md when you:
- Add a new Tauri command → update "Critical Conventions" pipeline section
- Add a client-side filter field → update "Client-Side Filter Fields" bullet
- Discover a new gotcha or workaround → add to "Known Gotchas" table
- Change a design pattern → update "Design System" section
- Change how tests are run → update "Testing" section
- Add a user-facing preference or constraint → add to "Observed Development Preferences"

### Always check docs/ARCHITECTURE.md when you:
- Change the database schema (new table, column, index, migration)
- Change the data flow (new step in the filter pipeline, sync pipeline, etc.)
- Change how React Query keys are structured
- Change how state flows between Zustand and React Query
- Add or remove a technical debt item
- Make a significant architectural decision with a "why"

### Always check tests/e2e/setup/tauri-mock.ts when you:
- Add a new Tauri command → add a mock response entry
- Change what an existing command returns → update the mock return value
- Add new fixture data needed by tests

### Always check README.md when you:
- Add a new user-facing feature (add to Features section)
- Change setup steps or prerequisites
- Add a new connector or integration (Zoom, Sheets Relay, etc.)
- Change keyboard shortcuts
- Change data storage paths

## How to update each file

### CLAUDE.md
Open the relevant section and edit in place. Keep entries concise — one line per gotcha, one code block per pattern. Do NOT append chronologically — edit the existing entry that's now wrong.

### docs/ARCHITECTURE.md
Edit the relevant section. If a schema table changed, update the table. If data flow changed, update the ASCII diagram. Mark resolved technical debt as done or remove it.

### tests/e2e/setup/tauri-mock.ts
Add to the `mockData` object in `buildTauriMockScript`. Keep mock values realistic — they're used in 39+ tests.

## Quality check

After updating:
1. Read back what you wrote — would a fresh agent understand it without context from this session?
2. Are there any "as discussed" or "see previous session" references? Remove them — docs must be self-contained.
3. Does the documentation describe WHY, not just WHAT?
