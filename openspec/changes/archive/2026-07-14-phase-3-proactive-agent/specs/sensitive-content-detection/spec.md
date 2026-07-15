## ADDED Requirements

### Requirement: PII detection

The system SHALL detect personally identifiable information in drafts.

#### Scenario: PII detected
- **WHEN** draft contains patterns matching SSN, phone numbers, or addresses
- **THEN** system flags draft with warning type "pii"
- **AND** highlights detected content

### Requirement: Credential detection

The system SHALL detect credentials and secrets in drafts.

#### Scenario: Credentials detected
- **WHEN** draft contains patterns matching API keys, passwords, or tokens
- **THEN** system flags draft with warning type "credentials"
- **AND** severity is set to "critical"

### Requirement: Financial data detection

The system SHALL detect financial information in drafts.

#### Scenario: Financial data detected
- **WHEN** draft contains patterns matching credit card numbers, bank accounts, or financial amounts with context
- **THEN** system flags draft with warning type "financial"

### Requirement: Non-blocking warnings

The system SHALL display warnings without blocking user actions.

#### Scenario: Warning displayed
- **WHEN** sensitive content is detected
- **THEN** warning banner appears above draft
- **AND** user can still copy, edit, or send draft

### Requirement: Warning dismissal

The system SHALL allow users to dismiss sensitive content warnings.

#### Scenario: Dismiss warning
- **WHEN** user clicks "Dismiss" on warning
- **THEN** warning is hidden for this draft
- **AND** dismissal is logged in audit log

### Requirement: Audit logging

The system SHALL log all sensitive content detections.

#### Scenario: Detection logged
- **WHEN** sensitive content is detected
- **THEN** audit log entry is created with:
  - action_type: "sensitive_content_detected"
  - entity_type: "draft"
  - details: { warning_type, content_preview (redacted), dismissed }

### Requirement: Scan on edit

The system SHALL re-scan drafts when content changes.

#### Scenario: Rescan on edit
- **WHEN** user edits draft content
- **AND** 2 seconds have passed since last keystroke
- **THEN** system re-scans for sensitive content
- **AND** updates warnings accordingly
