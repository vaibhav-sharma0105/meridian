# Meridian Product Roadmap: Agentic Executive Assistant

> **Version**: 1.0 (Draft)  
> **Last Updated**: 2026-07-07  
> **Status**: Under Review

---

## Executive Summary

Transform Meridian from a meeting-to-task converter into a **proactive, learning, local-first executive assistant** that anticipates needs, drafts actions, and executes with appropriate human oversight — becoming the trusted proxy that knows your work context, respects your preferences, and gets things done.

**Target Users**: Product managers, engineering managers, team leads, and knowledge workers who want to automate the administrative overhead of meetings, tasks, and cross-tool coordination while maintaining full control and data privacy.

---

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Local-first, always** | Data lives on user's machine. SQLCipher encryption at rest. External calls only for AI inference (user's choice of provider) and explicit integrations. |
| **Progressive autonomy** | Start Manual, let users increase autonomy per action type. High-risk actions always require approval. |
| **Transparent intelligence** | User can always see WHY the agent made a decision. Full audit log with 2-year retention. Agent activity always visible in UI. |
| **Graceful degradation** | Works offline (local tasks, docs, Ollama AI). Works without integrations. Every feature is opt-in enhancement. |
| **Learn, don't configure** | System observes patterns and adapts. Configuration is fallback, not default. Per-category reset available. |
| **Team-ready, solo-friendly** | Architecture supports team collaboration, shared skills, permissions. Solo use is the graceful fallback. |
| **Security by default** | Always-encrypted storage, OS keychain for credentials, sensitive content flagging, audit trails. |

---

## Architecture Decisions (Based on Requirements)

### Core Systems

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Vector storage** | Qdrant embedded | Handles millions of vectors, Rust-native, proven at scale |
| **Encryption** | SQLCipher (always on) | User requirement: always encrypted at rest |
| **Embedded model** | Bundle MiniLM-L6 (~80MB) | Works without setup, user can switch to Ollama/API |
| **Credential storage** | App settings table | Avoids macOS permission prompts on unsigned builds |
| **Scheduled tasks** | Background daemon (primary), system scheduler (fallback), wake-on-launch (minimum) | User priority order |
| **Update strategy** | Prompt user + auto-install critical security patches | Balance control with security |

### Agent Behavior

| Decision | Choice |
|----------|--------|
| **Default autonomy** | Manual (with clear UI guide on changing) |
| **High-risk threshold** | Any persistent external change (write/delete on external systems) |
| **Approval timeout** | User-configurable, archives if timeout crossed (invocable later with warning) |
| **AI disclosure** | Always disclose at end of message ("Drafted by Meridian") — user can remove manually |
| **Failure handling** | Immediate user-friendly notification + detailed logs in advanced panel |
| **Negative learning** | Reduce frequency + occasionally ask "Should I stop suggesting this?" |

### Integrations

| Decision | Choice |
|----------|--------|
| **Slack mode** | Both options available: Bot account AND user token (user chooses in settings) |
| **GitHub/Jira access** | Both read AND write, user-configurable per integration |
| **MCP write access** | Yes, external agents can create/update tasks (user-configurable) |
| **Email** | Draft-only (visible in UI, copy to clipboard / mailto) |
| **Webhook support** | Yes, for real-time updates from external systems |

### Learning System

| Priority | What It Learns |
|----------|----------------|
| 1 | Workflow sequences (common task chains) |
| 2 | Communication style (tone, length, formality) |
| 3 | Priority/assignee patterns |
| 4 | Time-of-day work preferences |
| 5 | Task estimation accuracy |
| 6 | Rejection patterns (what to stop suggesting) |

Learning data is exportable/importable, supports per-category and master reset.

### Data & Limits

| Parameter | Value |
|-----------|-------|
| Max file upload | 25 MB |
| Default storage cap | 5 GB (user-adjustable based on disk) |
| Audit log retention | 2 years |
| Chat history retention | User-configurable |
| Proactive suggestions | Max per day (user-configurable) |

