# Role Inference Specification

## Overview

Pattern-based detection of user role (Tech Lead, IC, PM, Manager) to personalize information surfacing, suggestions, and dashboard views.

## Data Model

### Database Schema

```sql
CREATE TABLE user_profile (
    id TEXT PRIMARY KEY DEFAULT 'default',  -- single-user app
    inferred_role TEXT,           -- 'tech_lead' | 'ic' | 'pm' | 'manager' | 'other'
    secondary_role TEXT,          -- if confidence > 0.3
    custom_role_description TEXT, -- free text for "Other"
    role_confirmed INTEGER DEFAULT 0,
    role_confirmed_at TEXT,
    role_scores TEXT,             -- JSON: {"tech_lead": 0.4, "ic": 0.3, ...}
    last_inference_at TEXT,
    productivity_patterns TEXT,   -- JSON: see productivity-patterns spec
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Extend pattern_observations for role signals
ALTER TABLE pattern_observations ADD COLUMN role_signal TEXT;
-- 'creates_tasks_for_others' | 'receives_assignments' | 'runs_meetings' | etc.
```

### Role Types

| Role | Description | Primary Signals |
|------|-------------|-----------------|
| `tech_lead` | Technical leadership, code reviews, team support | PR reviews > PR authors, creates tasks for others |
| `ic` | Individual contributor, execution-focused | Receives tasks, authors PRs, works on bugs |
| `pm` | Product manager, roadmap-focused | Views epics/roadmap items, creates tasks for others |
| `manager` | People manager, team-focused | Runs meetings, assigns follow-ups, team oversight |
| `other` | Custom role | User-defined |

## Inference Algorithm

### Signal Weights Matrix

```rust
pub const ROLE_WEIGHTS: [[f32; 4]; 8] = [
    //                      TechLead  IC    PM    Manager
    /* creates_for_others */  [0.3,  0.0,  0.4,   0.3],
    /* receives_tasks     */  [0.1,  0.5,  0.2,   0.1],
    /* runs_meetings      */  [0.3,  0.0,  0.3,   0.4],
    /* attends_meetings   */  [0.1,  0.4,  0.1,   0.1],
    /* reviews_prs        */  [0.4,  0.2,  0.0,   0.1],
    /* authors_prs        */  [0.2,  0.5,  0.0,   0.0],
    /* views_roadmap      */  [0.2,  0.0,  0.5,   0.2],
    /* works_on_bugs      */  [0.2,  0.5,  0.1,   0.0],
];
```

### Scoring Function

```rust
pub fn compute_role_scores(observations: &[RoleObservation]) -> RoleScores {
    let mut scores = RoleScores::default();
    let mut signal_counts: HashMap<String, i32> = HashMap::new();
    
    // Count observations per signal type
    for obs in observations {
        *signal_counts.entry(obs.signal.clone()).or_default() += 1;
    }
    
    // Normalize and apply weights
    let total = signal_counts.values().sum::<i32>() as f32;
    for (signal, count) in &signal_counts {
        let normalized = (*count as f32) / total;
        let weights = get_weights_for_signal(signal);
        scores.tech_lead += normalized * weights[0];
        scores.ic += normalized * weights[1];
        scores.pm += normalized * weights[2];
        scores.manager += normalized * weights[3];
    }
    
    // Normalize scores to sum to 1.0
    scores.normalize();
    scores
}
```

### Multi-Label Classification

```rust
pub fn classify_role(scores: &RoleScores) -> RoleClassification {
    let primary = scores.highest();
    let secondary = scores.second_highest();
    
    RoleClassification {
        primary: primary.role,
        primary_confidence: primary.score,
        secondary: if secondary.score > 0.3 { Some(secondary.role) } else { None },
        secondary_confidence: secondary.score,
    }
}
```

## Activity Thresholds

### Minimum for Inference

| Metric | Threshold | Rationale |
|--------|-----------|-----------|
| Task interactions | 20+ | Need enough task data |
| Meetings | 5+ | Need meeting behavior sample |
| Time period | ~1 week | Allow patterns to emerge |

### Confidence Display

```rust
pub fn get_inference_status(profile: &UserProfile) -> InferenceStatus {
    let task_count = count_task_interactions();
    let meeting_count = count_meeting_interactions();
    
    if task_count < 20 || meeting_count < 5 {
        InferenceStatus::Learning {
            message: "Getting to know your role...",
            progress: (task_count + meeting_count * 4) as f32 / 40.0,
        }
    } else if !profile.role_confirmed {
        InferenceStatus::PendingConfirmation {
            inferred: profile.inferred_role.clone(),
            confidence: profile.role_scores.highest().score,
        }
    } else {
        InferenceStatus::Confirmed {
            role: profile.inferred_role.clone(),
            secondary: profile.secondary_role.clone(),
        }
    }
}
```

## Role Drift Detection

### Detection Logic

