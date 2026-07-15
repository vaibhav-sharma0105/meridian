## ADDED Requirements

### Requirement: Export skill as directory package

The system SHALL allow exporting a skill as a directory package with `skill.md` inside:
- Export triggered from skill card overflow menu (Download icon → Export)
- Uses platform-native folder picker to select export location:
  - macOS: AppleScript `choose folder with prompt "Choose location to export skill"`
  - Windows/Linux: `rfd::FileDialog::new().pick_folder()`
- Creates a directory named after the skill (kebab-case slug)
- Writes `skill.md` (YAML+MD format) inside that directory
- Backend command: `export_skill_to_directory(skill_md_content, skill_name)`

#### Scenario: Export skill via folder picker
- **WHEN** user clicks "Export" from skill card menu
- **THEN** native folder picker opens to choose export location
- **WHEN** user selects a directory
- **THEN** skill directory created: `{selected_path}/{skill-slug}/skill.md`
- **AND** menu closes

#### Scenario: User cancels export
- **WHEN** user dismisses the folder picker dialog
- **THEN** no directory created
- **AND** menu closes silently (no error)

#### Scenario: Export error handling
- **WHEN** directory creation or file write fails
- **THEN** alert shown with specific error message
- **AND** menu closes

### Requirement: Export format (YAML+MD)

The exported `skill.md` SHALL follow the Anthropic skill standard format:
```yaml
---
name: Weekly Progress Report
description: Generate weekly summary every Monday
trigger:
  type: schedule
  cron: "0 9 * * 1"
action:
  type: summarize
settings:
  approval_mode: notify
  category: reporting
---

# Instructions

Summarize the week's progress using {{tasks}} and {{meetings}}.
```

The format is:
- YAML frontmatter (name, description, trigger, action, settings)
- Markdown body with `# Section` headings (Instructions, Context, Output Format, Examples)
- Serialized from skill data via `skillToSkillFile()` in `src/lib/skill-format.ts`

### Requirement: Import skill (folder-based upload)

The system SHALL support importing skills as folder packages:
- "Upload Skill" button in skills list toolbar replaces file-based import
- Uses `pick_folder_dialog` command for native folder picker
- Copies folder to `~/.meridian/skills/` via `install_skill_folder`
- Validates folder structure (see skill-folder-packages spec)
- Invalidates `["skill-folders"]` React Query cache

#### Scenario: Upload skill folder
- **WHEN** user clicks "Upload Skill" button
- **THEN** native folder picker opens
- **WHEN** user selects a valid skill folder
- **THEN** folder validated and copied to `~/.meridian/skills/`
- **AND** folders panel updates to show new package

#### Scenario: Upload invalid folder
- **WHEN** user selects a folder that fails validation
- **THEN** alert shown with specific validation error
- **AND** no files copied

#### Scenario: User cancels upload
- **WHEN** user dismisses the folder picker
- **THEN** no action taken

### Requirement: Built-in skills visibility toggle

The system SHALL provide a toggle to show/hide built-in (shared) skills:
- Small switch control with "Built-in" label in the toolbar
- When ON (default): all skills shown (user-created + built-in)
- When OFF: only user-created skills shown (filters out `shared === true`)
- Client-side filtering only — no backend call needed

#### Scenario: Hide built-in skills
- **WHEN** user toggles "Built-in" switch OFF
- **THEN** skills with `shared === true` are hidden from list
- **AND** search and category filters still apply to remaining skills

#### Scenario: Show built-in skills
- **WHEN** user toggles "Built-in" switch ON (default)
- **THEN** all skills shown regardless of `shared` flag

### Requirement: Clone shared skill

The system SHALL allow cloning built-in skills:
- Creates a new skill owned by current user
- Copies all configuration
- Independent from original (no sync)

#### Scenario: Clone skill
- **WHEN** user clicks "Clone" on a built-in skill
- **THEN** new skill created with user as owner
- **AND** name prefixed with "Copy of "
- **AND** shared=false on clone

#### Scenario: Edit cloned skill
- **WHEN** user edits their cloned skill
- **THEN** changes do not affect original
- **AND** original does not affect clone

## REMOVED Requirements

### Requirement: Community skills tab (REMOVED)

~~The system SHALL provide a "Community" tab for browsing shared skills.~~

**Reason:** Removed because there is no community backend or mechanism for users to upload skills to a shared store. The Community/My Skills toggle was confusing with no actionable path. Replaced with a simpler "Built-in" on/off toggle that controls visibility of seeded template skills.

### Requirement: Community skill discovery (REMOVED)

~~The system SHALL provide skill discovery with search, category filter, and sort by popularity.~~

**Reason:** No multi-user community exists. Discovery is limited to the 5 built-in templates and any imported `.skill.json` files.

### Requirement: Team visibility toggle (REMOVED)

~~The system SHALL support marking skills as shared with `shared` boolean for team visibility.~~

**Reason:** Meridian is a local-first single-user app. The `shared` field now only distinguishes built-in template skills from user-created ones. No multi-user sharing functionality exists.

### Requirement: Shared skill attribution (REMOVED)

~~The system SHALL track `cloned_from_id` and `original_owner` for provenance.~~

**Reason:** Single-user context makes provenance tracking unnecessary. Cloned skills are independent copies.

### Requirement: Browser-based download export (REMOVED)

~~The system SHALL export via `document.createElement("a").click()` with Blob URL.~~

**Reason:** Blob URL downloads do not work in Tauri v2 webview. Replaced with directory-based export using native folder picker + `export_skill_to_directory` command.

### Requirement: File-based JSON import (REMOVED)

~~The system SHALL support importing skills from `.json` / `.skill.json` files via hidden file input.~~

**Reason:** Replaced with folder-based upload that installs skill packages as directories in `~/.meridian/skills/`. The folder approach better aligns with the Anthropic skill standard format (directory with `skill.md`) and supports multi-file packages (scripts, configs, README).

### Requirement: Tauri save dialog export (REMOVED)

~~The system SHALL export via `save()` from `@tauri-apps/plugin-dialog` + `writeTextFile()` from `@tauri-apps/plugin-fs`.~~

**Reason:** Replaced with `export_skill_to_directory` Rust command that uses platform-native folder picker (AppleScript on macOS, rfd on Windows/Linux) to create a skill package directory with `skill.md` inside. This produces a folder that can be directly re-uploaded as a skill package.
