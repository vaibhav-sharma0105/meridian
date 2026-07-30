# Phase 8 Tasks: Advanced Intelligence

## 1. Database Schema

- [ ] 1.1 Create migration v017_advanced_intelligence.rs with `cross_project_links` table
- [ ] 1.2 Create `task_estimation_log` table for lifecycle event tracking
- [ ] 1.3 Create `usage_metrics` table for analytics aggregation
- [ ] 1.4 Create `onboarding_progress` table for tour tracking
- [ ] 1.5 Create `productivity_cache` table for time-of-day patterns
- [ ] 1.6 Add indexes on cross_project_links (source_id, target_id, link_type)
- [ ] 1.7 Register migration in db/migrations/mod.rs

## 2. Cross-Project Intelligence Module

- [ ] 2.1 Create `src-tauri/src/intelligence/mod.rs` module structure
- [ ] 2.2 Create `intelligence/models.rs` with CrossProjectLink, LinkType structs
- [ ] 2.3 Create `intelligence/repository.rs` for cross_project_links CRUD
- [ ] 2.4 Implement `intelligence/analysis.rs` with cross-project scanning logic
- [ ] 2.5 Implement explicit blocker detection (parse "blocked by" patterns)
- [ ] 2.6 Implement semantic similarity blocker detection using Qdrant
- [ ] 2.7 Implement duplicate meeting detection (title + attendee similarity)
- [ ] 2.8 Implement velocity computation across projects
- [ ] 2.9 Create Tauri commands: get_cross_project_links, create_link, delete_link, run_analysis

## 3. Cross-Project Frontend

- [ ] 3.1 Create `src/components/intelligence/CrossProjectPanel.tsx`
- [ ] 3.2 Add CrossProjectLinkBadge component for task cards
- [ ] 3.3 Add link creation modal with project/task search
- [ ] 3.4 Add cross-project blocker alerts in suggestion list
- [ ] 3.5 Create `src/hooks/useCrossProject.ts` with React Query hooks
- [ ] 3.6 Add TypeScript types for cross-project links in tauri.ts

## 4. Predictive Actions Module

- [ ] 4.1 Create `src-tauri/src/predictions/mod.rs` module structure
- [ ] 4.2 Create `predictions/models.rs` with Prediction, PredictionType structs
- [ ] 4.3 Implement `predictions/prefetch.rs` for document pre-loading
- [ ] 4.4 Implement `predictions/agenda.rs` for meeting agenda generation
- [ ] 4.5 Implement `predictions/blockers.rs` for deadline risk detection
- [ ] 4.6 Implement `predictions/workload.rs` for workload imbalance detection
- [ ] 4.7 Add prediction_confidence field to suggestions table
- [ ] 4.8 Create Tauri commands: get_predictions, trigger_prefetch, generate_agenda

## 5. Predictive Actions Frontend

- [ ] 5.1 Create `src/components/predictions/PredictionsPanel.tsx`
- [ ] 5.2 Add PredictionCard component with confidence indicator
- [ ] 5.3 Add deadline risk warnings on task cards
- [ ] 5.4 Add workload balance visualization in team view
- [ ] 5.5 Add pre-fetch status indicator for upcoming meetings
- [ ] 5.6 Create `src/hooks/usePredictions.ts` with React Query hooks

## 6. Time Optimization Module

- [ ] 6.1 Create `src-tauri/src/productivity/mod.rs` module structure
- [ ] 6.2 Implement `productivity/observation.rs` for time-based pattern recording
- [ ] 6.3 Implement `productivity/analysis.rs` for peak hours computation
- [ ] 6.4 Implement `productivity/suggestions.rs` for timing recommendations
- [ ] 6.5 Add time_of_day_productivity pattern type to pattern_observations
- [ ] 6.6 Add focus_session pattern type to pattern_observations
- [ ] 6.7 Create Tauri commands: get_productivity_profile, get_time_suggestions

## 7. Time Optimization Frontend

- [ ] 7.1 Create `src/components/analytics/ProductivitySection.tsx`
- [ ] 7.2 Add peak hours heatmap visualization
- [ ] 7.3 Add focus session distribution chart
- [ ] 7.4 Add timing suggestion tooltips on task creation
- [ ] 7.5 Add productivity trend comparison (this week vs average)

## 8. Task Estimation Module

- [ ] 8.1 Create `src-tauri/src/estimation/mod.rs` module structure
- [ ] 8.2 Implement `estimation/tracking.rs` for lifecycle event logging
- [ ] 8.3 Implement `estimation/accuracy.rs` for accuracy computation
- [ ] 8.4 Implement `estimation/suggestions.rs` for estimate recommendations
- [ ] 8.5 Add recency weighting to similar task matching
- [ ] 8.6 Add systematic bias detection and correction factor
- [ ] 8.7 Create Tauri commands: get_estimate_suggestion, get_estimation_accuracy

## 9. Task Estimation Frontend

- [ ] 9.1 Add estimate suggestion indicator on task form
- [ ] 9.2 Add "Why this estimate?" expandable explanation
- [ ] 9.3 Create `src/components/analytics/EstimationSection.tsx`
- [ ] 9.4 Add accuracy dashboard with breakdown by project/assignee
- [ ] 9.5 Add estimation improvement trend chart

