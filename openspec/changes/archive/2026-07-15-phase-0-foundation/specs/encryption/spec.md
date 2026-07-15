## ADDED Requirements

### Requirement: Always-Encrypted Database

The system SHALL encrypt all SQLite databases at rest using SQLCipher with AES-256 encryption. Encryption SHALL be enabled by default with no unencrypted mode available.

#### Scenario: Encrypted database creation
- **WHEN** application starts for the first time
- **THEN** system creates encrypted database with user-provided key or device-derived key

#### Scenario: Encrypted database access
- **WHEN** application opens existing database
- **THEN** system decrypts using stored key derivation parameters

#### Scenario: Reject unencrypted access
- **WHEN** attempt is made to open database without valid key
- **THEN** system returns authentication error and does not expose data

### Requirement: Encryption Key Management

The system SHALL support two key derivation modes: user password or device-bound key. User SHALL choose during onboarding.

#### Scenario: Password-based key derivation
- **WHEN** user chooses password mode during onboarding
- **THEN** system derives encryption key using PBKDF2 with user password and stored salt

#### Scenario: Device-bound key derivation
- **WHEN** user chooses device key mode during onboarding
- **THEN** system derives encryption key from device-specific identifier (machine ID + user ID)

#### Scenario: Key persistence
- **WHEN** encryption key is derived
- **THEN** system stores key derivation parameters (mode, salt) but never the key itself

### Requirement: Encryption Onboarding

The system SHALL present encryption setup during initial onboarding wizard before any data is created.

#### Scenario: First-run encryption setup
- **WHEN** user launches app for first time
- **THEN** onboarding wizard includes encryption mode selection step

#### Scenario: Explain tradeoffs
- **WHEN** encryption setup step is displayed
- **THEN** UI explains: password mode (more secure, requires password on new device) vs device mode (convenient, tied to this machine)

### Requirement: Database Migration

The system SHALL migrate existing unencrypted databases to encrypted format on upgrade.

#### Scenario: Detect unencrypted database
- **WHEN** app starts and finds unencrypted database from previous version
- **THEN** system prompts user to set up encryption before proceeding

#### Scenario: Migrate to encrypted
- **WHEN** user completes encryption setup with existing unencrypted database
- **THEN** system creates encrypted copy, verifies integrity, then removes unencrypted original

#### Scenario: Migration failure recovery
- **WHEN** migration fails mid-process
- **THEN** system preserves original unencrypted database and reports error

### Requirement: Password Change

The system SHALL allow users to change encryption password without data loss.

#### Scenario: Change password
- **WHEN** user provides current password and new password in settings
- **THEN** system re-encrypts database with new key derived from new password

#### Scenario: Reject invalid current password
- **WHEN** user provides incorrect current password
- **THEN** system rejects change request without modifying database

### Requirement: Encryption Status Display

The system SHALL display encryption status in settings.

#### Scenario: Show encryption mode
- **WHEN** user views security settings
- **THEN** UI shows current encryption mode (password or device) and database path

### Requirement: Safe Backup System

The system SHALL maintain backups in a location outside the main data directory to survive accidental deletion.

#### Scenario: Safe backup location
- **WHEN** database backup is created (migration, pre-restore)
- **THEN** system stores backup in ~/Documents/Meridian Backups (macOS) or Documents\Meridian Backups (Windows)
- **AND** backup survives deletion of ~/.meridian directory

#### Scenario: Dual backup during migration
- **WHEN** user migrates unencrypted database to encrypted
- **THEN** system creates backup in BOTH safe location and regular backup directory
- **AND** UI displays safe backup path on success or failure

#### Scenario: List safe backups
- **WHEN** user requests backup list
- **THEN** system returns all backups from safe backup directory with metadata

#### Scenario: Restore from safe backup
- **WHEN** user initiates restore from safe backup
- **THEN** system copies backup to database location and clears encryption config
