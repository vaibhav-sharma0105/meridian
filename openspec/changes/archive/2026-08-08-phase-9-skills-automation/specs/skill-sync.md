# Skill Sync Specification

## Overview

Import skills from connected GitHub repositories' `.claude/skills/` or `.agents/skills/` directories into Meridian's local skill storage.

## Data Model

### Database Schema Extensions (skills table)

```sql
ALTER TABLE skills ADD COLUMN sync_source TEXT;        -- "github:{owner}/{repo}"
ALTER TABLE skills ADD COLUMN sync_path TEXT;          -- path within repo
ALTER TABLE skills ADD COLUMN sync_commit TEXT;        -- last synced commit SHA
ALTER TABLE skills ADD COLUMN trust_state TEXT DEFAULT 'untrusted';  -- "untrusted"|"trusted"|"revoked"
ALTER TABLE skills ADD COLUMN trust_granted_at TEXT;
ALTER TABLE skills ADD COLUMN network_mode TEXT DEFAULT 'none';  -- "none"|"allowlist"|"full"
ALTER TABLE skills ADD COLUMN network_allowlist TEXT;  -- JSON array of allowed domains
ALTER TABLE skills ADD COLUMN last_sync_check TEXT;
ALTER TABLE skills ADD COLUMN content_hash TEXT;       -- SHA256 of skill content for change detection
```

### Skill Package Structure

A skill package in a repo may contain:
```
.claude/skills/my-skill/
├── skill.md          # Main skill file (Anthropic format)
├── script.py         # Optional executable script
├── template.hbs      # Optional Handlebars template
├── assets/           # Optional assets directory
│   └── ...
└── config.json       # Optional configuration
```

## API Endpoints

### List Importable Skills

```rust
#[tauri::command]
pub async fn list_importable_skills(
    integration_id: String,  // GitHub integration ID
    state: State<AppState>,
) -> Result<Vec<ImportableSkill>, String>
```

Returns skills found in the connected repo's skill directories.

### Import Skill

```rust
#[tauri::command]
pub async fn import_skill(
    integration_id: String,
    skill_path: String,      // Path in repo
    local_name: Option<String>,  // Rename on import
    state: State<AppState>,
) -> Result<Skill, String>
```

Copies skill to `~/.meridian/skills/{name}/` with fork-on-import semantics.

### Check for Updates

```rust
#[tauri::command]
pub async fn check_skill_updates(
    skill_id: String,
    state: State<AppState>,
) -> Result<SkillUpdateStatus, String>
```

Returns:
- `up_to_date`: Local matches remote
- `update_available`: Remote has newer commit
- `local_modified`: Local changes detected
- `conflict`: Both local and remote changed

### Sync Skill

```rust
#[tauri::command]
pub async fn sync_skill(
    skill_id: String,
    strategy: SyncStrategy,  // "keep_local" | "use_remote" | "manual"
    state: State<AppState>,
) -> Result<Skill, String>
```

## Git Operations

### Authentication
- Reuse OAuth token from connected GitHub integration
- If integration not connected, return error prompting connection

### Clone/Fetch Strategy
- Use sparse checkout to fetch only skill directories
- Cache fetched content in `~/.meridian/cache/repos/{owner}/{repo}/`
- TTL: 5 minutes for cached repo state

## Size Limits

| Threshold | Action |
|-----------|--------|
| < 10MB | Import normally |
| 10-50MB | Show warning, require confirmation |
| > 50MB | Block import with error |

## Naming Conflicts

When importing a skill with a name that already exists:
1. Block the import
2. Prompt user to rename via UI
3. Store original name in `sync_path` for update tracking
