## ADDED Requirements

### Requirement: Action types

The system SHALL support the following action types:
- `summarize`: Generate a summary of context (meetings, tasks, documents)
- `draft_message`: Create a draft communication (email, slack, etc.)
- `create_tasks`: Extract and create tasks from context
- `analyze`: Perform analysis and return insights
- `custom`: Execute custom prompt with context

#### Scenario: Summarize action
- **WHEN** skill has action type "summarize"
- **THEN** system generates a natural language summary
- **AND** output is saved to skill run record

#### Scenario: Draft message action
- **WHEN** skill has action type "draft_message"
- **THEN** system generates a draft
- **AND** applies communication style from user patterns
- **AND** creates DraftMessage record linked to skill run

#### Scenario: Create tasks action
- **WHEN** skill has action type "create_tasks"
- **THEN** system extracts tasks from context
- **AND** respects approval mode before creating

#### Scenario: Analyze action
- **WHEN** skill has action type "analyze"
- **THEN** system generates structured analysis
- **AND** output follows output_instructions format

### Requirement: Output format configuration

The system SHALL support output format options:
- `format`: "markdown", "json", "html", "plaintext"
- `template`: Optional Handlebars template for output
- `max_length`: Maximum output length in characters

#### Scenario: Markdown output
- **WHEN** skill has format "markdown"
- **THEN** output uses markdown formatting

#### Scenario: Template output
- **WHEN** skill has template "## {{title}}\n{{summary}}"
- **THEN** AI output is structured to fill template variables

### Requirement: Action with side effects

The system SHALL track which actions have side effects:
- Summarize: No side effects (read-only)
- Draft message: Creates draft record (requires no approval)
- Create tasks: Creates tasks (requires approval per autonomy setting)
- Analyze: No side effects (read-only)
- Custom: May have side effects (marked in skill definition)

#### Scenario: Read-only action
- **WHEN** skill action is "summarize" or "analyze"
- **THEN** execution never modifies database state beyond run log

#### Scenario: Side effect action
- **WHEN** skill action is "create_tasks"
- **THEN** system checks autonomy/approval before writing

### Requirement: Multi-step actions

The system SHALL support chained action sequences:
- `actions`: Array of action configs executed in order
- Output of step N available as input to step N+1
- Failure at any step stops chain and records partial results

#### Scenario: Summarize then draft
- **WHEN** skill has actions [summarize, draft_message]
- **THEN** system first summarizes
- **THEN** uses summary as context for draft generation

#### Scenario: Chain failure handling
- **WHEN** step 2 of 3 fails
- **THEN** system records steps 1 output
- **AND** marks run as "partial_failure"
- **AND** stores error details
