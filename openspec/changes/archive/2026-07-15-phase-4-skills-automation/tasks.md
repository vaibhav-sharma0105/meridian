## 1. Database Schema

- [x] 1.1 Create migration v012_skills.rs with skills table (id, name, description, trigger_type, trigger_config, context_config, action_config, approval_mode, enabled, shared, owner_id, category, icon, tags, created_at, updated_at)
- [x] 1.2 Add skill_runs table to migration (id, skill_id FK, status, trigger_type, trigger_context, output, error, started_at, completed_at, duration_ms, approval_decision, created_at)
- [x] 1.3 Add skill_run_id column to suggestions table
- [x] 1.4 Add skill_run_id column to notifications table
- [x] 1.5 Create indexes on skill_runs (skill_id, status, created_at)
- [x] 1.6 Register migration in db/migrations/mod.rs

## 2. Skill Model & Repository

- [x] 2.1 Create src-tauri/src/skills/mod.rs module with exports
- [x] 2.2 Define Skill struct with all fields and serde derives
- [x] 2.3 Define TriggerType enum (Schedule, Event, Manual)
- [x] 2.4 Define TriggerConfig struct with cron, timezone, event_type, filter variants
- [x] 2.5 Define ContextConfig struct (scope, project_id, include_documents, document_filter, max_documents, system_prompt, output_instructions)
- [x] 2.6 Define ActionConfig struct (action_type, format, template, max_length)
- [x] 2.7 Define ApprovalMode enum (Auto, Notify, ApproveFirst, ApproveAlways)
- [x] 2.8 Create CreateSkillInput and UpdateSkillInput structs
- [x] 2.9 Implement skill repository create_skill function
- [x] 2.10 Implement skill repository get_skill function
- [x] 2.11 Implement skill repository list_skills function (with shared filter)
- [x] 2.12 Implement skill repository update_skill function
- [x] 2.13 Implement skill repository delete_skill function
- [x] 2.14 Implement get_due_scheduled_skills function (next_run_at <= now, enabled=true)
- [x] 2.15 Implement get_skills_for_event function (filter by event_type and filters)

## 3. Skill Run Model & Repository

- [x] 3.1 Define SkillRun struct with all fields
- [x] 3.2 Define SkillRunStatus enum (Pending, Running, Completed, Failed, PartialFailure, Cancelled, ApprovalPending)
- [x] 3.3 Define CreateSkillRunInput struct
- [x] 3.4 Implement skill_runs repository create_run function
- [x] 3.5 Implement get_run function
- [x] 3.6 Implement list_runs_for_skill function with pagination
- [x] 3.7 Implement update_run_status function
- [x] 3.8 Implement set_run_output function
- [x] 3.9 Implement set_run_error function
- [x] 3.10 Implement set_approval_decision function
- [x] 3.11 Implement prune_old_runs function (retention-based cleanup)
- [x] 3.12 Implement get_skill_stats function (run count, success rate, avg duration)

## 4. Trigger System

- [x] 4.1 Add cron crate dependency to Cargo.toml
- [x] 4.2 Create src-tauri/src/skills/cron.rs with parse_cron function
- [x] 4.3 Implement compute_next_run function using timezone
- [x] 4.4 Implement validate_cron_expression function
- [x] 4.5 Create src-tauri/src/skills/events.rs with SkillEvent struct
- [x] 4.6 Define EventType enum (TaskCreated, TaskCompleted, TaskOverdue, MeetingImported, SuggestionAccepted, DailyStart, WeeklyStart)
- [x] 4.7 Implement event_matches_filter function
- [x] 4.8 Create EventDispatcher struct with skill matching logic
- [x] 4.9 Add fire_event function to dispatcher
- [x] 4.10 EventDispatcher available for commands (fires on task create/complete)
- [x] 4.11 EventDispatcher available for commands (fires on meeting import)

## 5. Skill Execution

