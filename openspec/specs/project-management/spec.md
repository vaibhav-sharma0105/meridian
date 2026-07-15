# project-management Specification

## Purpose
TBD - created by archiving change document-existing-system. Update Purpose after archive.
## Requirements
### Requirement: Project CRUD Operations

The system SHALL support create, read, update, and delete operations for projects. Each project SHALL have: id, name, description, color, and timestamps.

#### Scenario: Create project
- **WHEN** user creates project with name
- **THEN** system creates project record with unique id and default color

#### Scenario: Update project
- **WHEN** user edits project name, description, or color
- **THEN** system persists changes

#### Scenario: Delete project
- **WHEN** user deletes (archives) project
- **THEN** system archives project and navigates to null project (All Tasks)
- **AND** React Query projects cache is invalidated so sidebar updates without reload

#### Scenario: List projects
- **WHEN** user views sidebar
- **THEN** system displays all projects with task counts

### Requirement: Project-Scoped Views

The system SHALL scope task, meeting, and document views to the selected project.

#### Scenario: Select project
- **WHEN** user clicks project in sidebar
- **THEN** main canvas shows only tasks, meetings, and documents for that project

#### Scenario: Project task count
- **WHEN** project is displayed
- **THEN** badge shows count of open tasks in project

### Requirement: Project Color Coding

The system SHALL allow users to assign colors to projects for visual differentiation.

#### Scenario: Set project color
- **WHEN** user selects color in project settings
- **THEN** project displays with selected color in sidebar and headers

#### Scenario: Default color
- **WHEN** project is created without color selection
- **THEN** system assigns a default color from palette

### Requirement: Cross-Project Navigation

The system SHALL support viewing all tasks across projects via "All Tasks" view.

#### Scenario: View all tasks
- **WHEN** user selects "All Tasks" in sidebar
- **THEN** system displays tasks from all projects with project indicator

#### Scenario: Project indicator
- **WHEN** viewing all tasks
- **THEN** each task shows its parent project name/color

### Requirement: Project Settings

The system SHALL provide project settings panel for editing project metadata.

#### Scenario: Open project settings
- **WHEN** user clicks settings icon on project header
- **THEN** system shows project settings modal

#### Scenario: Edit project metadata
- **WHEN** user modifies name, description, or color in settings
- **THEN** changes are saved and reflected immediately in sidebar and all views
- **AND** React Query projects cache is invalidated so sidebar updates without reload

### Requirement: Default Project

The system SHALL create a default "Inbox" project for tasks without explicit project assignment.

#### Scenario: Default project on first run
- **WHEN** app initializes with no projects
- **THEN** system creates "Inbox" project

#### Scenario: Assign to default
- **WHEN** task is created without project selection
- **THEN** task is assigned to Inbox project

