## Why

Users often see tasks for their entire team when they only need their own assignments. Bulk operations for cleaning up tasks require tedious one-by-one actions. This phase adds smart defaults based on inferred role and efficient bulk task management with safety guardrails.

## What Changes

- **Smart Default View**: Task list defaults to "My Tasks" or "All Tasks" based on inferred role (IC sees own, Manager sees all)
- **Easy Override**: View toggle in task list header for quick switching; no settings page needed
- **Bulk Clear Operations**: Delete, Archive, or user-chooses for multiple tasks at once
- **Combined Filter Selection**: Multi-select for both projects AND assignees with live count
- **Preview Before Action**: Scrollable list of affected tasks shown before confirmation
- **Undoable Operations**: Bulk actions reversible for N minutes after execution

## Capabilities

### New Capabilities
- `smart-task-view`: Role-based default filtering with instant override
- `bulk-task-operations`: Multi-select task actions with preview and undo

### Modified Capabilities
- `task-filters`: Add role-aware defaults; persist override preference per project
- `governance`: Extend undo system for bulk operations

## Technical Specifications

### Bulk Operations

**Entry Point:**
- Checkbox column appears on hover over task list header
- "Select" button in header toggles bulk mode
- Shift+click for range selection; Cmd/Ctrl+click for individual toggle

**Bulk Undo Storage:**
- **Diff-based**: Store only changed fields, not full task snapshots
- **Compression**: Group tasks by operation type; store as single compressed JSON
- **Retention**: Undo data kept for 10 minutes, then purged
- **Size limit**: Max 500 tasks per bulk operation; larger operations split with warning

**Failure Handling:**
- **Atomic per-batch**: Operations processed in batches of 50
- **Partial success**: If batch fails, previous batches committed; show "X of Y completed"
- **Retry option**: "Retry failed items" button for transient errors
- **Rollback option**: If partial success, offer "Undo completed items"

**Undo UX:**
- UndoBar appears at bottom of screen: "Deleted 23 tasks. [Undo] (9:45 remaining)"
- Countdown timer visible
- Default timeout: 5 minutes (configurable in settings: 1/5/10/15 min)
- Clicking "Undo" restores immediately; bar dismisses on timeout

### Bulk Action Discoverability

**Progressive Disclosure:**
1. First visit: No checkbox column visible (clean UI)
2. On hover over any task: Subtle "☐ Select multiple" appears in header
3. Click: Checkbox column reveals for all tasks
4. After bulk action: Remember user knows about feature; show checkboxes on hover going forward

**Keyboard Shortcuts:**
- `Cmd/Ctrl + A`: Select all visible tasks
- `Cmd/Ctrl + Shift + A`: Deselect all
- `Delete/Backspace`: Open bulk delete dialog (when tasks selected)

### Preview Dialog

**Content:**
- Scrollable list showing task title, project, assignee, status
- Grouped by project if spanning multiple
- Live count: "23 tasks selected"
- Highlight: Tasks with subtasks shown with "⚠️ includes 5 subtasks" warning

**Actions:**
- Primary: "Delete All" (red) / "Archive All" (yellow)
- Secondary: "Let me choose" → expandable per-task action selector
- Cancel: "Keep these tasks"

### Role-Based Defaults

**Persistence:**
- Override persists per-project in `user_preferences`
- If user always switches to "All Tasks" in Project X, remember for next visit
- Role-based default only applies to new projects or projects without override

## Impact

- **Database**: Extend `action_history` with `operation_type: bulk`, `affected_ids: JSON`, `diff_data: BLOB (compressed)`; add `user_preferences` table for per-project view overrides
- **Backend**: BulkTaskOperations command with batched execution; diff-based snapshot capture; compressed storage; retry logic
- **Frontend**: TaskViewToggle in list header; BulkSelectMode with checkbox column; BulkPreviewDialog with grouping; UndoBar with countdown; keyboard shortcut handlers
- **Performance**: Bulk operations batched in groups of 50; undo data compressed; cleanup job purges expired undo data hourly
