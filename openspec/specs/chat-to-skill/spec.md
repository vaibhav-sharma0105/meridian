## ADDED Requirements

### Requirement: Natural language skill creation

The system SHALL allow creating skills via natural language:
- User describes desired automation in chat
- AI extracts structured skill definition
- Preview shown for confirmation before creating

#### Scenario: Describe skill in chat
- **WHEN** user types "Every Monday morning, summarize my open tasks"
- **THEN** AI extracts: trigger=schedule cron="0 9 * * 1", action=summarize, scope=global
- **AND** shows preview of extracted skill

#### Scenario: Ambiguous description
- **WHEN** user description is ambiguous
- **THEN** AI asks clarifying questions
- **AND** refines skill definition with answers

### Requirement: Skill preview and confirmation

The system SHALL show skill preview before creation:
- Structured view of extracted configuration
- Editable fields for refinement
- "Create Skill" and "Cancel" buttons

#### Scenario: Edit preview
- **WHEN** user sees skill preview
- **THEN** user can modify any field
- **AND** changes reflected in final skill

#### Scenario: Confirm creation
- **WHEN** user clicks "Create Skill"
- **THEN** skill is created with shown configuration
- **AND** confirmation toast shown

### Requirement: Context-aware extraction

The system SHALL use conversation context for extraction:
- Current project influences default scope
- Recent topics influence suggested actions
- User patterns inform default approval mode

#### Scenario: Project context
- **WHEN** user is viewing proj-1 and says "notify me when tasks are overdue"
- **THEN** extracted skill defaults to project_id=proj-1

#### Scenario: Pattern-informed defaults
- **WHEN** user has autonomy preference "approve_first"
- **THEN** extracted skill defaults to that approval mode

### Requirement: Supported natural language patterns

The system SHALL understand common automation patterns:
- Time-based: "every day at 5pm", "weekly on Friday", "monthly"
- Event-based: "when a meeting is imported", "after task completion"
- Action-based: "summarize", "send me a reminder", "create follow-up tasks"

#### Scenario: Time pattern parsing
- **WHEN** user says "every morning at 9"
- **THEN** extracts cron="0 9 * * *"

#### Scenario: Event pattern parsing
- **WHEN** user says "whenever I complete a task"
- **THEN** extracts trigger=event, event_type="task_completed"

#### Scenario: Combined pattern
- **WHEN** user says "every Friday, draft a weekly summary email"
- **THEN** extracts: trigger=schedule cron="0 9 * * 5", action=draft_message, channel=email

### Requirement: Skill refinement via chat

The system SHALL allow refining skills through conversation:
- User can say "make it weekly instead"
- AI updates relevant field and shows updated preview
- Multiple rounds of refinement supported

#### Scenario: Refine trigger
- **WHEN** preview shows daily schedule and user says "make it weekly"
- **THEN** AI updates cron to weekly
- **AND** shows updated preview

#### Scenario: Add filter
- **WHEN** user says "but only for high priority tasks"
- **THEN** AI adds filter { priority: ["critical", "high"] }
- **AND** shows updated preview
