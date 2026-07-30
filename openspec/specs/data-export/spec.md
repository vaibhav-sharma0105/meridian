# data-export Specification

## Purpose
TBD - created by archiving change phase-7-team-sync. Update Purpose after archive.
## Requirements
### Requirement: Full data export
The system SHALL export all user data to an encrypted portable format.

#### Scenario: Initiate export
- **WHEN** user clicks "Export Data" in settings
- **THEN** system prompts for export password
- **AND** system prompts for save location
- **AND** system shows content selection options

#### Scenario: Export content selection
- **WHEN** user initiates export
- **THEN** system offers checkboxes for:
  - Projects and tasks (always included)
  - Meetings and transcripts
  - Skills and skill history
  - Learned patterns
  - Team roster
  - Audit log (optional, can be large)
  - Document metadata (not file contents)
  - Vector embeddings (optional, can be large)

### Requirement: Export format
The system SHALL use an encrypted ZIP format with JSON contents.

#### Scenario: Create export archive
- **WHEN** export is confirmed
- **THEN** system creates ZIP containing:
  - manifest.json (version, timestamp, content list)
  - data/*.json (one file per entity type)
  - vectors/qdrant_snapshot/ (if selected)
  - checksum.sha256 (integrity verification)
- **AND** system encrypts ZIP with AES-256 using provided password

#### Scenario: Export versioning
- **WHEN** creating export
- **THEN** manifest.json includes:
  - export_version (format version)
  - app_version (Meridian version)
  - created_at (timestamp)
  - content_types (list of included data)

### Requirement: Export progress
The system SHALL show progress during export.

#### Scenario: Track export progress
- **WHEN** export is in progress
- **THEN** system shows progress bar with current step
- **AND** system shows estimated time remaining
- **AND** system allows cancellation

#### Scenario: Export completion
- **WHEN** export completes successfully
- **THEN** system shows success notification with file location
- **AND** system shows export size and content summary

### Requirement: Export error handling
The system SHALL handle export errors gracefully.

#### Scenario: Export fails
- **WHEN** export encounters error (disk full, permission denied)
- **THEN** system shows error message with details
- **AND** system cleans up partial export file
- **AND** system does not corrupt existing data

### Requirement: Standalone skill export
The system SHALL support exporting individual skills for sharing.

#### Scenario: Export single skill
- **WHEN** user clicks "Export" on a skill
- **THEN** system creates skill.md file in YAML+MD format
- **AND** file includes all skill configuration
- **AND** system prompts for save location

#### Scenario: Export shared skill
- **WHEN** user exports skill marked as shared
- **THEN** system includes owner attribution in export
- **AND** system strips owner_id from exported file

