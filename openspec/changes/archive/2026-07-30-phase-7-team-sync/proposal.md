## Why

Meridian is currently a single-user application with no concept of team collaboration. Users cannot share skills with teammates, leverage collective patterns for smarter suggestions, or export/import their data for backup or device migration. Phase 7 introduces the team layer and sync infrastructure, enabling shared intelligence while maintaining Meridian's local-first architecture.

## What Changes

- **Team Roster Management**: Maintain a list of team members with names, emails, roles, and source (manual entry, Slack workspace, Google Workspace)
- **Assignee Intelligence**: Smart assignee suggestions based on past task assignments, workload balance, expertise patterns, and team roster
- **Export/Import Sync**: Full data export (tasks, projects, meetings, documents metadata, skills, patterns, settings) as encrypted ZIP with JSON + Qdrant snapshot; import with merge or replace options
- **Shared Patterns**: Opt-in contribution to team-level aggregated patterns; team skills can leverage collective intelligence
- **Skill Sharing Enhancement**: Skills marked as "shared" become visible to team members who can clone and customize

## Capabilities

### New Capabilities
- `team-roster`: Team member management with manual entry and workspace integration (Slack, Google)
- `assignee-intelligence`: Smart assignee suggestions using patterns, workload analysis, and expertise matching
- `data-export`: Full data export to encrypted ZIP with selective content options
- `data-import`: Data import with merge/replace modes and conflict resolution
- `shared-patterns`: Team-level pattern aggregation and contribution system

### Modified Capabilities
- `skill-sharing`: Add team visibility, clone functionality, and master skill ownership
- `smart-defaults`: Extend assignee suggestions to use team roster and workload data
- `pattern-observation`: Add team contribution flag and aggregation support
- `pattern-aggregation`: Support team-level pattern models alongside personal patterns

## Impact

- **Database**: New `team_members` table; extend `pattern_models` with `scope` (personal/team) and `contributor_count`; extend `skills` with team sharing metadata
- **Backend**: New modules `src-tauri/src/team/` for roster management, `src-tauri/src/sync/` for export/import
- **Frontend**: New TeamSettings component, AssigneePicker with intelligence, Export/Import UI in settings
- **Integrations**: Slack and Google workspace connections can populate team roster
- **Skills**: Shared skills become discoverable and clonable by team members
- **Patterns**: Pattern learner gains team aggregation job; suggestions can use team patterns