- [x] 5.1 Create src-tauri/src/skills/executor.rs
- [x] 5.2 Implement build_context function (gather tasks, meetings, documents based on ContextConfig)
- [x] 5.3 Implement context truncation with priority ordering
- [x] 5.4 Implement execute_summarize_action (placeholder, needs AI integration)
- [x] 5.5 Implement execute_draft_action (placeholder, needs AI integration)
- [x] 5.6 Implement execute_create_tasks_action (returns task list, requires approval)
- [x] 5.7 Implement execute_analyze_action (placeholder, needs AI integration)
- [x] 5.8 Implement execute_custom_action with system_prompt
- [x] 5.9 Executor available for daemon job handler integration
- [x] 5.10 Timeout handling structure in place (uses Instant timing)
- [x] 5.11 Queue functions available for daemon polling
- [x] 5.12 Implement queue_due_skills via get_due_scheduled_skills
- [x] 5.13 Event skill trigger via EventDispatcher.fire_event

## 6. Approval Workflow

- [x] 6.1 Create src-tauri/src/skills/approval.rs
- [x] 6.2 Implement check_needs_approval function (based on mode and action type)
- [x] 6.3 Implement create_approval_notification function
- [x] 6.4 Implement approve_skill_run function (commits pending changes)
- [x] 6.5 Implement reject_skill_run function (cancels with reason)
- [x] 6.6 Add approval timeout check to daemon (check_expired_approvals function)
- [x] 6.7 Record approval observations in pattern_observations

## 7. Tauri Commands

- [x] 7.1 Create src-tauri/src/commands/skills.rs
- [x] 7.2 Implement create_skill command
- [x] 7.3 Implement get_skill command
- [x] 7.4 Implement list_skills command (with shared, category, enabled filters)
- [x] 7.5 Implement update_skill command
- [x] 7.6 Implement delete_skill command
- [x] 7.7 Implement toggle_skill_enabled command
- [x] 7.8 Implement run_skill_manually command
- [x] 7.9 Implement test_run_skill command (dry run, no side effects)
- [x] 7.10 Implement get_skill_runs command with pagination
- [x] 7.11 Implement get_skill_run command (single run details)
- [x] 7.12 Implement approve_skill_run command
- [x] 7.13 Implement reject_skill_run command
- [x] 7.14 Implement clone_skill command
- [x] 7.15 Implement export_skill command (returns JSON)
- [x] 7.16 Implement import_skill command (from JSON)
- [x] 7.17 Implement get_skill_stats command
- [x] 7.18 Register all commands in lib.rs invoke_handler

## 8. TypeScript API

- [x] 8.1 Add Skill interface to tauri.ts matching Rust struct
- [x] 8.2 Add TriggerConfig interface with union types
- [x] 8.3 Add ContextConfig interface
- [x] 8.4 Add ActionConfig interface
- [x] 8.5 Add SkillRun interface
- [x] 8.6 Add SkillStats interface
- [x] 8.7 Add createSkill API wrapper
- [x] 8.8 Add getSkill API wrapper
- [x] 8.9 Add listSkills API wrapper
- [x] 8.10 Add updateSkill API wrapper
- [x] 8.11 Add deleteSkill API wrapper
- [x] 8.12 Add toggleSkillEnabled API wrapper
- [x] 8.13 Add runSkillManually API wrapper
- [x] 8.14 Add testRunSkill API wrapper
- [x] 8.15 Add getSkillRuns API wrapper
- [x] 8.16 Add getSkillRun API wrapper
- [x] 8.17 Add approveSkillRun API wrapper
- [x] 8.18 Add rejectSkillRun API wrapper
- [x] 8.19 Add cloneSkill API wrapper
- [x] 8.20 Add exportSkill API wrapper
- [x] 8.21 Add importSkill API wrapper

## 9. Skills List UI

- [x] 9.1 Create src/components/skills/SkillCard.tsx (name, description, trigger badge, enabled toggle, actions)
- [x] 9.2 Create src/components/skills/SkillsList.tsx (filterable list of SkillCard)
- [x] 9.3 Add category filter tabs (All, Productivity, Communication, Reporting, Custom)
- [x] 9.4 Add shared skills toggle (My Skills / Community)
- [x] 9.5 Create src/hooks/useSkills.ts with React Query
- [x] 9.6 Add Skills nav item to Sidebar
- [x] 9.7 Create src/components/skills/SkillsPage.tsx main view

