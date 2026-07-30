## ADDED Requirements

### Requirement: Multi-factor assignee scoring
The system SHALL score potential assignees using multiple factors.

#### Scenario: Calculate assignee score
- **WHEN** user creates or edits a task
- **THEN** system calculates score for each team member based on:
  - Pattern score: historical assignment patterns for similar tasks
  - Workload score: inverse of current open task count
  - Expertise score: keyword match between member expertise and task
  - Recency score: recent activity on similar tasks
- **AND** system ranks members by combined weighted score

#### Scenario: Apply learned weights
- **WHEN** user has corrected assignee suggestions in the past
- **THEN** system adjusts factor weights based on corrections
- **AND** frequently overridden factors get lower weight

### Requirement: Assignee suggestions in UI
The system SHALL display intelligent assignee suggestions when creating/editing tasks.

#### Scenario: Show suggestions
- **WHEN** user focuses on assignee field
- **THEN** system displays top 3-5 suggested assignees
- **AND** each suggestion shows: name, confidence score, primary reason
- **AND** "Why?" tooltip explains the scoring factors

#### Scenario: Accept suggestion
- **WHEN** user clicks a suggested assignee
- **THEN** system sets assignee to selected member
- **AND** system records observation for pattern learning

#### Scenario: Override suggestion
- **WHEN** user types or selects different assignee than suggestions
- **THEN** system records override as negative signal
- **AND** system adjusts weights for future suggestions

### Requirement: Workload-aware suggestions
The system SHALL factor in current workload when suggesting assignees.

#### Scenario: Deprioritize overloaded members
- **WHEN** team member has workload_score > 0.8
- **THEN** system reduces their ranking in suggestions
- **AND** system shows workload warning if they're still selected

#### Scenario: Balance workload
- **WHEN** multiple members have similar pattern scores
- **THEN** system prioritizes member with lower workload
- **AND** system shows "better workload balance" as reason

### Requirement: Expertise matching
The system SHALL match task content to member expertise for suggestions.

#### Scenario: Extract task keywords
- **WHEN** calculating assignee scores
- **THEN** system extracts keywords from task title and description
- **AND** system matches against member expertise tags

#### Scenario: Learn expertise
- **WHEN** member completes tasks successfully
- **THEN** system updates expertise tags based on completed task keywords
- **AND** expertise confidence increases with repetition

### Requirement: Fallback for empty roster
The system SHALL provide graceful fallback when team roster is empty.

#### Scenario: No team members
- **WHEN** team_members table is empty
- **THEN** system falls back to smart_defaults pattern-based suggestions
- **AND** system shows hint to set up team roster

#### Scenario: Suggest from task history
- **WHEN** team roster is empty but tasks have assignee history
- **THEN** system suggests based on historical assignee strings
- **AND** system offers to add frequent assignees to roster
