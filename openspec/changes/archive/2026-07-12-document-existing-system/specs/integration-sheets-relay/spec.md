## ADDED Requirements

### Requirement: Sheets Relay Configuration

The system SHALL connect to Google Sheets Relay using an Apps Script web app URL and shared secret key.

#### Scenario: Configure Sheets Relay
- **WHEN** user enters Apps Script URL and secret key
- **THEN** system saves configuration in app_settings table

#### Scenario: Validate configuration
- **WHEN** configuration is saved
- **THEN** system tests connection by fetching data

### Requirement: Sheets Relay Data Polling

The system SHALL poll the Google Sheets Relay endpoint for new meeting data during sync.

#### Scenario: Fetch new rows
- **WHEN** sync is triggered
- **THEN** system calls Apps Script URL with auth key and since_ms parameter

#### Scenario: Incremental sync
- **WHEN** fetching data
- **THEN** system uses high-water mark timestamp to fetch only new rows

#### Scenario: Update watermark
- **WHEN** rows are fetched
- **THEN** system updates sheets_relay_last_sync_ms in app_settings

### Requirement: Sheets Relay Data Parsing

The system SHALL parse meeting data from Google Sheets rows including embedded JSON blobs.

#### Scenario: Parse row data
- **WHEN** row is received
- **THEN** system extracts: subject, sender, transcript, summary, timestamp

#### Scenario: Extract embedded JSON
- **WHEN** cell contains JSON blob
- **THEN** system detects and parses embedded JSON content

#### Scenario: Clean subject line
- **WHEN** subject contains "Meeting assets for ... are ready!"
- **THEN** system strips prefix and suffix to extract meeting title

### Requirement: Sheets Relay Deduplication

The system SHALL deduplicate Sheets Relay imports using source_email_id.

#### Scenario: Skip duplicate email
- **WHEN** row has source_email_id matching existing import
- **THEN** system skips import and counts as duplicate

### Requirement: Sheets Relay Connection Status

The system SHALL display Sheets Relay connection status in settings.

#### Scenario: Show configured status
- **WHEN** Sheets Relay is configured
- **THEN** UI shows configured URL (masked) and last sync time

#### Scenario: Show unconfigured status
- **WHEN** Sheets Relay is not configured
- **THEN** UI shows configuration form

### Requirement: Sheets Relay Architecture Documentation

The system SHALL document the Google Workspace setup required for Sheets Relay.

#### Scenario: Setup documentation
- **WHEN** user views Sheets Relay settings
- **THEN** link to CREDENTIALS_SETUP.md is provided

The architecture involves:
1. Google Workspace AI Studio monitors Gmail for Zoom AI summary emails
2. Extracts content and appends to Google Sheet
3. Apps Script web app exposes sheet data as JSON endpoint
4. Meridian polls the endpoint during sync