### UI/UX

| Decision | Choice |
|----------|--------|
| **Notification style** | Subtle (badge/panel) + toasts/sounds for severe items |
| **Configuration** | UI primary + JSON/YAML export/import for power users |
| **Skill editor** | Form-based primary, natural language as helper |
| **Agent activity** | Always visible in UI |
| **Feature visibility** | All features visible from start (no progressive disclosure hiding) |
| **Skill standards** | Follow Anthropic agent skills standard |

---

## Current State (From Audit)

### What's Built

| Feature | Status | Notes |
|---------|--------|-------|
| Documents upload/parsing | 80% | PDF/DOCX/PPTX/TXT/MD/CSV/VTT/SRT work. XLSX stubbed. |
| FTS5 keyword search | 100% | Fully functional |
| Embeddings | 60% | Ollama-only, manual trigger, no auto-embed |
| AI Chat | 100% | Multi-provider, context-aware (tasks/meetings/docs) |
| Zoom integration | 100% | OAuth + PKCE, syncs meetings/summaries/transcripts |
| Sheets Relay | 100% | Polls Apps Script for Gmail automation |
| MCP Server | 80% | Read-only (6 tools, 3 resources), needs write ops |
| Notifications | 30% | In-app only, desktop/email settings exist but not wired |

### What's Missing

| Feature | Status |
|---------|--------|
| GitHub integration | Not built |
| Jira integration | Not built |
| Slack integration | Not built (UI dropdown exists, no backend) |
| Background daemon | Not built |
| Scheduled jobs | Not built |
| Pattern learning | Not built |
| Skills/workflows | Not built |
| Desktop notifications | Plugin loaded, not wired |
| Encryption at rest | Not built (need SQLCipher) |
| Qdrant integration | Not built (currently sqlite-vec placeholder) |
| Bundled embedding model | Not built |
| Team/collaboration | Not built |
| Export/import sync | Not built |

---

## Phased Delivery Plan

### Phase 0: Foundation Hardening
**Theme**: Security, encryption, infrastructure  
**Duration**: 2-3 weeks  
**Prerequisite for everything else**

#### 0.1 SQLCipher Integration
- Replace rusqlite with sqlcipher
- Key derivation from user password or device key
- Migration path for existing unencrypted databases
- UI: Password setup during onboarding (or device-key option)

#### 0.2 Qdrant Embedded Integration
- Add qdrant-client with embedded mode
- Migrate from sqlite-vec to Qdrant
- Schema: collections per project + global collection
- Performance target: <100ms for 100K vectors

#### 0.3 Background Daemon Infrastructure
- Rust tokio-based background service
- IPC between main app and daemon (Unix socket / named pipe)
- Daemon lifecycle: start with app, optional "keep running" mode
- System scheduler integration (launchd plist / Windows Task Scheduler)
- Wake-on-launch fallback for missed jobs

#### 0.4 Audit Logging System
- `audit_log` table: timestamp, action_type, entity_type, entity_id, details (JSON), agent_initiated (bool)
- 2-year retention with auto-prune
- UI: Advanced panel showing filterable logs

---

### Phase 1: Intelligent Documents
**Theme**: Make documents actually useful for AI  
**Duration**: 3-4 weeks  
**Builds on**: Phase 0 (Qdrant)

#### 1.1 Bundled Embedding Model
- Ship MiniLM-L6-v2 (~80MB) with app
- ONNX runtime for cross-platform inference
- Settings UI: Choose between bundled / Ollama / OpenAI / Anthropic
- Auto-detect Ollama on startup

#### 1.2 Auto-Embed Pipeline
- On document upload: automatically queue for embedding
- Background job processes embedding queue
- Progress indicator in document card
- Retry logic for failures

#### 1.3 Multi-Provider Embeddings
- OpenAI `text-embedding-3-small` support
- Anthropic embeddings (when available)
- Ollama model selection
- Graceful fallback: bundled → Ollama → API

