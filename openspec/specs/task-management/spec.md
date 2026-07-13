# task-management Specification

## Purpose
TBD - created by archiving change document-existing-system. Update Purpose after archive.
## Requirements
### Requirement: Task CRUD Operations

The system SHALL provide create, read, update, and delete operations for tasks. Each task SHALL have: id, title, description, status (todo/in_progress/done/archived), priority (low/medium/high/critical), assignee, due_date, project_id, meeting_id, and timestamps.

#### Scenario: Create task
- **WHEN** user creates a task with title and project
- **THEN** system creates task with status "todo", generates unique id, sets created_at timestamp

#### Scenario: Update task fields
- **WHEN** user modifies task title, description, status, priority, assignee, or due_date
- **THEN** system persists changes and updates updated_at timestamp

#### Scenario: Delete task
- **WHEN** user deletes a task
- **THEN** system removes task from database

#### Scenario: Archive task
- **WHEN** user archives a task
- **THEN** system sets status to "archived" and task is hidden from default views

### Requirement: Task Filtering

The system SHALL support filtering tasks by status, priority, assignee, due_date range, and text search. Filters SHALL be combinable (AND logic).

#### Scenario: Filter by status
- **WHEN** user selects status filter "in_progress"
- **THEN** system displays only tasks with status "in_progress"

#### Scenario: Filter by multiple criteria
- **WHEN** user filters by status "todo" AND priority "high" AND assignee "John"
- **THEN** system displays only tasks matching all criteria

#### Scenario: Text search
- **WHEN** user enters search text "auth"
- **THEN** system displays tasks where title or description contains "auth" (case-insensitive)

#### Scenario: Filter by meeting
- **WHEN** user selects a meeting filter
- **THEN** system displays only tasks extracted from that meeting (client-side filter)

### Requirement: Task Views

The system SHALL provide three view modes for tasks: List, Kanban, and Table. View preference SHALL persist per user.

#### Scenario: List view
- **WHEN** user selects List view
- **THEN** system displays tasks as vertical cards grouped by status

#### Scenario: Kanban view
- **WHEN** user selects Kanban view
- **THEN** system displays tasks in columns by status with drag-and-drop between columns

#### Scenario: Table view
- **WHEN** user selects Table view
- **THEN** system displays tasks in tabular format with sortable columns

#### Scenario: View persistence
- **WHEN** user changes view mode
- **THEN** system saves preference and restores it on next session

### Requirement: Inline Task Editing

The system SHALL support inline editing of task title directly in List and Table views without opening a modal.

#### Scenario: Edit title inline
- **WHEN** user clicks on task title in list view
- **THEN** title becomes editable input field

#### Scenario: Save inline edit
- **WHEN** user presses Enter or clicks outside after editing
- **THEN** system saves the new title

#### Scenario: Cancel inline edit
- **WHEN** user presses Escape during inline edit
- **THEN** system reverts to original title without saving

### Requirement: Bulk Task Actions

The system SHALL support selecting multiple tasks and performing bulk actions: change status, change priority, archive, delete.

#### Scenario: Select multiple tasks
- **WHEN** user checks multiple task checkboxes
- **THEN** system shows bulk action bar with count and available actions

#### Scenario: Select all tasks
- **WHEN** user clicks "Select All" in bulk action bar
- **THEN** system selects all visible tasks matching current filters

#### Scenario: Bulk status change
- **WHEN** user selects tasks and chooses "Mark as Done"
- **THEN** system updates status to "done" for all selected tasks

#### Scenario: Bulk archive
- **WHEN** user selects tasks and chooses "Archive"
- **THEN** system archives all selected tasks

### Requirement: Task Priority Visualization

The system SHALL visually indicate task priority using colored left border: critical (red), high (orange), medium (yellow), low (gray).

#### Scenario: Display priority border
- **WHEN** task is rendered in any view
- **THEN** left border color reflects task priority

### Requirement: Drag and Drop Task Reordering

The system SHALL support drag-and-drop to change task status in Kanban view and reorder tasks within columns.

#### Scenario: Drag task between status columns
- **WHEN** user drags task from "Todo" column to "In Progress" column
- **THEN** system updates task status to "in_progress"

#### Scenario: Drag task to different project
- **WHEN** user drags task to different project drop zone
- **THEN** system moves task to target project

### Requirement: Task-Meeting Linkage

The system SHALL link tasks to their source meeting when extracted from meeting transcripts. Tasks SHALL display meeting reference.

#### Scenario: View task source meeting
- **WHEN** task was extracted from a meeting
- **THEN** task detail shows linked meeting title with navigation link