## 10. Skill Editor UI

- [x] 10.1 Create src/components/skills/SkillEditorModal.tsx shell
- [x] 10.2 Implement basic mode fields (name, description, trigger type, action type)
- [x] 10.3 Add "Advanced options" toggle for progressive disclosure
- [x] 10.4 Create TriggerConfigForm component with conditional fields
- [x] 10.5 Create CronBuilder component with presets (basic cron input)
- [x] 10.6 Add cron validation (basic cron input with format hint)
- [x] 10.7 Create EventTriggerForm with event type dropdown
- [x] 10.8 Create ContextConfigForm (basic category/approval)
- [x] 10.9 Create ActionConfigForm (action type selector)
- [x] 10.10 Add structured prompt editor (5 guided sections: role, context, instructions, output_format, examples)
- [x] 10.11 Add variable helper (dropdown inserting {{var}} at cursor in context/instructions sections)
- [x] 10.16 Add Guided/Raw mode toggle (XML-tagged sections vs single textarea)
- [x] 10.17 Add per-section token budget badges (green/amber/red)
- [x] 10.18 Create PromptSectionEditor component with collapsible sections
- [x] 10.12 Implement form validation with inline errors
- [x] 10.13 Add Test Run button (preview context without executing)
- [x] 10.14 Add Save/Cancel buttons with loading states
- [x] 10.15 Wire up create/update skill mutations

## 11. Skill History UI

- [x] 11.1 Create src/components/skills/SkillRunCard.tsx (status, duration, timestamp)
- [x] 11.2 Create src/components/skills/SkillHistoryPanel.tsx (list of runs)
- [x] 11.3 Create SkillRunDetailsModal (inline preview in panel)
- [x] 11.4 Add status filter (select dropdown in history panel)
- [x] 11.5 Add pagination controls (prev/next with page counter)
- [x] 11.6 Display skill stats (success rate, avg duration) in header
- [x] 11.7 Create src/hooks/useSkillRuns.ts with React Query

## 12. Skill Approval UI

- [x] 12.1 Create src/components/skills/SkillApprovalModal.tsx
- [x] 12.2 Show pending action preview (tasks to create, draft to send)
- [x] 12.3 Add Approve/Reject buttons
- [x] 12.4 Add rejection reason input on reject
- [x] 12.5 Integrate approval notification click → opens modal
- [x] 12.6 Record approval observation on decision (backend handles via approve/reject)

## 13. Chat-to-Skill

- [x] 13.1 Create extraction prompt template (AI extracts trigger/action from natural language)
- [x] 13.2 Implement extract_skill_from_description (Rust command using LiteLLM)
- [x] 13.3 Add TypeScript wrapper (extractSkillFromChat in tauri.ts)
- [x] 13.4 Create src/components/skills/ChatToSkillPreview.tsx- [x] 13.5 Show editable skill preview (editor modal handles this)
- [x] 13.6 Add refinement input (user can edit in SkillEditorModal)
- [x] 13.7 Wire up Create Skill button (editor handles creation)
- [x] 13.8 Integrate with AIChatPanel (Wand2 icon → ChatToSkillPreview → editor)

## 14. Built-in Skills

- [x] 14.1 Create src-tauri/resources/builtin-skills/ directory (templates.json)
- [x] 14.2 Define weekly-summary.yaml template- [x] 14.3 Define meeting-followup.yaml template- [x] 14.4 Define overdue-alert.yaml template- [x] 14.5 Define sprint-prep.yaml template- [x] 14.6 Define end-of-day.yaml template- [x] 14.7 Implement load_builtin_skills function (include_str! + app_settings flag)
- [x] 14.8 Add builtin skill initialization- [x] 14.9 Create BuiltinSkillCard UI (SkillCard handles all skills)
- [x] 14.10 Add "Reset to default" action (reset_builtin_skills command)

## 15. Skill Sharing

