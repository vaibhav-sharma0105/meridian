# data-import Specification

## Purpose
TBD - created by archiving change phase-7-team-sync. Update Purpose after archive.
## Requirements
### Requirement: Data import
The system SHALL import data from Meridian export files.

#### Scenario: Initiate import
- **WHEN** user clicks "Import Data" in settings
- **THEN** system prompts for export file selection
- **AND** system prompts for decryption password
- **AND** system validates file format and version

#### Scenario: Validate import file
- **WHEN** import file is selected
- **THEN** system verifies checksum integrity
- **AND** system checks export_version compatibility
- **AND** system rejects incompatible versions with clear error

### Requirement: Import mode selection
The system SHALL offer merge or replace import modes.

#### Scenario: Merge mode
- **WHEN** user selects "Merge with existing data"
- **THEN** system keeps all local data
- **AND** system adds non-conflicting imported items
- **AND** system presents conflicts for resolution

#### Scenario: Replace mode
- **WHEN** user selects "Replace all data"
- **THEN** system shows confirmation warning
- **AND** on confirm, system backs up existing database
- **AND** system replaces all data with imported content

### Requirement: Conflict detection
The system SHALL detect conflicts between local and imported data.

#### Scenario: Identify conflicts
- **WHEN** merge import is initiated
- **THEN** system scans for items with matching IDs
- **AND** system compares updated_at timestamps
- **AND** system builds conflict report grouped by type

#### Scenario: No conflicts
- **WHEN** no conflicting items exist
- **THEN** system proceeds directly to import
- **AND** system shows "No conflicts found" message

### Requirement: Conflict resolution UI
The system SHALL provide UI for resolving import conflicts.

#### Scenario: Display conflicts
- **WHEN** conflicts are detected
- **THEN** system shows conflict resolution modal
- **AND** conflicts are grouped by type (tasks, skills, etc.)
- **AND** each conflict shows: item name, local vs import timestamps, diff preview

#### Scenario: Resolve individual conflict
- **WHEN** user reviews a conflict
- **THEN** user can choose: "Keep Local" | "Use Import" | "Skip"
- **AND** choice is applied to that item only

#### Scenario: Bulk resolution
- **WHEN** user wants to resolve multiple conflicts
- **THEN** system offers: "Keep All Local" | "Use All Import"
- **AND** applies choice to all remaining conflicts of that type

### Requirement: Import execution
The system SHALL apply imports within a transaction.

#### Scenario: Apply import
- **WHEN** user confirms import after conflict resolution
- **THEN** system creates backup of current database
- **AND** system applies all imports in a transaction
- **AND** system rebuilds search indexes
- **AND** system restores Qdrant vectors if included

#### Scenario: Import rollback
- **WHEN** import fails mid-process
- **THEN** system rolls back all changes
- **AND** system restores from backup
- **AND** system shows error with details

### Requirement: Import progress
The system SHALL show progress during import.

#### Scenario: Track import progress
- **WHEN** import is in progress
- **THEN** system shows progress bar with current entity type
- **AND** system shows items imported / total
- **AND** system allows cancellation (triggers rollback)

### Requirement: Standalone skill import
The system SHALL support importing individual skill files.

#### Scenario: Import skill file
- **WHEN** user drops or selects .skill.md or .skill.json file
- **THEN** system parses skill definition
- **AND** system shows skill preview
- **AND** system creates new skill with generated ID

#### Scenario: Duplicate skill detection
- **WHEN** imported skill has same name as existing
- **THEN** system prompts: "Replace" | "Import as Copy" | "Cancel"
- **AND** "Import as Copy" appends "(imported)" to name

### Requirement: Pre-import backup
The system SHALL automatically backup before import.

#### Scenario: Create backup
- **WHEN** import is about to apply changes
- **THEN** system creates timestamped backup of database
- **AND** backup is stored in ~/.meridian/backups/
- **AND** system shows backup location in confirmation

#### Scenario: Restore from backup
- **WHEN** user wants to undo import
- **THEN** system lists available backups
- **AND** user can restore any backup
- **AND** restore replaces current database with backup

