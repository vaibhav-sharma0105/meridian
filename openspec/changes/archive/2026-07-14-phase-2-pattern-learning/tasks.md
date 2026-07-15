## 1. Database Schema

- [x] 1.1 Create migration v008_pattern_learning.rs with pattern_observations table
- [x] 1.2 Add pattern_models table to migration with confidence and model_data columns
- [x] 1.3 Add indexes on observation_type, project_id, and processed_at
- [x] 1.4 Register migration in db/connection.rs

## 2. Observation Infrastructure

- [x] 2.1 Create src-tauri/src/patterns/mod.rs module structure
- [x] 2.2 Implement PatternObservation struct with serde serialization
- [x] 2.3 Create observations repository with insert_observation function
- [x] 2.4 Add get_unprocessed_observations with pagination support
- [x] 2.5 Add mark_observations_processed batch function

## 3. Observation Recording Points

- [x] 3.1 Record task_completion observation in update_task when status changes to done
- [x] 3.2 Record priority_set observation when task priority changes
- [x] 3.3 Record assignee_set observation when task assignee changes
- [x] 3.4 Record draft_edit observation in AI chat when user modifies generated draft
- [x] 3.5 Add suggestion_dismissed observation recording for dismissed suggestions

## 4. Pattern Model Storage

- [x] 4.1 Implement PatternModel struct with model_data JSON field
- [x] 4.2 Create patterns repository with upsert_pattern_model function
- [x] 4.3 Add get_pattern_models_for_project query
- [x] 4.4 Add get_pattern_model_by_type function
- [x] 4.5 Implement delete_pattern_model for reset functionality

## 5. Aggregation Job

- [x] 5.1 Register aggregate_patterns job type in daemon jobs table
- [x] 5.2 Implement aggregation job handler in daemon/jobs.rs
- [x] 5.3 Add workflow sequence aggregation logic
- [x] 5.4 Add priority pattern aggregation logic
- [x] 5.5 Add assignee pattern aggregation logic
- [x] 5.6 Add communication style aggregation with diff analysis
- [x] 5.7 Implement confidence scoring algorithm
- [x] 5.8 Add pattern decay on aggregation (10% monthly for stale patterns)
- [x] 5.9 Schedule next aggregation job (15-minute interval)

## 6. Workflow Sequence Learning

- [x] 6.1 Detect task completion sequences within 5-minute window
- [x] 6.2 Store sequences in pattern_model with occurrence_count
- [x] 6.3 Create get_workflow_suggestions Tauri command
- [x] 6.4 Add negative_sequences tracking for dismissed suggestions
- [x] 6.5 Suppress suggestions after 3 dismissals

## 7. Communication Style Learning

- [x] 7.1 Implement text diff analysis for draft edits
- [x] 7.2 Calculate length_delta and formality_shift metrics
- [x] 7.3 Track common phrase additions and removals
- [x] 7.4 Create communication_style pattern model structure
- [x] 7.5 Add get_communication_style Tauri command
- [x] 7.6 Support context-aware styles (task_followup, meeting_summary, general)

## 8. Smart Defaults

- [x] 8.1 Implement keyword extraction from task titles
- [x] 8.2 Build priority_patterns in pattern model
- [x] 8.3 Build assignee_patterns in pattern model
- [x] 8.4 Create get_smart_defaults Tauri command
- [x] 8.5 Apply defaults when confidence >= 0.5

## 9. Frontend: Suggestion Components

- [x] 9.1 Create WorkflowSuggestion.tsx component with dismiss/accept actions
- [x] 9.2 Add suggestion display to TaskListView on task completion
- [x] 9.3 Create SmartDefaultIndicator.tsx for pre-filled fields
- [x] 9.4 Integrate defaults into task creation form
- [x] 9.5 Add StyleAppliedBadge to AI chat drafts with "View original" toggle

## 10. Frontend: Learning Settings

- [x] 10.1 Create LearningSettings.tsx panel component
- [x] 10.2 Add PatternCategoryCard showing confidence and observation count
- [x] 10.3 Implement WorkflowSequenceDetail view with sequence list
- [x] 10.4 Implement PriorityPatternsDetail with keyword mappings
- [x] 10.5 Implement CommunicationStyleDetail with style preferences
- [x] 10.6 Add per-category Reset button with confirmation dialog
- [x] 10.7 Add Reset All Learning button with double confirmation

## 11. Export/Import

- [x] 11.1 Create export_learning_data Tauri command returning JSON
- [x] 11.2 Create import_learning_data Tauri command with validation
- [x] 11.3 Add Export button to Learning settings with file download
- [x] 11.4 Add Import button with file picker and merge strategy

## 12. Observation Pruning

- [x] 12.1 Add prune_old_observations function (>90 days processed)
- [x] 12.2 Schedule pruning job in aggregation cycle
- [x] 12.3 Log pruning statistics for monitoring

## 13. TypeScript API

- [x] 13.1 Add PatternObservation type to tauri.ts
- [x] 13.2 Add PatternModel type with model_data union types
- [x] 13.3 Add getWorkflowSuggestions wrapper
- [x] 13.4 Add getSmartDefaults wrapper
- [x] 13.5 Add getCommunicationStyle wrapper
- [x] 13.6 Add exportLearningData and importLearningData wrappers
- [x] 13.7 Add resetPatternCategory and resetAllLearning wrappers

## 14. Testing

- [x] 14.1 Add unit tests for observation recording
- [x] 14.2 Add unit tests for aggregation logic
- [x] 14.3 Add unit tests for confidence scoring
- [x] 14.4 Add E2E test for workflow suggestion display
- [x] 14.5 Add E2E test for Learning settings panel
- [x] 14.6 Add E2E test mock data for pattern models

## 15. Documentation

- [x] 15.1 Update CLAUDE.md with pattern learning section
- [x] 15.2 Update docs/ARCHITECTURE.md with pattern data flow
- [x] 15.3 Add pattern learning entries to tauri-mock.ts
