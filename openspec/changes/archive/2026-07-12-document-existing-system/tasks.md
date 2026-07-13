## 1. Review and Validate Specs

- [ ] 1.1 Review task-management spec against actual TaskCard, TaskListView, TaskKanbanView, TaskTableView components
- [ ] 1.2 Review meeting-management spec against MeetingCard, MeetingIngest, useMeetings hook
- [ ] 1.3 Review document-management spec against DocFolder, DocUpload, DocCard, DocSearch components
- [ ] 1.4 Review ai-chat spec against AIChatPanel, AISettings, useAI hook, and ai/ Rust modules
- [ ] 1.5 Review project-management spec against ProjectCreate, ProjectSettings, useProjectStore
- [ ] 1.6 Review integration-zoom spec against zoom.rs connector and connections UI
- [ ] 1.7 Review integration-sheets-relay spec against sheets_relay.rs connector
- [ ] 1.8 Review integration-mcp-server spec against meridian-mcp binary and .mcp.json
- [ ] 1.9 Review notification-system spec against NotificationCenter, PendingImportCard, useNotificationStore

## 2. Address Spec Gaps

- [ ] 2.1 Update specs with any missing requirements discovered during review
- [ ] 2.2 Add missing scenarios for edge cases found in code
- [ ] 2.3 Document any undocumented limitations or known issues
- [ ] 2.4 Ensure all SHALL statements have corresponding scenarios

## 3. Archive to Main Specs

- [ ] 3.1 Run openspec validate to check spec format compliance
- [ ] 3.2 Run openspec archive to promote change specs to openspec/specs/
- [ ] 3.3 Verify all 9 capability specs exist in openspec/specs/
- [ ] 3.4 Update openspec/specs/index.md with capability list and descriptions

## 4. Documentation Updates

- [ ] 4.1 Update CLAUDE.md to reference OpenSpec specs as authoritative source
- [ ] 4.2 Add spec navigation guidance to docs/ARCHITECTURE.md
- [ ] 4.3 Create openspec/README.md explaining spec structure and conventions
