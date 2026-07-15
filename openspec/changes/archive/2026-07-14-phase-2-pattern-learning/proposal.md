## Why

Meridian currently treats every user interaction as isolated — it doesn't learn from patterns in how users work. Users repeatedly make the same corrections to AI suggestions, manually assign tasks following predictable patterns, and edit AI drafts to match their communication style. Phase 2 transforms Meridian from a reactive tool into an adaptive assistant that observes, learns, and improves over time — reducing friction and making suggestions increasingly relevant.

## What Changes

- **Observation pipeline**: Track user actions (task completions, edits, assignments, draft modifications) in a structured observation table
- **Pattern aggregation**: Background job processes raw observations into statistical pattern models
- **Workflow sequence suggestions**: Learn "after task A, user usually does B" and proactively suggest next actions
- **Communication style adaptation**: Analyze user edits to AI drafts to learn tone, length, and phrasing preferences
- **Priority/assignee defaults**: Learn task type → priority and task type → assignee patterns for smarter defaults
- **Learning management UI**: Settings panel to view learned patterns, reset by category, and export/import learning data
- **Negative learning**: Track rejected suggestions to reduce frequency and eventually ask "Stop suggesting this?"

## Capabilities

### New Capabilities

- `pattern-observation`: Infrastructure for recording user actions as structured observations with context
- `pattern-aggregation`: Background job that processes observations into statistical models per pattern type
- `workflow-sequences`: Learn and suggest task/action sequences based on historical patterns
- `communication-style`: Learn user's writing preferences from AI draft edits
- `smart-defaults`: Apply learned patterns for priority, assignee, and other task fields
- `learning-management`: UI for viewing, resetting, and exporting learned patterns

### Modified Capabilities

(none — this is new infrastructure)

## Impact

- **Database**: New tables `pattern_observations` and `pattern_models`
- **Daemon**: New job type for pattern aggregation processing
- **AI chat**: Drafts adapt to learned communication style
- **Task creation**: Default values informed by learned patterns
- **Settings UI**: New "Learning" section with pattern management
- **Storage**: Observation data grows with usage (will need pruning strategy)
