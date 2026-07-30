## Context

Meridian has evolved into an agentic assistant with skills, integrations, and proactive suggestions. These capabilities can take actions on behalf of the user — creating tasks, posting to Slack, filing GitHub issues. Currently, there is no unified governance layer:

- Skills have per-skill approval modes but no global override
- Integrations have no autonomy controls
- Risk is not systematically classified
- Users cannot easily undo agent actions
- There's no visibility into overall agent activity patterns

Phase 6 introduces the governance layer that sits between all agent capabilities and their execution, providing consistent autonomy control, risk assessment, approval flows, and undo capabilities.

**Existing Infrastructure:**
- `audit_log` table with action tracking (Phase 0)
- `skill_runs` table with approval workflow (Phase 4)
- `suggestions` table with acceptance tracking (Phase 3)
- Desktop notifications (Phase 5)
- Background daemon for job execution (Phase 0)

## Goals / Non-Goals

**Goals:**
- Unified autonomy control across all agent actions (skills, integrations, suggestions)
- Consistent risk classification for all actions
- Single approval queue for all pending actions
- Undo capability for reversible actions
- Dashboard for agent activity visibility and anomaly detection

**Non-Goals:**
- Team-level governance (Phase 7 scope)
- Real-time collaboration on approvals (single-user focus)
- ML-based risk prediction (rules + learned adjustments only)
- Per-field undo granularity (entity-level only)

## Decisions

### Decision 1: Autonomy Mode Architecture

**Choice:** Three-tier inheritance: Global → Integration → Skill

```
Global Autonomy Mode (app_settings.autonomy_mode)
    ↓ inherited by
Integration Autonomy (integrations.autonomy_mode) 
    ↓ inherited by
Skill Autonomy (skills.autonomy_mode)
```

**Alternatives Considered:**
- **Flat per-action configuration**: Too granular, configuration burden
- **Two-tier (global + skill only)**: Misses integration-level control which is important for external systems

**Rationale:** Users think hierarchically: "I trust internal actions more than external ones." Integration-level control naturally maps to this mental model. Skills can override when specific automation is trusted.

### Decision 2: Risk Classification Scoring

**Choice:** Weighted score with critical override

```rust
struct RiskScore {
    action_type_weight: u8,    // read=1, create=2, update=3, delete=5, external_send=4
    destination_score: u8,     // internal=1, team=2, external=3, executive=4
    content_score: u8,         // normal=1, sensitive=2, pii=3, financial=4
}

fn calculate_risk_level(score: RiskScore) -> RiskLevel {
    // Critical override: any maximum individual score
    if score.action_type_weight == 5 || score.destination_score == 4 || score.content_score == 4 {
        return RiskLevel::Critical;
    }
    
    let total = score.action_type_weight + score.destination_score + score.content_score;
    match total {
        1..=4 => RiskLevel::Low,
        5..=7 => RiskLevel::Medium,
        8..=10 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}
```

**Alternatives Considered:**
- **ML-based classification**: Requires training data, complexity
- **User-defined rules only**: Missing baseline protection

**Rationale:** Rules-based with learned adjustments provides deterministic baseline while adapting to user corrections. Critical override ensures dangerous actions always require maximum scrutiny.

### Decision 3: Unified Approval Queue

**Choice:** Single `pending_approvals` table replacing skill-specific approval

```sql
CREATE TABLE pending_approvals (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,        -- 'skill_execution', 'suggestion_action', 'integration_write'
    action_config TEXT NOT NULL,      -- JSON: full action details
    source_type TEXT,                 -- 'skill', 'suggestion', 'integration', 'mcp'
    source_id TEXT,                   -- skill_id, suggestion_id, integration_id
    risk_level TEXT NOT NULL,
    autonomy_mode TEXT NOT NULL,      -- effective mode at time of creation
    context TEXT,                     -- JSON: additional context for approval UI
    timeout_at TEXT,
    status TEXT DEFAULT 'pending',    -- 'pending', 'approved', 'rejected', 'archived', 'executed'
    resolved_by TEXT,                 -- 'user', 'timeout', 'bulk'
    resolution_reason TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    resolved_at TEXT
);
```

**Alternatives Considered:**
- **Keep separate approval per feature**: Inconsistent UX, duplicate logic
- **Approval as audit log entries**: Mixes concerns, query complexity

**Rationale:** Single queue enables: unified UI, bulk actions, consistent timeout handling, cross-feature approval metrics.

### Decision 4: Undo via Reversal Actions

**Choice:** Create reversal action, not true rollback