#### 1.4 Enhanced Semantic Search
- Hybrid retrieval: Qdrant similarity + FTS5 keyword
- RRF (Reciprocal Rank Fusion) for result merging
- Project-scoped + global document search
- AI chat automatically retrieves relevant docs

#### 1.5 Document Improvements
- XLSX parsing (proper implementation)
- Improved PDF extraction (use pdf-extract crate)
- Document preview in UI
- Bulk upload support

---

### Phase 2: Pattern Learning Engine
**Theme**: Meridian starts understanding you  
**Duration**: 3-4 weeks  
**Builds on**: Phase 1

#### 2.1 Observation Infrastructure
- `pattern_observations` table: type, observation (JSON), context, timestamp
- `pattern_models` table: aggregated/processed patterns per type
- Background job to process observations → models
- Export/import as JSON

#### 2.2 Workflow Sequence Learning (Priority 1)
- Track: task A completed → task B started patterns
- Track: meeting about X → tasks Y, Z created
- Suggest: "Last time after X, you did Y. Create similar tasks?"
- Confidence scoring based on repetition count

#### 2.3 Communication Style Learning (Priority 2)
- Analyze user edits to AI drafts
- Extract: tone (formal/casual), length preference, common phrases
- Apply learned style to future drafts
- UI: "Your communication style" summary in settings

#### 2.4 Priority/Assignee Pattern Learning (Priority 3)
- Track: task type X usually assigned to Y
- Track: task with keyword Z usually high priority
- Apply as defaults for new task creation
- Show confidence: "Suggesting high priority based on past patterns"

#### 2.5 Learning Management UI
- View all learned patterns by category
- Per-category reset button
- Master reset button
- Export/import learned data
- "Stop learning this" per pattern

---

### Phase 3: Proactive Agent Core
**Theme**: From reactive to proactive  
**Duration**: 4-5 weeks  
**Builds on**: Phase 2 (patterns inform suggestions)

#### 3.1 Suggestion Engine
- Background job analyzes current state
- Generates suggestions based on:
  - Overdue tasks
  - Stale tasks (no progress)
  - Upcoming meetings without agenda
  - Patterns (workflow sequences)
- Respects max suggestions/day limit
- Stores in `suggestions` table

#### 3.2 Suggestion UI
- New section in notification panel
- Each suggestion: summary, "Do it" button, "Dismiss" button, "Stop suggesting this" option
- Suggestion detail view with reasoning
- Toast for high-severity suggestions

#### 3.3 Draft Communications
- Task contains "follow up", "send", "share" → auto-generate draft
- Draft stored with task (not sent)
- UI: Draft tab in task detail
- Copy to clipboard, edit inline
- "Drafted by Meridian" signature (removable)

#### 3.4 Smart Task Plans
- On task creation: AI evaluates complexity
- Simple tasks: generate draft action/message
- Medium tasks: suggest subtasks
- Complex tasks: flag for human decomposition
- Plan stored with task, editable
- User corrections train the pattern learner

#### 3.5 Sensitive Content Detection
- Scan drafts for PII, financial data, credentials
- Flag with non-intrusive warning (not blocking)
- Log detections in audit log
- User can dismiss flag

---

### Phase 4: Skills & Automation
**Theme**: Repeatable intelligent workflows  
**Duration**: 5-6 weeks  
**Builds on**: Phase 3 (skills use suggestion + draft capabilities)

#### 4.1 Skill Engine Core
```typescript
interface Skill {
  id: string;
  name: string;
  description: string;
  
  trigger: {
    type: 'schedule' | 'event' | 'manual';
    schedule?: string;  // cron
    event?: 'task_created' | 'meeting_imported' | 'task_overdue' | ...;
  };
  
  context: {
    scope: 'project' | 'all';
    project_id?: string;
    include_documents: boolean;
    custom_instructions?: string;
  };
  
  action: {
    type: 'summarize' | 'draft_message' | 'create_tasks' | 'analyze' | 'custom';
    output_format: 'notification' | 'document' | 'draft_message' | 'tasks';
  };
  
  approval: {
    mode: 'always_ask' | 'auto_with_timeout' | 'confidence_based';
    timeout_seconds?: number;
  };
  
  shared: boolean;  // team visibility
  owner_id: string;
}
```