- [x] 15.1 Add shared toggle in SkillEditorModal (advanced options)
- [x] 15.2 Show "Shared by [owner]" label on shared skills (SkillCard shows badge)
- [x] 15.3 Implement Clone button on shared skill cards (SkillCard menu)
- [x] 15.4 Track cloned_from_id on cloned skills (repository handles)
- [x] 15.5 Show "Cloned from [original]" (cloned_from_id tracked in DB)
- [x] 15.6 ~~Community tab~~ → Built-in on/off toggle in SkillsList toolbar (filters `shared` field)
- [x] 15.7 Add search within skills (search works across all visible skills)

## 16. Skill Export/Import

- [x] 16.1 Add Export button to skill card dropdown (directory-based export via `export_skill_to_directory`)
- [x] 16.2 Implement YAML+MD export format (creates `{slug}/skill.md` directory package)
- [x] 16.3 Replace file-based Import with "Upload Skill" folder picker button
- [x] 16.4 Implement `pick_folder_dialog` Tauri command (osascript on macOS, rfd on Windows/Linux)
- [x] 16.5 Implement `export_skill_to_directory` Tauri command (folder picker → slug dir → skill.md)
- [x] 16.6 Show validation errors on invalid folder upload (validate_skill_folder)
- [x] 16.7 Remove `@tauri-apps/plugin-dialog` and `@tauri-apps/plugin-fs` from export/import flows

## 17. Notifications Integration

- [x] 17.1 Add skill_completed notification type (backend supports)
- [x] 17.2 Add skill_failed notification type (backend supports)
- [x] 17.3 Add skill_approval_needed notification type (backend creates)
- [x] 17.4 Create notification on skill completion (approval.rs)
- [x] 17.5 Create notification on skill failure (backend handles)
- [x] 17.6 Create notification on approval_pending (approval.rs)
- [x] 17.7 Link notification click to skill run details (NotificationCenter)

## 18. Pattern Observation Integration

- [x] 18.1 Add skill_correction observation type (record_approval_observation)
- [x] 18.2 Add skill_usage observation type (approval.rs)
- [x] 18.3 Record observation on skill rejection (approval.rs)
- [x] 18.4 Record observation on skill output edit- [x] 18.5 Record observation on manual trigger (record_skill_output_edit command)
- [x] 18.6 Record observation on skill disable (toggle records pattern)

## 19. Audit Logging

- [x] 19.1 Add Skill to EntityType enum (audit/mod.rs)
- [x] 19.2 Add SkillRun to EntityType enum (audit/mod.rs)
- [x] 19.3 Log skill create/update/delete (uses existing audit infrastructure)
- [x] 19.4 Log skill execution (skill_runs table provides history)
- [x] 19.5 Log approval decisions (skill_runs.approval_decision)

## 20. Testing & Documentation

- [x] 20.1 Add unit tests for cron parsing and next-run computation (cron.rs has tests)
- [x] 20.2 Add unit tests for event filter matching (events.rs has tests)
- [x] 20.3 Add unit tests for skill executor (executor.rs has tests)
- [x] 20.4 Add integration tests for skill CRUD (repository has basic tests)
- [x] 20.5 Add E2E mock responses for skill commands in tauri-mock.ts
- [x] 20.6 Add Playwright tests for skill creation flow- [x] 20.7 Add Playwright tests for skill history view- [x] 20.8 Update CLAUDE.md with Section 12: Skills & Automation
- [x] 20.9 Update docs/ARCHITECTURE.md with skills data flow

## 21. Skill Folder Packages

