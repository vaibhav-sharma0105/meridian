## Why

Meridian has evolved through seven phases from a meeting-to-task converter into a proactive, learning executive assistant. With pattern learning, skills automation, external integrations, and governance in place, the final phase transforms Meridian into an indispensable tool that anticipates needs across projects, predicts blockers before they happen, learns from task estimation patterns, and provides deep visibility into usage and productivity metrics.

## What Changes

- Cross-project intelligence engine that detects blockers spanning multiple projects, identifies duplicate/related meetings for consolidation, and analyzes velocity trends
- Predictive actions system that pre-fetches relevant documents before meetings, auto-drafts agendas from open tasks and attendees, predicts potential blockers, and suggests reassignments based on workload
- Time-of-day optimization that learns when users are most productive for different task types and suggests optimal scheduling
- Task estimation learning that tracks actual vs estimated completion times and provides increasingly accurate estimates for similar tasks
- Enhanced onboarding with interactive tour of agentic features, demo mode showing Meridian in action, and gradual feature introduction
- Usage analytics dashboard with activity metrics, AI token consumption, storage breakdown, productivity insights, and exportable reports

## Capabilities

### New Capabilities
- `cross-project-intelligence`: Analyze relationships and blockers across projects, detect meeting consolidation opportunities, compute cross-project velocity metrics
- `predictive-actions`: Pre-fetch documents before meetings, auto-draft agendas, predict blockers, suggest workload-based reassignments
- `time-optimization`: Learn user productivity patterns by time of day, suggest optimal work scheduling, adapt skill timing
- `task-estimation`: Track actual vs estimated task duration, learn estimation accuracy patterns, provide smart estimates for new tasks
- `onboarding-expansion`: Interactive tour of autonomy settings, demo mode with sample data, progressive feature tooltips
- `usage-analytics`: Activity metrics dashboard, AI usage tracking, storage breakdown, productivity insights, data export

### Modified Capabilities
- `suggestion-engine`: Extend to generate cross-project and predictive suggestions
- `pattern-observation`: Add time-of-day and estimation observation types
- `skill-execution`: Integrate with predictive pre-fetch for document context

## Impact

- **Database**: New tables for cross-project links, estimation history, usage metrics; extend pattern_observations with new types
- **Daemon**: New jobs for cross-project analysis, predictive pre-fetch, usage aggregation
- **Frontend**: New Analytics view in sidebar, enhanced onboarding wizard, prediction indicators on tasks
- **AI**: Additional context building for cross-project queries, estimation prompts
- **Performance**: Cross-project queries need efficient indexing; usage metrics need daily aggregation to avoid query overhead
