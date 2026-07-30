# Phase 6 Tasks: Autonomy & Governance

## 1. Database Schema

- [x] 1.1 Create migration v015_governance.rs with `pending_approvals` table
- [x] 1.2 Create `action_history` table for undo tracking
- [x] 1.3 Create `governance_metrics` table for dashboard aggregates
- [x] 1.4 Add columns to `audit_log`: risk_level, autonomy_mode, autonomy_source, approval_id, undo_action_id
- [x] 1.5 Add `autonomy_mode` column to `integrations` table (nullable, default NULL = inherit)
- [x] 1.6 Add `autonomy_mode` column to `skills` table (nullable, default NULL = inherit)
- [x] 1.7 Register migration in db/migrations/mod.rs

## 2. Governance Module Structure

- [x] 2.1 Create `src-tauri/src/governance/mod.rs` module structure
- [x] 2.2 Create `governance/models.rs` with RiskLevel, AutonomyMode, PendingApproval, ActionHistory structs
- [x] 2.3 Create `governance/repository.rs` for governance table CRUD

## 3. Risk Classification Engine

- [x] 3.1 Implement `governance/risk.rs` with RiskScore struct and calculate_risk_level()
- [x] 3.2 Add action type classification (read=1, create=2, update=3, external_send=4, delete=5)
- [x] 3.3 Add destination scoring (internal=1, team=2, external=3, executive=4)
- [x] 3.4 Add content scoring using existing sensitive content detection
- [x] 3.5 Implement critical override logic (any max score → Critical)
- [x] 3.6 Add learned adjustment support (user can bump/lower risk for specific contacts/channels)

## 4. Autonomy Controller

- [x] 4.1 Implement `governance/autonomy.rs` with AutonomyController
- [x] 4.2 Implement resolve_effective_autonomy() with inheritance chain (global → integration → skill)
- [x] 4.3 Implement should_require_approval(risk_level, autonomy_mode) decision logic
- [x] 4.4 Add get_autonomy_setting and set_autonomy_setting Tauri commands
- [x] 4.5 Add global autonomy mode to app_settings (Manual/Supervised/Autonomous, default Supervised)

## 5. Approval Flow

- [x] 5.1 Implement `governance/approval.rs` with approval queue operations
- [x] 5.2 Implement create_pending_approval() with timeout calculation
- [x] 5.3 Implement approve_action() → execute and create action_history entry
- [x] 5.4 Implement reject_action() with reason recording
- [x] 5.5 Implement archive_expired_approvals() for timeout handling
- [x] 5.6 Add Tauri commands: get_pending_approvals, approve_action, reject_action, bulk_approve, bulk_reject

## 6. Undo System

- [x] 6.1 Implement `governance/undo.rs` with action history operations
- [x] 6.2 Implement capture_action_state() to snapshot before_state/after_state
- [x] 6.3 Implement undo_action() to create reversal action
- [x] 6.4 Implement get_recent_undoable_actions() for undo bar
- [x] 6.5 Add Tauri commands: undo_action, get_action_history, get_undoable_actions
- [x] 6.6 Mark external actions (Slack, GitHub, Jira) as undoable=false

## 7. Integration Points

- [x] 7.1 Hook skill execution through autonomy controller in skills/executor.rs
- [x] 7.2 Hook suggestion acceptance through autonomy controller in suggestions/
- [x] 7.3 Hook integration write operations through autonomy controller
- [x] 7.4 Hook MCP write tools through autonomy controller in meridian-mcp/
- [x] 7.5 Update audit logging calls to include risk_level, autonomy_mode, approval_id
- [x] 7.6 Create action_history entries for all agent-initiated mutations

## 8. Governance Commands

- [x] 8.1 Create `src-tauri/src/commands/governance.rs` with all governance commands
- [x] 8.2 Register governance commands in lib.rs invoke_handler
- [x] 8.3 Add TypeScript wrappers in src/lib/tauri.ts for all governance commands
- [x] 8.4 Add types: RiskLevel, AutonomyMode, PendingApproval, ActionHistory, GovernanceMetrics

## 9. Daemon Jobs

- [x] 9.1 Add `check_approval_timeouts` job handler (runs every minute)
- [x] 9.2 Add `aggregate_governance_metrics` job handler (runs daily)
- [x] 9.3 Add `detect_anomalies` job handler (runs hourly)
- [x] 9.4 Register jobs in daemon/jobs.rs handle_job()
- [x] 9.5 Schedule jobs on app startup

## 10. Frontend: Autonomy Settings

- [x] 10.1 Create `src/components/governance/AutonomySettings.tsx` with global mode selector
- [x] 10.2 Add per-integration autonomy override in IntegrationsPage
- [x] 10.3 Add per-skill autonomy override in SkillEditorModal
- [x] 10.4 Create `src/hooks/useGovernance.ts` with React Query hooks
- [x] 10.5 Add Zustand store for governance state if needed (using React Query instead)

## 11. Frontend: Approval Queue

- [x] 11.1 Create `src/components/governance/ApprovalQueue.tsx` with pending items list
- [x] 11.2 Add approval detail panel showing action config, risk level, context
- [x] 11.3 Add approve/reject buttons with confirmation
- [x] 11.4 Add bulk approve/reject functionality
- [x] 11.5 Add timeout countdown display
- [x] 11.6 Add sidebar badge showing pending approval count

## 12. Frontend: Undo Bar

- [x] 12.1 Create `src/components/governance/UndoBar.tsx` toast component
- [x] 12.2 Show undo bar for 10 seconds after reversible agent actions
- [x] 12.3 Implement undo button that calls undo_action
- [x] 12.4 Add action description and countdown timer
- [x] 12.5 Stack multiple undos if actions happen in sequence

## 13. Frontend: Action History

- [x] 13.1 Create `src/components/governance/ActionHistoryPanel.tsx`
- [x] 13.2 Add filters: by entity type, by action type, by date range
- [x] 13.3 Show before/after state diff for undone actions
- [x] 13.4 Add "Undo" button for recent undoable actions

## 14. Frontend: Governance Dashboard

- [x] 14.1 Create `src/components/governance/GovernanceDashboard.tsx` main view
- [x] 14.2 Add activity summary card (actions today/this week/by type)
- [x] 14.3 Add autonomy breakdown card (auto-executed vs approved)
- [x] 14.4 Add approval rate metrics card
- [x] 14.5 Add risk distribution chart
- [x] 14.6 Add anomaly alerts section
- [x] 14.7 Add integration activity breakdown
- [x] 14.8 Add skill activity breakdown
- [x] 14.9 Add time range selector
- [x] 14.10 Add export functionality

## 15. Testing

- [x] 15.1 Add Rust unit tests for risk classification
- [x] 15.2 Add Rust unit tests for autonomy resolution
- [x] 15.3 Add Rust unit tests for approval flow
- [x] 15.4 Add Rust unit tests for undo system
- [x] 15.5 Add Playwright E2E tests for autonomy settings
- [x] 15.6 Add Playwright E2E tests for approval queue
- [x] 15.7 Add Playwright E2E tests for undo bar
- [x] 15.8 Update Tauri mock in tests/e2e/setup/tauri-mock.ts

## 16. Documentation

- [x] 16.1 Update CLAUDE.md with governance architecture
- [x] 16.2 Update docs/ARCHITECTURE.md with governance data flow
- [x] 16.3 Add governance section to README with user guide
