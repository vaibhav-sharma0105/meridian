# Message Center

Persistent storage and UI for skill results, integration digests, and pinned AI chat content.

## ADDED Requirements

### Requirement: Message Storage

The system MUST store messages with metadata for routing, retention, and AI context visibility.

#### Scenario: Skill result creates message
Given a skill executes successfully with output
When the skill run completes
Then a message is created in `message_center` table
And the message includes reference to output files (not copies)
And a notification is sent with "View full result" deep link

#### Scenario: AI chat with file attachment auto-pins
Given an AI chat response contains generated files
When the response is rendered
Then the message is auto-pinned to Message Center
And files are stored in `created_files/` directory
And message stores references to file paths

#### Scenario: Long AI response suggests pinning
Given an AI chat response exceeds 500 words
And the response is not auto-pinned
When the response is rendered
Then a one-click "Pin to Message Center" option is shown

### Requirement: Dual Retention Model

The system MUST separate AI context window from Message Center persistence.

#### Scenario: AI context respects window
Given `ai_context_days` is set to 30
And a message was created 45 days ago
When building AI prompt context
Then the 45-day-old message is NOT included
And messages from the last 30 days ARE included

#### Scenario: Old messages remain browsable
Given `message_retention` is set to "forever"
And a message was created 6 months ago
When user opens Message Center
Then the 6-month-old message is visible and searchable

#### Scenario: Message soft-delete
Given a user deletes a message
When the delete action completes
Then `deleted_at` timestamp is set
And the message is hidden from UI
And the message is recoverable for 30 days
And after 30 days the message is hard-deleted

#### Scenario: File cleanup on hard-delete
Given a message references files in `created_files/`
And no other messages reference those files
When the message is hard-deleted
Then the referenced files are also deleted

### Requirement: Message Center UI

The system MUST provide a dedicated sidebar view for browsing messages.

#### Scenario: View Message Center
Given the user clicks the Message Center icon
When the sidebar opens
Then messages are displayed in reverse chronological order
And each message shows type, timestamp, and preview
And search/filter controls are available

#### Scenario: Filter by message type
Given the Message Center is open
When user filters by "Skill Results"
Then only skill result messages are displayed
And digest and pinned chat messages are hidden

#### Scenario: Storage usage indicator
Given the user opens Message Center settings
When storage is calculated
Then total storage used is displayed
And a warning appears if usage exceeds 500MB
And archival is suggested if usage exceeds 1GB

### Requirement: Message Types and Routing

The system MUST route content to appropriate destinations based on type.

#### Scenario: Brief status to notifications only
Given a brief status update (< 100 chars, no files)
When the update is generated
Then it appears in notifications only
And no Message Center entry is created

#### Scenario: Digest to Message Center
Given a daily or weekly digest is generated
When the digest is complete
Then it is stored in Message Center
And a notification links to the full digest
