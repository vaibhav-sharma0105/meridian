## ADDED Requirements

### Requirement: Zoom OAuth Authentication

The system SHALL authenticate with Zoom using OAuth 2.0 with PKCE flow. Client credentials SHALL be provided via environment variables.

#### Scenario: Initiate OAuth flow
- **WHEN** user clicks "Connect Zoom"
- **THEN** system opens browser to Zoom authorization URL with PKCE challenge

#### Scenario: Handle OAuth callback
- **WHEN** Zoom redirects with authorization code
- **THEN** system exchanges code for access and refresh tokens

#### Scenario: Store tokens
- **WHEN** tokens are received
- **THEN** system stores access token and refresh token in OS keychain

#### Scenario: Token refresh
- **WHEN** access token is expired
- **THEN** system uses refresh token to obtain new access token

### Requirement: Zoom Meeting Sync

The system SHALL sync past meetings from Zoom API on user-triggered sync.

#### Scenario: Sync meetings
- **WHEN** user triggers sync (manual or on app launch)
- **THEN** system fetches past meetings from Zoom API

#### Scenario: Pagination
- **WHEN** user has many meetings
- **THEN** system paginates through results (max 20 pages / 1000 meetings)

#### Scenario: Initial sync window
- **WHEN** syncing for first time
- **THEN** system fetches meetings from last 14 days

#### Scenario: Incremental sync
- **WHEN** syncing after previous sync
- **THEN** system fetches meetings since last sync timestamp

### Requirement: Meeting Summary Fetch

The system SHALL fetch AI-generated meeting summaries from Zoom for each synced meeting.

#### Scenario: Fetch summary
- **WHEN** meeting is synced
- **THEN** system calls Zoom Meeting Summary API

#### Scenario: Summary not available
- **WHEN** meeting has no AI summary
- **THEN** system proceeds without summary

### Requirement: Meeting Transcript Fetch

The system SHALL fetch meeting transcripts from Zoom cloud recordings.

#### Scenario: Fetch transcript
- **WHEN** meeting has cloud recording with transcript
- **THEN** system downloads VTT transcript file

#### Scenario: No transcript
- **WHEN** meeting has no transcript recording
- **THEN** system proceeds without transcript

### Requirement: Zoom Rate Limiting

The system SHALL respect Zoom API rate limits with throttling between requests.

#### Scenario: Throttle requests
- **WHEN** fetching multiple meetings
- **THEN** system waits 100ms between API calls per meeting

### Requirement: Zoom Connection Status

The system SHALL display Zoom connection status and last sync time in settings.

#### Scenario: Show connected status
- **WHEN** Zoom is connected
- **THEN** UI shows "Connected" with last sync timestamp

#### Scenario: Show disconnected status
- **WHEN** Zoom is not connected
- **THEN** UI shows "Not Connected" with connect button

### Requirement: Zoom Disconnect

The system SHALL allow users to disconnect Zoom integration.

#### Scenario: Disconnect Zoom
- **WHEN** user clicks "Disconnect"
- **THEN** system removes tokens from keychain and clears connection status

### Requirement: Zoom Environment Configuration

The system SHALL require Zoom client credentials via ZOOM_CLIENT_ID and ZOOM_CLIENT_SECRET environment variables.

#### Scenario: Missing credentials
- **WHEN** environment variables are not set
- **THEN** Zoom connection is unavailable with appropriate message
