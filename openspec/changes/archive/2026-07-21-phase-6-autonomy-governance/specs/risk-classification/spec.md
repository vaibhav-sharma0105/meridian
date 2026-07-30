## ADDED Requirements

### Requirement: Action risk levels
The system SHALL classify every agent action into one of four risk levels: low, medium, high, critical.

#### Scenario: Low risk action
- **WHEN** agent performs a read-only action (view tasks, search documents)
- **THEN** system classifies action as "low" risk

#### Scenario: Medium risk action
- **WHEN** agent performs internal write action (create task, update task status)
- **THEN** system classifies action as "medium" risk

#### Scenario: High risk action
- **WHEN** agent performs external write action (create GitHub issue, post Slack message)
- **THEN** system classifies action as "high" risk

#### Scenario: Critical risk action
- **WHEN** agent performs delete action OR sends to external executives OR sends to first-contact recipients
- **THEN** system classifies action as "critical" risk

### Requirement: Action type classification
The system SHALL classify actions by type: read, create, update, delete, external_send.

#### Scenario: Classify read action
- **WHEN** agent calls get_tasks, search_documents, or list_meetings
- **THEN** system classifies action_type as "read"

#### Scenario: Classify create action
- **WHEN** agent calls create_task, create_meeting_note, or create_suggestion
- **THEN** system classifies action_type as "create"

#### Scenario: Classify external_send action
- **WHEN** agent sends message via Slack, creates GitHub issue, or posts Jira comment
- **THEN** system classifies action_type as "external_send"

### Requirement: Destination risk scoring
The system SHALL score destination risk: internal < team < external < executive.

#### Scenario: Internal destination
- **WHEN** action target is local data only (task, meeting, document)
- **THEN** destination_risk is "internal" (score: 1)

#### Scenario: Team destination
- **WHEN** action target is team channel or known team members
- **THEN** destination_risk is "team" (score: 2)

#### Scenario: External destination
- **WHEN** action target is external system or unknown recipients
- **THEN** destination_risk is "external" (score: 3)

#### Scenario: Executive destination
- **WHEN** action target includes executive-tagged contacts or external decision-makers
- **THEN** destination_risk is "executive" (score: 4)

### Requirement: Content risk scoring
The system SHALL score content risk: normal < sensitive < pii < financial.

#### Scenario: Normal content
- **WHEN** content contains no flagged patterns
- **THEN** content_risk is "normal" (score: 1)

#### Scenario: Sensitive content
- **WHEN** content contains keywords flagged as sensitive (confidential, internal-only)
- **THEN** content_risk is "sensitive" (score: 2)

#### Scenario: PII content
- **WHEN** content contains personally identifiable information patterns
- **THEN** content_risk is "pii" (score: 3)

#### Scenario: Financial content
- **WHEN** content contains financial data patterns (account numbers, amounts, contracts)
- **THEN** content_risk is "financial" (score: 4)

### Requirement: Combined risk calculation
The system SHALL combine action_type, destination_risk, and content_risk into final risk_level.

#### Scenario: Calculate combined risk
- **WHEN** action has action_type="external_send", destination="team", content="normal"
- **THEN** system calculates risk_score = action_weight + destination_score + content_score
- **AND** maps score to risk_level (low: 1-3, medium: 4-6, high: 7-9, critical: 10+)

#### Scenario: Critical override
- **WHEN** any individual risk factor is at maximum (delete action, executive destination, financial content)
- **THEN** final risk_level is "critical" regardless of combined score

### Requirement: Risk rules configuration
The system SHALL allow users to customize risk classification rules.

#### Scenario: Mark channel as high-risk
- **WHEN** user marks Slack channel #executive-team as "always high-risk"
- **THEN** any message to that channel is classified as high-risk minimum

#### Scenario: Mark action type as low-risk
- **WHEN** user marks "create_task" as "always low-risk"
- **THEN** create_task actions are classified as low-risk regardless of other factors

### Requirement: Learned risk adjustments
The system SHALL learn from user corrections to risk classifications.

#### Scenario: User corrects risk level
- **WHEN** user overrides an action's risk level during approval
- **THEN** system records the correction pattern
- **AND** applies learned adjustment to similar future actions

#### Scenario: Apply learned risk
- **WHEN** similar action pattern has 3+ user corrections
- **THEN** system adjusts base risk calculation by learned factor
