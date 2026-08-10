# Role Inference

Pattern-based detection of user role to personalize information surfacing and suggestions.

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
