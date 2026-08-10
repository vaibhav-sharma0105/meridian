# Phase 10: Message Center & Role - Design Document

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend                                 │
├─────────────────────────────────────────────────────────────────┤
│  MessageCenterView  │  RoleConfirmation   │  ProductivityInsights│
│  MessageCard        │  RoleIndicator      │  TimeSuggestion      │
│  StorageUsageBar    │  RoleDriftAlert     │  PrivacySettings     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri Commands                              │
├─────────────────────────────────────────────────────────────────┤
│  get_messages       │  get_user_profile   │  get_time_suggestion │
│  pin_message        │  confirm_role       │  get_productivity    │
│  delete_message     │  change_role        │  clear_productivity  │
│  get_storage_stats  │  get_inference_status│  export_productivity│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Backend Modules                             │
├──────────────────┬──────────────────┬───────────────────────────┤
│  messages/       │  role/           │  productivity/            │
│  - routing.rs    │  - inference.rs  │  - patterns.rs            │
│  - retention.rs  │  - scoring.rs    │  - suggestions.rs         │
│  - storage.rs    │  - drift.rs      │  - aggregation.rs         │
└──────────────────┴──────────────────┴───────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Storage                                   │
├─────────────────────────────────────────────────────────────────┤
│  SQLite: message_center       - Messages with retention metadata │
│  SQLite: user_profile         - Role scores, productivity data   │
│  SQLite: pattern_observations - Extended with role/time signals  │
│  ~/.meridian/created_files/   - Referenced files (not copied)    │
└─────────────────────────────────────────────────────────────────┘
```

## Database Migration (v021)

```sql
-- Message Center table
CREATE TABLE IF NOT EXISTS message_center (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    message_type TEXT NOT NULL,  -- 'skill_result' | 'digest' | 'pinned_chat' | 'integration_sync'
    title TEXT NOT NULL,
    content TEXT,
    source_id TEXT,
    source_type TEXT,
    auto_pinned INTEGER DEFAULT 0,
    pinned_reason TEXT,
    file_refs TEXT,              -- JSON array of file paths
    ai_visible_until TEXT,       -- For AI context window cutoff
    deleted_at TEXT,             -- Soft-delete
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_message_center_project ON message_center(project_id, created_at DESC);
CREATE INDEX idx_message_center_type ON message_center(message_type, created_at DESC);
CREATE INDEX idx_message_center_deleted ON message_center(deleted_at) WHERE deleted_at IS NOT NULL;

-- User Profile table
CREATE TABLE IF NOT EXISTS user_profile (
    id TEXT PRIMARY KEY DEFAULT 'default',
    inferred_role TEXT,
    secondary_role TEXT,
    custom_role_description TEXT,
    role_confirmed INTEGER DEFAULT 0,
    role_confirmed_at TEXT,
    role_scores TEXT,            -- JSON: {"tech_lead": 0.4, ...}
    last_inference_at TEXT,
    productivity_patterns TEXT,  -- JSON: peak hours, completions by hour
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Extend pattern_observations
ALTER TABLE pattern_observations ADD COLUMN role_signal TEXT;
ALTER TABLE pattern_observations ADD COLUMN completion_hour INTEGER;
ALTER TABLE pattern_observations ADD COLUMN completion_day_of_week INTEGER;
ALTER TABLE pattern_observations ADD COLUMN task_category TEXT;

-- Indexes for role/productivity queries
CREATE INDEX idx_pattern_obs_role ON pattern_observations(role_signal) 
    WHERE role_signal IS NOT NULL;
CREATE INDEX idx_pattern_obs_productivity ON pattern_observations(completion_hour, task_category) 
    WHERE completion_hour IS NOT NULL;
```

## Module Structure

### src-tauri/src/messages/mod.rs

```rust
mod routing;
mod retention;
mod storage;
mod repository;

pub use routing::{route_content, RoutingDecision};
pub use retention::{RetentionPolicy, cleanup_expired_messages};
pub use storage::{StorageStats, calculate_storage_usage};
```

### src-tauri/src/messages/routing.rs

```rust
pub enum RoutingDecision {
    NotificationOnly,
    MessageCenterWithNotification,
    AutoPin { reason: String },
    SuggestPin,
    None,
}

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
                RoutingDecision::AutoPin { reason: "file_attachment".into() }
            } else if word_count > 500 {
                RoutingDecision::SuggestPin
            } else {
                RoutingDecision::None
            }
        }
        // ... other content types
    }
}
```

### src-tauri/src/role/inference.rs

```rust
pub struct RoleScores {
    pub tech_lead: f32,
    pub ic: f32,
    pub pm: f32,
    pub manager: f32,
}

