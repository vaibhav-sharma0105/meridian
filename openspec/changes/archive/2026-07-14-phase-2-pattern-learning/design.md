## Context

Meridian Phase 1 established document intelligence with semantic search and daemon-based embedding jobs. Phase 2 extends this foundation to learn from user behavior patterns. The daemon job infrastructure, Qdrant integration, and existing audit logging provide the scaffolding for pattern observation and aggregation.

Current state:
- All user actions pass through Tauri commands (task CRUD, meeting imports, AI chat)
- Daemon worker polls `daemon_jobs` table and processes background work
- Audit log captures entity changes but lacks behavioral context
- AI drafts are generated but not tracked for user modifications

## Goals / Non-Goals

**Goals:**
- Learn user behavior patterns passively without explicit training
- Improve suggestions incrementally as patterns strengthen
- Maintain transparency — users see what's learned and can reset
- Keep learning local-first (no data leaves the device)
- Support pattern portability (export/import across machines)

**Non-Goals:**
- Real-time learning (batched aggregation is sufficient)
- Cross-project pattern transfer (patterns are project-scoped)
- Complex ML models (statistical aggregation only)
- Learning from external sources (only local actions)

## Decisions

### 1. Observation Storage Schema

**Decision**: Store raw observations in `pattern_observations` table with JSON `context_data` for flexible payloads.

```sql
CREATE TABLE pattern_observations (
    id TEXT PRIMARY KEY,
    observation_type TEXT NOT NULL,  -- task_completion, priority_set, draft_edit, etc.
    entity_type TEXT,                -- task, meeting, project
    entity_id TEXT,
    project_id TEXT,
    context_data TEXT NOT NULL,      -- JSON blob with type-specific data
    created_at TEXT NOT NULL,
    processed_at TEXT                -- NULL until aggregation processes it
);
```

**Rationale**: Flexible JSON context allows different observation types without schema changes. The `processed_at` marker enables reliable batch processing without re-processing.

**Alternatives considered**:
- Typed tables per observation type: Too rigid, each new observation type needs migration
- Single `action_log` with flat columns: Doesn't capture rich context needed for patterns

### 2. Pattern Model Structure

**Decision**: Store aggregated patterns in `pattern_models` with JSON `model_data` per pattern type.

```sql
CREATE TABLE pattern_models (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,      -- workflow_sequence, communication_style, priority_default, assignee_default
    project_id TEXT,                 -- NULL for global patterns
    model_data TEXT NOT NULL,        -- JSON with type-specific aggregated data
    confidence REAL NOT NULL,        -- 0.0 to 1.0
    observation_count INTEGER NOT NULL,
    last_updated TEXT NOT NULL
);
```

**Rationale**: Single table with JSON payloads allows pattern-specific schemas to evolve independently. Confidence score enables threshold-based suggestions.

### 3. Observation Recording Points

**Decision**: Record observations at Tauri command level, not React component level.

**Recording points**:
| Action | Observation Type | Context Data |
|--------|-----------------|--------------|
| Task marked done | `task_completion` | preceding_action, time_since_last_completion |
| Priority changed | `priority_set` | old_priority, new_priority, task_keywords |
| Assignee changed | `assignee_set` | old_assignee, new_assignee, task_keywords |
| AI draft edited | `draft_edit` | original_text, edited_text, context_type |
| Suggestion dismissed | `suggestion_dismissed` | suggestion_type, suggestion_content |

