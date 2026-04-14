Add a new filter field to Meridian's task filter system.

Arguments: $ARGUMENTS
Expected format: "field_name: description — backend|client-side"

Meridian filters flow through a strict pipeline. Follow every step.

## Determine: backend or client-side?

**Backend filter** — The Rust SQL can handle it efficiently (single value match, date range, text search).
**Client-side filter** — Requires array IN logic, join, or cross-project logic that the dynamic SQL builder doesn't support.

## Step 1 — Add the field to TaskFilters in tauri.ts

Open `src/lib/tauri.ts`, find the `TaskFilters` interface, and add:
```typescript
export interface TaskFilters {
  // ... existing fields ...
  your_field?: YourType;  // comment: backend|client-side only
}
```

## Step 2A — Backend filter: add SQL in tasks.rs

Open `src-tauri/src/db/repositories/tasks.rs`. Add to BOTH `get_tasks_for_project` AND `get_all_tasks` (they have identical filter logic):

```rust
if let Some(value) = &filters.your_field {
    conditions.push(format!("your_column = ?{}", param_idx));
    bind_values.push(Box::new(value.clone()));
    param_idx += 1;
}
```

The Rust `TaskFilters` struct in `src-tauri/src/models/task.rs` must also have the new field:
```rust
pub your_field: Option<YourType>,
```

## Step 2B — Client-side filter: strip in useTasks + apply after fetch

Open `src/hooks/useTasks.ts`. Add to the strip object:
```typescript
const backendFilters = {
  ...effectiveFilters,
  project_id: undefined,
  meeting_ids: undefined,
  your_field: undefined,  // ← add here
};
```

Then apply the filter after the fetch inside `queryFn`:
```typescript
if (effectiveFilters.your_field) {
  tasks = tasks.filter(t => /* your condition */);
}
```

## Step 3 — Add UI in TaskFilters.tsx

Open `src/components/tasks/TaskFilters.tsx`.

**For a simple select:** Follow the same pattern as status/priority — show `ActiveChip` when active, show `<select>` when inactive.

**For a popover (multi-select or date range):** Follow the `MeetingFilter` or `DateFilter` pattern:
- `useRef` for outside-click detection
- `useEffect` + `document.addEventListener("mousedown", ...)` when open
- Button trigger with active state using `activeCls` classes
- Dropdown with `absolute top-full mt-1 z-50 animate-fade-in`

**Active chip pattern:**
```tsx
{filters.your_field ? (
  <ActiveChip label="Your Label" onClear={() => setFilters({ your_field: undefined })} />
) : (
  <select value="" onChange={(e) => setFilters({ your_field: e.target.value || undefined })}>
    ...
  </select>
)}
```

Add to the `hasFilters` check and the `clearAll()` function.

## Step 4 — Update Playwright mock (if backend)

If it's a backend filter, no mock changes needed (the mock returns static data regardless of filters).

If it's a client-side filter that changes which mock items are returned, update `tests/e2e/setup/tauri-mock.ts` fixture data if needed.

## Step 5 — Add a Playwright test

Open the relevant spec file in `tests/e2e/task-filters.spec.ts` and add tests for:
- Filter control is visible
- Selecting a value shows active state
- Clear resets the filter

## Step 6 — Update documentation

Update `CLAUDE.md` "Client-Side Filter Fields" section if you added a new client-side-only field.