pub struct RoleClassification {
    pub primary: String,
    pub primary_confidence: f32,
    pub secondary: Option<String>,
    pub secondary_confidence: f32,
}

pub fn compute_role_scores(observations: &[RoleObservation]) -> RoleScores;
pub fn classify_role(scores: &RoleScores) -> RoleClassification;
pub fn detect_role_drift(current: &RoleScores, historical: &RoleScores) -> Option<RoleDriftAlert>;
```

### src-tauri/src/productivity/patterns.rs

```rust
pub struct ProductivityPatterns {
    pub task_completions_by_hour: HashMap<String, [u32; 24]>,
    pub peak_hours: HashMap<String, Vec<u8>>,
    pub low_productivity_hours: Vec<u8>,
    pub total_completions: u32,
    pub tracking_enabled: bool,
}

pub fn aggregate_patterns(conn: &Connection) -> Result<ProductivityPatterns, String>;
pub fn get_effective_peak_hours(patterns: &ProductivityPatterns, category: &str) -> Vec<u8>;
pub fn suggest_task_time(task: &Task, patterns: &ProductivityPatterns) -> Option<TimeSuggestion>;
```

## Frontend Components

### MessageCenterView

Location: `src/components/messages/MessageCenterView.tsx`

Features:
- Sidebar view accessible from main nav
- Message list with type icons (skill, digest, chat, integration)
- Search bar with type filter dropdown
- "New messages" indicator badge
- Pagination or infinite scroll

### RoleConfirmation

Location: `src/components/role/RoleConfirmation.tsx`

Modal wizard:
1. Show inferred role with confidence bar
2. Option to confirm, change, or select "Other"
3. If "Other": free-text description field
4. Immediate reorder of My Activity on confirmation

### RoleIndicator

Location: `src/components/role/RoleIndicator.tsx`

Features:
- Small badge in My Activity header
- Shows primary role (and secondary if present)
- Hover tooltip: "Showing [Role] view — focusing on [priorities]"
- [Change] link opens role selection modal

### ProductivityInsights

Location: `src/components/settings/ProductivityInsights.tsx`

Features:
- Hour-by-hour heatmap of task completions
- Peak hours highlight
- "Still learning..." message if < 50 completions
- Privacy toggle
- Export/Clear data buttons

### TimeSuggestion

Location: `src/components/tasks/TimeSuggestion.tsx`

Inline component on task creation:
- "Best time: 9-10 AM" badge
- Hover shows reasoning
- Click to apply to due date

## Daemon Jobs

### Message Cleanup

```rust
// Runs daily at 3 AM
pub fn schedule_message_cleanup_job(conn: &Connection) {
    create_job(conn, DaemonJob {
        job_type: "cleanup_messages",
        schedule: "0 3 * * *",  // 3 AM daily
        ..
    });
}

pub fn process_message_cleanup_job(conn: &Connection) -> JobResult {
    let stats = cleanup_expired_messages(conn)?;
    let orphan_files = cleanup_orphaned_files(conn)?;
    JobResult::Success { 
        message: format!("Cleaned {} messages, {} files", stats.deleted, orphan_files) 
    }
}
```

### Role Inference

```rust
// Runs weekly
pub fn schedule_role_inference_job(conn: &Connection) {
    create_job(conn, DaemonJob {
        job_type: "infer_role",
        schedule: "0 0 * * 0",  // Sunday midnight
        ..
    });
}

