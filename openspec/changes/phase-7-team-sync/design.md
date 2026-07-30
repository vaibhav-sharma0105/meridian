## Context

Meridian has evolved through Phases 0-6 into a capable single-user agentic assistant with skills, integrations, pattern learning, and governance. However, several limitations emerge when users work in teams:

1. **No shared intelligence**: Each user's learned patterns are siloed; teams can't benefit from collective workflow knowledge
2. **Skill sharing is cosmetic**: The `shared` flag exists on skills but has no functional team discovery or cloning
3. **No team context for assignments**: Assignee suggestions don't consider team roster, workload, or expertise
4. **No data portability**: Users can't backup, migrate, or sync their Meridian data

**Existing Infrastructure:**
- `skills` table with `shared` boolean and `owner_id` (unused)
- `pattern_models` table with personal patterns
- Slack and Google integrations can provide team member data
- SQLCipher encryption already in place for secure export

**Constraints:**
- Local-first architecture must be preserved — no central server
- Team features must gracefully degrade to solo mode
- Export/import must handle encrypted data securely
- Pattern sharing must be opt-in with clear privacy controls

## Goals / Non-Goals

**Goals:**
- Enable team roster management with workspace integration
- Provide intelligent assignee suggestions based on patterns and workload
- Support full data export/import with encryption
- Allow opt-in contribution to team-level patterns
- Make skill sharing functional with discovery and cloning

**Non-Goals:**
- Real-time collaboration or live sync between users (requires server)
- Team permissions or access control (single-user app)
- Cloud backup service (user provides storage)
- Automated conflict resolution (user decides on merge)
- Team analytics dashboard (Phase 8 scope)

## Decisions

### Decision 1: Team Roster Architecture

**Choice:** Local team_members table populated from multiple sources

```sql
CREATE TABLE team_members (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    source TEXT NOT NULL,  -- 'manual', 'slack', 'google'
    source_id TEXT,        -- external ID for dedup
    role TEXT DEFAULT 'member',  -- 'admin', 'member'
    expertise TEXT,        -- JSON array of tags
    workload_score REAL,   -- computed from assigned tasks
    metadata TEXT,         -- JSON for source-specific data
    last_synced_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source, source_id)
);
```

**Alternatives Considered:**
- **Use assignee strings directly**: Loses deduplication, no workload tracking
- **External team service**: Violates local-first principle

**Rationale:** A dedicated table enables deduplication across sources (same person in Slack and Google), workload computation, and expertise tracking. The unique constraint on (source, source_id) prevents duplicates during sync.

### Decision 2: Assignee Intelligence Algorithm

**Choice:** Multi-factor scoring with learned weights

```rust
struct AssigneeScore {
    pattern_score: f64,      // from smart_defaults patterns
    workload_score: f64,     // inverse of current task count
    expertise_score: f64,    // keyword match with task
    recency_score: f64,      // recently active on similar tasks
}

fn calculate_assignee_score(member: &TeamMember, task: &Task, patterns: &PatternModels) -> f64 {
    let weights = patterns.get_assignee_weights();  // learned from corrections
    
    weights.pattern * pattern_score(member, task, patterns)
        + weights.workload * workload_score(member)
        + weights.expertise * expertise_score(member, task)
        + weights.recency * recency_score(member, task)
}
```

**Alternatives Considered:**
- **Pattern-only**: Ignores current workload, may overload individuals
- **Round-robin**: Fair but ignores expertise and context

**Rationale:** Multi-factor scoring balances historical patterns with current state. Weights are learned from user corrections — if user often overrides workload-based suggestions, workload weight decreases.

### Decision 3: Export Format

**Choice:** Encrypted ZIP with JSON manifests + Qdrant snapshot

```
meridian-export-2026-07-21.zip (encrypted with user password)
├── manifest.json           # version, timestamp, content inventory
├── data/
│   ├── projects.json
│   ├── tasks.json
│   ├── meetings.json
│   ├── skills.json
│   ├── patterns.json
│   ├── team_members.json
│   ├── app_settings.json
│   └── audit_log.json      # optional, can be large
├── documents/
│   └── metadata.json       # doc metadata, not file contents
├── vectors/
│   └── qdrant_snapshot/    # Qdrant collection snapshot
└── checksum.sha256
```

**Alternatives Considered:**
- **SQLite dump**: Ties format to schema version, harder to merge
- **Protobuf**: More efficient but less debuggable
- **Unencrypted**: Security risk for sensitive data

**Rationale:** JSON is human-readable for debugging and schema-agnostic for version compatibility. ZIP provides compression. User-provided password encryption ensures data security. Qdrant snapshot format is native and efficient.

### Decision 4: Import Merge Strategy

**Choice:** Three-phase import with conflict UI

```
Phase 1: Inventory
  - Parse manifest, validate version compatibility
  - Scan for conflicts (same ID exists locally)
  - Build conflict report

Phase 2: User Decision
  - Show conflicts grouped by type (tasks, skills, etc.)
  - Options per conflict: Keep Local | Use Import | Skip
  - Bulk actions: Keep All Local | Use All Import

Phase 3: Apply
  - Insert non-conflicting items
  - Apply user decisions for conflicts
  - Update ID mappings for references
  - Rebuild Qdrant index if vector data imported
```

**Alternatives Considered:**
- **Last-write-wins**: Loses data silently
- **Automatic merge**: Complex, error-prone for conflicts

**Rationale:** Explicit user decision for conflicts ensures no data loss. Phased approach allows preview before commitment. ID mapping handles foreign key references across imported entities.

### Decision 5: Shared Patterns Architecture

**Choice:** Dual-layer patterns with contribution opt-in

