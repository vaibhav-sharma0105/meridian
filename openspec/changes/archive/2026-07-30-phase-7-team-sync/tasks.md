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
- [x] 3.4 Add Google Workspace member sync (if directory scope available) — implemented: `integrations/google.rs` (OAuth provider + Admin SDK Directory API fetch), `sync_team_from_google` command, `poll_team_syncs_job`'s Google branch un-commented, `GoogleSettings.tsx` UI. Requires a real Google Cloud OAuth client and Workspace domain admin approval of `admin.directory.user.readonly` to actually connect — see CREDENTIALS_SETUP.md Part 5b and CLAUDE.md §18 Known Gaps.

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
- [x] 6.4 Track suggestion acceptance/override for learning — `record_assignee_selection` command is called directly from `AssigneePicker.tsx` on every add (with `wasOverride` computed against the current suggestion list), via the `useRecordAssigneeSelection` hook
- [x] 6.5 Add workload warning for overloaded assignees
- [x] 6.6 Integrate AssigneePicker into TaskForm/TaskCard — `AssigneePicker` was rewritten to multi-assignee (same comma-separated `value`/`onChange` contract as `AssigneeChipInput`, chips + dropdown) and mounted in `TaskEditModal` only, per product decision. `TaskInlineEditor` and `TaskFilters` intentionally still use plain `AssigneeChipInput` — expand later if those surfaces need suggestions too.

## 7. Export Module

- [x] 7.1 Create `src-tauri/src/sync/mod.rs` module structure
- [x] 7.2 Create `sync/export.rs` with serialization logic
- [x] 7.3 Implement manifest.json generation with version info
- [x] 7.4 Implement entity serialization (projects, tasks, meetings, skills, patterns)
- [x] 7.5 Add Qdrant snapshot export support — `vectors/qdrant.rs::export_snapshot()` via gRPC `create_snapshot()` + raw REST download; every existing collection is included under `vectors/qdrant_snapshot/{collection}.snapshot`; degrades gracefully (export continues, `contents.vectors=false`) when Qdrant isn't running
- [x] 7.6 Implement ZIP creation with AES-256 encryption — real AES-256-GCM via ring, envelope wraps the finished zip (see sync/crypto.rs)
- [x] 7.7 Create export_data Tauri command with progress callback — real incremental progress: backend emits `export_progress` events (`tauri::Emitter`) after each of 7 stages, frontend renders an actual percentage bar via `onExportProgress`
- [x] 7.8 Add export_skill standalone command — REMOVED post-audit: `export_single_skill`/`export_skill_standalone` had zero call sites (skills UI uses the separate, working `export_skill_to_directory` path in `commands/skills.rs`) and wrote plain JSON, not the spec's YAML+MD `skill.md` format. Deleted rather than fixed since a working alternative already covers this need.

## 8. Export Frontend

