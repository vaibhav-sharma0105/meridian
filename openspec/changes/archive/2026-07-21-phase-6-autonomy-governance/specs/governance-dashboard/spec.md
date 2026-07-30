## ADDED Requirements

### Requirement: Agent activity summary
The system SHALL display a summary of recent agent activity.

#### Scenario: View activity summary
- **WHEN** user opens governance dashboard
- **THEN** system displays: total actions today, actions this week, actions by type breakdown

#### Scenario: Activity timeline
- **WHEN** user views activity section
- **THEN** system displays timeline chart of actions over selected period (day/week/month)

### Requirement: Actions by autonomy level
The system SHALL display breakdown of actions by autonomy mode that executed them.

#### Scenario: View autonomy breakdown
- **WHEN** user views autonomy metrics
- **THEN** system displays: count of auto-executed (low-risk), count of auto-executed (medium-risk), count requiring approval, count executed after approval

#### Scenario: Compare autonomy modes
- **WHEN** user selects time period
- **THEN** system shows how actions would differ under each autonomy mode
- **AND** shows potential time saved/manual effort for each mode

### Requirement: Approval rate metrics
The system SHALL track and display approval and rejection rates.

#### Scenario: View approval metrics
- **WHEN** user opens approval metrics section
- **THEN** system displays: approval rate %, rejection rate %, timeout/archive rate %

#### Scenario: Approval by category
- **WHEN** user expands approval metrics
- **THEN** system shows approval rate by: action_type, integration, skill, risk_level

#### Scenario: Rejection reasons
- **WHEN** user views rejection details
- **THEN** system shows breakdown of rejection reasons (if provided)

### Requirement: Risk distribution
The system SHALL display distribution of actions by risk level.

#### Scenario: View risk distribution
- **WHEN** user opens risk metrics section
- **THEN** system displays pie/bar chart: % low-risk, % medium-risk, % high-risk, % critical-risk

#### Scenario: Risk trend
- **WHEN** user views risk over time
- **THEN** system shows trend line of average risk level over selected period

### Requirement: Anomaly detection
The system SHALL detect and flag unusual patterns in agent activity.

#### Scenario: Detect activity spike
- **WHEN** agent actions exceed 2x the daily average
- **THEN** system flags anomaly in dashboard
- **AND** shows "Unusual activity: [X] actions today vs [Y] average"

#### Scenario: Detect high-risk increase
- **WHEN** proportion of high/critical-risk actions increases significantly
- **THEN** system flags anomaly with warning indicator
- **AND** shows "Risk level increase: [X]% high-risk vs [Y]% typical"

#### Scenario: Detect rejection spike
- **WHEN** rejection rate exceeds 2x typical rate
- **THEN** system flags anomaly
- **AND** shows "High rejection rate: Review agent suggestions"

### Requirement: Integration activity breakdown
The system SHALL display activity metrics per integration.

#### Scenario: View integration metrics
- **WHEN** user expands integration section
- **THEN** system displays for each integration: action count, approval rate, last activity

#### Scenario: Integration health indicator
- **WHEN** integration has high rejection rate or errors
- **THEN** system shows yellow/red health indicator

### Requirement: Skill activity breakdown
The system SHALL display activity metrics per skill.

#### Scenario: View skill metrics
- **WHEN** user expands skills section
- **THEN** system displays for each skill: run count, success rate, approval rate, average execution time

#### Scenario: Skill effectiveness
- **WHEN** user views skill detail
- **THEN** system shows: outputs generated, user edits to outputs, acceptance rate

### Requirement: Dashboard time range selection
The system SHALL allow users to select time range for all dashboard metrics.

#### Scenario: Select time range
- **WHEN** user selects "Last 7 days" from time range picker
- **THEN** all dashboard metrics update to show data for that period

#### Scenario: Compare periods
- **WHEN** user enables "Compare to previous period"
- **THEN** metrics show current vs previous period with delta indicators

### Requirement: Dashboard export
The system SHALL allow exporting dashboard data.

#### Scenario: Export dashboard data
- **WHEN** user clicks "Export" in dashboard
- **THEN** system generates CSV/JSON with all visible metrics
- **AND** file is downloadable