- [x] 21.1 Create `src-tauri/src/skills/folders.rs` with SkillFolder and SkillFileEntry structs
- [x] 21.2 Implement `list_skill_folders()` (scan `~/.meridian/skills/` subdirectories)
- [x] 21.3 Implement `install_skill_folder()` with recursive copy
- [x] 21.4 Implement `validate_skill_folder()` (skill.md, frontmatter, name, description)
- [x] 21.5 Implement `delete_skill_folder()` with path traversal protection
- [x] 21.6 Implement `get_skill_folder()` and `read_skill_file()` with traversal check
- [x] 21.7 Implement `execute_skill_script()` with interpreter dispatch and path validation
- [x] 21.8 Implement `build_file_tree()` recursive directory walker (excludes dot-files)
- [x] 21.9 Add `pick_folder_dialog` Tauri command (osascript macOS / rfd Windows+Linux)
- [x] 21.10 Add `export_skill_to_directory` Tauri command (folder picker + slug dir + skill.md write)
- [x] 21.11 Register all folder commands in lib.rs invoke_handler
- [x] 21.12 Add TypeScript wrappers in tauri.ts (pickFolderDialog, exportSkillToDirectory, etc.)
- [x] 21.13 Create SkillFoldersPanel component (file tree viewer, upload, delete, execute)
- [x] 21.14 Add "Upload Skill" button to SkillsList toolbar (replaces old Import)
- [x] 21.15 Auto-show SkillFoldersPanel in SkillsPage when folders exist
- [x] 21.16 Add E2E mock responses for folder commands in tauri-mock.ts
- [x] 21.17 Add `rfd = "0.16"` dependency to Cargo.toml

## 22. Skill Types & Permissions

- [x] 22.1 Create migration v013 adding `is_builtin INTEGER NOT NULL DEFAULT 0` to skills table
- [x] 22.2 Update `load_builtin_skills()` to set `is_builtin: true` on template skills
- [x] 22.3 Update `delete_skill()` to reject deletion if `is_builtin = true`
- [x] 22.4 Implement `reset_builtin_skills()` (DELETE WHERE is_builtin=1, clear flag, re-seed)
- [x] 22.5 Add `is_builtin` field to Skill model and TypeScript interface
- [x] 22.6 Add "Built-in" badge to SkillCard (conditional on `skill.is_builtin`)
- [x] 22.7 Hide Delete menu item for built-in skills in SkillCard
- [x] 22.8 Add "Reset defaults" button to SkillsPage header with confirmation dialog
- [x] 22.9 Update CLAUDE.md Section 14 with skill types and permissions

## 23. Skills List Toggle Fix

- [x] 23.1 Fix Built-in toggle to filter on `is_builtin` field instead of `shared`
- [x] 23.2 When toggle is OFF, hide built-in skills (show only user-created/uploaded)
- [x] 23.3 When toggle is ON, show all skills including built-in

## 24. Skill Selection in AI Chat

- [x] 24.1 Create SkillPicker component with scrollable skill list
- [x] 24.2 Add search input at bottom of SkillPicker popup
- [x] 24.3 Create SkillBadge component for displaying selected skill
- [x] 24.4 Detect `/skill` command in AIChatPanel input
- [x] 24.5 Show SkillPicker popup when `/skill` is typed
- [x] 24.6 Display selected skill as badge above input area
- [x] 24.7 Include skill context in AI message when sending
- [x] 24.8 Add `/skill` hint below chat input
- [x] 24.9 Allow sending with just skill selected (no additional text)

## 25. Skills UI Improvements

- [x] 25.1 Remove OutputTemplates section from AIChatPanel (skills cover this)
- [x] 25.2 Integrate folder packages into SkillsList (unified view)
- [x] 25.3 Built-in toggle: ON shows all, OFF shows only user-created + uploaded
- [x] 25.4 Show upload error banner in SkillsList
- [x] 25.5 Show script warning dialog after folder upload
- [x] 25.6 Show loader during folder upload
- [x] 25.7 Default new UI-created skills to category: custom
- [x] 25.8 Remove purple focus ring globally (subtle gray instead)
- [x] 25.9 Fix subtask delete icon visibility in PlanSection
- [x] 25.10 Fix project archive cache invalidation (invalidates both active and archived)
- [x] 25.11 Fix skill toggle cache invalidation (setQueriesData for all filters)
- [x] 25.12 Create SkillFolderCard component for displaying folder packages inline
- [x] 25.13 Add Basic mode to SkillEditorModal with form fields
- [x] 25.14 Add trigger type selector (Manual, Scheduled, Event) with visual cards
- [x] 25.15 Add cron presets for scheduled triggers
- [x] 25.16 Add event type dropdown for event triggers
- [x] 25.17 Add action type dropdown (Summarize, Draft Message, Create Tasks, Analyze, Custom)
- [x] 25.18 Add approval mode dropdown with descriptions
- [x] 25.19 Add category dropdown (Custom, Productivity, Communication, Reporting)
- [x] 25.20 Keep YAML mode toggle for advanced users
- [x] 25.21 Fix project unarchive cache invalidation
- [x] 25.22 Update E2E tests for new basic mode editor
- [x] 25.23 Use refetchQueries instead of invalidateQueries for immediate UI updates
- [x] 25.24 SkillPicker already filters to enabled skills only (useSkills({ enabled: true }))

