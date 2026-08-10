# Productivity Patterns Specification

## Overview

Learn when users are most productive for different task types and suggest optimal scheduling based on historical patterns.

## Data Model

### Database Schema

Productivity data stored in `user_profile.productivity_patterns` as JSON:

```json
{
  "task_completions_by_hour": {
    "focus_work": [0, 0, 0, 0, 0, 2, 5, 12, 18, 22, 15, 8, 5, 3, 8, 12, 10, 5, 2, 1, 0, 0, 0, 0],
    "meetings": [0, 0, 0, 0, 0, 0, 0, 2, 5, 15, 20, 12, 8, 10, 18, 15, 8, 3, 0, 0, 0, 0, 0, 0],
    "quick_tasks": [0, 0, 0, 0, 0, 1, 3, 8, 12, 15, 10, 5, 3, 5, 10, 12, 8, 4, 2, 1, 0, 0, 0, 0]
  },
  "peak_hours": {
    "focus_work": [9, 10, 11],
    "meetings": [10, 14, 15],
    "quick_tasks": [9, 14, 15]
  },
  "low_productivity_hours": [12, 13, 17, 18],
  "total_completions": 127,
  "last_aggregation": "2026-08-10T00:00:00Z",
  "tracking_enabled": true
}
```

### Pattern Observations Extension

```sql
ALTER TABLE pattern_observations ADD COLUMN completion_hour INTEGER;
ALTER TABLE pattern_observations ADD COLUMN completion_day_of_week INTEGER;  -- 0=Sun, 6=Sat
ALTER TABLE pattern_observations ADD COLUMN task_category TEXT;  -- 'focus_work' | 'meetings' | 'quick_tasks'
```

### Task Category Classification

```rust
pub fn classify_task_category(task: &Task) -> TaskCategory {
    // Based on estimated duration and task properties
    if task.estimated_minutes.unwrap_or(60) >= 60 {
        TaskCategory::FocusWork
    } else if task.title.to_lowercase().contains("meeting") 
           || task.source_type == Some("meeting") {
        TaskCategory::Meetings
    } else {
        TaskCategory::QuickTasks
    }
}
```

## Aggregation Algorithm

### Daemon Job

```rust
pub fn aggregate_productivity_patterns(conn: &Connection) -> Result<(), String> {
    let observations = get_unprocessed_completion_observations(conn)?;
    let mut patterns = get_current_patterns(conn)?;
    
    for obs in observations {
        let hour = obs.completion_hour;
        let category = &obs.task_category;
        
        patterns.task_completions_by_hour
            .get_mut(category)
            .map(|hours| hours[hour as usize] += 1);
        
        patterns.total_completions += 1;
    }
    
    // Recalculate peak hours
    for (category, hours) in &patterns.task_completions_by_hour {
        let mut indexed: Vec<_> = hours.iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.cmp(a.1));
        patterns.peak_hours.insert(
            category.clone(),
            indexed.iter().take(3).map(|(i, _)| *i as u8).collect()
        );
    }
    
    // Identify low-productivity hours (bottom 25%)
    let all_hours: Vec<u32> = patterns.task_completions_by_hour
        .values()
        .flat_map(|h| h.iter().cloned())
        .collect();
    // ... calculate threshold and mark low hours
    
    patterns.last_aggregation = Utc::now();
    save_patterns(conn, &patterns)?;
    mark_observations_processed(conn, &observations)?;
    
    Ok(())
}
```

### Cold Start Defaults

When `total_completions < 50`:

```rust
pub const DEFAULT_PEAK_HOURS: &[u8] = &[9, 10, 14, 15];  // 9-11am, 2-4pm
pub const DEFAULT_LOW_HOURS: &[u8] = &[12, 13, 17, 18];  // Lunch, end of day

pub fn get_effective_peak_hours(patterns: &ProductivityPatterns, category: &str) -> Vec<u8> {
    if patterns.total_completions < 50 {
        DEFAULT_PEAK_HOURS.to_vec()
    } else {
        patterns.peak_hours.get(category).cloned().unwrap_or_default()
    }
}
```

## Scheduling Suggestions

### Suggestion Generation

```rust
pub fn suggest_task_time(
    task: &Task,
    patterns: &ProductivityPatterns,
    calendar: &CalendarView,
) -> Option<TimeSuggestion> {
    let category = classify_task_category(task);
    let peak_hours = get_effective_peak_hours(patterns, &category.to_string());
    
    // Find available slots in peak hours
    for hour in peak_hours {
        if calendar.is_available(hour) {
            return Some(TimeSuggestion {
                suggested_hour: hour,
                reason: format!(
                    "You typically complete {} best around {}",
                    category.display_name(),
                    format_hour(hour)
                ),
                confidence: if patterns.total_completions >= 50 { 
                    Confidence::High 
                } else { 
                    Confidence::Default 
                },
            });
        }
    }
    
    None
}
```

