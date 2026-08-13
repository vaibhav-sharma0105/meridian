# Phase 10: Message Center & Role - Implementation Tasks

## Database Migration

- [x] Create v021 migration with `message_center` table schema
- [x] Create `user_profile` table for role scores and productivity patterns
- [x] Extend `pattern_observations` with role_signal, completion_hour, completion_day_of_week, task_category columns
- [x] Add indexes for message_center, role, and productivity queries

## Backend: Message Center Module

- [x] Create `src-tauri/src/messages/mod.rs` module structure
- [x] Implement `messages/models.rs` with Message, MessageType, RoutingDecision structs
- [x] Implement `messages/repository.rs` with CRUD operations
- [x] Implement `messages/routing.rs` with route_content() decision logic
- [x] Implement `messages/retention.rs` with cleanup_expired_messages() daemon job
- [x] Implement `messages/storage.rs` with calculate_storage_usage() and file reference tracking
- [x] Implement `archive_old_files` setting — `messages/archive.rs` zips `created_files/YYYY-MM-DD/` dirs older than `archive_after_days` into `created_files/archive/{date}.zip`; opt-in (v023, default off), runs inside the `cleanup_messages` job, refuses to clobber an existing archive

## Backend: Message Center Commands

- [x] Add `get_messages` command with filters and pagination
- [x] Add `pin_message` command for manual pinning from AI chat
- [x] Add `delete_message` command (soft-delete)
- [x] Add `get_storage_stats` command
- [x] Register all message commands in lib.rs

## Backend: Role Inference Module

- [x] Create `src-tauri/src/role/mod.rs` module structure
- [x] Implement `role/models.rs` with RoleScores, RoleClassification, InferenceStatus
- [x] Implement `role/inference.rs` with compute_role_scores() and classify_role()
- [x] Implement `role/drift.rs` with detect_role_drift()
- [x] Implement `role/repository.rs` for user_profile CRUD

## Backend: Role Commands

- [x] Add `get_user_profile` command
- [x] Add `confirm_role` command
- [x] Add `change_role` command
- [x] Add `get_role_inference_status` command
- [x] Add `dismiss_role_drift_alert` command
- [x] Register all role commands in lib.rs

## Backend: Productivity Patterns Module

- [x] Create `src-tauri/src/productivity/mod.rs` module structure
- [x] Implement `productivity/models.rs` with ProductivityPatterns, TimeSuggestion
- [x] Implement `productivity/patterns.rs` with aggregate_patterns() and get_effective_peak_hours()
- [x] Implement `productivity/suggestions.rs` with suggest_task_time()

## Backend: Productivity Commands

- [x] Add `get_productivity_insights` command
- [x] Add `get_time_suggestion` command
- [x] Add `update_productivity_settings` command
- [x] Add `export_productivity_data` command
- [x] Add `clear_productivity_data` command
- [x] Register all productivity commands in lib.rs

## Backend: Daemon Jobs

- [x] Add `cleanup_messages` job to daemon/jobs.rs (daily at 4 AM)
- [x] Add `infer_role` job to daemon/jobs.rs (daily)
- [x] Add `aggregate_productivity` job to daemon/jobs.rs (every 6 hours)
- [x] Add `generate_digest` job to daemon/jobs.rs (daily at 6 AM UTC)
- [x] Initialize jobs in daemon/worker.rs

## Backend: Integration Points

- [x] Hook message creation into skill execution (skill_runs completion)
- [x] Hook message creation into AI chat (pin_message from chat; sandboxed skill file output auto-pins with file_refs)
- [x] Hook role observation recording into task create/update
- [x] Hook productivity observation recording into task completion
- [x] Wire Message Center into AI chat context (dual retention model — `build_project_context_full`)
- [x] Wire role drift detection into `infer_role` daemon job + `get_role_drift_alert` command
- [x] Create `integration_sync` messages when a sync brings in new items
- [x] Expose `suggest_meeting_batching` as `get_meeting_batching_suggestion`
- [x] Update suggestions weighting based on user role — `role/weighting.rs` applied in `get_pending_suggestions`; only `overdue_task` and `meeting_followup` map to the spec's table (its other two rows have no producer), everything else weighs 1.0. Spec scenarios added to role-inference under Role-Based Personalization.
- [x] Digest producer — `messages/digest.rs` (stats + markdown) and the daily `generate_digest` daemon job (6 AM UTC, after the 4 AM cleanup so a digest is never swept immediately); skips empty windows rather than posting a blank digest
- [x] Send a notification with a "View full result" deep link when a message is created — `notifications.message_id` (v022), `create_notification_for_message()`, wired into `complete_skill_run` (previously created no notification at all) and the integration-sync job (previously created an unlinked one); NotificationCenter renders the link and `openMessageCenter(messageId)` scrolls to and highlights the target message

