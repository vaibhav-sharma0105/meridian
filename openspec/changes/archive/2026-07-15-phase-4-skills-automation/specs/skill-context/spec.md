## ADDED Requirements

### Requirement: Scope configuration

The system SHALL support skill scope settings:
- `scope`: "global" (all projects) or "project" (specific project)
- `project_id`: Required when scope is "project"
- `include_archived`: Whether to include archived items in context (default false)

#### Scenario: Project-scoped skill
- **WHEN** skill has scope "project" with project_id "proj-1"
- **THEN** execution context only includes data from proj-1

#### Scenario: Global skill
- **WHEN** skill has scope "global"
- **THEN** execution context includes data from all projects

#### Scenario: Exclude archived by default
- **WHEN** skill runs without include_archived set
- **THEN** archived tasks and meetings are excluded from context

### Requirement: Document inclusion

The system SHALL allow skills to specify which documents to include:
- `include_documents`: boolean (default true for summarize/analyze actions)
- `document_filter`: Optional regex pattern for filename filtering
- `max_documents`: Maximum documents to include (default 10)

#### Scenario: Include all project documents
- **WHEN** skill has include_documents true with no filter
- **THEN** up to max_documents are included in context

#### Scenario: Filter documents by pattern
- **WHEN** skill has document_filter ".*\\.pdf$"
- **THEN** only PDF documents are included in context

### Requirement: Custom instructions

The system SHALL support custom instructions that guide skill behavior:
- `system_prompt`: Base instruction for the skill (required for custom actions)
- `output_instructions`: How to format output (markdown, json, etc.)
- `persona`: Optional persona description (e.g., "You are a project manager")

#### Scenario: Custom system prompt
- **WHEN** skill defines system_prompt "Summarize in bullet points"
- **THEN** AI execution uses this as base instruction

#### Scenario: Output format instruction
- **WHEN** skill defines output_instructions "Respond in JSON with keys: summary, actions"
- **THEN** AI output follows the specified format

### Requirement: Context limits

The system SHALL enforce context limits following graceful degradation:
- `max_tokens`: Maximum context tokens (default 8000)
- `priority_order`: Which content to prioritize when truncating
- System truncates least-important content first

#### Scenario: Context exceeds limit
- **WHEN** context would exceed max_tokens
- **THEN** system truncates starting from lowest-priority content
- **AND** logs truncation in execution details

#### Scenario: Priority ordering
- **WHEN** context is truncated with priority_order ["documents", "meetings", "tasks"]
- **THEN** tasks are included first, then meetings, then documents