pub fn process_role_inference_job(conn: &Connection) -> JobResult {
    let observations = get_recent_role_observations(conn, days: 30)?;
    let scores = compute_role_scores(&observations);
    update_user_profile_scores(conn, &scores)?;
    
    if let Some(drift) = detect_role_drift(&scores, &get_historical_scores(conn)?) {
        create_notification(conn, Notification {
            title: "Your role may have changed",
            body: format!("Based on recent activity, you might be a {}", drift.suggested_role),
            ..
        })?;
    }
    
    JobResult::Success { message: "Role inference complete" }
}
```

### Productivity Aggregation

```rust
// Runs daily
pub fn schedule_productivity_aggregation_job(conn: &Connection) {
    create_job(conn, DaemonJob {
        job_type: "aggregate_productivity",
        schedule: "0 1 * * *",  // 1 AM daily
        ..
    });
}

pub fn process_productivity_aggregation_job(conn: &Connection) -> JobResult {
    let profile = get_user_profile(conn)?;
    if !profile.productivity_tracking_enabled {
        return JobResult::Skipped { reason: "Tracking disabled" };
    }
    
    let patterns = aggregate_patterns(conn)?;
    update_user_profile_patterns(conn, &patterns)?;
    
    JobResult::Success { message: format!("Aggregated {} completions", patterns.total_completions) }
}
```

## MCP Tools

Add to `meridian-mcp/src/handlers.rs`:

```rust
// Create a report (stored in Message Center)
Tool {
    name: "create_report",
    description: "Create a report that will be stored in Message Center",
    parameters: json!({
        "title": "string",
        "content": "string (markdown)",
        "project_id": "string (optional)"
    }),
}

// Get reports from Message Center
Tool {
    name: "get_reports",
    description: "Get reports from Message Center",
    parameters: json!({
        "project_id": "string (optional)",
        "limit": "number (default 10)"
    }),
}

// Draft a message (creates pinned chat)
Tool {
    name: "draft_message",
    description: "Draft a message to be stored in Message Center for later review",
    parameters: json!({
        "title": "string",
        "content": "string",
        "recipient_hint": "string (optional, for context)"
    }),
}
```

## Privacy Considerations

1. **Productivity data is local-only**: No productivity patterns leave the device
2. **Aggregated not raw**: Only hourly aggregates stored, not individual timestamps
3. **Opt-out available**: Single toggle disables all productivity tracking
4. **Clear data**: Users can delete all productivity data with one action
5. **Role inference transparent**: Users always see why a role was inferred
6. **No external signals**: Role inference only uses local task/meeting data, not integration data

## Data Flow

### Message Creation

```
Skill completes / AI responds / Digest generated
                    │
                    ▼
            route_content()
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
NotificationOnly  AutoPin      SuggestPin
    │               │               │
    ▼               ▼               ▼
create_notif   create_message   show_pin_btn
                + create_notif      │
                                    ▼
                              user clicks?
                                    │
                                    ▼
                              create_message
```

### Role Inference

```
Task created/completed, Meeting attended, PR reviewed
                    │
                    ▼
            record_observation()
            (role_signal column)
                    │
                    ▼
            Weekly daemon job
                    │
                    ▼
            compute_role_scores()
                    │
                    ▼
            update_user_profile()
                    │
    ┌───────────────┴───────────────┐
    ▼                               ▼
First time?                    Drift detected?
    │                               │
    ▼                               ▼
RoleConfirmation             RoleDriftAlert
    prompt                      notification
```

## Testing Strategy

### Unit Tests

- `messages/routing.rs`: Test all content type → routing decision mappings
- `role/inference.rs`: Test scoring algorithm with known observation sets
- `productivity/patterns.rs`: Test aggregation with mock data

### Integration Tests

- Message Center CRUD operations
- Retention policy enforcement
- File reference cleanup

### E2E Tests

- Pin AI chat response → appears in Message Center
- Role confirmation flow
- Productivity insights render with mock data
