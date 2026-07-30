## Why

Skill execution results, integration digests, and important AI chat responses currently have no persistent home. Users cannot review past outputs or find relevant information later. Additionally, Meridian treats all users identically, but a Tech Lead needs different information surfaced than an IC engineer. This phase introduces a Message Center for persistent outputs and role-based personalization.

## What Changes

- **Message Center**: Dedicated sidebar view for skill results, integration digests, and pinned AI chat highlights
- **Notification Integration**: Notifications show "View full result" link that opens Message Center
- **Time-Based Retention**: Configurable retention (N days, default 30) for message center content
- **Role Inference**: Learn user role from patterns (task creation vs receiving, meeting running vs attending, PR authoring vs reviewing)
- **Role Confirmation**: One-time prompt after ~1 week of usage to confirm inferred role
- **Role Drift**: Continuous adaptation with periodic check-in when significant behavior shift detected
- **Inline Role Adjustment**: My Activity shows current role view with inline [Change] link for quick switching without settings page
- **Time-of-Day Patterns**: Learn when user is most productive for different task types
- **Optimal Scheduling**: Suggest best times for task types based on productivity patterns

## Capabilities

### New Capabilities
- `message-center`: Persistent storage and UI for skill results, digests, and pinned content
- `role-inference`: Pattern-based detection of user role (Tech Lead, IC, PM, Manager)
- `role-adjustment`: Inline [Change] link in My Activity for quick role switching without settings
- `productivity-patterns`: Time-of-day learning for task type productivity
- `optimal-scheduling`: Suggest timing based on learned productivity patterns

### Modified Capabilities
- `notifications`: Add deep link to Message Center for full result viewing
- `suggestions`: Weight suggestions based on inferred role
- `my-activity-dashboard`: Filter and prioritize based on role

## Technical Specifications

### Message Center vs Notifications Routing

| Content Type | Destination | Rationale |
|--------------|-------------|-----------|
| Brief status updates | Notifications only | Transient, no need to persist |
| Skill results with output | Message Center + Notification link | User may want to revisit |
| AI chat with generated files | Message Center (auto-pinned) | Files need persistent home |
| Integration sync summary | Notifications only | Routine, unless errors |
| Integration sync with new items | Message Center + Notification link | User may want to review items |
| Approval requests | Notifications only | Action required, time-sensitive |
| Daily/weekly digests | Message Center | Reference material |

### Auto-Pinning Rules
- AI chat responses containing file attachments → auto-pinned to Message Center
- AI chat responses > 500 words → suggest pinning with one-click option
- Skill results marked `important: true` in skill config → auto-pinned

### Storage Strategy
- **References not copies**: Message Center stores reference to files, not duplicates; files live in `created_files/`
- **File cleanup**: When message expires (retention exceeded), associated files in `created_files/` also cleaned up unless referenced by another message
- **Size monitoring**: Show storage used by Message Center in settings; warn if > 500MB

### Role Inference Model

**Signals and Weights:**

| Signal | Tech Lead | IC | PM | Manager |
|--------|-----------|----|----|---------|
| Creates tasks for others | 0.3 | 0.0 | 0.4 | 0.3 |
| Receives task assignments | 0.1 | 0.5 | 0.2 | 0.1 |
| Runs meetings (assigns follow-ups) | 0.3 | 0.0 | 0.3 | 0.4 |
| Attends meetings (receives follow-ups) | 0.1 | 0.4 | 0.1 | 0.1 |
| Reviews PRs (GitHub) | 0.4 | 0.2 | 0.0 | 0.1 |
| Authors PRs (GitHub) | 0.2 | 0.5 | 0.0 | 0.0 |
| Views roadmap/epic items (Jira) | 0.2 | 0.0 | 0.5 | 0.2 |
| Works on bugs/tasks (Jira) | 0.2 | 0.5 | 0.1 | 0.0 |

**Multi-Label Classification**: User can have multiple roles (e.g., "IC + Tech Lead" for senior engineers); show primary with secondary if confidence > 0.3
**Minimum Activity**: Require 20+ task interactions AND 5+ meetings before confident inference; until then, show "Getting to know your role..." with manual selection option
**"Other" Option**: Always available; user can describe custom role in free text

### Productivity Patterns Cold Start
- **Minimum Data**: Require 50+ task completions with timestamps before suggesting optimal times
- **Fallback**: Until threshold met, use general productivity research defaults (9-11am, 2-4pm for deep work)
- **Opt-Out**: User can disable productivity tracking in settings; setting clearly visible

### Role Value Proposition
- **Immediate Benefit**: When role confirmed, My Activity immediately reorders (Manager sees team items first; IC sees own assignments first)
- **Explanation**: One-line tooltip "Showing Tech Lead view — focusing on reviews and team blockers"

## Impact

- **Database**: New `message_center` table with content, type, retention metadata, `auto_pinned` flag; extend `pattern_observations` with time-of-day data; add `user_profile` table for role scores and productivity patterns
- **Backend**: New `src-tauri/src/messages/` module with routing rules and auto-pin logic; role inference engine with weighted scoring; productivity aggregation daemon job; file cleanup on message expiration
- **Frontend**: MessageCenter sidebar view with search/filter; RoleConfirmation prompt component with manual selection; ProductivityInsights in settings; storage usage indicator
- **MCP**: Add `create_report`, `get_reports`, `draft_message` tools
- **AI**: Role context included in prompts; time-of-day awareness for scheduling suggestions
