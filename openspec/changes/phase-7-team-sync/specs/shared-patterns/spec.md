## ADDED Requirements

### Requirement: Pattern contribution opt-in
The system SHALL allow users to opt-in to contributing anonymized patterns.

#### Scenario: Enable contribution
- **WHEN** user enables "Contribute to team patterns" in settings
- **THEN** system marks future observations as contributable
- **AND** system shows explanation of what is shared

#### Scenario: Disable contribution
- **WHEN** user disables contribution
- **THEN** system stops marking new observations as contributable
- **AND** existing contributions remain (not retroactively removed)

### Requirement: Pattern anonymization
The system SHALL anonymize patterns before contribution.

#### Scenario: Anonymize observation
- **WHEN** pattern observation is marked for contribution
- **THEN** system removes: entity IDs, project names, assignee names
- **AND** system preserves: action type, category keywords, timing patterns
- **AND** system hashes unique strings for deduplication

#### Scenario: Exclude sensitive content
- **WHEN** observation relates to content flagged as sensitive
- **THEN** system excludes it from contribution
- **AND** sensitive content never leaves local database

### Requirement: Pattern contribution storage
The system SHALL store anonymized patterns for export.

#### Scenario: Store contribution
- **WHEN** anonymized pattern is created
- **THEN** system stores in pattern_contributions table
- **AND** system records: pattern_type, observation_hash, contributed_at
- **AND** duplicate hashes are ignored (dedup)

#### Scenario: Export contributions
- **WHEN** user exports data with patterns selected
- **THEN** export includes pattern_contributions
- **AND** contributions can be shared with teammates

### Requirement: Team pattern import
The system SHALL merge imported pattern contributions into team patterns.

#### Scenario: Import team patterns
- **WHEN** import file contains pattern_contributions
- **AND** import is from different user (different device ID)
- **THEN** system creates team-scope pattern_models from contributions
- **AND** system increments contributor_count

#### Scenario: Aggregate team patterns
- **WHEN** multiple imports contribute similar patterns
- **THEN** system increases confidence of team pattern
- **AND** system tracks contributor_count for that pattern

### Requirement: Dual-layer pattern resolution
The system SHALL query both personal and team patterns.

#### Scenario: Query patterns
- **WHEN** system needs pattern suggestions (assignee, workflow, etc.)
- **THEN** system queries personal patterns first
- **AND** system queries team patterns as fallback/supplement
- **AND** personal patterns take priority in case of conflict

#### Scenario: Display pattern source
- **WHEN** suggestion is based on team pattern
- **THEN** system indicates "Based on team patterns" in UI
- **AND** system shows contributor_count for transparency

### Requirement: Pattern scope in database
The system SHALL distinguish personal and team patterns in storage.

#### Scenario: Pattern model scope
- **WHEN** pattern model is created
- **THEN** system sets scope to 'personal' (default) or 'team'
- **AND** team patterns include contributor_count

#### Scenario: Query by scope
- **WHEN** querying patterns
- **THEN** system can filter by scope
- **AND** system can query both scopes with priority order

### Requirement: Team pattern visibility
The system SHALL show team patterns in learning management UI.

#### Scenario: View team patterns
- **WHEN** user opens learning management settings
- **THEN** system shows separate section for team patterns
- **AND** each pattern shows: type, confidence, contributor_count

#### Scenario: Disable team patterns
- **WHEN** user toggles "Use team patterns" off
- **THEN** system excludes team patterns from suggestions
- **AND** personal patterns still apply
