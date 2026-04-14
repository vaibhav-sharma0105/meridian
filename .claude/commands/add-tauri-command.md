Add a new Tauri IPC command to Meridian.

Arguments: $ARGUMENTS
Expected format: "command_name: description of what it does"

Follow these steps exactly — missing any step causes a silent runtime failure:

## Step 1 — Write the Rust function

Determine which domain file in `src-tauri/src/commands/` the command belongs to (tasks, meetings, projects, ai, settings, connections, documents).

Add the function following this exact pattern:
```rust
#[tauri::command]
pub async fn your_command_name(
    required_arg: String,
    optional_arg: Option<String>,
    state: State<'_, AppState>,
) -> Result<ReturnType, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    repo::your_repository_function(&conn, &required_arg, optional_arg.as_deref())
}
```

Rules:
- Always `pub async fn` + `#[tauri::command]`
- Return type is always `Result<T, String>` — String is a human-readable error message
- DB access: lock the mutex, pass a reference to the repository function
- Validation logic goes HERE (before the DB call), not in the repository

## Step 2 — Add repository SQL (if needed)

If the command needs new SQL, add a function to the corresponding file in `src-tauri/src/db/repositories/`.

- All SQL lives in repositories — never in command files
- Use parameterized queries (never string interpolation for user input)
- Dynamic WHERE clauses: build `conditions: Vec<String>` + `bind_values: Vec<Box<dyn ToSql>>`

## Step 3 — Register in lib.rs ★ CRITICAL ★

Open `src-tauri/src/lib.rs` and add the command to the `.invoke_handler(tauri::generate_handler![...])` list.

Without this step, the command exists in Rust but is invisible to the frontend. There is NO compile error — the failure is a silent runtime "command not found".

## Step 4 — Add TypeScript wrapper in tauri.ts

Open `src/lib/tauri.ts` and add:
```typescript
export const yourCommandName = (requiredArg: string, optionalArg?: string) =>
  invoke<ReturnType>("your_command_name", { requiredArg, optionalArg });
```

The argument names in the `invoke` object must exactly match the Rust function parameter names (snake_case).

## Step 5 — Use from a hook or component

Always import from `@/lib/tauri`. Never call `invoke()` directly from a component.

## Step 6 — Add to Playwright mock

Open `tests/e2e/setup/tauri-mock.ts` and add a mock response in `mockData`:
```typescript
your_command_name: { /* mock return value */ },
```

## Step 7 — Update documentation

- Add an entry to `CLAUDE.md` "Known Gotchas" if this command has non-obvious behavior
- Update `docs/ARCHITECTURE.md` if this changes the data flow or schema
