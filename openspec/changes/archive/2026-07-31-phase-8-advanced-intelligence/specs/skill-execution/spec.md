## MODIFIED Requirements

### Requirement: Approval workflow

The system SHALL support approval modes integrated with the autonomy controller:
- `auto`: Execute immediately, no approval needed
- `notify`: Execute and notify user of results
- `approve_first`: Require approval before executing side effects
- `approve_always`: Require approval for every execution
- Approval mode is overridden by autonomy controller when autonomy mode is more restrictive
- **NEW**: Skills can request predictive document pre-fetch before execution

#### Scenario: Auto approval mode
- **WHEN** skill has approval "auto"
- **AND** autonomy mode allows automatic execution for this risk level
- **THEN** executes immediately without user interaction

#### Scenario: Autonomy overrides skill approval
- **WHEN** skill has approval "auto"
- **AND** global autonomy mode is "Manual"
- **THEN** system requires approval regardless of skill setting
- **AND** action is queued in pending_approvals

#### Scenario: Approve first mode
- **WHEN** skill has approval "approve_first" and action creates tasks
- **THEN** system shows preview of tasks to create
- **AND** waits for user approval or rejection
- **AND** only creates tasks if approved

#### Scenario: Risk-based approval routing
- **WHEN** skill action is classified as high-risk or critical-risk
- **THEN** system routes to approval queue regardless of skill approval setting
- **AND** action is not executed until approved

#### Scenario: Approval timeout
- **WHEN** approval_pending exceeds configured timeout (default 24 hours)
- **THEN** system archives the action (does not execute)
- **AND** notifies user of timeout
- **AND** action is retrievable from archived approvals

#### Scenario: Predictive pre-fetch for scheduled skills
- **WHEN** skill is scheduled to run in next 30 minutes
- **AND** skill context includes documents
- **THEN** system pre-fetches relevant document embeddings into memory cache
- **AND** skill execution uses cached context for faster response

## ADDED Requirements

### Requirement: Predictive document context
The system SHALL pre-fetch document context for scheduled skills.

#### Scenario: Pre-fetch before execution
- **WHEN** skill has include_documents: true in context_config
- **AND** skill is scheduled to run within 30 minutes
- **THEN** system identifies relevant documents from project
- **AND** pre-loads document chunks into memory cache

#### Scenario: Use pre-fetched context
- **WHEN** skill executes after pre-fetch
- **THEN** system uses cached document context
- **AND** document retrieval is faster (no Qdrant query needed)

#### Scenario: Cache expiration
- **WHEN** pre-fetched context is older than 30 minutes
- **THEN** system refreshes from Qdrant
- **AND** stale cache is discarded

### Requirement: Cross-project skill context
The system SHALL support skills that operate across projects.

#### Scenario: Skill with global scope
- **WHEN** skill has scope: 'global' in context_config
- **THEN** skill context includes data from all projects
- **AND** cross-project links are included in context

#### Scenario: Cross-project analysis skill
- **WHEN** skill action analyzes relationships across projects
- **THEN** system provides cross_project_links in context
- **AND** skill can generate cross-project suggestions

### Requirement: Skill execution timing optimization
The system SHALL optimize skill execution timing based on user patterns.

#### Scenario: Suggest optimal skill timing
- **WHEN** user creates a scheduled skill
- **AND** productivity patterns indicate better times for skill output type
- **THEN** system suggests optimal schedule time
- **AND** explains recommendation: "Based on when you typically review reports"

#### Scenario: Defer low-priority skill during peak hours
- **WHEN** low-priority skill is scheduled during user's peak productivity hours
- **AND** user has pattern indicating focus time preference
- **THEN** system suggests deferring to off-peak time
- **AND** user can override suggestion
