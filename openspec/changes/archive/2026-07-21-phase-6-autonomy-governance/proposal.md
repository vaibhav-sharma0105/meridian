## Why

Meridian now has skills, integrations, and proactive suggestions that can take actions on behalf of the user. Without governance, users lack visibility into what the agent is doing, cannot control risk levels, and have no way to undo mistakes. Phase 6 introduces the trust layer — autonomy controls, risk classification, approval workflows, and undo capabilities — so users can progressively delegate more to Meridian while maintaining oversight.

## What Changes

- **Autonomy Controller**: Global autonomy mode (Manual/Supervised/Autonomous) with per-integration and per-skill overrides
- **Risk Classification Engine**: Classify every agent action by risk level (low/medium/high/critical) based on action type, destination, and content
- **Approval Flow**: Pending approvals queue with configurable timeouts, bulk approve/reject, and archived actions retrievable with warning
- **Undo System**: Instant undo for last action, action history with selective undo, clear marking of non-undoable actions
- **Governance Dashboard**: Agent activity summary, actions by autonomy level, approval/rejection rates, risk distribution, anomaly detection

## Capabilities

### New Capabilities
- `autonomy-controller`: Global and granular autonomy mode settings (Manual/Supervised/Autonomous) with inheritance and overrides
- `risk-classification`: Rules-based and learned risk classification for agent actions with category and destination risk scoring
- `approval-flow`: Pending approvals queue, timeout handling, bulk actions, and archived action retrieval
- `undo-system`: Reversible action tracking, instant undo, action history, and reversal execution
- `governance-dashboard`: Agent activity metrics, approval rates, risk distribution charts, and anomaly detection

### Modified Capabilities
- `audit-logging`: Add risk_level field to audit entries, link to approval records
- `skill-execution`: Integrate approval flow for high-risk skill actions
- `suggestion-engine`: Route suggestions through approval flow based on risk level

## Impact

- **Database**: New tables (`pending_approvals`, `action_history`), extended `audit_log` with risk_level
- **Backend**: New modules (`src-tauri/src/autonomy/`, `src-tauri/src/governance/`)
- **Frontend**: New components (`AutonomySettings`, `ApprovalQueue`, `UndoBar`, `GovernanceDashboard`)
- **Skills**: All skill executions check autonomy mode before executing
- **Integrations**: All integration writes check autonomy mode and risk level
- **MCP Server**: Respect autonomy settings for external agent actions
