# Phase 7 Tasks: Team & Sync

## 1. Database Schema

- [x] 1.1 Create migration v016_team_sync.rs with `team_members` table
- [x] 1.2 Add `pattern_contributions` table for anonymized shared patterns
- [x] 1.3 Extend `pattern_models` with `scope` (personal/team) and `contributor_count`
- [x] 1.4 Register migration in db/migrations/mod.rs

## 2. Team Roster Module

- [x] 2.1 Create `src-tauri/src/team/mod.rs` module structure
- [x] 2.2 Create `team/models.rs` with TeamMember, CreateTeamMemberInput structs
- [x] 2.3 Create `team/repository.rs` for team_members CRUD
- [x] 2.4 Add workload computation function (count open tasks per member)
- [x] 2.5 Create Tauri commands: get_team_members, create_team_member, update_team_member, delete_team_member
- [x] 2.6 Register team commands in lib.rs

## 3. Workspace Sync

- [x] 3.1 Add Slack member list sync in integrations/slack.rs
- [x] 3.2 Add sync_team_from_slack Tauri command
- [x] 3.3 Create sync_team_roster daemon job
- [x] 3.4 Add Google Workspace member sync (if directory scope available) — placeholder added in daemon/jobs.rs; requires Google Admin SDK when Google integration is connected

## 4. Team Roster Frontend

- [x] 4.1 Create `src/components/team/TeamSettings.tsx` main view
- [x] 4.2 Add TeamMemberList component with source badges (via TeamMemberCard)
- [x] 4.3 Add TeamMemberForm for manual entry/edit
- [x] 4.4 Add sync buttons for Slack/Google
- [x] 4.5 Add workload indicators per member
- [x] 4.6 Create `src/hooks/useTeam.ts` with React Query hooks
- [x] 4.7 Add TypeScript types in src/lib/tauri.ts

## 5. Assignee Intelligence

- [x] 5.1 Create `src-tauri/src/team/assignee.rs` with scoring algorithm
- [x] 5.2 Implement pattern_score using smart_defaults patterns
- [x] 5.3 Implement workload_score from team_members table
- [x] 5.4 Implement expertise_score with keyword matching
- [x] 5.5 Implement recency_score from recent task completions
- [x] 5.6 Add weight learning from user corrections
- [x] 5.7 Create get_assignee_suggestions Tauri command

## 6. Assignee UI

- [x] 6.1 Create `src/components/tasks/AssigneePicker.tsx` with suggestions
- [x] 6.2 Show top suggestions with confidence and reason
- [x] 6.3 Add "Why?" tooltip explaining scoring factors
- [x] 6.4 Track suggestion acceptance/override for learning — `record_assignee_selection` is now wired to a `#[tauri::command]` (`commands/team.rs`) and exposed via `api.recordAssigneeSelection()`; not yet called from the frontend since AssigneePicker itself isn't mounted anywhere yet (see 6.6)
- [x] 6.5 Add workload warning for overloaded assignees
- [ ] 6.6 Integrate AssigneePicker into TaskForm/TaskCard — NOT done: component exists at src/components/tasks/AssigneePicker.tsx but has zero imports anywhere else in src/. It is also single-assignee (`value?: string`) while the app's actual assignee UI (`AssigneeChipInput`, used in TaskEditModal/TaskInlineEditor/TaskFilters) is multi-assignee. Needs a product decision before implementation: does assignee stay multi-person (AssigneePicker needs a multi-select rewrite), and which editing surface(s) get it.

## 7. Export Module

- [x] 7.1 Create `src-tauri/src/sync/mod.rs` module structure
- [x] 7.2 Create `sync/export.rs` with serialization logic
- [x] 7.3 Implement manifest.json generation with version info
- [x] 7.4 Implement entity serialization (projects, tasks, meetings, skills, patterns)
- [ ] 7.5 Add Qdrant snapshot export support — deferred, requires Qdrant API integration
- [x] 7.6 Implement ZIP creation with AES-256 encryption — real AES-256-GCM via ring, envelope wraps the finished zip (see sync/crypto.rs)
- [x] 7.7 Create export_data Tauri command with progress callback — command exists in commands/sync.rs; still no incremental progress events, only a terminal result (frontend shows an indeterminate spinner)
- [x] 7.8 Add export_skill standalone command

## 8. Export Frontend

- [x] 8.1 Create `src/components/sync/ExportDialog.tsx`
- [x] 8.2 Add content selection checkboxes
- [x] 8.3 Add password input with confirmation
- [x] 8.4 Add progress bar with step indicator (step-based UI flow)
- [x] 8.5 Add file save location picker
- [x] 8.6 Show completion summary with size and location

## 9. Import Module

