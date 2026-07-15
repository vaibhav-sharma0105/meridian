## ADDED Requirements

### Requirement: Skill correction observation

The system SHALL record when users correct or reject skill outputs:
- Approval rejection recorded with rejection reason
- Output edits recorded with before/after diff
- Observations used to improve skill effectiveness

#### Scenario: Skill approval rejected
- **WHEN** user rejects skill execution in approval workflow
- **THEN** system records observation with pattern_type "skill_correction"
- **AND** observation includes skill_id, rejection_reason, and context

#### Scenario: Skill output edited
- **WHEN** user edits skill-generated draft before sending
- **THEN** system records observation with pattern_type "skill_correction"
- **AND** observation includes original and edited content

### Requirement: Skill usage observation

The system SHALL record skill execution patterns:
- Which skills user runs most frequently
- When skills are disabled after poor results
- Manual trigger patterns (time of day, context)

#### Scenario: Manual skill trigger recorded
- **WHEN** user manually triggers a skill
- **THEN** system records observation with pattern_type "skill_usage"
- **AND** includes time_of_day, current_project_id, and preceding_action

#### Scenario: Skill disabled recorded
- **WHEN** user disables a skill
- **THEN** system records observation with pattern_type "skill_usage"
- **AND** includes skill_id, days_active, run_count, and success_rate

## MODIFIED Requirements

### Requirement: Observation types

The system SHALL support the following observation pattern_types: task_completion, task_creation, task_assignment, draft_edit, suggestion_accepted, suggestion_dismissed, priority_change, workflow_sequence, skill_correction, skill_usage.

#### Scenario: All observation types stored
- **WHEN** any supported action type occurs
- **THEN** system records observation with appropriate pattern_type and structured JSON payload
