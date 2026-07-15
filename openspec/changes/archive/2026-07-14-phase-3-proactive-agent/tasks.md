## 1. Database Schema

- [x] 1.1 Create migration v010_proactive_agent.rs with suggestions table
- [x] 1.2 Add draft_messages table to migration
- [x] 1.3 Add plan_complexity, plan_data, plan_generated_at columns to tasks
- [x] 1.4 Add indexes on suggestions(status, project_id) and draft_messages(task_id)
- [x] 1.5 Register migration in db/migrations/mod.rs

## 2. Suggestions Repository

- [x] 2.1 Create src-tauri/src/suggestions/mod.rs module structure
- [x] 2.2 Implement Suggestion struct with serde serialization
- [x] 2.3 Create suggestions repository with create_suggestion function
- [x] 2.4 Add get_pending_suggestions with project filter
- [x] 2.5 Add update_suggestion_status function
- [x] 2.6 Add get_suggestions_count_today for daily limit
- [x] 2.7 Add delete_old_suggestions for cleanup

## 3. Suggestion Engine

- [x] 3.1 Create generate_suggestions job handler in daemon/jobs.rs
- [x] 3.2 Implement overdue task detection (>24 hours past due)
- [x] 3.3 Implement stale task detection (>7 days no update, in_progress)
- [x] 3.4 Implement meeting follow-up detection (>24 hours, no linked tasks)
- [x] 3.5 Integrate workflow sequence patterns into suggestions
- [x] 3.6 Add daily suggestion limit enforcement (default: 10)
- [x] 3.7 Schedule suggestion job every 30 minutes
- [x] 3.8 Add suggestion settings (max_per_day) to app_settings

## 4. Suggestion Commands

- [x] 4.1 Create src-tauri/src/commands/suggestions.rs
- [x] 4.2 Add get_pending_suggestions Tauri command
- [x] 4.3 Add accept_suggestion Tauri command
- [x] 4.4 Add dismiss_suggestion Tauri command
- [x] 4.5 Add stop_suggesting Tauri command (records negative pattern)
- [x] 4.6 Register commands in lib.rs

## 5. Drafts Repository

- [x] 5.1 Create src-tauri/src/drafts/mod.rs module structure
- [x] 5.2 Implement DraftMessage struct with serde serialization
- [x] 5.3 Create drafts repository with create_draft function
- [x] 5.4 Add get_drafts_for_task function
- [x] 5.5 Add update_draft function
- [x] 5.6 Add delete_draft function

## 6. Draft Generation

- [x] 6.1 Add draft generation prompt template to AI module
- [x] 6.2 Implement generate_draft function with style adaptation
- [x] 6.3 Create auto_generate_draft_for_task function
- [x] 6.4 Add action keyword detection (follow up, send, share, email, message)
- [x] 6.5 Integrate communication style patterns into draft generation
- [x] 6.6 Add "Drafted by Meridian" signature handling

## 7. Draft Commands

- [x] 7.1 Create src-tauri/src/commands/drafts.rs
- [x] 7.2 Add get_drafts_for_task Tauri command
- [x] 7.3 Add generate_draft Tauri command
- [x] 7.4 Add update_draft Tauri command
- [x] 7.5 Add delete_draft Tauri command
- [x] 7.6 Register commands in lib.rs

## 8. Smart Task Plans

- [x] 8.1 Add complexity evaluation prompt template
- [x] 8.2 Implement evaluate_task_complexity function
- [x] 8.3 Add plan generation for simple tasks (draft action)
- [x] 8.4 Add plan generation for medium tasks (subtask suggestions)
- [x] 8.5 Add plan generation for complex tasks (decomposition flag)
- [x] 8.6 Update create_task to trigger plan generation
- [x] 8.7 Add accept_plan Tauri command (creates subtasks)
- [x] 8.8 Record plan corrections as observations

## 9. Sensitive Content Detection

- [x] 9.1 Create src-tauri/src/sensitive/mod.rs module
- [x] 9.2 Implement PII detection patterns (SSN, phone, address)
- [x] 9.3 Implement credential detection patterns (API keys, passwords)
- [x] 9.4 Implement financial data detection patterns (credit cards, bank accounts)
- [x] 9.5 Create scan_content function returning warnings array
- [x] 9.6 Add scan_draft Tauri command
- [x] 9.7 Log detections to audit log
- [x] 9.8 Register commands in lib.rs

## 10. TypeScript API

- [x] 10.1 Add Suggestion type to tauri.ts
- [x] 10.2 Add DraftMessage type to tauri.ts
- [x] 10.3 Add getPendingSuggestions wrapper
- [x] 10.4 Add acceptSuggestion, dismissSuggestion, stopSuggesting wrappers
- [x] 10.5 Add getDraftsForTask, generateDraft, updateDraft wrappers
- [x] 10.6 Add scanDraft wrapper for sensitive content
- [x] 10.7 Add TaskPlan type with complexity and suggested_subtasks

## 11. Suggestion UI

- [x] 11.1 Create SuggestionCard.tsx component
- [x] 11.2 Add severity indicator (info/warning/critical colors)
- [x] 11.3 Add "Do it", "Dismiss", "Stop suggesting" action buttons
- [x] 11.4 Create SuggestionsList.tsx for notification panel
- [x] 11.5 Add suggestions section to NotificationCenter
- [x] 11.6 Implement suggestion detail expansion (show reasoning)
- [x] 11.7 Add toast notification for warning-severity suggestions

## 12. Drafts UI

- [x] 12.1 Create DraftEditor.tsx component with inline editing
- [x] 12.2 Add AI signature toggle checkbox
- [x] 12.3 Add "Copy to clipboard" button with toast feedback
- [x] 12.4 Create DraftsTab.tsx for task detail
- [x] 12.5 Add Drafts tab to TaskEditModal
- [x] 12.6 Add draft generation trigger on task save (if action keywords detected)

## 13. Plan UI

- [x] 13.1 Create PlanSection.tsx component
- [x] 13.2 Add complexity indicator badge (simple/medium/complex)
- [x] 13.3 Add suggested subtasks list with edit capability
- [x] 13.4 Add "Create subtasks" button for medium tasks
- [x] 13.5 Add "This needs breakdown" indicator for complex tasks
- [x] 13.6 Integrate PlanSection into TaskEditModal/TaskInlineEditor

## 14. Sensitive Content UI

- [x] 14.1 Create SensitiveWarning.tsx banner component
- [x] 14.2 Add warning type icons (PII, credentials, financial)
- [x] 14.3 Add "Dismiss" button with audit logging
- [x] 14.4 Integrate warnings into DraftEditor
- [x] 14.5 Add debounced rescan on draft content change

## 15. Testing

- [x] 15.1 Add unit tests for suggestion generation logic
- [x] 15.2 Add unit tests for sensitive content patterns
- [x] 15.3 Add unit tests for complexity evaluation
- [x] 15.4 Add E2E test for suggestion display in notifications
- [x] 15.5 Add E2E test for draft generation and editing
- [x] 15.6 Add E2E test mock data for suggestions and drafts

## 16. Documentation

- [x] 16.1 Update CLAUDE.md with proactive agent section
- [x] 16.2 Update docs/ARCHITECTURE.md with suggestion flow
- [x] 16.3 Add suggestion and draft entries to tauri-mock.ts