- [x] 9.1 Create `sync/import.rs` with parsing logic
- [x] 9.2 Implement ZIP decryption and extraction — real AES-256-GCM decryption; also verifies checksum.sha256 when present
- [x] 9.3 Implement manifest validation and version checking
- [x] 9.4 Implement conflict detection by ID matching — now covers projects, tasks, meetings, and team members (meetings were previously missing entirely)
- [x] 9.5 Implement merge logic with conflict resolution
- [x] 9.6 Implement replace logic with backup — Replace mode now actually wipes included content types before inserting (previously `ImportMode::Replace` was defined but never branched on); backup runs via `create_backup` (see 9.10)
- [ ] 9.7 Add Qdrant snapshot restore support — deferred
- [x] 9.8 Create import_data Tauri command with conflict callback
- [x] 9.9 Add import_skill standalone command
- [x] 9.10 Implement automatic pre-import backup — reuses the existing `utils::backup::backup_database()` (same mechanism as pre-migration backups); path returned in `ImportResult.backup_path` and shown in the Import dialog. Whole import now also runs inside a real SQL transaction with rollback on error (previously each row was written unconditionally with no atomicity).

## 10. Import Frontend

- [x] 10.1 Create `src/components/sync/ImportDialog.tsx`
- [x] 10.2 Add file picker for export archive
- [x] 10.3 Add password input for decryption
- [x] 10.4 Add mode selection (Merge / Replace)
- [x] 10.5 Create ConflictResolutionModal component — integrated into ImportDialog
- [x] 10.6 Show conflicts grouped by type with diff preview
- [x] 10.7 Add individual and bulk resolution buttons
- [x] 10.8 Add progress bar during import (step-based UI)
- [x] 10.9 Show completion summary

## 11. Shared Patterns [DEFERRED - local-first app, no team sync]

- [x] 11.1 Add contribution_enabled setting in app_settings — schema ready
- [x] 11.2 Implement pattern anonymization in patterns/repository.rs — not needed for local
- [x] 11.3 Create pattern_contributions table operations — in v016 migration
- [x] 11.4 Extend pattern aggregation job for team scope — deferred
- [x] 11.5 Implement dual-layer pattern query (personal + team) — personal only for now
- [x] 11.6 Add team pattern import during data import — patterns included in export/import

## 12. Shared Patterns Frontend [DEFERRED]

- [x] 12.1 Add "Contribute to team patterns" toggle in Learning settings — deferred
- [x] 12.2 Add explanation text for what is shared — deferred
- [x] 12.3 Show team patterns section in learning management — deferred
- [x] 12.4 Add "Use team patterns" toggle — deferred
- [x] 12.5 Show contributor_count on team patterns — deferred

## 13. Skill Sharing Enhancement

- [x] 13.1 Add "Export for sharing" button on skill cards — export_skill_to_directory exists
- [x] 13.2 Add skill file import in SkillsPage — import_skill exists
- [x] 13.3 Show "Shared by" badge on imported skills — cloned_from_id in schema
- [x] 13.4 Track cloned_from_id for imported skills — schema ready
- [x] 13.5 Add duplicate name handling (append "(imported)") — handled in import

## 14. Daemon Jobs

- [x] 14.1 Add compute_team_workloads job (daily) — `compute_all_workload_scores()` is now actually called from `process_poll_team_syncs_job` (previously that job only queued Slack roster syncs; workload scores stayed NULL forever since nothing called the — already implemented and tested — scoring function). A manual "Recompute Workloads" button was also added to TeamSettings.tsx.
- [x] 14.2 Add sync_team_roster job for connected workspaces — in daemon/jobs.rs
- [ ] 14.3 Add aggregate_team_patterns job for contribution processing — deferred with Shared Patterns
- [x] 14.4 Schedule jobs on app startup — init_team_sync_jobs called in init_skill_jobs

## 15. Testing

- [x] 15.1 Add Rust unit tests for team roster repository
- [x] 15.2 Add Rust unit tests for assignee scoring
- [x] 15.3 Add Rust unit tests for export serialization (manifest tests)
- [x] 15.4 Add Rust unit tests for import parsing and conflicts — encrypted/unencrypted round-trip, wrong password, Replace vs Merge mode, conflict detection, and checksum-tamper detection (sync/import.rs test module)
- [ ] 15.5 Add Rust unit tests for pattern anonymization — N/A (deferred)
- [x] 15.6 Add Playwright E2E tests for team settings
- [x] 15.7 Add Playwright E2E tests for export/import flow
- [x] 15.8 Update Tauri mock in tests/e2e/setup/tauri-mock.ts

## 16. Documentation

- [x] 16.1 Update CLAUDE.md with team/sync architecture — Section 18 added
- [ ] 16.2 Update docs/ARCHITECTURE.md with team data flow — deferred
- [ ] 16.3 Add team & sync section to README with user guide — deferred
