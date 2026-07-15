## ADDED Requirements

### Requirement: Skill data model

The system SHALL store skills with the following structure conforming to Anthropic's agent skills standard:
- `id`: Unique identifier
- `name`: Human-readable name (max 100 chars)
- `description`: Purpose and behavior description
- `trigger`: Trigger configuration (schedule/event/manual)
- `context`: Execution context (scope, documents, instructions)
- `action`: What the skill does (type, output format)
- `approval`: Approval mode and timeout
- `enabled`: Active/inactive toggle
- `shared`: Team visibility flag
- `owner_id`: Creator identifier
- `created_at`, `updated_at`: Timestamps

#### Scenario: Create skill with all fields
- **WHEN** user creates a skill with name, trigger, context, action, and approval settings
- **THEN** system stores the skill with a generated UUID
- **AND** sets enabled=true and shared=false as defaults

#### Scenario: Skill name validation
- **WHEN** user creates a skill with empty or >100 char name
- **THEN** system rejects with validation error

### Requirement: Skill CRUD operations

The system SHALL provide create, read, update, and delete operations for skills.

#### Scenario: List user's skills
- **WHEN** user requests their skills
- **THEN** system returns all skills where owner_id matches OR shared=true

#### Scenario: Update skill
- **WHEN** user updates a skill they own
- **THEN** system updates fields and sets updated_at timestamp

#### Scenario: Delete skill
- **WHEN** user deletes a skill they own
- **THEN** system removes the skill and all associated run history

#### Scenario: Cannot modify unowned skill
- **WHEN** user attempts to update/delete a skill they don't own
- **THEN** system rejects with permission error

### Requirement: Skill metadata

The system SHALL support skill metadata following Anthropic's progressive disclosure pattern:
- `version`: Skill definition version for compatibility
- `category`: Grouping (productivity, communication, reporting, custom)
- `icon`: Optional emoji or icon identifier
- `tags`: Array of searchable tags

#### Scenario: Filter skills by category
- **WHEN** user filters skills by category "productivity"
- **THEN** system returns only skills with matching category

#### Scenario: Search skills by tag
- **WHEN** user searches skills with tag "meeting"
- **THEN** system returns skills containing that tag
