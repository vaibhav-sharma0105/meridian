## ADDED Requirements

### Requirement: Productivity pattern observation
The system SHALL observe user productivity patterns by time of day.

#### Scenario: Record task completion timing
- **WHEN** user completes a task
- **THEN** system records: completion time (hour of day), day of week, task type/category, time spent (if tracked)

#### Scenario: Record focus session patterns
- **WHEN** user works on tasks without switching for 30+ minutes
- **THEN** system records focus session: start time, duration, task types worked on

#### Scenario: Record meeting scheduling patterns
- **WHEN** user creates or imports meetings
- **THEN** system records: preferred meeting times, meeting-free periods

### Requirement: Productivity pattern analysis
The system SHALL analyze patterns to determine optimal work times.

#### Scenario: Identify peak productivity hours
- **WHEN** user has 30+ completed tasks with timing data
- **THEN** system computes: hours with highest completion rate, hours with longest focus sessions, hours with fastest task completion

#### Scenario: Identify task type preferences by time
- **WHEN** analyzing productivity patterns
- **THEN** system determines: times when user prefers high-priority tasks, times when user handles routine tasks, times user avoids meetings

#### Scenario: Insufficient data handling
- **WHEN** user has fewer than 30 completed tasks
- **THEN** system displays "Learning your patterns..." with progress indicator
- **AND** system does not make time-based suggestions

### Requirement: Time-based suggestions
The system SHALL suggest optimal timing for tasks and focus work.

#### Scenario: Suggest focus time for complex task
- **WHEN** user creates a high-priority or complex task
- **AND** user's productivity patterns show peak focus hours
- **THEN** system suggests: "Best tackled during [peak hours] based on your patterns"

#### Scenario: Warn about suboptimal scheduling
- **WHEN** user schedules a meeting during their peak focus time
- **AND** meeting is not marked as high priority
- **THEN** system shows non-blocking warning: "This conflicts with your usual focus time"

#### Scenario: Suggest task reordering
- **WHEN** user views task list in morning
- **AND** patterns show afternoon is better for current task types
- **THEN** system suggests: "Based on your patterns, consider [other tasks] first"

### Requirement: Skill timing adaptation
The system SHALL adjust skill execution timing based on productivity patterns.

#### Scenario: Optimize skill schedule for productivity
- **WHEN** user creates a scheduled skill
- **THEN** system suggests optimal time based on: skill output type (summary → morning, cleanup → evening), user's alert/engaged hours

#### Scenario: Avoid low-productivity times for important skills
- **WHEN** a skill generates high-priority suggestions or drafts
- **THEN** system recommends scheduling when user is typically most responsive
- **AND** system explains recommendation based on patterns

### Requirement: Productivity insights display
The system SHALL display productivity insights in settings and analytics.

#### Scenario: View personal productivity profile
- **WHEN** user opens Productivity section in Analytics
- **THEN** system displays: peak hours heatmap, task completion by hour chart, focus session distribution

#### Scenario: Compare to baseline
- **WHEN** displaying productivity insights
- **THEN** system shows: this week vs average, trends over time, days with anomalies (unusually high/low productivity)

#### Scenario: Export productivity data
- **WHEN** user clicks export in Productivity section
- **THEN** system exports: hourly completion rates, focus session logs, pattern summary
