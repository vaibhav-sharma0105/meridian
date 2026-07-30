## ADDED Requirements

### Requirement: Cross-project blocker detection
The system SHALL detect when tasks in one project are blocked by tasks in another project.

#### Scenario: Detect explicit blocker reference
- **WHEN** a task description contains "blocked by [ProjectName]/[TaskTitle]" or a task ID from another project
- **THEN** system creates a cross_project_link with link_type 'blocks'
- **AND** system sets detected_by to 'pattern'
- **AND** system generates a suggestion alerting user to the blocker

#### Scenario: Detect semantic blocker relationship
- **WHEN** cross-project analysis job runs
- **AND** an open task in Project A has high embedding similarity to an incomplete task in Project B
- **AND** the Project A task contains blocking language ("waiting for", "depends on", "need X first")
- **THEN** system creates a cross_project_link with confidence score
- **AND** system sets detected_by to 'ai'

#### Scenario: User confirms or dismisses detected blocker
- **WHEN** user views a detected blocker suggestion
- **THEN** user can confirm (increases confidence) or dismiss (removes link)
- **AND** user dismissal is recorded to avoid re-detecting same relationship

### Requirement: Related task discovery
The system SHALL identify related tasks across projects based on content similarity.

#### Scenario: Find semantically related tasks
- **WHEN** cross-project analysis job runs
- **THEN** system computes embedding similarity between open tasks across all projects
- **AND** system creates 'related_to' links for pairs with similarity above 0.75 threshold
- **AND** links are visible in task detail view

#### Scenario: Suggest task consolidation
- **WHEN** two or more tasks across projects have similarity above 0.9
- **THEN** system generates a suggestion to consolidate or link the tasks
- **AND** suggestion shows both task titles and projects

### Requirement: Meeting consolidation detection
The system SHALL identify potentially duplicate or related meetings across projects.

#### Scenario: Detect duplicate meetings
- **WHEN** cross-project analysis job runs
- **AND** two meetings within 7 days have title similarity above 0.85
- **AND** meetings have overlapping attendees
- **THEN** system creates 'duplicate_meeting' link
- **AND** system generates suggestion to consolidate

#### Scenario: Detect recurring topic across meetings
- **WHEN** three or more meetings across projects discuss the same topic (based on transcript/summary similarity)
- **THEN** system generates insight: "Topic X discussed in N meetings across M projects"

### Requirement: Cross-project velocity analysis
The system SHALL compute and display velocity metrics across projects.

#### Scenario: Calculate cross-project velocity
- **WHEN** user opens Analytics dashboard
- **THEN** system displays tasks completed per week across all projects
- **AND** system shows trend (increasing, stable, decreasing)
- **AND** system highlights projects with declining velocity

#### Scenario: Compare project velocities
- **WHEN** user selects multiple projects in Analytics
- **THEN** system displays side-by-side velocity comparison
- **AND** system normalizes by team size if team roster data available

### Requirement: Cross-project link management
The system SHALL allow users to view, create, and remove cross-project links.

#### Scenario: View cross-project links
- **WHEN** user opens a task that has cross-project links
- **THEN** system displays linked tasks from other projects
- **AND** each link shows: project name, task title, link type, confidence

#### Scenario: Manually create cross-project link
- **WHEN** user clicks "Link to other project" on a task
- **THEN** system shows searchable list of tasks from other projects
- **AND** user can select and specify link type (blocks, related_to)
- **AND** system creates link with detected_by 'user'

#### Scenario: Remove cross-project link
- **WHEN** user clicks remove on a cross-project link
- **THEN** system deletes the link
- **AND** system records removal to prevent AI re-detection
