# Productivity Patterns

Time-of-day learning for task type productivity and optimal scheduling suggestions.

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
