## ADDED Requirements

### Requirement: Skill editor modal

The system SHALL provide a modal editor for creating and editing skills:
- Opens from "New Skill" button or skill card edit action
- Form-based interface with progressive disclosure
- Basic fields: name, description, trigger type, action type
- Structured prompt editor with guided sections
- Advanced options (collapsed by default): category, approval mode

#### Scenario: Open new skill editor
- **WHEN** user clicks "New Skill"
- **THEN** modal opens with guided prompt mode active
- **AND** Role and Instructions sections expanded by default

#### Scenario: Toggle advanced mode
- **WHEN** user clicks "Advanced options"
- **THEN** editor expands to show category and approval mode
- **AND** state persists for session

### Requirement: Structured prompt editor

The system SHALL provide a sectioned prompt editor aligned with Anthropic Agent Skills Best Practices:
- Five sections: Role, Context, Instructions, Output Format, Examples
- Each section collapsible with token budget indicator
- Role and Instructions required (at least one must have content)
- Context and Instructions sections include {{variable}} insertion helper
- Two modes: Guided (collapsible sections) and Raw (XML textarea)

#### Scenario: Guided mode editing
- **WHEN** editor is in guided mode (default)
- **THEN** 5 collapsible sections shown with per-section token badges
- **AND** Role and Instructions are expanded by default
- **AND** Context, Output Format, Examples are collapsed

#### Scenario: Token budget warnings
- **WHEN** section content exceeds 80% of its budget
- **THEN** token badge turns amber
- **WHEN** section exceeds 100% budget
- **THEN** badge turns red with "over budget" message

#### Scenario: Variable insertion
- **WHEN** user clicks "var" button in Context or Instructions section
- **THEN** dropdown shows available variables (tasks, meetings, project_name, date, overdue_count, completed_today)
- **AND** selecting one inserts it at cursor position

#### Scenario: Raw mode editing
- **WHEN** user switches to raw mode via code icon
- **THEN** single textarea shows XML-tagged content
- **AND** switching back parses XML into sections

#### Scenario: Backward compatibility
- **WHEN** editing a skill with plain-text system_prompt (no XML tags)
- **THEN** entire content is placed in Instructions section
- **AND** other sections remain empty

### Requirement: Trigger configuration UI

The system SHALL provide trigger-specific configuration:
- Schedule: Cron input field with placeholder example
- Event: Event type dropdown (task_created, task_completed, task_overdue, meeting_imported)
- Manual: No additional config needed

#### Scenario: Cron input
- **WHEN** user selects trigger type "schedule"
- **THEN** UI shows cron expression input field
- **AND** validates on submit (required, non-empty)

#### Scenario: Event type selection
- **WHEN** user selects trigger type "event"
- **THEN** UI shows event type dropdown

### Requirement: Action configuration UI

The system SHALL provide action type selection:
- Summarize, Draft Message, Create Tasks, Analyze, Custom
- Displayed as a dropdown select

### Requirement: Test run capability

The system SHALL allow testing skills before saving:
- "Test" button visible only when editing existing skills
- Executes skill in preview mode (context only, no side effects)
- Shows generated output or error in preview pane below form

#### Scenario: Test run skill
- **WHEN** user clicks "Test" button
- **THEN** system executes skill with current config
- **AND** shows context preview in result pane
- **AND** does NOT create tasks or send messages

#### Scenario: Test run failure
- **WHEN** test run encounters error
- **THEN** error banner shown at top of form
- **AND** skill can still be saved

### Requirement: Form validation

The system SHALL validate skill configuration:
- Name: Required, 1-64 characters
- Trigger: Cron required for schedule type
- Prompt: At least role or instructions must be provided
- Errors shown inline per field

#### Scenario: Validation errors shown
- **WHEN** user tries to save with invalid config
- **THEN** validation errors shown inline below fields
- **AND** save button remains enabled (disabled only when name is empty)

#### Scenario: Valid skill saves
- **WHEN** all required fields are valid
- **THEN** clicking save creates/updates skill and closes modal

### Requirement: Error handling

The system SHALL display errors from save operations:
- Error banner at top of form with AlertCircle icon
- Message includes specific error from backend
- Banner persists until dismissed or new save attempt