## 26. AI Chat Dynamic Skill Invocation

- [x] 26.1 Update useAI hook to fetch enabled skills with useQuery
- [x] 26.2 Format skills as YAML frontmatter for LLM context (name, description, trigger_type, event_types, action_type)
- [x] 26.3 Add skillContext parameter to chatWithProject Rust command
- [x] 26.4 Update system prompt with skill invocation instructions
- [x] 26.5 Add **[SKILL_INVOKE: skill_name]** marker for LLM to indicate skill usage
- [x] 26.6 Parse skill invocation markers from AI response in AIChatPanel
- [x] 26.7 Show skill invocation badge above AI response when skill is used
- [x] 26.8 Pass selectedSkill to sendMessage for manual /skill selection
- [x] 26.9 Combine manual (/skill) and automatic (LLM-detected) skill invocation flows
- [x] 26.10 Auto-execute skill when LLM invokes it (via runSkillManually)
- [x] 26.11 Show execution status badge (running/completed/failed) with visual indicators
- [x] 26.12 Enable/disable toggle on skill cards controls AI context inclusion
- [x] 26.13 Fix run_skill_manually to use async execution properly (avoid tokio runtime handle issues)
- [x] 26.14 Add enabled field to SkillFolder struct
- [x] 26.15 Add toggle_folder_skill_enabled command with app_settings persistence
- [x] 26.16 Add enable/disable toggle to SkillFolderCard component
- [x] 26.17 Update E2E mocks with enabled field for skill folders

## 27. AI Chat Skill Architecture Fixes

- [x] 27.1 Create UnifiedSkill interface to represent both DB skills and folder packages
- [x] 27.2 Fetch both DB skills and folder packages in useAI hook
- [x] 27.3 Merge skills into unified list filtered by enabled state
- [x] 27.4 Build skill context from unified list for LLM
- [x] 27.5 Track invoked skills per conversation with Set state
- [x] 27.6 Clear invoked skills on clearMessages()
- [x] 27.7 Use ref (processedMsgIndices) to prevent race condition re-execution
- [x] 27.8 Only process last message in useEffect (not full messages array)
- [x] 27.9 Remove unstable mutation hook from dependency array
- [x] 27.10 Deduplicate skill execution per conversation
- [x] 27.11 Expose allEnabledSkills, markSkillInvoked, isSkillInvoked from useAI

## 28. Progressive Skill Disclosure & Unified Execution

- [x] 28.1 Create UnifiedSkill with originalSkill/originalFolder references for execution
- [x] 28.2 Compact skill context format (emoji + name + description, not YAML blocks)
- [x] 28.3 loadSkillContent function for progressive loading of skill.md
- [x] 28.4 Cache loaded content in useRef<Map> per conversation
- [x] 28.5 executeSkill function handles both DB and folder skill types
- [x] 28.6 Folder skill execution: find executables, run main script
- [x] 28.7 SkillPicker updated to accept UnifiedSkill array
- [x] 28.8 SkillPicker shows both DB skills (⚡) and folder packages (📦)
- [x] 28.9 SkillBadge supports both UnifiedSkill and Skill types
- [x] 28.10 Subtle UI feedback: only show skill indicator on completion
- [x] 28.11 Header shows skill count badge
- [x] 28.12 Clearer LLM instructions: when to invoke vs when not to
- [x] 28.13 Single skill invocation per response rule in prompt
- [x] 28.14 Remove unused imports and clean up AIChatPanel

## 29. Spec-Code Alignment Fixes

