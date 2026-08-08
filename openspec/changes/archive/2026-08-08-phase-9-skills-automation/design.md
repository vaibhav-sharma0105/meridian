# Phase 9: Skills & Automation - Design Document

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend                                 │
├─────────────────────────────────────────────────────────────────┤
│  SkillImportWizard  │  SkillTrustSettings  │  ChatToSkillDialog │
│  FilePreviewCard    │  SyncDiffView        │  StarterTemplates  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri Commands                              │
├─────────────────────────────────────────────────────────────────┤
│  list_importable_skills  │  import_skill    │  sync_skill       │
│  execute_skill_sandboxed │  create_from_chat│  queue_skill      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Backend Modules                             │
├──────────────────┬──────────────────┬───────────────────────────┤
│  skills/sync.rs  │  skills/sandbox.rs│  skills/chat_extract.rs  │
│  - Git operations│  - Docker runner  │  - Pattern detection     │
│  - Fork-on-import│  - macOS sandbox  │  - Skill generation      │
│  - Update check  │  - Linux fallback │  - Clarification flow    │
└──────────────────┴──────────────────┴───────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Storage                                   │
├─────────────────────────────────────────────────────────────────┤
│  ~/.meridian/skills/{name}/     - Local skill files             │
│  ~/.meridian/created_files/     - Generated file outputs        │
│  ~/.meridian/cache/repos/       - Cached repo content           │
│  SQLite: skills table           - Skill metadata & trust state  │
└─────────────────────────────────────────────────────────────────┘
```

## Database Migration (v020)

```sql
-- Extend skills table for sync and trust
ALTER TABLE skills ADD COLUMN sync_source TEXT;
ALTER TABLE skills ADD COLUMN sync_path TEXT;
ALTER TABLE skills ADD COLUMN sync_commit TEXT;
ALTER TABLE skills ADD COLUMN trust_state TEXT DEFAULT 'untrusted';
ALTER TABLE skills ADD COLUMN trust_granted_at TEXT;
ALTER TABLE skills ADD COLUMN network_mode TEXT DEFAULT 'none';
ALTER TABLE skills ADD COLUMN network_allowlist TEXT;
ALTER TABLE skills ADD COLUMN last_sync_check TEXT;
ALTER TABLE skills ADD COLUMN content_hash TEXT;
ALTER TABLE skills ADD COLUMN source_conversation_id TEXT;

-- Skill execution outputs
CREATE TABLE IF NOT EXISTS skill_outputs (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    skill_run_id TEXT REFERENCES skill_runs(id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_skill_outputs_skill ON skill_outputs(skill_id, created_at DESC);

-- Skill execution queue for MCP
CREATE TABLE IF NOT EXISTS skill_queue (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    inputs TEXT,  -- JSON
    status TEXT DEFAULT 'pending',  -- pending, running, completed, failed
    result TEXT,  -- JSON
    error TEXT,
    queued_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX idx_skill_queue_status ON skill_queue(status, queued_at);
```

## Module Structure

### src-tauri/src/skills/sync.rs

```rust
pub struct ImportableSkill {
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub size_bytes: u64,
    pub has_scripts: bool,
}

pub async fn list_importable_skills(
    conn: &Connection,
    integration_id: &str,
) -> Result<Vec<ImportableSkill>, String>;

pub async fn import_skill(
    conn: &Connection,
    integration_id: &str,
    skill_path: &str,
    local_name: Option<&str>,
) -> Result<Skill, String>;

pub async fn check_for_updates(
    conn: &Connection,
    skill_id: &str,
) -> Result<UpdateStatus, String>;

pub async fn sync_skill(
    conn: &Connection,
    skill_id: &str,
    strategy: SyncStrategy,
) -> Result<Skill, String>;
```

### src-tauri/src/skills/sandbox.rs

```rust
pub enum SandboxBackend {
    Docker,
    MacOSSandbox,
    Firejail,
    Bubblewrap,
    ProcessIsolation,
}

pub struct SandboxConfig {
    pub timeout_secs: u64,
    pub memory_mb: u64,
    pub network_mode: NetworkMode,
    pub network_allowlist: Vec<String>,
}

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub output_files: Vec<OutputFile>,
    pub duration_ms: u64,
}

pub fn detect_backend() -> SandboxBackend;

pub async fn execute_in_sandbox(
    skill_path: &Path,
    script_name: &str,
    inputs: &Value,
    config: &SandboxConfig,
) -> Result<ExecutionResult, String>;
```

### src-tauri/src/skills/chat_extract.rs

```rust
pub struct PatternDetection {
    pub confidence: f64,
    pub trigger_type: String,
    pub detected_inputs: Vec<DetectedInput>,
    pub output_format: String,
}

pub struct DetectedInput {
    pub name: String,
    pub inferred_type: String,
    pub example: Option<String>,
}

pub async fn detect_pattern(
    conversation: &[ChatMessage],
    litellm: &LiteLLMClient,
) -> Result<Option<PatternDetection>, String>;

pub async fn generate_skill_from_chat(
    conversation: &[ChatMessage],
    clarifications: &ClarificationAnswers,
    litellm: &LiteLLMClient,
) -> Result<String, String>;  // Returns skill.md content
```

## Frontend Components

### SkillImportWizard

Location: `src/components/skills/SkillImportWizard.tsx`

Steps:
1. Select connected repo
2. Browse available skills (tree view)
3. Review skill details (size, has scripts, permissions)
4. Confirm import (with rename if conflict)

### SkillTrustSettings

Location: `src/components/settings/SkillTrustSettings.tsx`

Features:
- List all skills with trust state badge
- One-click revoke
- View permission details (network mode, etc.)
- Trust history log

### ChatToSkillDialog

Location: `src/components/chat/ChatToSkillDialog.tsx`

Wizard flow:
1. Show detected pattern with confidence
2. Ask clarification questions (trigger, scope, inputs, output)
3. Show generated skill preview (editable)
4. Save or cancel

### FilePreviewCard

Location: `src/components/chat/FilePreviewCard.tsx`

For generated files in chat:
- File icon based on type
- File name and size
- Preview button (images, text, PDF)
- Download button

## Starter Templates

Built-in skills created on first run:

1. **Repo Watch** - Monitor repository for changes
2. **Weekly Status Report** - Generate status from tasks/meetings
3. **Meeting Prep** - Prepare agenda from tasks and notes
4. **Task Cleanup** - Archive old completed tasks

Location: `src-tauri/src/skills/templates/`

## MCP Tools

Add to `meridian-mcp/src/handlers.rs`:

```rust
// Queue a skill for execution
Tool {
    name: "queue_skill",
    description: "Queue a skill for background execution",
    parameters: json!({
        "skill_id": "string",
        "inputs": "object (optional)"
    }),
}

// Get queued skill result
Tool {
    name: "get_skill_result",
    description: "Get the result of a queued skill execution",
    parameters: json!({
        "execution_id": "string"
    }),
}

// Bulk create tasks (useful for skills)
Tool {
    name: "bulk_create_tasks",
    description: "Create multiple tasks at once",
    parameters: json!({
        "project_id": "string",
        "tasks": "array of task objects"
    }),
}
```

## Security Considerations

1. **Trust boundary**: Untrusted skills cannot execute scripts
2. **Network isolation**: Default to no network; allowlist requires explicit domains
3. **File system isolation**: Scripts can only write to `/output` directory
4. **Resource limits**: Prevent runaway processes with timeouts and memory caps
5. **Content hashing**: Detect changes and auto-revoke trust
6. **Size limits**: Prevent large imports that could exhaust disk space
