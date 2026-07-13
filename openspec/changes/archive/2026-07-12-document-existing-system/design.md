## Context

Meridian is a Tauri v2 desktop app with a React/TypeScript frontend and Rust backend. The codebase has grown organically with features spanning task management, meeting intelligence, document RAG, multi-provider AI chat, and integrations (Zoom, Sheets Relay, MCP).

Currently, understanding the system requires reading code directly. This creates friction for:
- AI agents trying to work on the codebase
- Future development requiring context on existing behavior
- Onboarding new contributors

This change backfills formal OpenSpec specifications for all existing features, establishing the foundation for spec-driven development going forward.

**Current State:**
- 9 major feature areas without formal specs
- Behavior documented only in code and CLAUDE.md
- No standardized format for requirements and scenarios

## Goals / Non-Goals

**Goals:**
- Create authoritative specifications for all existing Meridian features
- Establish testable scenarios for each requirement
- Enable AI agents to understand system contracts without reading implementation
- Set the pattern for future feature specs

**Non-Goals:**
- No code changes — this is documentation only
- No new features or behavior modifications
- No architecture changes or refactoring
- No test implementation (specs define scenarios, tests come later)

## Decisions

### Decision 1: Spec Granularity

**Choice:** One spec file per major feature/capability (9 total), not one file per component.

**Rationale:** Feature-level specs are more useful for understanding system behavior than component-level specs. A developer asking "how does task filtering work?" wants one place to look, not scattered component docs.

**Alternatives Considered:**
- Component-level specs (TaskCard.spec.md, TaskFilters.spec.md, etc.) — too granular, loses cohesion
- Single monolithic spec — too large, hard to navigate and maintain

### Decision 2: Scenario Format

**Choice:** Use WHEN/THEN format without explicit GIVEN clause.

**Rationale:** Most Meridian scenarios have implicit context (user is logged in, project is selected). Explicit GIVEN clauses add verbosity without clarity. The requirement description provides context.

**Alternatives Considered:**
- Full GIVEN/WHEN/THEN — more formal but verbose
- Prose descriptions — less testable

### Decision 3: Integration Specs Separation

**Choice:** Separate spec files for each integration (Zoom, Sheets Relay, MCP) rather than a combined "integrations" spec.

**Rationale:** Each integration has distinct auth flows, data models, and sync logic. Combining them would create a confusing document. Separate specs enable focused reading and independent evolution.

### Decision 4: Document Existing Behavior Only

**Choice:** Specs describe current behavior, including known limitations (e.g., "XLSX parsing returns placeholder text").

**Rationale:** Specs should be accurate to what exists today, not aspirational. Future changes will create new spec deltas. Documenting limitations explicitly prevents confusion.

## Risks / Trade-offs

**Risk: Specs become stale** → Mitigation: Establish practice of updating specs with each code change. OpenSpec's archive workflow enforces spec updates when completing changes.

**Risk: Over-specification** → Mitigation: Focus on user-visible behavior and API contracts. Internal implementation details are intentionally omitted.

**Risk: Missing edge cases** → Mitigation: Specs can be extended. Initial pass covers happy paths and major edge cases. Community/usage will reveal gaps.

**Trade-off: Time investment** → Documentation takes time but enables faster future development and AI assistance. ROI increases with each agent interaction.

## Spec Organization

```
openspec/
├── specs/                          # Main specs (archived from changes)
│   ├── task-management/
│   │   └── spec.md
│   ├── meeting-management/
│   │   └── spec.md
│   ├── document-management/
│   │   └── spec.md
│   ├── ai-chat/
│   │   └── spec.md
│   ├── project-management/
│   │   └── spec.md
│   ├── integration-zoom/
│   │   └── spec.md
│   ├── integration-sheets-relay/
│   │   └── spec.md
│   ├── integration-mcp-server/
│   │   └── spec.md
│   └── notification-system/
│       └── spec.md
└── changes/
    └── document-existing-system/   # This change
        ├── proposal.md
        ├── design.md
        ├── tasks.md
        └── specs/                  # Delta specs to be archived
            └── [same structure]
```

## Migration Plan

1. Create specs in change directory (done)
2. Review specs for accuracy against codebase
3. Archive change to promote specs to main openspec/specs/
4. Future changes reference these specs and create deltas

No code migration required — this is additive documentation.

## Open Questions

1. **Spec versioning**: Should specs include version numbers tied to app versions?
2. **Cross-references**: Should specs link to each other (e.g., task-management referencing meeting-management)?
3. **Code references**: Should specs include file paths to implementation, or stay implementation-agnostic?

These can be resolved during review or in follow-up changes.