**Rationale**: Backend recording is reliable (can't be bypassed), captures all actions regardless of UI path, and keeps frontend simple.

### 4. Aggregation Job Scheduling

**Decision**: Run pattern aggregation every 15 minutes via daemon job queue.

**Process**:
1. Daemon checks for `aggregate_patterns` job type
2. Job queries unprocessed observations (WHERE processed_at IS NULL)
3. Groups by pattern_type and project_id
4. Updates or creates pattern_models with new statistics
5. Marks observations as processed
6. Reschedules self for next 15-minute interval

**Rationale**: Batch processing is efficient, 15-minute lag is acceptable for pattern suggestions, uses existing daemon infrastructure.

**Alternatives considered**:
- Real-time aggregation: Wasteful for patterns that need multiple observations
- Daily aggregation: Too slow for user feedback
- On-demand aggregation: Unpredictable load, may lag during inactive periods

### 5. Confidence Scoring Algorithm

**Decision**: Use weighted confidence based on observation count, recency, and consistency.

```
base_confidence = min(observation_count / 10, 1.0)  -- Caps at 10 observations
recency_weight = exp(-days_since_last_observation / 30)  -- Decays over 30 days
consistency = matching_observations / total_observations
confidence = base_confidence * recency_weight * consistency
```

**Rationale**: Simple formula that captures key factors. Easy to tune thresholds.

### 6. Suggestion Thresholds

**Decision**: Different confidence thresholds per suggestion type.

| Suggestion Type | Minimum Confidence | Rationale |
|-----------------|-------------------|-----------|
| Workflow sequence | 0.5 | Lower threshold — easy to dismiss |
| Priority default | 0.5 | Pre-filled but editable |
| Assignee default | 0.5 | Pre-filled but editable |
| Communication style | 0.6 | Higher threshold — directly affects output |

### 7. Negative Learning

**Decision**: Track dismissed suggestions and suppress after 3 dismissals.

**Process**:
1. Suggestion dismissed → record `suggestion_dismissed` observation
2. On 3rd dismissal, move to `negative_patterns` in model_data
3. Negative patterns are never suggested
4. On 4th dismissal, show "Stop learning this?" prompt
5. User can clear negative patterns via Learning settings

**Rationale**: Balances learning from mistakes without over-persisting incorrect patterns.

### 8. Communication Style Analysis

**Decision**: Use diff-based analysis to detect style patterns.

**Extracted features**:
- `length_delta`: (edited_length - original_length) / original_length
- `formality_shift`: Detect formal→casual or casual→formal phrase replacements
- `added_phrases`: Common text additions across multiple edits
- `removed_phrases`: Common text removals across multiple edits

**Rationale**: Diff-based analysis is lightweight, doesn't require NLP models, captures measurable style dimensions.

### 9. Learning Management Architecture

**Decision**: Dedicated Settings section with category-based management.

**Structure**:
```
Settings > Learning
├── Pattern Categories (list with confidence scores)
│   ├── Workflow Sequences → detail view
│   ├── Communication Style → detail view
│   ├── Priority Defaults → detail view
│   └── Assignee Defaults → detail view
├── Actions
│   ├── Export Learning Data (JSON download)
│   ├── Import Learning Data (JSON upload)
│   └── Reset All Learning (destructive, requires confirmation)
```

**Rationale**: Transparency builds trust. Users can see exactly what's learned and control it.

## Risks / Trade-offs

**[Risk] Observation data grows unboundedly**
→ Mitigation: Prune processed observations older than 90 days. Pattern models retain aggregated statistics, not raw data.

**[Risk] False patterns from insufficient data**
→ Mitigation: Confidence thresholds prevent suggestions until patterns are statistically meaningful. Low observation count caps confidence at 0.3.

**[Risk] Stale patterns persist after behavior changes**
→ Mitigation: Confidence decay (10% per month of inactivity) naturally deprecates unused patterns.

**[Risk] Export/import could transfer patterns to wrong project**
→ Mitigation: Import validates project_id references. Option to import as "global" patterns only.

**[Risk] Performance impact from observation recording**
→ Mitigation: Observation inserts are async, fire-and-forget. No blocking on user actions.

**[Trade-off] Project-scoped patterns limit reuse**
→ Accepted: Project-specific workflows outweigh cross-project generalization for MVP. Global patterns can be added later.

**[Trade-off] Statistical patterns lack semantic understanding**
→ Accepted: Simple aggregation works for MVP. Vector-based pattern similarity could enhance later.

## Migration Plan

1. **Database migration**: Add `pattern_observations` and `pattern_models` tables (non-breaking)
2. **Observation recording**: Add recording calls to existing Tauri commands (no API changes)
3. **Aggregation job**: Register new daemon job type
4. **UI integration**: Add suggestion components to existing views
5. **Settings**: Add Learning section to Settings panel
6. **Rollback**: Drop new tables, remove recording calls — existing functionality unaffected

No data migration required. Patterns build from scratch after deployment.

## Open Questions

1. **Should patterns transfer across project archives/restores?** — Current design loses patterns on archive
2. **What's the right observation pruning age?** — 90 days proposed but needs validation
3. **Should there be a "learning paused" global toggle?** — For users who want to try without commitment
