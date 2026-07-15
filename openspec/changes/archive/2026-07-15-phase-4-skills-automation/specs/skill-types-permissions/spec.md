## ADDED Requirements

### Requirement: Three-tier skill type system

The system SHALL distinguish three skill types with different permission models:

| Type | Editable | Deletable | Source |
|------|----------|-----------|--------|
| Built-in | Yes | No (reset only) | `resources/builtin-skills/templates.json`, loaded on first launch |
| User-created | Yes | Yes | Created via editor, imported, or cloned |
| Folder packages | No (read-only) | Yes | Uploaded from `~/.meridian/skills/` directory |

#### Scenario: Built-in skill protection
- **WHEN** user attempts to delete a built-in skill
- **THEN** delete option not shown in skill card menu
- **AND** `delete_skill` command rejects with error if called directly

#### Scenario: User-created skill full control
- **WHEN** user creates or imports a skill
- **THEN** skill can be edited, cloned, exported, and deleted
- **AND** all CRUD operations available from card menu

#### Scenario: Folder package read-only
- **WHEN** user views a folder package in the UI
- **THEN** file contents viewable but not editable inline
- **AND** package can be deleted (removes folder from disk)
- **AND** executable scripts can be run with confirmation

### Requirement: Built-in skill flag (`is_builtin`)

The system SHALL track built-in status with a database column:
- Migration v013 adds `is_builtin INTEGER NOT NULL DEFAULT 0` to skills table
- `load_builtin_skills()` sets `is_builtin: true` when seeding templates
- Column exposed in Skill model and TypeScript interface

#### Scenario: Database migration
- **WHEN** app starts after v013 migration applied
- **THEN** `skills` table has `is_builtin` column defaulting to 0
- **AND** existing user skills remain `is_builtin = 0`
- **AND** built-in skills created with `is_builtin = 1`

### Requirement: Built-in skill delete protection

The system SHALL prevent deletion of built-in skills:
- `delete_skill` command checks `is_builtin` flag before proceeding
- If `is_builtin = true`, returns error "Cannot delete built-in skill. Use 'Reset defaults' to restore."
- UI hides the Delete option from skill card menu for built-in skills

#### Scenario: Attempt delete via command
- **WHEN** `delete_skill` called with a built-in skill ID
- **THEN** error returned: cannot delete built-in skill
- **AND** skill remains in database unchanged

#### Scenario: UI hides delete for built-in
- **WHEN** rendering skill card menu for a built-in skill
- **THEN** Delete button not rendered
- **AND** separator above delete not rendered
- **AND** Edit, History, Clone, Export still available

### Requirement: Built-in skill reset mechanism

The system SHALL support resetting built-in skills to defaults:
- "Reset defaults" button in Skills page header
- Confirmation dialog before proceeding
- `reset_builtin_skills` command:
  1. Deletes all skills WHERE `is_builtin = 1`
  2. Clears `app_settings.builtin_skills_initialized` flag
  3. Re-loads templates from `resources/builtin-skills/templates.json`
- Returns list of newly created skill IDs
- User-created skills untouched

#### Scenario: Reset built-in skills
- **WHEN** user clicks "Reset defaults" and confirms
- **THEN** all built-in skills deleted and re-created from templates
- **AND** user-created skills preserved
- **AND** skills list refreshes to show fresh templates

#### Scenario: Cancel reset
- **WHEN** user clicks "Reset defaults" then cancels confirmation
- **THEN** no changes made

### Requirement: Built-in skill initialization

The system SHALL auto-load built-in templates on first launch:
- Gated by `app_settings.builtin_skills_initialized = "true"`
- `initialize_builtin_skills` command checks gate, loads templates if not set
- Templates embedded at compile time via `include_str!("../../resources/builtin-skills/templates.json")`
- After loading, sets gate flag to prevent re-initialization

#### Scenario: First launch
- **WHEN** app starts with `builtin_skills_initialized` not set
- **THEN** 5 built-in skills created from templates
- **AND** `builtin_skills_initialized = "true"` saved to settings
- **AND** all created skills have `is_builtin = true`

#### Scenario: Subsequent launches
- **WHEN** app starts with `builtin_skills_initialized = "true"`
- **THEN** no skills created
- **AND** existing skills unchanged

### Requirement: Built-in badge in UI

The system SHALL visually distinguish built-in skills:
- Small "Built-in" badge shown next to skill name on card
- Styled: `text-[10px] px-1.5 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500`
- Only visible when `skill.is_builtin === true`

#### Scenario: Built-in skill card
- **WHEN** rendering a skill card with `is_builtin = true`
- **THEN** "Built-in" badge appears after skill name
- **AND** no Delete option in menu

#### Scenario: User skill card
- **WHEN** rendering a skill card with `is_builtin = false`
- **THEN** no "Built-in" badge shown
- **AND** all menu options including Delete available

## Implementation Notes

**Key files:**
- `src-tauri/src/db/migrations/v013_skills_builtin.rs` — Adds `is_builtin` column
- `src-tauri/src/skills/builtin.rs` — `load_builtin_skills()`, `reset_builtin_skills()` with `include_str!()`
- `src-tauri/src/skills/repository.rs` — `delete_skill()` checks `is_builtin` flag
- `src-tauri/src/commands/skills.rs` — `initialize_builtin_skills`, `reset_builtin_skills` commands
- `src-tauri/resources/builtin-skills/templates.json` — 5 skill template definitions
- `src/components/skills/SkillCard.tsx` — Conditional "Built-in" badge, conditional Delete menu item
- `src/components/skills/SkillsPage.tsx` — "Reset defaults" button with confirmation