```sql
CREATE TABLE action_history (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    before_state TEXT,           -- JSON snapshot (null for external actions)
    after_state TEXT,            -- JSON snapshot
    undoable BOOLEAN DEFAULT TRUE,
    undo_action_id TEXT,         -- FK to reversal action if undone
    audit_log_id TEXT,           -- FK to audit_log entry
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

Undo create → delete. Undo update → update with before_state. Undo delete → create with before_state.

**Alternatives Considered:**
- **True rollback with transactions**: Complex for external actions, state drift risk
- **Event sourcing**: Architectural overhaul, overkill for this use case

**Rationale:** Reversal is simpler, auditable, and handles the reality that external actions (Slack messages, GitHub issues) cannot be truly undone.

### Decision 5: Governance Dashboard Data Source

**Choice:** Query audit_log with extended fields + materialized aggregates

Extend audit_log:
```sql
ALTER TABLE audit_log ADD COLUMN risk_level TEXT;
ALTER TABLE audit_log ADD COLUMN autonomy_mode TEXT;
ALTER TABLE audit_log ADD COLUMN autonomy_source TEXT;
ALTER TABLE audit_log ADD COLUMN approval_id TEXT;
ALTER TABLE audit_log ADD COLUMN undo_action_id TEXT;
```

Materialized daily aggregates (computed by daemon job):
```sql
CREATE TABLE governance_metrics (
    date TEXT NOT NULL,
    metric_type TEXT NOT NULL,   -- 'action_count', 'risk_distribution', 'approval_rate'
    breakdown_key TEXT,          -- e.g., 'low', 'medium', 'high' for risk_distribution
    value INTEGER,
    PRIMARY KEY (date, metric_type, breakdown_key)
);
```

**Alternatives Considered:**
- **Real-time aggregation only**: Performance concerns at scale
- **Separate analytics database**: Complexity for single-user app

**Rationale:** Daily aggregates provide fast dashboard load while audit_log enables drill-down. Background job computes aggregates overnight.

## Data Flow

```
Agent Action Initiated
        ↓
┌───────────────────────────────────────────────────┐
│           Risk Classification Engine              │
│  - Classify action type, destination, content     │
│  - Apply learned adjustments                      │
│  - Return risk_level                              │
└───────────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────────┐
│            Autonomy Controller                    │
│  - Resolve effective autonomy mode                │
│  - Compare risk_level vs autonomy allowance       │
│  - Decision: execute | queue_approval             │
└───────────────────────────────────────────────────┘
        ↓                    ↓
   [Execute]          [Queue Approval]
        ↓                    ↓
┌─────────────────┐  ┌──────────────────────────────┐
│ Action History  │  │    pending_approvals         │
│ - before_state  │  │  - Wait for user decision    │
│ - after_state   │  │  - Timeout → archive         │
└─────────────────┘  └──────────────────────────────┘
        ↓                    ↓
┌───────────────────────────────────────────────────┐
│              Audit Log Entry                      │
│  - risk_level, autonomy_mode, approval_id         │
└───────────────────────────────────────────────────┘
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Approval fatigue in Manual mode | Default to Supervised, show stats on "would have auto-approved" |
| Undo creates inconsistent state | Warn about dependent actions, allow user to proceed anyway |
| Risk classification misses edge cases | Critical override for any maximum score, learned adjustments |
| Dashboard performance at scale | Materialized daily aggregates, paginated queries |
| External actions not undoable | Clear marking, pre-execution warning for high-risk |

## Migration Plan

### Phase 1: Database & Core
1. Create `pending_approvals` table
2. Create `action_history` table
3. Create `governance_metrics` table
4. Extend `audit_log` with new columns
5. Add `autonomy_mode` column to `integrations` and `skills` tables

### Phase 2: Backend Modules
1. Implement risk classification engine (`src-tauri/src/governance/risk.rs`)
2. Implement autonomy controller (`src-tauri/src/governance/autonomy.rs`)
3. Implement approval flow (`src-tauri/src/governance/approval.rs`)
4. Implement undo system (`src-tauri/src/governance/undo.rs`)
5. Add governance commands to lib.rs

### Phase 3: Integration Points
1. Hook skill execution through autonomy controller
2. Hook suggestion acceptance through autonomy controller
3. Hook integration writes through autonomy controller
4. Hook MCP write operations through autonomy controller

### Phase 4: Frontend
1. Autonomy settings UI
2. Approval queue UI
3. Undo bar component
4. Action history panel
5. Governance dashboard

### Phase 5: Daemon Jobs
1. Approval timeout checker (runs every minute)
2. Governance metrics aggregator (runs daily)
3. Anomaly detection (runs hourly)

## Open Questions

1. **Approval timeout default**: 5 minutes vs 30 minutes vs 24 hours? Configurable, but what default?
2. **Undo window**: How long should instant undo bar be visible? 10 seconds proposed.
3. **Anomaly thresholds**: What multiplier triggers anomaly flag? 2x proposed but may need tuning.
4. **Risk rule editing UI**: How granular should user customization be? Start with channel/contact marking only?