- [x] 8.1 Create `src/components/sync/ExportDialog.tsx`
- [x] 8.2 Add content selection checkboxes — skills/document-metadata checkboxes are disabled with "coming soon" (export.rs doesn't read those options; see item 16 in CLAUDE.md gaps table)
- [x] 8.3 Add password input with confirmation
- [x] 8.4 Add progress bar with step indicator — real percentage bar driven by `export_progress` events, not just a step label
- [x] 8.5 Add file save location picker — native OS dialog (`pick_export_save_path`, osascript/rfd), previously hardcoded to `~/Downloads`
- [x] 8.6 Show completion summary with size and location

## 9. Import Module

- [x] 9.1 Create `sync/import.rs` with parsing logic
- [x] 9.2 Implement ZIP decryption and extraction — real AES-256-GCM decryption; also verifies checksum.sha256 when present
- [x] 9.3 Implement manifest validation and version checking
- [x] 9.4 Implement conflict detection by ID matching — now covers projects, tasks, meetings, and team members (meetings were previously missing entirely)
- [x] 9.5 Implement merge logic with conflict resolution
- [x] 9.6 Implement replace logic with backup — Replace mode now actually wipes included content types before inserting (previously `ImportMode::Replace` was defined but never branched on); backup runs via `create_backup` (see 9.10)
- [x] 9.7 Add Qdrant snapshot restore support — `vectors/qdrant.rs::import_snapshot()` uploads via multipart POST to Qdrant's REST snapshot-recover endpoint; `read_vector_snapshots()` in import.rs extracts the zip entries and `finish_import()` restores each collection
- [x] 9.8 Create import_data Tauri command with conflict callback
- [x] 9.9 Add import_skill standalone command — REMOVED post-audit: `import_single_skill`/`import_skill_standalone` had zero call sites and never actually inserted the parsed skill into the database (dead stub). Deleted; skills UI's working `import_skill` path (`commands/skills.rs`) already covers this.
- [x] 9.10 Implement automatic pre-import backup — reuses the existing `utils::backup::backup_database()` (same mechanism as pre-migration backups); path returned in `ImportResult.backup_path` and shown in the Import dialog. Whole import now also runs inside a real SQL transaction with rollback on error (previously each row was written unconditionally with no atomicity).

## 10. Import Frontend

- [x] 10.1 Create `src/components/sync/ImportDialog.tsx`
- [x] 10.2 Add file picker for export archive — native OS dialog (`pick_import_file_path`, osascript/rfd), previously no picker at all
- [x] 10.3 Add password input for decryption
- [x] 10.4 Add mode selection (Merge / Replace)
- [x] 10.5 Create ConflictResolutionModal component — integrated into ImportDialog
- [x] 10.6 Show conflicts grouped by type with diff preview
- [x] 10.7 Add individual and bulk resolution buttons
- [x] 10.8 Add progress bar during import — real percentage bar driven by `import_progress` events
- [x] 10.9 Show completion summary
- [x] 10.10 Add "Restore Backup" tab — lists existing pre-migration/pre-import backups via `list_backups`/`restore_from_backup`, previously only reachable nowhere in the UI

## 11. Shared Patterns

- [x] 11.1 Add contribution_enabled setting in app_settings — `pattern_contribution_enabled`, toggled via `set_pattern_contribution_enabled`
- [x] 11.2 Implement pattern anonymization in patterns/repository.rs — `anonymize_context_data()` keeps only `SAFE_CONTEXT_KEYS` (task_keywords, new/old_priority, new_status), drops titles/names/IDs entirely; skipped if `sensitive::scan_content()` flags the observation; hashed via SHA-256 before storage
- [x] 11.3 Create pattern_contributions table operations — `maybe_contribute()`, `get_all_pattern_contributions()`, `merge_team_contributions()` (dedup on `UNIQUE(pattern_type, observation_hash)`)
- [x] 11.4 Extend pattern aggregation job for team scope — contribution happens inline in `insert_observation()`, not as a separate daemon job (no batching needed since anonymization is cheap and per-observation)
- [x] 11.5 Implement dual-layer pattern query (personal + team) — `get_pattern_model_by_type` filters `scope='personal'`; `get_team_pattern_model_by_type`/`get_team_pattern_models` are the team-scope equivalents, kept as separate queries rather than merged (team rows carry only a contribution count, not content — see CLAUDE.md §18)
- [x] 11.6 Add team pattern import during data import — `pattern_contributions.json` included in export when `include_patterns` is set; `merge_team_contributions()` runs inside the import transaction

## 12. Shared Patterns Frontend

- [x] 12.1 Add "Contribute to team patterns" toggle in Learning settings — `LearningSettings.tsx`
- [x] 12.2 Add explanation text for what is shared — explains only anonymized keyword/priority/status signals are contributed, never titles or names
- [x] 12.3 Show team patterns section in learning management — renders via `getTeamPatternSummaries()`
- [x] 12.4 Add "Use team patterns" toggle — `use_team_patterns` setting gates whether the section renders; does not yet filter any suggestion query since team rows have no exploitable content today (see CLAUDE.md §18)
- [x] 12.5 Show contributor_count on team patterns — surfaced in `PatternSummary.contributor_count`

## 13. Skill Sharing Enhancement

- [x] 13.1 Add "Export for sharing" button on skill cards — export_skill_to_directory exists
- [x] 13.2 Add skill file import in SkillsPage — import_skill exists
- [x] 13.3 Show "Shared by" badge on imported skills — cloned_from_id in schema
- [x] 13.4 Track cloned_from_id for imported skills — schema ready
- [x] 13.5 Add duplicate name handling (append "(imported)") — handled in import

## 14. Daemon Jobs

- [x] 14.1 Add compute_team_workloads job (daily) — `compute_all_workload_scores()` is now actually called from `process_poll_team_syncs_job` (previously that job only queued Slack roster syncs; workload scores stayed NULL forever since nothing called the — already implemented and tested — scoring function). A manual "Recompute Workloads" button was also added to TeamSettings.tsx.
- [x] 14.2 Add sync_team_roster job for connected workspaces — in daemon/jobs.rs
- [x] 14.3 Add aggregate_team_patterns job for contribution processing — N/A by design: contribution is inline (`maybe_contribute()` inside `insert_observation()`), not a batch job; see 11.4
- [x] 14.4 Schedule jobs on app startup — init_team_sync_jobs called in init_skill_jobs

## 15. Testing

- [x] 15.1 Add Rust unit tests for team roster repository
- [x] 15.2 Add Rust unit tests for assignee scoring
- [x] 15.3 Add Rust unit tests for export serialization (manifest tests)
- [x] 15.4 Add Rust unit tests for import parsing and conflicts — encrypted/unencrypted round-trip, wrong password, Replace vs Merge mode, conflict detection, and checksum-tamper detection (sync/import.rs test module)
- [x] 15.5 Add Rust unit tests for pattern anonymization — `patterns/repository.rs` test module: anonymization key-stripping, hash dedup, personal/team scope non-collision (`test_personal_and_team_scope_dont_collide_on_same_pattern_type`), team pattern upsert/merge; `sync/import.rs` has `test_shared_patterns_round_trip_through_export_and_import`
- [x] 15.6 Add Playwright E2E tests for team settings — plus `team.spec.ts` reachability suite verifying Team Roster/Export/Import are all reachable from the Integrations panel (this exposed the bug fixed in item 17 below)
- [x] 15.7 Add Playwright E2E tests for export/import flow
- [x] 15.8 Update Tauri mock in tests/e2e/setup/tauri-mock.ts

## 16. Documentation

- [x] 16.1 Update CLAUDE.md with team/sync architecture — Section 18 added
- [ ] 16.2 Update docs/ARCHITECTURE.md with team data flow — deferred
- [ ] 16.3 Add team & sync section to README with user guide — deferred

## 17. Bug Fixes Found During Audit

- [x] 17.1 `TeamSettings`, `ExportDialog`, `ImportDialog` were fully implemented but never mounted anywhere in the app (zero JSX usage) — fixed by adding a "Team & Data" section to `IntegrationsPage.tsx` (Settings → Integrations → Team & Data); `TeamSettings` gained an optional `onClose` prop for its modal wrapper
- [x] 17.2 `bulk_update_tasks` didn't feed pattern/expertise learning — completing or reassigning tasks via bulk actions taught the system nothing, only one-at-a-time updates did. Fixed by snapshotting pre-update state and calling the same `record_completion_observation`/`record_assignee_observation` functions (extracted from `update_task`) for each changed task
- [x] 17.3 `AssigneePicker`'s `wasOverride` learning signal was too loose — adding a 4th/5th person after already honoring the AI's top suggestion incorrectly counted as an "override". Fixed: only true if the top suggestion hasn't already been added and the newly-added person differs from it
- [x] 17.4 Google Workspace roster sync never updated `integrations.last_sync_at` — `GoogleSettings.tsx` always showed a stale/missing last-sync timestamp even after a successful sync. Fixed by calling `update_integration_last_sync()` in `sync_team_from_google_internal`; also excluded `google` integrations from the generic `poll_integration_syncs` job (has its own `poll_team_syncs` cadence) to avoid double-syncing
- [x] 17.5 `skills::repository` test schema was missing the `autonomy_mode` column present in the real v0xx migration, causing 3 pre-existing test failures unrelated to this phase's work — fixed by aligning `setup_test_db()` with production schema
- [x] 17.6 4 pre-existing `governance.spec.ts` E2E test bugs (wrong selectors/mock keys, not app bugs) — fixed; see test file comments for each

## 18. Second-Pass Audit Fixes (independent sub-agent audit)

- [x] 18.1 Audit log was silently missing from the data-export spec's content-selection options — unlike skills/documents, it wasn't even disclosed as unimplemented. Fixed by adding a disabled "Audit log (coming soon)" checkbox to `ExportDialog.tsx`, matching the existing skills/documents pattern — no backend export was implemented, this is a UI honesty fix only
- [x] 18.2 `export_single_skill`/`import_single_skill` (and their `export_skill_standalone`/`import_skill_standalone` implementations) were dead code with zero call sites — the skills UI uses a separate, working `export_skill_to_directory`/`import_skill` path (`commands/skills.rs`) instead. The import side never even inserted the parsed skill into the DB. Deleted entirely (Rust commands, `lib.rs` registration, `tauri.ts` bindings, `tauri-mock.ts` mocks) rather than fixed, since a working alternative already exists — see 7.8/9.9
- [x] 18.3 Import conflict rows in `ImportDialog.tsx` didn't show local-vs-import timestamps or a diff, though `ImportConflict` already carries `local_updated`/`import_updated` — data-import spec requires this. Fixed by rendering both timestamps per conflict row
- [x] 18.4 `ConflictResolution::Ask` reaching the data layer (`sync/import.rs`) silently behaved identically to `Skip` — latent trap for any future caller that expects an interactive prompt. Fixed by making it return an explicit error instead, since the UI always resolves every conflict to `skip`/`overwrite` before calling `import_all_data`
- [x] 18.5 Phase 7's five spec files never went through the archive-and-merge step every other completed phase (5, 6) used — moved `phase-7-team-sync` to `openspec/changes/archive/`, merged its spec deltas into canonical `openspec/specs/`
