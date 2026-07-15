## ADDED Requirements

### Requirement: Edit pattern analysis

The system SHALL analyze draft_edit observations to extract style patterns including: average_length_delta, formality_shifts, common_phrase_additions, and common_phrase_removals.

#### Scenario: Length preference detected
- **WHEN** user consistently shortens AI drafts by 30%+
- **THEN** model records length_preference: "concise" with magnitude

#### Scenario: Formality shift detected
- **WHEN** user consistently changes formal phrases to casual equivalents
- **THEN** model records formality_preference: "casual"

### Requirement: Style model structure

Communication style model_data SHALL contain: length_preference (concise/verbose/neutral), formality_level (formal/casual/neutral), common_additions (phrases user adds), common_removals (phrases user removes), signature_patterns (consistent endings).

#### Scenario: Phrase patterns captured
- **WHEN** user adds "Thanks!" to 5+ drafts
- **THEN** "Thanks!" is added to common_additions with occurrence count

### Requirement: Style application

The system SHALL apply learned communication style to AI-generated drafts when style confidence >= 0.6.

#### Scenario: Style applied to draft
- **WHEN** AI generates a draft
- **AND** communication style confidence >= 0.6
- **THEN** draft is adjusted to match learned length_preference, formality_level, and includes common_additions

#### Scenario: Style not applied at low confidence
- **WHEN** communication style confidence < 0.6
- **THEN** AI generates draft without style adjustments

### Requirement: Style transparency

The system SHALL indicate when a draft has been styled, showing "Adjusted to match your style" with option to see original.

#### Scenario: User can see original
- **WHEN** styled draft is displayed
- **THEN** "View original" link reveals un-styled version

### Requirement: Context-aware style

The system SHALL maintain separate style models for different contexts: task_followup, meeting_summary, general_message.

#### Scenario: Context-specific style
- **WHEN** user edits meeting summaries differently than task follow-ups
- **THEN** system maintains separate style models per context
- **AND** applies appropriate style based on draft context