#### 4.2 Built-in Skills (Ship with App)
| Skill | Trigger | Action |
|-------|---------|--------|
| Weekly Summary | Monday 8am | Summarize completions + priorities |
| Meeting Follow-up | 24h post-meeting | Check task progress, draft reminder |
| Overdue Alert | Daily 9am | List overdue, suggest actions |
| Sprint Prep | Monday | Aggregate upcoming, flag blockers |
| End of Day | 6pm | Draft standup from today's activity |

#### 4.3 Skill Editor UI
- List view of all skills (personal + shared)
- Form-based editor (primary)
- Natural language helper: "Describe what you want..."
- Cron builder UI (no manual cron syntax needed)
- Test run button (dry run)
- Enable/disable toggle

#### 4.4 Chat-to-Skill Creation
- User describes workflow in AI chat
- AI parses → structured skill definition
- Shows preview for confirmation
- Creates skill, editable via form

#### 4.5 Skill Execution & History
- Background daemon runs scheduled skills
- Execution log per skill
- Output stored (notification, draft, tasks created)
- Retry on transient failure
- Respect autonomy settings for approval

#### 4.6 Skill Sharing (Team Feature)
- Mark skill as "shared"
- Team members see in their skill list
- Can clone and customize
- Original owner maintains master

#### 4.7 Skill Export/Import
- Export as JSON/YAML
- Import with validation
- Skill marketplace foundation (future)

---

### Phase 5: External Integrations
**Theme**: Connect to where work happens  
**Duration**: 6-8 weeks  
**Builds on**: Phase 4 (skills can trigger integration actions)

#### 5.1 Integration Framework
- `integrations` table: type, config (encrypted), permissions, status
- `integration_cache` table: cached external data
- Unified sync interface
- Per-integration autonomy settings
- Webhook receiver (local HTTP server with security token)

#### 5.2 GitHub Integration
- **Auth**: OAuth App or Personal Access Token
- **Read**: Issues assigned, PRs authored/reviewing, repo activity
- **Write**: Create issue, comment on PR, update issue status
- **Sync**: Incremental via `since` parameter
- **Link**: Meridian task ↔ GitHub issue bidirectional
- **UI**: GitHub section in task detail, connection settings

#### 5.3 Jira Integration
- **Auth**: Atlassian OAuth or API token
- **Read**: Issues assigned, sprint context, issue links
- **Write**: Create issue, update status, add comment
- **Sync**: JQL-based incremental sync
- **Link**: Meridian task ↔ Jira issue
- **MCP**: Leverage existing Atlassian Rovo MCP where possible

#### 5.4 Slack Integration
- **Auth**: Slack App with Socket Mode (no public endpoint)
- **Bot mode**: Sends as "Meridian Bot"
- **User token mode**: Sends as user (optional, user enables)
- **Read**: Monitor channels for action items
- **Write**: Send messages, post in channels
- **Autonomy levels** (per channel):
  1. Draft only (create task drafts from messages)
  2. Notify with approval (show pending messages)
  3. Time-delayed send (10 min default, user-configurable)
  4. Auto-send low-risk (based on confidence)
- **Proxy mode**: Act as user in specified channels
- **High-risk override**: Always approve for executives, external, first contact

#### 5.5 Desktop Notifications
- Wire up existing Tauri notification plugin
- Notification severity levels: info, warning, critical
- Critical: toast + sound
- Warning: toast
- Info: badge only
- User preferences in settings

#### 5.6 MCP Server Enhancement
- Add write operations: create_task, update_task, create_meeting_note
- Add skill operations: run_skill, list_skills
- Permission system: user configures what external agents can do
- Audit all MCP actions

---