```sql
ALTER TABLE pattern_models ADD COLUMN scope TEXT DEFAULT 'personal';  -- 'personal' | 'team'
ALTER TABLE pattern_models ADD COLUMN contributor_count INTEGER DEFAULT 1;

CREATE TABLE pattern_contributions (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    observation_hash TEXT NOT NULL,  -- anonymized observation
    contributed_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pattern_type, observation_hash)
);
```

**Contribution Flow:**
1. User enables "Contribute to team patterns" in settings
2. Pattern aggregation job anonymizes observations (removes entity IDs, keeps keywords)
3. Anonymized observations stored in `pattern_contributions`
4. On export, `pattern_contributions` can be shared
5. On import from teammate, merge into team-scope `pattern_models`

**Alternatives Considered:**
- **Central pattern server**: Violates local-first
- **Real-time pattern sync**: Requires always-on connection

**Rationale:** Export-based sharing maintains local-first architecture. Anonymization protects privacy while preserving pattern value. Dual-layer (personal + team) ensures individual patterns aren't overwritten.

### Decision 6: Skill Sharing Discovery

**Choice:** Export-based skill marketplace

Since there's no central server, skill sharing works via export:

1. User marks skill as `shared: true`
2. User exports skill as standalone `.skill.json` or `.skill.md` file
3. Teammate imports skill file
4. Imported skill has `cloned_from_id` pointing to original (if known)
5. Original owner's changes don't auto-propagate (no sync)

**UI Enhancement:**
- Skills list shows "Shared by [owner]" badge
- "Export for sharing" button on shared skills
- Import accepts skill files directly (not just full export)

**Alternatives Considered:**
- **Network discovery**: Requires LAN/server
- **Skill URL/link**: Requires hosting

**Rationale:** File-based sharing is simple, works offline, and aligns with local-first. Users can share via any file transfer method (email, Slack, Drive).

## Data Flow

```
Team Roster Population
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  Manual Entry ───┐                                         │
│                  │                                         │
│  Slack Sync ─────┼──→ team_members table ──→ Assignee     │
│                  │         ↓                 Suggestions  │
│  Google Sync ────┘    workload_score                      │
│                       (computed from                       │
│                        assigned tasks)                     │
└────────────────────────────────────────────────────────────┘

Pattern Sharing
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  User A observations ──→ anonymize ──→ pattern_contributions
│                                              ↓              │
│                                         export.zip          │
│                                              ↓              │
│  User B import ──────────────────────→ team pattern_models │
│                                              ↓              │
│                                    enhanced suggestions     │
└────────────────────────────────────────────────────────────┘

Export/Import
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  Export Request                                            │
│       ↓                                                    │
│  Gather: projects, tasks, meetings, skills, patterns       │
│       ↓                                                    │
│  Serialize to JSON                                         │
│       ↓                                                    │
│  Snapshot Qdrant collections                               │
│       ↓                                                    │
│  Create ZIP, encrypt with password                         │
│       ↓                                                    │
│  Save to user-selected location                            │
│                                                            │
│  Import Request                                            │
│       ↓                                                    │
│  Decrypt ZIP with password                                 │
│       ↓                                                    │
│  Parse manifest, check version                             │
│       ↓                                                    │
│  Detect conflicts                                          │
│       ↓                                                    │
│  Show conflict resolution UI                               │
│       ↓                                                    │
│  Apply changes, rebuild indexes                            │
└────────────────────────────────────────────────────────────┘
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Team roster gets stale | Periodic sync job for Slack/Google; manual refresh button; stale indicator (>30 days) |
| Export files are large | Selective export (exclude audit log, vectors); compression; progress indicator |
| Import corrupts data | Transaction-based import; automatic backup before import; rollback on failure |
| Pattern anonymization leaks PII | Review anonymization rules; exclude sensitive content flagged observations |
| Workload calculation expensive | Compute on schedule (not real-time); cache in team_members.workload_score |
| Skill clones diverge from original | Show "cloned from" badge; no automatic sync (explicit design choice) |
| Merge conflicts overwhelm user | Group by type; bulk actions; "smart merge" suggestions based on timestamps |

## Migration Plan

### Phase 1: Database & Core
1. Add `team_members` table (v016 migration)
2. Extend `pattern_models` with scope and contributor_count
3. Add `pattern_contributions` table
4. Extend `skills` table for sharing metadata

### Phase 2: Team Roster
1. Implement team roster repository and commands
2. Add Slack workspace member sync
3. Add Google Workspace member sync (if integration exists)
4. Build TeamSettings UI component

### Phase 3: Assignee Intelligence
1. Implement multi-factor scoring algorithm
2. Extend AssigneePicker component with suggestions
3. Add workload computation job
4. Learn weights from user corrections

### Phase 4: Export/Import
1. Implement export serialization and encryption
2. Implement import parsing and validation
3. Build conflict detection and resolution UI
4. Add Qdrant snapshot/restore support

### Phase 5: Shared Patterns & Skills
1. Implement pattern anonymization
2. Add contribution toggle in settings
3. Extend skill export for standalone sharing
4. Build skill import for single files

## Open Questions

1. **Workspace sync permissions**: Do Slack/Google integrations already have required scopes for member lists? May need additional OAuth scopes.

2. **Workload time window**: Should workload consider all open tasks or only recent (last 30 days)? Propose: configurable, default 30 days.

3. **Pattern anonymization depth**: How aggressively to anonymize? Propose: remove entity IDs, keep category keywords, hash unique strings.

4. **Export password vs device key**: Should export use user-provided password or device-derived key? Propose: user password (portable).

5. **Import version compatibility**: How many versions back should import support? Propose: current and previous major version.
