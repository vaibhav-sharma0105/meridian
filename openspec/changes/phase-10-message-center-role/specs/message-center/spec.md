# Message Center Specification

## Overview

Persistent storage and UI for skill results, integration digests, and pinned AI chat content. Separates AI context window (bounded) from user-browsable history (indefinite).

## Data Model

### Database Schema

```sql
CREATE TABLE message_center (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    message_type TEXT NOT NULL,  -- 'skill_result' | 'digest' | 'pinned_chat' | 'integration_sync'
    title TEXT NOT NULL,
    content TEXT,                -- markdown content or summary
    source_id TEXT,              -- skill_run_id, chat_message_id, etc.
    source_type TEXT,            -- 'skill' | 'ai_chat' | 'integration'
    auto_pinned INTEGER DEFAULT 0,
    pinned_reason TEXT,          -- 'file_attachment' | 'long_response' | 'important_skill'
    file_refs TEXT,              -- JSON array of file paths in created_files/
    ai_visible_until TEXT,       -- ISO timestamp; NULL = always visible to AI
    deleted_at TEXT,             -- soft-delete timestamp
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_message_center_project ON message_center(project_id, created_at DESC);
CREATE INDEX idx_message_center_type ON message_center(message_type, created_at DESC);
CREATE INDEX idx_message_center_ai_visible ON message_center(ai_visible_until) WHERE ai_visible_until IS NOT NULL;
```

### Message Types

| Type | Source | Auto-Pin | AI Context |
|------|--------|----------|------------|
| `skill_result` | Skill execution | If `important: true` in config | Yes |
| `digest` | Daily/weekly job | Always | Yes |
| `pinned_chat` | AI chat response | If files or >500 words | Yes |
| `integration_sync` | GitHub/Jira/Slack sync | If new items found | No |

## Dual Retention Model

### AI Context Window

Messages included in AI prompts based on `ai_context_days` setting:

```rust
pub fn get_ai_context_messages(
    conn: &Connection,
    project_id: &str,
    ai_context_days: i64,
) -> Result<Vec<Message>, String> {
    let cutoff = Utc::now() - Duration::days(ai_context_days);
    // Query where created_at > cutoff AND deleted_at IS NULL
    // AND (ai_visible_until IS NULL OR ai_visible_until > now)
}
```

### Message Center Persistence

User-configurable via `app_settings`:

| Setting | Options | Default |
|---------|---------|---------|
| `ai_context_days` | 7 / 30 / 90 | 30 |
| `message_retention` | 90d / 1y / forever | forever |
| `archive_old_files` | true / false | false |

### Cleanup Daemon Job

```rust
pub fn cleanup_expired_messages(conn: &Connection) -> Result<CleanupStats, String> {
    // 1. Hard-delete messages where deleted_at < now - 30 days
    // 2. If message_retention != "forever":
    //    Soft-delete messages where created_at < now - retention
    // 3. For hard-deleted messages, cleanup orphaned files
}
```

## File Reference Management

### Storage Structure

```
~/.meridian/created_files/
├── 2026-08-10/
│   ├── report_143022.pdf
│   └── analysis_143156.csv
└── 2026-08-11/
    └── summary_091530.md
```

### Reference Tracking

- Messages store `file_refs` as JSON array of relative paths
- Files are NOT duplicated — Message Center stores references only
- Orphan detection: file deleted only when no `message_center.file_refs` contains it

## Routing Rules

### Decision Logic

```rust
pub fn route_content(content: &Content) -> RoutingDecision {
    match content {
        Content::SkillResult { has_output, important, .. } => {
            if has_output || important {
                RoutingDecision::MessageCenterWithNotification
            } else {
                RoutingDecision::NotificationOnly
            }
        }
        Content::AiChat { has_files, word_count, .. } => {
            if has_files {
                RoutingDecision::AutoPin { reason: "file_attachment" }
            } else if word_count > 500 {
                RoutingDecision::SuggestPin
            } else {
                RoutingDecision::None
            }
        }
        Content::Digest { .. } => RoutingDecision::MessageCenterWithNotification,
        Content::IntegrationSync { new_items, .. } => {
            if new_items > 0 {
                RoutingDecision::MessageCenterWithNotification
            } else {
                RoutingDecision::NotificationOnly
            }
        }
        Content::BriefStatus { .. } => RoutingDecision::NotificationOnly,
        Content::ApprovalRequest { .. } => RoutingDecision::NotificationOnly,
    }
}
```

## API Endpoints

### Commands

```rust
#[tauri::command]
pub async fn get_messages(
    project_id: String,
    filters: MessageFilters,
    pagination: Pagination,
    state: State<AppState>,
) -> Result<PaginatedMessages, String>

#[tauri::command]
pub async fn pin_message(
    source_type: String,  // "ai_chat"
    source_id: String,    // chat message ID
    state: State<AppState>,
) -> Result<Message, String>

#[tauri::command]
pub async fn delete_message(
    message_id: String,
    state: State<AppState>,
) -> Result<(), String>  // Soft-delete

#[tauri::command]
pub async fn get_storage_stats(
    state: State<AppState>,
) -> Result<StorageStats, String>
```

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