```rust
pub fn detect_role_drift(
    current_scores: &RoleScores,
    historical_scores: &RoleScores,
    window_days: i64,
) -> Option<RoleDriftAlert> {
    let score_change = current_scores.difference(historical_scores);
    
    // Significant drift if any role score changed by > 0.2 over 2 weeks
    if score_change.max_delta() > 0.2 && window_days >= 14 {
        Some(RoleDriftAlert {
            previous_role: historical_scores.highest().role,
            suggested_role: current_scores.highest().role,
            confidence: current_scores.highest().score,
        })
    } else {
        None
    }
}
```

## Personalization Effects

### My Activity Dashboard Ordering

```rust
pub fn order_activity_items(items: Vec<ActivityItem>, role: &str) -> Vec<ActivityItem> {
    match role {
        "manager" => {
            // Team items first, then personal
            items.sort_by_key(|i| (!i.is_team_item, i.created_at))
        }
        "ic" => {
            // Personal assignments first, then others
            items.sort_by_key(|i| (!i.is_assigned_to_me, i.created_at))
        }
        "tech_lead" => {
            // PR reviews and team blockers first
            items.sort_by_key(|i| (
                !i.is_review_request,
                !i.is_blocker,
                i.created_at
            ))
        }
        _ => items.sort_by_key(|i| i.created_at),
    }
    items
}
```

### Suggestion Weighting

Suggestions from `suggestions` table are weighted by role relevance:

| Suggestion Type | Tech Lead | IC | PM | Manager |
|-----------------|-----------|----|----|---------|
| PR review needed | 1.5x | 1.0x | 0.5x | 0.8x |
| Task overdue | 1.0x | 1.5x | 1.2x | 1.0x |
| Meeting follow-up | 1.2x | 1.0x | 1.3x | 1.5x |
| Team velocity drop | 1.3x | 0.5x | 1.5x | 1.5x |

## API Endpoints

### Commands

```rust
#[tauri::command]
pub async fn get_user_profile(
    state: State<AppState>,
) -> Result<UserProfile, String>

#[tauri::command]
pub async fn confirm_role(
    role: String,
    custom_description: Option<String>,
    state: State<AppState>,
) -> Result<UserProfile, String>

#[tauri::command]
pub async fn change_role(
    role: String,
    custom_description: Option<String>,
    state: State<AppState>,
) -> Result<UserProfile, String>

#[tauri::command]
pub async fn dismiss_role_drift_alert(
    state: State<AppState>,
) -> Result<(), String>

#[tauri::command]
pub async fn get_role_inference_status(
    state: State<AppState>,
) -> Result<InferenceStatus, String>
```

## ADDED Requirements

### Requirement: Role Detection

The system MUST infer user role from behavioral patterns.

#### Scenario: Detect Tech Lead from PR reviews
Given a user has reviewed 15+ PRs in the last 30 days
And the user has authored 5 PRs in the same period
When role inference runs
Then "Tech Lead" has a score >= 0.4
And the user's PR review activity contributes 0.4 weight to Tech Lead

#### Scenario: Detect IC from task assignments
Given a user receives task assignments frequently
And the user rarely creates tasks for others
And the user authors more PRs than reviews
When role inference runs
Then "IC" has the highest score

#### Scenario: Multi-label classification
Given a user exhibits both IC and Tech Lead behaviors
And IC score is 0.5 and Tech Lead score is 0.35
When displaying role
Then primary role shows as "IC"
And secondary role shows as "Tech Lead" (since > 0.3 threshold)

#### Scenario: Minimum activity threshold
Given a user has fewer than 20 task interactions
Or the user has attended fewer than 5 meetings
When role inference runs
Then display "Getting to know your role..."
And offer manual role selection option

### Requirement: Role Confirmation

The system MUST confirm inferred role with the user.

#### Scenario: One-time role confirmation
Given the user has used Meridian for ~1 week
And role inference has sufficient confidence
When the user next opens the app
Then a role confirmation prompt appears
And the inferred role is pre-selected
And the user can confirm, change, or select "Other"

#### Scenario: Other role with free text
Given the user selects "Other" during role confirmation
When they submit their role
Then a free-text field accepts their custom role description
And the description is stored in `user_profile`

### Requirement: Role Drift Detection

The system MUST adapt to changing user behavior.

#### Scenario: Significant behavior shift
Given a user was classified as IC
And their recent behavior shows 3x more PR reviews than before
And they now create tasks for others regularly
When role drift is detected (score change > 0.2 over 2 weeks)
Then a "Your role may have changed" prompt appears
And the new inferred role is suggested

### Requirement: Role-Based Personalization

The system MUST adjust information surfacing based on role.

#### Scenario: Manager sees team items first
Given a user's role is "Manager"
When they view My Activity dashboard
Then team member items appear before personal items
And meeting follow-ups are prioritized

#### Scenario: IC sees own assignments first
Given a user's role is "IC"
When they view My Activity dashboard
Then personal task assignments appear first
And own PR status is prioritized

#### Scenario: Inline role adjustment
Given the user is viewing My Activity
When they see the role indicator
Then a [Change] link is visible
And clicking it allows quick role switching without navigating to settings

#### Scenario: Role tooltip explanation
Given a role-based view is active
When the user hovers over the role indicator
Then a tooltip explains the current view (e.g., "Showing Tech Lead view — focusing on reviews and team blockers")
