## ADDED Requirements

### Requirement: GitHub OAuth App authentication
The system SHALL authenticate with GitHub using OAuth App flow with PKCE extension.

#### Scenario: Complete GitHub OAuth
- **WHEN** user initiates GitHub connection
- **THEN** system redirects to GitHub OAuth with required scopes (repo, read:user)
- **THEN** on callback, system exchanges code for access token

### Requirement: Fetch assigned issues
The system SHALL fetch GitHub issues assigned to the authenticated user across configured repositories.

#### Scenario: Sync assigned issues
- **WHEN** GitHub sync runs
- **THEN** system fetches issues where assignee matches authenticated user
- **THEN** system stores issues in integration cache

### Requirement: Fetch authored and reviewing PRs
The system SHALL fetch pull requests authored by or requesting review from the authenticated user.

#### Scenario: Sync pull requests
- **WHEN** GitHub sync runs
- **THEN** system fetches PRs authored by user and PRs where user is requested reviewer
- **THEN** system stores PRs in integration cache

### Requirement: Create GitHub issue
The system SHALL allow creating GitHub issues from Meridian tasks with title, body, labels, and assignees.

#### Scenario: Create issue from task
- **WHEN** user or skill triggers "Create GitHub Issue" for a task
- **THEN** system creates issue in configured repository
- **THEN** system creates bidirectional link between task and issue

### Requirement: Comment on GitHub PR
The system SHALL allow posting comments on pull requests.

#### Scenario: Post PR comment
- **WHEN** user or skill triggers "Comment on PR" action
- **THEN** system posts comment via GitHub API
- **THEN** system logs action in audit log

### Requirement: Bidirectional task-issue linking
The system SHALL maintain links between Meridian tasks and GitHub issues, syncing status changes.

#### Scenario: Link task to existing issue
- **WHEN** user links Meridian task to GitHub issue URL
- **THEN** system creates entry in `integration_links` table

#### Scenario: Sync issue closure to task
- **WHEN** linked GitHub issue is closed
- **THEN** system updates linked Meridian task status to "done" (if autonomy allows)

#### Scenario: Sync task completion to issue
- **WHEN** linked Meridian task is marked done
- **THEN** system closes linked GitHub issue (if autonomy allows)

### Requirement: GitHub repository configuration
The system SHALL allow users to configure which repositories to sync and monitor.

#### Scenario: Add repository to sync
- **WHEN** user adds repository to GitHub integration
- **THEN** system includes repository in subsequent syncs

### Requirement: Incremental sync
The system SHALL use GitHub's `since` parameter to fetch only changes since last sync.

#### Scenario: Incremental issue sync
- **WHEN** sync runs after initial sync
- **THEN** system requests only issues updated since last sync timestamp