### Meeting Batching Suggestion

```rust
pub fn suggest_meeting_batching(
    schedule: &DaySchedule,
    patterns: &ProductivityPatterns,
) -> Option<BatchingSuggestion> {
    let meeting_count = schedule.meetings.len();
    let scattered = schedule.has_scattered_meetings();  // > 2 hour gaps between meetings
    
    if meeting_count >= 3 && scattered {
        Some(BatchingSuggestion {
            message: "Consider batching your meetings to protect focus time",
            suggested_block: patterns.peak_hours.get("meetings").cloned(),
            freed_hours: calculate_freed_hours(schedule),
        })
    } else {
        None
    }
}
```

## Privacy Controls

### Settings

```rust
pub struct ProductivitySettings {
    pub tracking_enabled: bool,        // Default: true
    pub show_suggestions: bool,        // Default: true
    pub data_retention_days: u32,      // Default: 365
}
```

### Data Export

```rust
#[tauri::command]
pub async fn export_productivity_data(
    state: State<AppState>,
) -> Result<ProductivityExport, String> {
    // Returns aggregated patterns only, not raw timestamps
    let profile = get_user_profile(conn)?;
    Ok(ProductivityExport {
        peak_hours: profile.productivity_patterns.peak_hours.clone(),
        total_data_points: profile.productivity_patterns.total_completions,
        tracking_since: profile.created_at.clone(),
    })
}
```

### Data Deletion

```rust
#[tauri::command]
pub async fn clear_productivity_data(
    state: State<AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    
    // Clear pattern observations with completion data
    conn.execute(
        "UPDATE pattern_observations SET completion_hour = NULL, 
         completion_day_of_week = NULL, task_category = NULL",
        [],
    )?;
    
    // Reset user profile patterns
    conn.execute(
        "UPDATE user_profile SET productivity_patterns = '{}'",
        [],
    )?;
    
    Ok(())
}
```

## API Endpoints

### Commands

```rust
#[tauri::command]
pub async fn get_productivity_insights(
    state: State<AppState>,
) -> Result<ProductivityInsights, String>

#[tauri::command]
pub async fn get_time_suggestion(
    task_id: String,
    state: State<AppState>,
) -> Result<Option<TimeSuggestion>, String>

#[tauri::command]
pub async fn update_productivity_settings(
    settings: ProductivitySettings,
    state: State<AppState>,
) -> Result<(), String>

#[tauri::command]
pub async fn export_productivity_data(
    state: State<AppState>,
) -> Result<ProductivityExport, String>

#[tauri::command]
pub async fn clear_productivity_data(
    state: State<AppState>,
) -> Result<(), String>
```

## ADDED Requirements

### Requirement: Productivity Tracking

The system MUST learn when users are most productive for different task types.

#### Scenario: Track task completion times
Given a user completes a task
When the completion is recorded
Then the timestamp and task type are stored in `pattern_observations`
And the day-of-week and hour are extracted

#### Scenario: Minimum data threshold
Given a user has fewer than 50 task completions
When productivity patterns are requested
Then display "Still learning your patterns..."
And use research-based defaults (9-11am, 2-4pm for deep work)

#### Scenario: Sufficient data for patterns
Given a user has 50+ task completions with timestamps
When productivity analysis runs
Then identify peak productivity hours by task type
And identify low-productivity hours
And store aggregated patterns in `user_profile`

### Requirement: Optimal Scheduling

The system MUST suggest best times for task types.

#### Scenario: Suggest deep work time
Given the user's pattern shows highest focus work completion at 9-11am
When the user creates a focus-intensive task
Then suggest scheduling it for morning hours
And explain "You typically complete focus work best in the morning"

#### Scenario: Suggest meeting-free blocks
Given the user's pattern shows context-switching reduces afternoon productivity
When viewing schedule with afternoon meetings
Then suggest batching meetings
And highlight potential focus blocks

### Requirement: Privacy Controls

The system MUST allow users to control productivity tracking.

#### Scenario: Opt-out of tracking
Given the user opens settings
When they toggle "Productivity tracking" off
Then no new timestamps are stored for pattern analysis
And existing patterns are retained (or optionally cleared)
And the setting is clearly visible and easy to find

#### Scenario: View collected data
Given the user wants to see their productivity data
When they open Productivity Insights in settings
Then aggregated patterns are displayed (not raw timestamps)
And they can export or delete their data
