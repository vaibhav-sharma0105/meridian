## ADDED Requirements

### Requirement: Draft storage

The system SHALL store drafts in a `draft_messages` table with fields: id, task_id, channel, recipient, subject, body, ai_signature, status, created_at, sent_at.

#### Scenario: Draft created
- **WHEN** system generates a draft for a task
- **THEN** draft is stored with status "draft" and ai_signature=true

### Requirement: Auto-draft trigger

The system SHALL auto-generate drafts for tasks containing action keywords.

#### Scenario: Follow-up task detected
- **WHEN** task title or description contains "follow up", "send", "share", "email", or "message"
- **AND** task has no existing draft
- **THEN** system generates draft using AI with task context

### Requirement: Draft content generation

The system SHALL generate contextual draft content using AI.

#### Scenario: Draft generated
- **WHEN** auto-draft is triggered
- **THEN** AI generates draft body based on task title, description, and meeting context
- **AND** draft includes appropriate greeting and sign-off

### Requirement: AI signature

The system SHALL append "Drafted by Meridian" signature to generated drafts.

#### Scenario: Signature included
- **WHEN** draft is generated
- **THEN** signature line "Drafted by Meridian" is appended
- **AND** user can remove signature via checkbox

### Requirement: Drafts tab in task detail

The system SHALL show a Drafts tab in task detail view when drafts exist.

#### Scenario: View drafts
- **WHEN** user opens task with drafts
- **THEN** Drafts tab shows all drafts for this task
- **AND** each draft shows channel, recipient (if set), and preview

### Requirement: Inline draft editing

The system SHALL allow inline editing of draft content.

#### Scenario: Edit draft
- **WHEN** user clicks on draft
- **THEN** draft body becomes editable
- **AND** changes auto-save after 1 second of inactivity

### Requirement: Copy to clipboard

The system SHALL allow copying draft content to clipboard.

#### Scenario: Copy draft
- **WHEN** user clicks "Copy" on draft
- **THEN** draft body is copied to clipboard
- **AND** toast confirms "Copied to clipboard"

### Requirement: Communication style application

The system SHALL apply learned communication style to generated drafts when confidence >= 0.6.

#### Scenario: Style applied
- **WHEN** draft is generated
- **AND** communication_style pattern confidence >= 0.6
- **THEN** draft is adjusted to match learned length_preference and formality
- **AND** common phrases are included