## Frontend: TypeScript API

- [x] Add Message types and API functions to tauri.ts
- [x] Add UserProfile types and API functions to tauri.ts
- [x] Add ProductivityPatterns types and API functions to tauri.ts
- [x] Add event listeners for role_drift_alert

## Frontend: Message Center

- [x] Create `src/hooks/useMessages.ts` with React Query hooks
- [x] Create `src/components/messages/MessageCenterView.tsx` sidebar view
- [x] Create `src/components/messages/MessageCard.tsx` for individual messages
- [x] Create `src/components/messages/MessageFilters.tsx` for type/search filtering
- [x] Create `src/components/messages/StorageUsageBar.tsx` indicator
- [x] Add Message Center icon to sidebar navigation
- [x] Add "Pin to Message Center" button to AI chat responses

## Frontend: Role

- [x] Create `src/hooks/useRole.ts` with React Query hooks
- [x] Create `src/components/role/RoleConfirmation.tsx` modal wizard
- [x] Create `src/components/role/RoleIndicator.tsx` badge with tooltip
- [x] Create `src/components/role/RoleDriftAlert.tsx` notification component
- [x] Add RoleIndicator to MyActivityDashboard header
- [x] Add user identity (`display_name`/`user_email`/`user_aliases` on `user_profile`, v022) + `update_user_identity` command + `IdentitySettings` panel — prerequisite the spec assumed but never defined; without it "my items" vs "team items" is undecidable
- [x] Implement role-based ordering in MyActivityDashboard (`role/ordering.rs`, applied in `get_attention_items`; severity stays primary, role rule breaks ties, falls back to severity+recency when identity is unset or role is pm/unconfirmed)

## Frontend: Productivity

- [x] Create `src/hooks/useProductivity.ts` with React Query hooks
- [x] Create `src/components/productivity/ProductivityInsights.tsx` settings panel
- [x] Create `src/components/productivity/TimeSuggestion.tsx` inline component
- [x] Add ProductivityInsights to Settings page
- [x] Add TimeSuggestion to task creation flow (quick-add row in `TaskListView`, below the smart-defaults hint)
- [x] Add meeting-batching UI consuming `get_meeting_batching_suggestion` (`MeetingBatchingSuggestion`, above the meetings list; renders nothing when the day is not fragmented)

## MCP Tools

- [x] Add `create_report` tool to meridian-mcp (writes a `digest` message + linked notification; gated on the new `create_report` MCP permission)
- [x] Add `get_reports` tool to meridian-mcp (reads back `source_type = 'mcp'` messages only, so agents don't get unrelated skill/sync traffic)
- [x] Add `draft_message` tool to meridian-mcp (parks a `pinned_chat` draft for review, never sends; returns `sent: false`)

## Testing

- [x] Add unit tests for message routing logic (10 tests in routing.rs)
- [x] Add unit tests for role inference scoring (7 tests in inference.rs, 4 in drift.rs)
- [x] Add unit tests for productivity aggregation (6 tests in patterns.rs)
- [x] Add unit tests for Message Center → AI context (9 tests in extractor.rs, incl. UTF-8 truncation)
- [x] Add frontend/backend command contract test (`src-tauri/tests/command_contract.rs`)
- [x] Add E2E tests for Message Center UI (13 tests, message-center.spec.ts)
- [x] Add E2E tests for role confirmation flow (9 tests, role-inference.spec.ts)
- [x] Add E2E tests for the notification deep link + identity settings (5 tests, message-deep-link.spec.ts)
- [x] Add unit tests for role ordering (9 tests in role/ordering.rs) and digest rendering (4 tests in messages/digest.rs)
- [x] Add unit tests for suggestion weighting (6 tests in role/weighting.rs) and file archival (6 tests in messages/archive.rs)
- [x] Add `src-tauri/tests/migration_contract.rs` (6 tests) — nothing previously executed migration SQL, so a broken migration only surfaced as a blank window at startup
- [x] Add `mountWithMocks` fixture helper for per-test mock overrides
- [x] Add Phase 10 mocks to tests/e2e/setup/tauri-mock.ts

## Documentation

- [x] Update CLAUDE.md (Section 20, dual retention model, gotchas)
- [x] Update docs/ARCHITECTURE.md (data flows, v021 schema, contract enforcement)