### Phase 6: Autonomy & Governance
**Theme**: Trust through control  
**Duration**: 4-5 weeks  
**Builds on**: Phase 5 (governs all agent actions)

#### 6.1 Autonomy Controller
- Global autonomy mode: Manual / Supervised / Autonomous
- Per-integration override
- Per-skill override
- Per-action-type override

| Mode | Behavior |
|------|----------|
| Manual | Agent suggests, user executes everything |
| Supervised | Low-risk auto, medium asks, high-risk always asks |
| Autonomous | Low/medium auto, high-risk asks |

#### 6.2 Risk Classification Engine
- Classify every agent action
- Rules-based + learned (from user corrections)
- Categories: read, create, update, delete, external_send
- Destination risk: internal < team < external < executive
- Content risk: normal < sensitive < PII < financial

#### 6.3 Approval Flow
- Pending approvals queue
- Approval UI with context and reasoning
- Configurable timeout (default: 5 min)
- Timeout behavior: archive (retrievable with warning)
- Bulk approve/reject

#### 6.4 Undo System
- Last action: instant undo button
- Action history: scrollable list with selective undo
- Undo creates reversal (not true rollback)
- Some actions non-undoable (external sends) — marked clearly

#### 6.5 Governance Dashboard
- Agent activity summary
- Actions taken by autonomy level
- Approval rate / rejection rate
- Risk distribution
- Anomaly detection (unusual patterns)

---

### Phase 7: Team & Sync
**Theme**: Beyond single user  
**Duration**: 5-6 weeks  
**Builds on**: Phase 6

#### 7.1 Team Roster
- Manual team member entry
- Pull from Slack workspace
- Pull from Google Workspace (if connected)
- Role assignment (admin, member)
- Used for: assignee suggestions, permission, shared skills

#### 7.2 Assignee Intelligence
- Combine: past tasks + team roster + workspace
- Suggest based on: task type patterns, workload balance, expertise
- Show confidence and reasoning
- Learn from corrections

