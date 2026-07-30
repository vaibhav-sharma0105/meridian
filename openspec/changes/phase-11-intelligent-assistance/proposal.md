## Why

Meridian's auto-learning system exists but isn't visibly helping users. Pattern observations happen silently; suggestions exist but aren't surfaced effectively. Users don't feel the system learning or improving. This phase makes learning visible and genuinely useful through smart task list organization, intelligent defaults, and predictive assistance.

## What Changes

- **Needs Attention Section**: Smart task grouping at top of task list showing overdue, stale, and follow-up items
- **Action-Oriented UI**: Each attention item has [→ Do] button for immediate action, not just observation
- **My Activity Digest**: AI-generated summary of what needs attention (no interrupting nudges)
- **Pre-Accepted Chips**: Task creation shows suggested values as pre-filled chips based on confidence level:
  - High confidence (>85%): Pre-filled in field with info tooltip explaining why
  - Medium confidence (50-85%): Ghost text suggestion, tab to accept
  - Low confidence (<50%): Empty field, no guess (preserve trust)
- **Click to Remove**: Click any suggestion chip to remove it; brief feedback shown ("Got it — won't suggest X")
- **Clear All Escape**: One-click to clear all suggestions on task creation
- **Draft Generation**: AI drafts for Slack/email when integrations connected, adapted to learned style
- **Pre-Fetch Documents**: Background job fetches relevant docs 30 min before meetings
- **Auto-Draft Agendas**: Generate meeting agenda from open tasks and attendee assignments
- **Task Estimation**: Track actual vs estimated duration; provide smart estimates for new tasks

## Capabilities

### New Capabilities
- `needs-attention`: Smart grouping of tasks requiring user action with severity ordering
- `smart-chips`: Pre-accepted suggestion chips on task creation with confidence-based display (high=prefill, medium=ghost, low=empty); click to remove
- `learning-feedback`: Visible feedback when user rejects suggestion ("Got it — won't suggest X")
- `predictive-prefetch`: Background document pre-fetch before scheduled meetings
- `auto-agenda`: Meeting agenda generation from task and attendee context
- `task-estimation`: Duration tracking and smart estimate suggestions

### Modified Capabilities
- `suggestions`: Surface through Needs Attention section, not separate notifications
- `pattern-learning`: Add estimation patterns; feedback loop for suggestion accuracy
- `draft-generation`: Integrate with learned communication style; activate on integration connection

## Technical Specifications

### Needs Attention Section

**Computation Strategy:**
- Pre-computed on task/meeting changes, stored in `attention_items` table
- Updated incrementally when individual tasks change; full recompute daily at 3am
- Query is simple SELECT on pre-computed table, not real-time aggregation

**Severity Ordering:**
1. Critical: Overdue > 7 days, high-priority stale > 3 days
2. Warning: Overdue 1-7 days, any priority stale > 5 days, meeting follow-up > 48 hours
3. Info: Approaching deadline (within 2 days), low-activity tasks

**Attention Fatigue Prevention:**
- Default view: Top 3 items only
- Collapsible: "See N more" expands full list
- Focus Mode toggle: When enabled, shows only Critical items
- Weekly digest: If user ignores section for 7 days, prompt to snooze or adjust thresholds

**Action Buttons (Contextual):**
| Item Type | Primary Action | Secondary |
|-----------|----------------|-----------|
| Overdue task | "Extend deadline" | "Mark done" |
| Stale task | "Add update" | "Mark done" |
| Meeting follow-up | "Create tasks" | "Dismiss" |
| Approaching deadline | "View task" | — |

### Confidence Calibration

**Historical Tracking:**
- Store last 100 suggestions per pattern type with accept/reject outcome
- `confidence_accuracy` = accepted / (accepted + rejected) over last 30 days
- Adjust thresholds per user: if user accepts 90% of "medium" suggestions, promote to "high" display

**User-Adjustable:**
- Settings page: "Suggestion sensitivity" slider (Conservative / Balanced / Aggressive)
- Conservative: only >90% confidence shown; Aggressive: show >40% confidence
- Default: Balanced (thresholds as specified)

**Learning Rate:**
- Single rejection: confidence -= 5% for that pattern
- Single acceptance: confidence += 2% (slower positive learning prevents overconfidence)
- Minimum 10 observations before showing any suggestion for a pattern

### Ghost Text Visual Distinction
- Style: Italic, color `text-zinc-400`, slightly smaller font (0.9em)
- Label: Small "Suggested" tag above field when ghost text present
- Behavior: Tab to accept fills field with normal styling; typing replaces ghost text immediately

### Pre-Fetch Strategy

**Meeting Source:**
- Use meetings in Meridian database (imported from Zoom/Sheets Relay)
- No external calendar integration required; works with existing meeting records
- Pre-fetch timing: 30 minutes before meeting `start_time`

**Document Selection:**
- Match meeting title keywords against document embeddings
- Match attendee names against task assignees → fetch those task-related docs
- Limit: Top 5 most relevant documents per meeting

**Cache Behavior:**
- Pre-fetched docs loaded into memory, not re-embedded
- Cache cleared after meeting end time + 1 hour

### Agenda Generation Quality

**Input Quality Check:**
- Require at least 2 tasks linked to meeting OR 2 attendees with open tasks
- If insufficient: show "Not enough context for agenda" with manual create option

**Human Review:**
- Always show "Draft Agenda" with edit capability before saving
- Highlight sections AI is uncertain about (e.g., "Discussion topics - please review")
- "Regenerate" button if user wants different approach

### Task Estimation Bootstrap

**Duration Tracking:**
- Track time from `status: in_progress` to `status: done`
- Only count if duration > 5 minutes (filter out quick updates)
- Store in `task_estimation_log` with task keywords, priority, assignee

**Estimate Display:**
- Show estimate only after 20+ similar tasks completed
- Format: "Usually takes ~2 hours" (rounded to nearest sensible unit)
- Basis: Median duration of similar tasks (by keyword overlap > 50%)

**Optional Estimate Field:**
- Add optional "Estimated time" field to task creation
- Not required; only used to improve model accuracy when provided
- Compare estimate vs actual in UI: "You estimated 2h, took 3h"

## Impact

- **Database**: New `attention_items` table for pre-computed attention items; `task_estimation_log` for duration tracking; `suggestion_outcomes` for confidence calibration; extend `suggestions` with action metadata
- **Backend**: NeedsAttention aggregator with incremental updates; smart chips confidence scorer with historical tracking; prefetch daemon job using meeting records; agenda generator with quality checks
- **Frontend**: NeedsAttentionSection with collapsible view and focus mode; SmartChips component with ghost text styling; LearningFeedbackToast; AgendaPreview with edit capability; sensitivity slider in settings
- **MCP**: Add `get_suggestions`, `get_user_patterns`, `get_assignee_suggestions` tools
- **UX**: Zero-friction design — suggestions pre-accepted, one click to dismiss, visible learning; attention fatigue prevention built in
