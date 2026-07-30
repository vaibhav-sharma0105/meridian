## ADDED Requirements

### Requirement: Slack Socket Mode connection
The system SHALL connect to Slack using Socket Mode (WebSocket) to avoid requiring a public endpoint.

#### Scenario: Establish Socket Mode connection
- **WHEN** Slack integration is connected
- **THEN** system establishes WebSocket connection to Slack
- **THEN** system receives events without public URL

### Requirement: Bot mode messaging
The system SHALL support sending messages as a Slack bot with clear bot identity.

#### Scenario: Send bot message
- **WHEN** skill or user triggers Slack message in bot mode
- **THEN** message appears from "Meridian Bot" with bot badge

### Requirement: User token mode messaging
The system SHALL optionally support sending messages as the authenticated user (requires additional OAuth scope).

#### Scenario: Send user message
- **WHEN** user enables user token mode and triggers message
- **THEN** message appears from user's identity (no bot badge)

### Requirement: Channel-level autonomy
The system SHALL allow configuring autonomy mode per Slack channel independently.

#### Scenario: Configure channel autonomy
- **WHEN** user sets #general to "draft only" mode
- **THEN** messages to #general require manual sending
- **WHEN** user sets #bot-updates to "auto-send"
- **THEN** messages to #bot-updates send automatically

### Requirement: Draft-before-send for high-risk channels
The system SHALL default to draft mode for channels marked as high-risk (external, executives, first contact).

#### Scenario: High-risk channel detection
- **WHEN** user connects channel with external guests
- **THEN** system defaults channel to "draft only" mode

### Requirement: Time-delayed sending
The system SHALL support configurable send delay for supervised channels (default 10 minutes).

#### Scenario: Delayed send
- **WHEN** message is triggered for channel with delay
- **THEN** system queues message and shows countdown
- **THEN** user can cancel before delay expires
- **THEN** message sends after delay if not cancelled

### Requirement: Channel monitoring for action items
The system SHALL monitor configured channels for messages that may require follow-up actions.

#### Scenario: Detect action item in message
- **WHEN** message contains action indicators ("can you", "please", "@user")
- **THEN** system creates suggestion to add task

### Requirement: Message drafts UI
The system SHALL display pending Slack message drafts with edit, send, and cancel actions.

#### Scenario: View pending drafts
- **WHEN** user opens Slack integration panel
- **THEN** system displays queued drafts with destination channel and content

#### Scenario: Edit draft before sending
- **WHEN** user edits pending draft
- **THEN** system updates draft content
- **THEN** user can send or cancel

### Requirement: Slack workspace configuration
The system SHALL allow connecting to one Slack workspace with channel selection.

#### Scenario: Configure channels to monitor
- **WHEN** user selects channels to monitor
- **THEN** system subscribes to events for those channels

### Requirement: Message history sync
The system SHALL NOT sync full message history, only process real-time events to protect privacy.

#### Scenario: New message event
- **WHEN** message is posted to monitored channel
- **THEN** system processes for action items
- **THEN** system does NOT store message content long-term