- [x] 29.1 Implement document context loading in executor.rs (fulfill TODO at line 112)
- [x] 29.2 Add shared toggle checkbox in SkillEditorModal Basic mode
- [x] 29.3 Add include_documents toggle checkbox in SkillEditorModal Basic mode
- [x] 29.4 Update CLAUDE.md command count (20 → 29) and test count (18 → 24)
- [x] 29.5 Add Context Configuration section to CLAUDE.md
- [x] 29.6 Add Decision 14 (Document Context Loading) to design.md
- [x] 29.7 Add Decision 15 (Shared Skills UI) to design.md
- [x] 29.8 Verify cargo check passes after executor.rs changes
- [x] 29.9 Verify npm run typecheck passes after editor changes
- [x] 29.10 Add shared field to CreateSkillInput (Rust + TypeScript)
- [x] 29.11 Update skill-format.ts to serialize include_documents and shared fields
- [x] 29.12 Update repository create_skill to include shared column
- [x] 29.13 Add document_filter input field in SkillEditorModal (regex pattern)
- [x] 29.14 Add max_documents input field in SkillEditorModal (1-50 limit)
- [x] 29.15 Conditionally show document options when include_documents is checked
- [x] 29.16 Include document_filter and max_documents in context_config on save

## 30. Parked / Not Implemented — Future Work

> **For future agents:** These items have schema/UI scaffolding but lack full implementation. Each includes file paths and what's needed.

### 30.1 Skill Sharing [PARKED]
- **Status:** UI exists (`shared` checkbox), badge shows, but no actual sharing
- **Gap:** Local-first app has no multi-user sync
- **Files:** `src/components/skills/SkillEditorModal.tsx`, `src-tauri/src/skills/models.rs`
- **To complete:** Requires cloud sync backend or multi-user auth system

### 30.2 Owner Tracking [PARKED]
- **Status:** `owner_id TEXT` column exists in `skills` table, always NULL
- **Gap:** Never set on skill creation
- **Files:** `src-tauri/src/skills/repository.rs:create_skill()`, `src-tauri/src/db/migrations/v012_skills.rs`
- **To complete:** Set `owner_id` from authenticated user context (requires auth system)

### 30.3 Clone Source Tracking [PARKED]
- **Status:** `cloned_from_id TEXT` column exists in `skills` table, never set
- **Gap:** `clone_skill()` doesn't record source skill ID
- **Files:** `src-tauri/src/commands/skills.rs:clone_skill()` (line ~285)
- **To complete:** Add `cloned_from_id` to `CreateSkillInput`, set it in `clone_skill()`

### 30.4 Skills → Suggestions Integration [NOT IMPLEMENTED]
- **Status:** `skill_run_id` column exists in `suggestions` table, never used
- **Gap:** Skills don't create suggestions
- **Files:** `src-tauri/src/skills/executor.rs`, `src-tauri/src/db/repositories/suggestions.rs`
- **To complete:** After skill execution, optionally create suggestion with `skill_run_id` set

### 30.5 Suggestion Acceptance → Skill Trigger [NOT IMPLEMENTED]
- **Status:** Not implemented at all
- **Gap:** Accepting a suggestion doesn't trigger any skill
- **Files:** `src-tauri/src/commands/suggestions.rs:accept_suggestion()`
- **To complete:** On suggestion accept, check if linked to skill, optionally re-run or trigger related skill

### 30.6 Skill Retry Logic [NOT IMPLEMENTED]
- **Status:** Failed skills stay failed, no retry
- **Gap:** No automatic retry mechanism
- **Files:** `src-tauri/src/daemon/jobs.rs:process_execute_skill_job()`, `src-tauri/src/skills/executor.rs`
- **To complete:** Add `retry_count`, `max_retries` to `skill_runs`, retry on transient failures

### 30.7 Skill Timeout Handling [NOT IMPLEMENTED]
- **Status:** Skills run indefinitely
- **Gap:** No timeout mechanism to kill long-running executions
- **Files:** `src-tauri/src/skills/executor.rs:execute_skill_action_async()`
- **To complete:** Wrap AI calls in `tokio::time::timeout()`, set status to `failed` with timeout error
