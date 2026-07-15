## ADDED Requirements

### Requirement: Built-in skills shipped with app

The system SHALL include pre-configured skills that users can enable:
- Weekly Summary
- Meeting Follow-up
- Overdue Alert
- Sprint Prep
- End of Day

#### Scenario: Enable built-in skill
- **WHEN** user enables a built-in skill
- **THEN** skill becomes active with default configuration
- **AND** appears in user's skill list

#### Scenario: Customize built-in skill
- **WHEN** user edits a built-in skill
- **THEN** changes saved to user's copy
- **AND** does not affect system template

### Requirement: Weekly Summary skill

The system SHALL provide Weekly Summary skill:
- Trigger: Schedule, every Friday at 5pm (configurable)
- Scope: Global (all projects)
- Action: Summarize tasks completed, tasks remaining, meetings held
- Output: Markdown summary with sections

#### Scenario: Weekly summary runs
- **WHEN** Friday 5pm arrives
- **THEN** skill generates summary of week's activity
- **AND** creates notification with summary content

### Requirement: Meeting Follow-up skill

The system SHALL provide Meeting Follow-up skill:
- Trigger: Event, meeting_imported
- Scope: Project of imported meeting
- Action: Analyze transcript, suggest follow-up tasks
- Approval: approve_first (default)

#### Scenario: Meeting follow-up triggered
- **WHEN** meeting is imported
- **THEN** skill analyzes transcript
- **AND** generates suggested follow-up tasks
- **AND** waits for user approval before creating

### Requirement: Overdue Alert skill

The system SHALL provide Overdue Alert skill:
- Trigger: Schedule, daily at 9am
- Scope: Global
- Action: Find overdue tasks, generate alert summary
- Output: List of overdue tasks with age and assignee

#### Scenario: Overdue alert runs
- **WHEN** daily 9am check runs
- **AND** overdue tasks exist
- **THEN** notification created with overdue task list

#### Scenario: No overdue tasks
- **WHEN** daily check runs
- **AND** no overdue tasks
- **THEN** no notification created (silent)

### Requirement: Sprint Prep skill

The system SHALL provide Sprint Prep skill:
- Trigger: Schedule, Monday 8am (configurable)
- Scope: Project (user selects)
- Action: Summarize backlog, suggest sprint goals
- Output: Sprint planning brief

#### Scenario: Sprint prep runs
- **WHEN** Monday 8am arrives
- **THEN** skill analyzes project backlog
- **AND** generates suggested sprint scope
- **AND** notifies user with prep summary

### Requirement: End of Day skill

The system SHALL provide End of Day skill:
- Trigger: Schedule, 5pm weekdays (configurable)
- Scope: Global
- Action: Summarize day's progress, highlight blockers
- Output: Daily wrap-up summary

#### Scenario: End of day runs
- **WHEN** 5pm weekday arrives
- **THEN** skill summarizes tasks touched today
- **AND** highlights tasks with blockers or stalls
- **AND** notifies user with daily summary

### Requirement: Built-in skill management

The system SHALL manage built-in skill lifecycle:
- New built-in skills added via app updates
- User-modified built-ins preserved across updates
- Users can "reset to default" if desired

#### Scenario: Reset to default
- **WHEN** user clicks "Reset to default"
- **THEN** skill configuration restored to built-in template
- **AND** run history preserved

#### Scenario: New built-in added
- **WHEN** app updates with new built-in skill
- **THEN** skill appears in available skills list
- **AND** disabled by default until user enables
