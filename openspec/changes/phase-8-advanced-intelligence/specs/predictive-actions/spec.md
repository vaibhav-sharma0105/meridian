## ADDED Requirements

### Requirement: Pre-meeting document fetch
The system SHALL pre-fetch relevant documents before scheduled meetings.

#### Scenario: Identify upcoming meeting needing documents
- **WHEN** predictive pre-fetch job runs (every 15 minutes)
- **AND** a meeting is scheduled within the next 30 minutes
- **THEN** system identifies documents relevant to meeting topic and attendees

#### Scenario: Pre-load document context
- **WHEN** relevant documents are identified for upcoming meeting
- **THEN** system pre-loads document embeddings into memory cache
- **AND** system marks documents as "pre-fetched for [Meeting Name]"
- **AND** AI chat queries during/after meeting use cached context

#### Scenario: Document relevance criteria
- **WHEN** determining document relevance for a meeting
- **THEN** system considers: meeting title keywords, linked task documents, attendee's recent document access, project documents

### Requirement: Automatic agenda drafting
The system SHALL draft meeting agendas from open tasks and context.

#### Scenario: Generate agenda for scheduled meeting
- **WHEN** a meeting is 24 hours away
- **AND** meeting has linked tasks OR attendees have open tasks
- **THEN** system drafts an agenda based on: open tasks by priority, recent decisions needing follow-up, blocked items requiring discussion
- **AND** system creates a draft document attached to meeting

#### Scenario: Agenda includes attendee context
- **WHEN** generating agenda
- **AND** team roster has data for attendees
- **THEN** agenda includes: each attendee's relevant open tasks, items where attendee is blocker, topics attendee raised in recent meetings

#### Scenario: User edits or dismisses agenda
- **WHEN** user views drafted agenda
- **THEN** user can edit inline, approve as-is, or dismiss
- **AND** edits inform future agenda generation patterns

### Requirement: Blocker prediction
The system SHALL predict potential blockers before they become critical.

#### Scenario: Predict deadline risk
- **WHEN** a task has due date within 3 days
- **AND** task has dependencies (subtasks, blocked_by links, integration links)
- **AND** any dependency is not complete
- **THEN** system generates prediction: "Task X at risk - dependency Y incomplete"
- **AND** prediction includes suggested actions (reassign, extend deadline, escalate)

#### Scenario: Predict workload bottleneck
- **WHEN** an assignee has more than 5 tasks due in next 7 days
- **AND** their recent completion rate is below 3 tasks/week
- **THEN** system generates prediction: "[Assignee] may be overloaded"
- **AND** system suggests tasks that could be reassigned

#### Scenario: Predict meeting follow-up gap
- **WHEN** a meeting occurred 48+ hours ago
- **AND** meeting had extracted tasks
- **AND** none of those tasks have been started
- **THEN** system generates prediction: "Meeting [Name] follow-ups stalled"

### Requirement: Workload-based reassignment suggestions
The system SHALL suggest task reassignments based on workload balance.

#### Scenario: Suggest reassignment for overloaded assignee
- **WHEN** predictive analysis detects workload imbalance
- **THEN** system suggests specific tasks to reassign
- **AND** suggestion includes: alternative assignees with capacity, task compatibility based on past assignments

#### Scenario: Respect expertise in reassignment
- **WHEN** generating reassignment suggestion
- **THEN** system only suggests assignees who have completed similar tasks
- **OR** system indicates "No expert available" if no match found

#### Scenario: User acts on reassignment suggestion
- **WHEN** user accepts reassignment suggestion
- **THEN** system updates task assignee
- **AND** system optionally notifies via Slack if integration connected

### Requirement: Predictive insights dashboard
The system SHALL display predictions and pre-fetch status in a dedicated view.

#### Scenario: View active predictions
- **WHEN** user opens Predictions panel (within Analytics or as tab)
- **THEN** system displays: deadline risks, workload alerts, stalled follow-ups
- **AND** each prediction shows confidence and suggested action

#### Scenario: View pre-fetch status
- **WHEN** user has meetings scheduled today
- **THEN** system shows which meetings have pre-fetched documents
- **AND** user can manually trigger pre-fetch for any meeting
