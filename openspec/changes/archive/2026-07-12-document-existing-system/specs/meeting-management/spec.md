## ADDED Requirements

### Requirement: Meeting Import Sources

The system SHALL support importing meetings from multiple sources: manual text paste, Zoom integration, and Google Sheets Relay (Gmail automation).

#### Scenario: Manual meeting import
- **WHEN** user pastes meeting transcript text
- **THEN** system creates meeting record with pasted content as transcript

#### Scenario: Zoom meeting import
- **WHEN** user syncs Zoom connection
- **THEN** system imports meetings from Zoom API with summaries and transcripts

#### Scenario: Sheets Relay meeting import
- **WHEN** user syncs Sheets Relay connection
- **THEN** system imports meetings from Google Sheet rows populated by Gmail automation

### Requirement: Pending Import Review

The system SHALL stage imported meetings as "pending imports" requiring user approval before creating meeting records.

#### Scenario: View pending imports
- **WHEN** sync discovers new meetings
- **THEN** system shows pending imports in notification center with preview

#### Scenario: Approve pending import
- **WHEN** user selects project and clicks "Import"
- **THEN** system creates meeting record and extracts tasks using AI

#### Scenario: Dismiss pending import
- **WHEN** user dismisses a pending import
- **THEN** system marks import as dismissed and hides from notification center

#### Scenario: Import type selection
- **WHEN** user approves import
- **THEN** user can choose to import "Summary" only or "Full Transcript"

### Requirement: Meeting Deduplication

The system SHALL prevent duplicate meeting imports using external identifiers: Zoom meeting ID for Zoom, email ID for Sheets Relay.

#### Scenario: Skip duplicate Zoom meeting
- **WHEN** sync encounters meeting with existing external_meeting_id
- **THEN** system skips import and increments skipped_duplicates count

#### Scenario: Skip duplicate Sheets Relay meeting
- **WHEN** sync encounters row with existing source_email_id
- **THEN** system skips import and increments skipped_duplicates count

### Requirement: Meeting Record Storage

The system SHALL store meeting records with: id, title, date, source, transcript, ai_summary, project_id, external_meeting_id, attendees, and timestamps.

#### Scenario: Create meeting record
- **WHEN** pending import is approved
- **THEN** system creates meeting with all metadata from import source

#### Scenario: Store AI summary
- **WHEN** meeting has AI-generated summary (from Zoom or extraction)
- **THEN** system stores summary separately from transcript

### Requirement: AI Task Extraction from Meetings

The system SHALL use AI to extract actionable tasks from meeting transcripts during import.

#### Scenario: Extract tasks from transcript
- **WHEN** meeting is imported with transcript
- **THEN** AI analyzes transcript and creates task records linked to meeting

#### Scenario: Extract tasks from summary
- **WHEN** meeting is imported with summary only
- **THEN** AI extracts tasks from summary content

#### Scenario: Task extraction failure
- **WHEN** AI extraction fails
- **THEN** system creates meeting without tasks and logs error

### Requirement: Meeting List View

The system SHALL display meetings in a list view with title, date, source indicator, task count, and health status.

#### Scenario: View project meetings
- **WHEN** user navigates to project meetings tab
- **THEN** system displays meetings sorted by date descending

#### Scenario: Meeting health badge
- **WHEN** meeting has associated tasks
- **THEN** system shows health badge based on task completion status

### Requirement: Meeting Detail View

The system SHALL provide meeting detail view showing transcript/summary content and linked tasks.

#### Scenario: View meeting detail
- **WHEN** user clicks on meeting
- **THEN** system shows meeting transcript, summary, attendees, and linked tasks

#### Scenario: Navigate to linked tasks
- **WHEN** user clicks on task in meeting detail
- **THEN** system navigates to task detail view
