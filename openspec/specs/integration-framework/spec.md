## ADDED Requirements

### Requirement: Integration registry
The system SHALL maintain a registry of available integrations with their configuration schemas, OAuth endpoints, and capability definitions.

#### Scenario: List available integrations
- **WHEN** user opens Integration Hub
- **THEN** system displays all supported integrations (GitHub, Jira, Slack) with connection status

### Requirement: OAuth authentication flow
The system SHALL support OAuth 2.0 authentication with PKCE for integrations requiring user authorization.

#### Scenario: GitHub OAuth connection
- **WHEN** user clicks "Connect GitHub"
- **THEN** system opens browser to GitHub OAuth consent page
- **THEN** on approval, system receives callback and stores encrypted tokens

#### Scenario: OAuth token refresh
- **WHEN** access token is expired and refresh token is valid
- **THEN** system automatically refreshes tokens without user intervention

### Requirement: Encrypted credential storage
The system SHALL store all integration credentials (OAuth tokens, API keys) encrypted in the database using the existing SQLCipher encryption.

#### Scenario: Store integration credentials
- **WHEN** user completes OAuth flow
- **THEN** system stores tokens in `integrations` table with `config` column encrypted

#### Scenario: Retrieve credentials for API call
- **WHEN** system needs to make API call to integration
- **THEN** system decrypts credentials from database and uses them

### Requirement: Per-integration autonomy settings
The system SHALL allow users to configure autonomy mode (manual/supervised/autonomous) for each connected integration independently.

#### Scenario: Set integration to supervised mode
- **WHEN** user sets GitHub integration to "supervised" mode
- **THEN** low-risk actions execute automatically, high-risk actions require approval

### Requirement: Integration sync scheduling
The system SHALL support configurable sync intervals for each integration with background daemon execution.

#### Scenario: Scheduled sync execution
- **WHEN** sync interval elapses for GitHub integration
- **THEN** daemon queues and executes sync job

#### Scenario: Manual sync trigger
- **WHEN** user clicks "Sync Now" for an integration
- **THEN** system immediately queues sync job regardless of schedule

### Requirement: Webhook receiver
The system SHALL run a local HTTP server to receive webhook callbacks from external systems with security token validation.

#### Scenario: Receive valid webhook
- **WHEN** external system sends POST to webhook endpoint with valid security token
- **THEN** system processes payload and triggers appropriate actions

#### Scenario: Reject invalid webhook
- **WHEN** request arrives without valid security token
- **THEN** system returns 401 Unauthorized and logs attempt

### Requirement: Integration audit logging
The system SHALL log all integration actions (syncs, creates, updates, sends) to the audit log with integration context.

#### Scenario: Log external write action
- **WHEN** system creates issue in GitHub via integration
- **THEN** audit log entry includes integration_id, action_type, entity details, and timestamp

### Requirement: Integration cache
The system SHALL cache fetched external data to reduce API calls and enable offline viewing.

#### Scenario: Cache integration data
- **WHEN** sync fetches issues from GitHub
- **THEN** system stores in `integration_cache` table with sync timestamp

#### Scenario: Serve cached data when offline
- **WHEN** user views linked GitHub issues while offline
- **THEN** system displays cached data with "Last synced" timestamp
