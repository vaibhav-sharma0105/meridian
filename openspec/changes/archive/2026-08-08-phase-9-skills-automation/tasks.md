# Phase 9: Skills & Automation - Tasks

## 1. Database Migration

- [x] 1.1 Create v020_skills_automation.rs migration with sync/trust columns on skills table
- [x] 1.2 Create skill_outputs table for generated file tracking
- [x] 1.3 Create skill_queue table for MCP async execution
- [x] 1.4 Register migration in mod.rs and test

## 2. Skill Sync Backend

- [x] 2.1 Create src-tauri/src/skills/sync.rs module with ImportableSkill struct
- [x] 2.2 Implement list_importable_skills() - scan repo skill directories via GitHub API
- [x] 2.3 Implement import_skill() - download and save to ~/.meridian/skills/{name}/
- [x] 2.4 Implement check_for_updates() - compare local commit SHA with remote HEAD
- [x] 2.5 Implement sync_skill() with keep_local/use_remote/manual strategies
- [x] 2.6 Add content_hash computation for trust revocation detection
- [x] 2.7 Add Tauri commands for sync operations in commands/skills.rs

## 3. Sandbox Execution Backend

- [x] 3.1 Create src-tauri/src/skills/sandbox.rs with SandboxBackend enum
- [x] 3.2 Implement detect_backend() - check Docker, sandbox-exec, firejail availability
- [x] 3.3 Implement Docker execution with resource limits and network modes
- [x] 3.4 Implement macOS sandbox-exec fallback with deny-network profile
- [x] 3.5 Implement Linux firejail/bubblewrap fallback
- [x] 3.6 Implement process isolation fallback with restricted environment
- [x] 3.7 Implement output file capture and move to ~/.meridian/created_files/
- [x] 3.8 Add execute_skill_sandboxed Tauri command

## 4. Trust Model

- [x] 4.1 Add trust state management functions in skills/repository.rs
- [x] 4.2 Implement auto-revocation on content_hash change
- [x] 4.3 Implement network mode escalation detection
- [x] 4.4 Add grant_trust and revoke_trust Tauri commands
- [x] 4.5 Add trust check before sandbox execution

## 5. Chat-to-Skill Backend

- [x] 5.1 Create src-tauri/src/skills/chat_extract.rs module
- [x] 5.2 Implement detect_pattern() with LLM-based analysis
- [x] 5.3 Implement generate_skill_from_chat() to create skill.md content
- [x] 5.4 Add create_skill_from_conversation Tauri command
- [x] 5.5 Store source_conversation_id for traceability

## 6. Multi-Skill Execution

- [x] 6.1 Add skill matching with embedding similarity in executor.rs
- [x] 6.2 Implement dependency resolution (topological sort on depends_on)
- [x] 6.3 Implement skill chaining with output piping
- [x] 6.4 Add confidence thresholds and fallback behavior
- [x] 6.5 Update execute_skill to support multi-skill chains

## 7. MCP Tools

- [x] 7.1 Add queue_skill tool handler in meridian-mcp
- [x] 7.2 Add get_skill_result tool handler
- [x] 7.3 Add bulk_create_tasks tool handler
- [x] 7.4 Implement background queue processing

## 8. Frontend - Skill Import

- [x] 8.1 Create SkillImportWizard.tsx component with repo selection
- [x] 8.2 Add skill tree browser with size and script indicators
- [x] 8.3 Add conflict detection and rename dialog
- [x] 8.4 Add import confirmation with permission review
- [x] 8.5 Add useSkillSync hook for import operations

## 9. Frontend - Trust Settings

- [x] 9.1 Create SkillTrustSettings.tsx in settings
- [x] 9.2 Add skill list with trust state badges
- [x] 9.3 Add one-click revoke functionality
- [x] 9.4 Add permission details view (network mode, allowlist)

## 10. Frontend - Chat Integration

- [x] 10.1 Create ChatToSkillDialog.tsx with wizard steps
- [x] 10.2 Add inline skill creation suggestion in chat
- [x] 10.3 Create FilePreviewCard.tsx for generated files
- [x] 10.4 Add download functionality for skill outputs
- [x] 10.5 Add skill execution progress indicators in chat

## 11. Frontend - Sync UI

- [x] 11.1 Add SyncDiffView.tsx for update previews
- [x] 11.2 Add "Update available" badge on skills list
- [x] 11.3 Add "Sync now" button with conflict resolution options
- [x] 11.4 Add sync status indicators per skill

## 12. Starter Templates

- [x] 12.1 Create repo-watch template skill
- [x] 12.2 Create weekly-status-report template skill
- [x] 12.3 Create meeting-prep template skill
- [x] 12.4 Create task-cleanup template skill
- [x] 12.5 Add template installation on first run
- [x] 12.6 Add "Import from templates" option in Skills page

## 13. Testing

- [x] 13.1 Add unit tests for sync operations
- [x] 13.2 Add unit tests for sandbox execution
- [x] 13.3 Add unit tests for trust model
- [x] 13.4 Add E2E tests for skill import flow
- [x] 13.5 Add E2E tests for skill execution
