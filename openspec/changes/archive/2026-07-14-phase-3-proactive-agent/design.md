## Context

Phase 2 established pattern learning infrastructure. Phase 3 builds on this to make Meridian proactive — surfacing suggestions, auto-drafting communications, and intelligently planning tasks. The daemon job infrastructure, pattern models, and audit logging from earlier phases provide the foundation.

Current state:
- Pattern learning captures workflow sequences, priorities, and communication style
- Daemon worker processes background jobs
- Notification panel exists for pending imports
- AI chat generates context-aware responses
- Tasks have basic fields but no planning or draft associations

## Goals / Non-Goals

**Goals:**
- Surface actionable suggestions without overwhelming users (daily limit)
- Auto-draft communications that match user's style
- Intelligently decompose tasks based on complexity
- Detect sensitive content before accidental sharing
- Maintain transparency — users see why suggestions are made

**Non-Goals:**
- Automated sending (drafts only, never auto-send)
- Real-time notification push (suggestions appear on next panel open)
- Complex NLP for sensitive content (regex-based patterns sufficient for MVP)
- Subtask hierarchies beyond one level

## Decisions

### 1. Suggestions Table Schema

**Decision**: Store suggestions with action_config JSON for polymorphic actions.

```sql
CREATE TABLE suggestions (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,           -- overdue_task, stale_task, meeting_followup, workflow_sequence, decompose_task
    title TEXT NOT NULL,
    description TEXT,
    reasoning TEXT,               -- Explanation of why this was suggested
    action_config TEXT,           -- JSON: { task_id, suggested_title, etc. }
    severity TEXT DEFAULT 'info', -- info, warning, critical
    status TEXT DEFAULT 'pending',-- pending, accepted, dismissed, expired
    project_id TEXT,
    created_at TEXT NOT NULL,
    acted_at TEXT
);
```

**Rationale**: Flexible action_config allows different suggestion types to carry different payloads without schema changes.

### 2. Draft Messages Table Schema

**Decision**: Store drafts linked to tasks with channel flexibility.

```sql
CREATE TABLE draft_messages (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    channel TEXT NOT NULL,        -- email, slack, clipboard (placeholder for future integrations)
    recipient TEXT,
    subject TEXT,
    body TEXT NOT NULL,
    ai_signature INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft',  -- draft, sent, archived
    sensitive_warnings TEXT,      -- JSON array of detected warnings
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    sent_at TEXT
);
```

### 3. Task Plan Fields

**Decision**: Extend tasks table with plan fields rather than separate table.

```sql
ALTER TABLE tasks ADD COLUMN plan_complexity TEXT;  -- simple, medium, complex
ALTER TABLE tasks ADD COLUMN plan_data TEXT;        -- JSON with suggestions/subtasks
ALTER TABLE tasks ADD COLUMN plan_generated_at TEXT;
```

**Rationale**: Plans are 1:1 with tasks. Avoids join complexity for common task queries.

### 4. Suggestion Generation Architecture

**Decision**: Single daemon job runs every 30 minutes, generates all suggestion types.

```
generate_suggestions job
  ├── Query overdue tasks → create overdue_task suggestions
  ├── Query stale tasks → create stale_task suggestions
  ├── Query meetings without tasks → create meeting_followup suggestions
  ├── Check recent completions against workflow patterns → create workflow_sequence suggestions
  └── Enforce daily limit, queue overflow for tomorrow
```

**Rationale**: Single job is simpler than multiple scheduled jobs. 30-minute interval balances freshness with resource usage.

### 5. Sensitive Content Detection

**Decision**: Regex-based pattern matching, no ML models.

**Patterns:**
| Type | Pattern Examples |
|------|-----------------|
| SSN | `\d{3}-\d{2}-\d{4}` |
| Phone | `\d{3}[-.]?\d{3}[-.]?\d{4}` |
| Credit Card | `\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}` |
| API Key | `sk-[a-zA-Z0-9]{20,}`, `api[_-]?key[=:]\s*\S+` |
| Password | `password[=:]\s*\S+` (case insensitive) |

**Rationale**: Regex is fast, runs client-side without network, catches common cases. False positives are acceptable since warnings are non-blocking.

### 6. Draft Generation Prompt

**Decision**: Structured prompt with task context and style adaptation.

```
Generate a draft {channel} message for the following task:

Task: {title}
Description: {description}
Meeting context: {meeting_summary if linked}

Style preferences:
- Length: {length_preference}
- Formality: {formality_level}
- Common phrases to include: {common_additions}

Generate a professional message that accomplishes the task.
End with signature line: "Drafted by Meridian"
```

### 7. Complexity Evaluation Prompt

**Decision**: Simple prompt with clear criteria.

```
Evaluate the complexity of this task:

Title: {title}
Description: {description}

Classify as:
- SIMPLE: Single action, can be done immediately (send email, make call, quick review)
- MEDIUM: 2-5 discrete steps, can be broken into subtasks
- COMPLEX: Requires research, multiple stakeholders, or unclear scope

Respond with JSON: { "complexity": "simple|medium|complex", "reasoning": "...", "suggested_subtasks": [...] }
```

### 8. UI Integration Points

**Decision**: Leverage existing components with minimal new UI.

| Feature | Integration Point |
|---------|------------------|
| Suggestions | New section in NotificationCenter |
| Drafts tab | New tab in TaskEditModal/TaskInlineEditor |
| Plan section | Collapsible section in task detail below description |
| Sensitive warnings | Banner component above draft editor |

## Risks / Trade-offs

**[Risk] Too many suggestions overwhelm users**
→ Mitigation: Daily limit (default 10), severity-based prioritization, "stop suggesting" option

**[Risk] Draft quality varies**
→ Mitigation: Communication style adaptation, easy editing, clear AI attribution

**[Risk] False positives in sensitive content detection**
→ Mitigation: Non-blocking warnings, easy dismissal, audit for patterns to refine

**[Risk] Complexity evaluation inconsistent**
→ Mitigation: User corrections feed back into learning, plan editing before acceptance

**[Trade-off] Regex vs ML for sensitive content**
→ Accepted: Regex is fast and offline. ML would add model weight and latency for marginal accuracy improvement.

**[Trade-off] Drafts stored locally, not sent**
→ Accepted: Avoids integration complexity for MVP. Clipboard copy covers immediate need. Future phases add Slack/email integrations.

## Migration Plan

1. **Database migration**: Add `suggestions` table, `draft_messages` table, task plan columns
2. **Daemon job**: Register `generate_suggestions` job type
3. **AI prompts**: Add draft generation and complexity evaluation prompts
4. **UI components**: Add SuggestionCard, DraftEditor, PlanSection, SensitiveWarning
5. **Integration**: Wire into NotificationCenter and TaskEditModal
6. **Rollback**: Drop new tables and columns — no data loss risk for existing functionality

## Open Questions

1. **Should suggestions expire?** — Currently no expiration; stale suggestions might accumulate
2. **Draft versioning?** — Currently overwrites; should we keep edit history?
3. **Subtask linking?** — How to visually indicate subtask relationships in task list?
