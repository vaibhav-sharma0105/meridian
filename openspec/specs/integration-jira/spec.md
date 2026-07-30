## ADDED Requirements

### Requirement: Atlassian authentication
The system SHALL support both Atlassian OAuth 2.0 and API token authentication for Jira Cloud.

#### Scenario: OAuth authentication
- **WHEN** user chooses OAuth connection
- **THEN** system redirects to Atlassian OAuth consent
- **THEN** on approval, system stores encrypted tokens

#### Scenario: API token authentication
- **WHEN** user provides email and API token
- **THEN** system validates credentials with Jira API
- **THEN** system stores encrypted token

### Requirement: Fetch assigned issues
The system SHALL fetch Jira issues assigned to the authenticated user using JQL queries.

#### Scenario: Sync assigned issues
- **WHEN** Jira sync runs
- **THEN** system executes JQL: `assignee = currentUser() ORDER BY updated DESC`
- **THEN** system stores issues in integration cache

### Requirement: Fetch sprint context
The system SHALL fetch active sprint information for configured boards.

#### Scenario: Sync sprint data
- **WHEN** Jira sync runs for a Scrum board
- **THEN** system fetches active sprint with issues
- **THEN** system stores sprint context in cache

### Requirement: Create Jira issue
The system SHALL allow creating Jira issues from Meridian tasks with project, issue type, summary, and description.

#### Scenario: Create issue from task
- **WHEN** user or skill triggers "Create Jira Issue"
- **THEN** system prompts for project and issue type
- **THEN** system creates issue via Jira API
- **THEN** system creates bidirectional link

### Requirement: Update Jira issue status
The system SHALL allow transitioning Jira issues through workflow states.

#### Scenario: Transition issue status
- **WHEN** user or skill triggers status change for linked issue
- **THEN** system fetches available transitions
- **THEN** system executes transition via API

### Requirement: Add Jira comment
The system SHALL allow adding comments to Jira issues.

#### Scenario: Post issue comment
- **WHEN** user or skill triggers "Add Comment" action
- **THEN** system posts comment via Jira API
- **THEN** system logs action in audit log

### Requirement: Bidirectional task-issue linking
The system SHALL maintain links between Meridian tasks and Jira issues with status sync.

#### Scenario: Sync Jira resolution to task
- **WHEN** linked Jira issue transitions to Done
- **THEN** system updates linked Meridian task to "done" (if autonomy allows)

### Requirement: Jira project configuration
The system SHALL allow users to configure which Jira projects to sync.

#### Scenario: Configure project sync
- **WHEN** user adds project to Jira integration
- **THEN** system includes project in JQL queries

### Requirement: Issue link visualization
The system SHALL display linked Jira issue details inline on Meridian tasks.

#### Scenario: View linked issue on task
- **WHEN** user views task with linked Jira issue
- **THEN** system displays issue key, status, and link to Jira