#### 7.3 Export/Import Sync
- Full export: tasks, projects, meetings, documents (metadata), skills, patterns, settings
- Format: encrypted ZIP with JSON + Qdrant snapshot
- Import: merge or replace options
- Conflict resolution UI for merge
- Foundation for future cloud sync (user's own storage)

#### 7.4 Shared Patterns (Team)
- Team-level pattern models (aggregate of members)
- Individual patterns remain personal
- Opt-in to contribute to team patterns
- Team skills use team patterns

---

### Phase 8: Advanced Intelligence
**Theme**: Meridian becomes indispensable  
**Duration**: Ongoing  
**Builds on**: All previous phases

#### 8.1 Cross-Project Intelligence
- "Task X in Project A blocked by Task Y in Project B"
- "3 meetings about same topic — consolidate?"
- "Quarterly velocity 20% lower — here's why"

#### 8.2 Predictive Actions
- Pre-fetch docs before meeting
- Draft agenda from open tasks + attendees
- Predict blockers before they happen
- Suggest reassignment based on workload

#### 8.3 Time-of-Day Optimization (Priority 4)
- Learn: user writes best in morning, meetings in afternoon
- Suggest: schedule focused work time
- Skills: adjust timing based on patterns

#### 8.4 Task Estimation (Priority 5)
- Learn: actual vs. estimated completion
- Suggest: "Similar tasks took 3 days, not 1"
- Trend: getting better or worse at estimation?

#### 8.5 Onboarding Expansion
- Expanded wizard for agentic features
- Interactive tour of autonomy settings
- "See Meridian in action" demo mode
- Gradual feature introduction (tooltips, not hiding)

#### 8.6 Usage Analytics Dashboard
- **Activity metrics**: Tasks completed over time, meetings processed, documents indexed
- **AI usage**: Tokens consumed (by provider), embeddings generated, chat interactions
- **Storage**: Database size, document storage, vector storage, breakdown by project
- **Productivity insights**: Tasks completed vs created, average completion time, overdue trends
- **Learning activity**: Patterns learned, suggestions accepted/dismissed, style adaptations
- **Export**: CSV/JSON export of usage data for personal analytics
- **Time range**: Daily, weekly, monthly, custom range views
- **Project breakdown**: Usage stats per project with comparison view

---

## Technical Architecture

### Database Schema (Additions)

```sql
-- Encrypted with SQLCipher

-- Audit logging
CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,
  timestamp TEXT NOT NULL,
  action_type TEXT NOT NULL,  -- 'create', 'update', 'delete', 'send', 'approve', 'reject'
  entity_type TEXT NOT NULL,  -- 'task', 'skill', 'message', 'integration'
  entity_id TEXT,
  details TEXT,  -- JSON
  agent_initiated BOOLEAN DEFAULT FALSE,
  autonomy_mode TEXT,
  risk_level TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Pattern learning
CREATE TABLE pattern_observations (
  id TEXT PRIMARY KEY,
  pattern_type TEXT NOT NULL,
  observation TEXT NOT NULL,  -- JSON
  context TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pattern_models (
  id TEXT PRIMARY KEY,
  pattern_type TEXT NOT NULL UNIQUE,
  model_data TEXT NOT NULL,  -- aggregated patterns
  confidence REAL DEFAULT 0.5,
  observation_count INTEGER DEFAULT 0,
  last_updated TEXT
);

-- Skills
CREATE TABLE skills (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  trigger_type TEXT NOT NULL,
  trigger_config TEXT,  -- JSON (cron, event type, etc.)
  context_config TEXT,  -- JSON
  action_config TEXT,  -- JSON
  approval_mode TEXT DEFAULT 'always_ask',
  approval_timeout INTEGER,
  enabled BOOLEAN DEFAULT TRUE,
  shared BOOLEAN DEFAULT FALSE,
  owner_id TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT
);

CREATE TABLE skill_runs (
  id TEXT PRIMARY KEY,
  skill_id TEXT NOT NULL REFERENCES skills(id),
  status TEXT NOT NULL,  -- 'pending', 'running', 'completed', 'failed', 'cancelled'
  output TEXT,  -- JSON
  error TEXT,
  started_at TEXT,
  completed_at TEXT
);

-- Suggestions
CREATE TABLE suggestions (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  reasoning TEXT,
  action_config TEXT,  -- JSON: what to do if accepted
  severity TEXT DEFAULT 'info',
  status TEXT DEFAULT 'pending',  -- 'pending', 'accepted', 'dismissed', 'expired'
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  acted_at TEXT
);

-- Integrations
CREATE TABLE integrations (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,  -- 'github', 'jira', 'slack'
  name TEXT NOT NULL,
  config TEXT NOT NULL,  -- encrypted JSON
  permissions TEXT,  -- JSON array
  autonomy_mode TEXT DEFAULT 'manual',
  status TEXT DEFAULT 'disconnected',
  last_sync TEXT,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE integration_cache (
  id TEXT PRIMARY KEY,
  integration_id TEXT NOT NULL REFERENCES integrations(id),
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  data TEXT NOT NULL,  -- JSON
  synced_at TEXT,
  UNIQUE(integration_id, entity_type, entity_id)
);

-- Pending approvals
CREATE TABLE pending_approvals (
  id TEXT PRIMARY KEY,
  action_type TEXT NOT NULL,
  action_config TEXT NOT NULL,  -- JSON
  context TEXT,  -- JSON
  risk_level TEXT NOT NULL,
  timeout_at TEXT,
  status TEXT DEFAULT 'pending',  -- 'pending', 'approved', 'rejected', 'archived'
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  resolved_at TEXT
);

-- Team
CREATE TABLE team_members (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT,
  source TEXT,  -- 'manual', 'slack', 'google'
  source_id TEXT,
  role TEXT DEFAULT 'member',
  metadata TEXT,  -- JSON
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Draft messages
CREATE TABLE draft_messages (
  id TEXT PRIMARY KEY,
  task_id TEXT REFERENCES tasks(id),
  channel TEXT NOT NULL,  -- 'slack', 'email'
  recipient TEXT,
  subject TEXT,
  body TEXT NOT NULL,
  ai_signature BOOLEAN DEFAULT TRUE,
  status TEXT DEFAULT 'draft',  -- 'draft', 'sent', 'archived'
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  sent_at TEXT
);
```

### File Structure (Additions)

```
~/.meridian/
├── meridian.db           # SQLCipher encrypted
├── qdrant/               # Qdrant embedded data
│   ├── collection_global/
│   └── collection_project_*/
├── models/
│   └── minilm-l6-v2.onnx # Bundled embedding model
├── exports/
│   └── [timestamped exports]
├── documents/
│   └── [hashed folders]
└── daemon.sock           # IPC socket for background daemon
```

### Daemon Architecture

```
┌─────────────────────────────────────────────┐
│              MERIDIAN MAIN APP              │
│  (Tauri + React)                            │
└─────────────────────┬───────────────────────┘
                      │ IPC (Unix socket)
                      ▼
┌─────────────────────────────────────────────┐
│            MERIDIAN DAEMON                  │
│  (Rust tokio service)                       │
├─────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐           │
│  │  Scheduler  │  │  Job Queue  │           │
│  │  (cron)     │  │  (embedding,│           │
│  │             │  │   sync, etc)│           │
│  └──────┬──────┘  └──────┬──────┘           │
│         │                │                   │
│         ▼                ▼                   │
│  ┌────────────────────────────────┐         │
│  │        Job Executor            │         │
│  │  - Skill runs                  │         │
│  │  - Integration syncs           │         │
│  │  - Embedding generation        │         │
│  │  - Pattern processing          │         │
│  └────────────────────────────────┘         │
│                    │                         │
│                    ▼                         │
│  ┌────────────────────────────────┐         │
│  │    Notification Emitter        │         │
│  │  - Desktop notifications       │         │
│  │  - IPC to main app             │         │
│  └────────────────────────────────┘         │
└─────────────────────────────────────────────┘
```

---

## UI/UX Guidelines

### Information Architecture

```
Sidebar
├── Projects (expandable)
│   └── [Project list]
├── Inbox (pending imports + suggestions)
├── Skills
├── Integrations
└── Settings
    ├── General
    ├── AI & Models
    ├── Autonomy
    ├── Learning (view/reset patterns)
    ├── Team
    ├── Integrations
    ├── Notifications
    ├── Export/Import
    └── Advanced (logs, daemon status)
```

### Design Principles

1. **Agent activity always visible** — Dedicated panel or status bar showing current agent state
2. **Autonomy clear at a glance** — Color-coded indicator of current mode
3. **Reasoning accessible** — Every AI action has expandable "Why?" section
4. **Approval flow non-blocking** — Badge count, not modal interrupts
5. **Progressive complexity** — Basic features prominent, advanced in dedicated sections
6. **Consistent patterns** — Same interaction patterns across all features

### Key Screens

| Screen | Purpose |
|--------|---------|
| **Inbox** | Pending imports, suggestions, approvals — single triage point |
| **Skill Editor** | Form-based with NL helper, cron builder, test run |
| **Autonomy Settings** | Visual diagram of what auto-runs vs. asks |
| **Learning Dashboard** | View patterns by category, reset controls |
| **Integration Hub** | Connect, configure permissions, view sync status |
| **Approval Queue** | Pending actions with context, bulk actions |
| **Activity Log** | Filterable audit log, export capability |

---

## Security Model

### Data Protection

| Layer | Mechanism |
|-------|-----------|
| **At rest** | SQLCipher (AES-256) for all databases |
| **Credentials** | App settings table (encrypted with DB) |
| **In transit** | HTTPS for all external API calls |
| **Vector data** | Qdrant embedded (encrypted with SQLCipher key) |

### Audit & Compliance

| Requirement | Implementation |
|-------------|----------------|
| **Action logging** | All agent actions logged with timestamp, context |
| **Retention** | 2 years, auto-prune |
| **Export** | Full audit log export for compliance |
| **Sensitive content** | Flagging (non-blocking), logged |

### Access Control

| Scope | Control |
|-------|---------|
| **MCP access** | User configures: read-only, read-write, or disabled |
| **Integration permissions** | Per-integration: what it can read/write |
| **Autonomy per channel** | Slack channels have individual autonomy settings |
| **Team roles** | Admin (full), Member (limited shared skill edit) |

---

## Performance Targets

| Operation | Target |
|-----------|--------|
| App startup | < 1s (daemon already running) |
| Document embedding (per page) | < 500ms |
| Semantic search (100K vectors) | < 100ms |
| Skill execution (simple) | < 2s |
| Pattern query | < 50ms |
| Full sync (all integrations) | < 30s |

---

## Success Metrics

### Quantitative

| Metric | Target |
|--------|--------|
| Tasks with AI-generated plans | 80%+ |
| Draft acceptance rate | 70%+ (with minor edits) |
| Skill automation rate | 50%+ of weekly admin tasks |
| Suggestion acceptance | 60%+ acted on (not dismissed) |
| Time saved per user | 5+ hours/week |

### Qualitative

- User says "Meridian knew what I needed"
- User trusts agent to act on low-risk items
- User spends less time on admin, more on actual work
- Adoption spreads via team skill sharing

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| AI hallucination in plans | Always cite sources, human approval for actions |
| Over-automation anxiety | Manual default, clear autonomy controls |
| Integration complexity | MCP-first, graceful degradation |
| Learning wrong patterns | Easy reset, transparent display, per-category control |
| Performance at scale | Qdrant for vectors, background daemon, lazy loading |
| Security breach | SQLCipher always-on, audit logging, no external data send without approval |
| Daemon resource usage | Configurable: "Keep running" vs "Launch on demand" |

---

## Open Questions for Implementation

1. **Team identity**: Where does "team" concept come from? Is there a team ID, or just shared device?
2. **Multi-device conflict**: If same user on two devices, how to merge learned patterns?
3. **Skill versioning**: When built-in skills update, how to handle user customizations?
4. **Webhook security**: Token-based? Signed payloads? What if user's IP changes?

---

## Next Steps

1. **Review this document** — Adjust priorities, scope, timing
2. **Create OpenSpecs** — Detailed spec per phase, starting with Phase 0
3. **Technical spikes** — Validate SQLCipher migration, Qdrant embedded, daemon IPC
4. **UI wireframes** — Key screens before implementation
5. **Implementation** — Phase by phase, feature-complete before next phase

---

## Appendix: Feature-to-Phase Mapping

| Feature | Phase |
|---------|-------|
| SQLCipher encryption | 0 |
| Qdrant integration | 0 |
| Background daemon | 0 |
| Audit logging | 0 |
| Bundled embedding model | 1 |
| Auto-embed documents | 1 |
| Multi-provider embeddings | 1 |
| Enhanced semantic search | 1 |
| Workflow sequence learning | 2 |
| Communication style learning | 2 |
| Pattern management UI | 2 |
| Suggestion engine | 3 |
| Draft communications | 3 |
| Smart task plans | 3 |
| Skill engine | 4 |
| Built-in skills | 4 |
| Chat-to-skill | 4 |
| Skill sharing | 4 |
| GitHub integration | 5 |
| Jira integration | 5 |
| Slack integration | 5 |
| Desktop notifications | 5 |
| MCP write operations | 5 |
| Autonomy controller | 6 |
| Risk classification | 6 |
| Approval flow | 6 |
| Undo system | 6 |
| Team roster | 7 |
| Assignee intelligence | 7 |
| Export/import sync | 7 |
| Cross-project intelligence | 8 |
| Predictive actions | 8 |
| Usage analytics dashboard | 8 |

