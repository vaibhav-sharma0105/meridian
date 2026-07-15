## ADDED Requirements

### Requirement: Suggestions section in notifications

The system SHALL display a "Suggestions" section in the notification panel showing pending suggestions.

#### Scenario: Suggestions visible
- **WHEN** user opens notification panel
- **THEN** suggestions section shows pending suggestions sorted by severity (warning > info)
- **AND** each suggestion shows title, description snippet, and severity indicator

#### Scenario: Empty state with explainer
- **WHEN** user opens notification panel and no suggestions exist
- **THEN** system shows explainer card with lightbulb icon and "Smart suggestions" heading
- **AND** describes when suggestions appear: overdue/stale tasks, meetings without follow-ups, detected workflow patterns
- **AND** includes italicized note that suggestions appear automatically as user works

### Requirement: Accept suggestion action

The system SHALL allow users to accept suggestions with a "Do it" button.

#### Scenario: Accept overdue task suggestion
- **WHEN** user clicks "Do it" on overdue_task suggestion
- **THEN** system opens the task in detail view
- **AND** suggestion status changes to "accepted"

#### Scenario: Accept workflow sequence suggestion
- **WHEN** user clicks "Do it" on workflow_sequence suggestion
- **THEN** system creates the suggested task
- **AND** suggestion status changes to "accepted"

### Requirement: Dismiss suggestion action

The system SHALL allow users to dismiss suggestions.

#### Scenario: Dismiss suggestion
- **WHEN** user clicks "Dismiss" on a suggestion
- **THEN** suggestion status changes to "dismissed"
- **AND** suggestion is removed from view

### Requirement: Stop suggesting action

The system SHALL allow users to stop similar suggestions with "Stop suggesting this" option.

#### Scenario: Stop suggesting pattern
- **WHEN** user clicks "Stop suggesting this"
- **THEN** system records negative pattern for this suggestion type
- **AND** similar suggestions are suppressed in future

### Requirement: Suggestion detail view

The system SHALL show expanded reasoning when user clicks on a suggestion.

#### Scenario: View reasoning
- **WHEN** user clicks on suggestion card
- **THEN** system shows full description and reasoning
- **AND** shows action buttons (Do it, Dismiss, Stop suggesting)

### Requirement: High-severity toast

The system SHALL show toast notification for high-severity suggestions.

#### Scenario: Warning toast
- **WHEN** suggestion with severity "warning" is created
- **THEN** system shows toast notification with summary
- **AND** toast links to suggestion in notification panel