## 10. Onboarding Expansion

- [ ] 10.1 Create `src-tauri/src/onboarding/mod.rs` module structure
- [ ] 10.2 Implement tour progress tracking in onboarding_progress table
- [ ] 10.3 Create Tauri commands: get_tour_progress, update_tour_progress, reset_tour
- [ ] 10.4 Create `src/components/onboarding/AgenticTour.tsx` component
- [ ] 10.5 Create tour step components (AutonomyStep, SkillsStep, GovernanceStep)
- [ ] 10.6 Implement demo mode data generation
- [ ] 10.7 Add demo mode UI indicator and exit button
- [ ] 10.8 Create tooltip system for first-time feature encounters
- [ ] 10.9 Add "Take Agentic Tour" button after basic onboarding
- [ ] 10.10 Add "Retake Tour" option in Settings

## 11. Usage Analytics Module

- [ ] 11.1 Create `src-tauri/src/analytics/mod.rs` module structure
- [ ] 11.2 Implement `analytics/tracking.rs` for activity recording
- [ ] 11.3 Implement `analytics/ai_usage.rs` for token tracking
- [ ] 11.4 Implement `analytics/storage.rs` for storage computation
- [ ] 11.5 Implement `analytics/aggregation.rs` for daily rollups
- [ ] 11.6 Implement `analytics/export.rs` for CSV/JSON export
- [ ] 11.7 Create Tauri commands: get_usage_metrics, export_analytics

## 12. Usage Analytics Frontend

- [ ] 12.1 Create `src/components/analytics/AnalyticsDashboard.tsx` main view
- [ ] 12.2 Add Analytics nav item to sidebar
- [ ] 12.3 Create ActivitySection with task/meeting/document charts
- [ ] 12.4 Create AIUsageSection with token consumption and cost estimates
- [ ] 12.5 Create StorageSection with breakdown visualization
- [ ] 12.6 Create AutomationSection with skill execution metrics
- [ ] 12.7 Add time range selector (today/week/month/custom)
- [ ] 12.8 Add project filter for scoped analytics
- [ ] 12.9 Add comparison toggle (vs previous period)
- [ ] 12.10 Add export button with format selection (CSV/JSON)

## 13. Daemon Jobs

- [ ] 13.1 Add cross_project_analysis job (every 6 hours)
- [ ] 13.2 Add predictive_prefetch job (every 15 minutes)
- [ ] 13.3 Add usage_aggregation job (daily at midnight)
- [ ] 13.4 Add estimation_log_lifecycle events on task status changes
- [ ] 13.5 Add productivity_observation recording on task completion
- [ ] 13.6 Schedule jobs on daemon startup

## 14. Suggestion Engine Extensions

- [ ] 14.1 Add cross_project_blocker suggestion type
- [ ] 14.2 Add consolidation suggestion type
- [ ] 14.3 Add velocity_alert suggestion type
- [ ] 14.4 Add deadline_risk suggestion type
- [ ] 14.5 Add workload_rebalance suggestion type
- [ ] 14.6 Add meeting_prep suggestion type
- [ ] 14.7 Add source field to suggestions (pattern/prediction/cross_project/rule)
- [ ] 14.8 Add prediction_confidence field to suggestions

## 15. Pattern Observation Extensions

- [ ] 15.1 Add time_of_day_productivity observation type
- [ ] 15.2 Add task_estimation observation type
- [ ] 15.3 Add focus_session observation type
- [ ] 15.4 Extend pattern aggregation job for new observation types
- [ ] 15.5 Add productivity_profile to pattern_models
- [ ] 15.6 Add estimation_accuracy to pattern_models

## 16. Skill Execution Extensions

- [ ] 16.1 Add pre-fetch trigger for skills with include_documents
- [ ] 16.2 Add cross-project context support for global scope skills
- [ ] 16.3 Add timing optimization suggestions for skill scheduling
- [ ] 16.4 Integrate with productivity patterns for skill timing

## 17. Data Migration

- [ ] 17.1 Backfill task_estimation_log from existing task created_at/completed_at
- [ ] 17.2 Initial usage_metrics aggregation from audit_log history
- [ ] 17.3 Set onboarding_progress for existing users (mark basic complete)

## 18. Testing

- [ ] 18.1 Add Rust unit tests for cross-project link repository
- [ ] 18.2 Add Rust unit tests for prediction engine
- [ ] 18.3 Add Rust unit tests for estimation accuracy computation
- [ ] 18.4 Add Rust unit tests for usage aggregation
- [ ] 18.5 Add Playwright E2E tests for Analytics dashboard
- [ ] 18.6 Add Playwright E2E tests for Agentic Tour
- [ ] 18.7 Add Playwright E2E tests for cross-project links
- [ ] 18.8 Update Tauri mock in tests/e2e/setup/tauri-mock.ts

## 19. Documentation

- [ ] 19.1 Update CLAUDE.md with intelligence/predictions/analytics architecture
- [ ] 19.2 Update docs/ARCHITECTURE.md with new modules and data flow
- [ ] 19.3 Add analytics and predictions sections to README
