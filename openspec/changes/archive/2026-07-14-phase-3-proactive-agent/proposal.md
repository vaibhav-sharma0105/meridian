## Why

Meridian currently waits for users to ask before acting — it doesn't proactively identify opportunities to help. Users miss overdue tasks, forget follow-ups, and manually draft repetitive communications. Phase 3 transforms Meridian from a reactive tool into a proactive assistant that surfaces actionable suggestions, auto-drafts communications, and intelligently plans tasks — reducing cognitive load and ensuring nothing falls through the cracks.

## What Changes

- **Suggestion engine**: Background job analyzes tasks, meetings, and patterns to generate actionable suggestions (overdue alerts, stale task reminders, follow-up prompts)
- **Suggestion UI**: New section in notification panel with accept/dismiss/stop-suggesting actions and reasoning transparency
- **Draft communications**: Auto-generate drafts for tasks containing action words ("follow up", "send", "share") with "Drafted by Meridian" signature
- **Smart task plans**: AI evaluates task complexity and suggests subtasks or draft actions; user corrections feed back into pattern learning
- **Sensitive content detection**: Scan drafts for PII, credentials, and financial data with non-blocking warnings

## Capabilities

### New Capabilities

- `suggestion-engine`: Background analysis and suggestion generation with configurable limits and pattern integration
- `suggestion-ui`: Notification panel integration with accept/dismiss actions and reasoning display
- `draft-communications`: Auto-draft generation for action tasks with inline editing and clipboard support
- `smart-task-plans`: AI complexity evaluation, subtask suggestions, and plan storage with learning feedback
- `sensitive-content-detection`: PII/credential/financial data scanning with warning UI and audit logging

### Modified Capabilities

- `pattern-observation`: Add observation type for suggestion dismissals to train negative learning
- `workflow-sequences`: Integrate learned sequences into suggestion generation

## Impact

- **Database**: New `suggestions` and `draft_messages` tables; extend `tasks` with plan fields
- **Daemon**: New suggestion generation job running on schedule
- **AI**: New prompts for draft generation, complexity evaluation, and sensitive content detection
- **Notification UI**: Extend notification panel with suggestions section
- **Task detail UI**: Add Drafts tab and Plan section
- **Audit log**: Log suggestion actions and sensitive content detections
- **Settings**: Add suggestion frequency/limit configuration
