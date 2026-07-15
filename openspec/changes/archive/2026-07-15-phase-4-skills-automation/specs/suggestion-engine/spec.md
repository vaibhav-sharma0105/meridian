## ADDED Requirements

### Requirement: Skill-generated suggestions

The system SHALL allow skills to create suggestions:
- Skills with action type "analyze" can output suggestions
- Skill run output parsed for suggestion extraction
- Suggestions linked to source skill via skill_run_id

#### Scenario: Skill creates suggestion
- **WHEN** skill completes with suggestion in output
- **THEN** system creates suggestion with skill_run_id reference
- **AND** suggestion includes skill name in reasoning

### Requirement: Suggestion triggers skill execution

The system SHALL support triggering skills from suggestion acceptance:
- Suggestion action_config can include skill_id
- Accepting suggestion queues skill execution
- Skill receives suggestion context as input

#### Scenario: Accept suggestion runs skill
- **WHEN** user accepts suggestion with action_config.skill_id
- **THEN** system queues skill execution
- **AND** passes suggestion data as trigger context

## MODIFIED Requirements

### Requirement: Suggestion storage

The system SHALL store suggestions in a `suggestions` table with fields: id, type, title, description, reasoning, action_config (JSON), severity, status, created_at, acted_at, skill_run_id (optional FK to skill_runs).

#### Scenario: Suggestion created
- **WHEN** suggestion engine detects actionable opportunity
- **THEN** suggestion is stored with status "pending" and severity level

#### Scenario: Skill-originated suggestion
- **WHEN** skill run generates suggestion
- **THEN** suggestion includes skill_run_id linking to source execution
