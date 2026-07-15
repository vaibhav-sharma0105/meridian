## ADDED Requirements

### Requirement: Skill folder packages

The system SHALL support installing skill packages as filesystem directories in `~/.meridian/skills/`:
- Each subfolder is an independent skill package
- Packages contain a `skill.md` file (required) plus optional scripts, configs, and READMEs
- Packages are read-only in the UI (view file contents, no inline editing)
- Packages are deletable (removes entire folder from disk)
- Auto-discovered at runtime — no registration step

#### Scenario: List installed skill folders
- **WHEN** user navigates to Skills page
- **AND** `~/.meridian/skills/` contains one or more subdirectories
- **THEN** the Skill Folders panel appears below the skills list
- **AND** each folder shown with name, description (from skill.md frontmatter), and file tree

#### Scenario: No folders installed
- **WHEN** `~/.meridian/skills/` is empty or does not exist
- **THEN** the Skill Folders panel is hidden entirely
- **AND** no empty-state placeholder shown

### Requirement: Upload skill folder

The system SHALL allow uploading (installing) a skill folder from any local directory:
- Triggered via "Upload Skill" button in skills list toolbar
- Uses platform-native folder picker:
  - macOS: AppleScript `choose folder` (via `osascript`) for reliable directory selection
  - Windows/Linux: `rfd::FileDialog::pick_folder()` (Rust file dialog)
- Copies selected folder into `~/.meridian/skills/{folder_name}/`
- Validates folder structure before installation (see Validation requirement)

#### Scenario: Upload via macOS folder picker
- **WHEN** user clicks "Upload Skill" on macOS
- **THEN** native macOS folder picker opens (AppleScript-based)
- **WHEN** user selects a valid folder
- **THEN** folder is copied to `~/.meridian/skills/`
- **AND** skill folders query cache invalidated
- **AND** new folder appears in panel

#### Scenario: Upload via Windows/Linux folder picker
- **WHEN** user clicks "Upload Skill" on Windows or Linux
- **THEN** `rfd::FileDialog` folder picker opens
- **WHEN** user selects a valid folder
- **THEN** folder is copied to `~/.meridian/skills/`

#### Scenario: User cancels folder picker
- **WHEN** user dismisses the folder picker dialog
- **THEN** no action taken
- **AND** no error shown

#### Scenario: Folder already exists
- **WHEN** user selects a folder whose name matches an existing installation
- **THEN** error alert shown: "Skill folder '{name}' already exists"
- **AND** no files overwritten

### Requirement: Skill folder validation

The system SHALL validate uploaded folders before installation:
- `skill.md` file must exist in root
- `skill.md` must start with `---` (YAML frontmatter)
- Frontmatter must be closed with a second `---`
- Frontmatter must contain `name:` field
- Frontmatter must contain `description:` field

#### Scenario: Missing skill.md
- **WHEN** user uploads a folder without `skill.md`
- **THEN** error: "Invalid skill folder: missing 'skill.md'. A valid skill package must contain a skill.md file with YAML frontmatter (name, trigger, action) and markdown body (# Instructions section at minimum)."

#### Scenario: Missing frontmatter
- **WHEN** `skill.md` does not start with `---`
- **THEN** error: "Invalid skill.md: missing YAML frontmatter. File must start with '---' followed by YAML metadata (name, trigger, action) and end with '---'."

#### Scenario: Unclosed frontmatter
- **WHEN** `skill.md` starts with `---` but has no closing `---`
- **THEN** error: "Invalid skill.md: YAML frontmatter not closed. Expected a second '---' line after the metadata block."

#### Scenario: Missing name field
- **WHEN** frontmatter does not contain `name:`
- **THEN** error: "Invalid skill.md: missing required 'name' field in frontmatter."

#### Scenario: Missing description field
- **WHEN** frontmatter does not contain `description:`
- **THEN** error: "Invalid skill.md: missing required 'description' field in frontmatter."

### Requirement: Skill folder file tree viewer

The system SHALL display folder contents as an interactive file tree:
- Progressive disclosure: directories expand/collapse on click
- Hidden files (dot-prefixed) excluded from display
- Files sorted alphabetically within each directory
- Executable scripts highlighted (based on extension)
- File size shown for non-directory entries

#### Scenario: View file tree
- **WHEN** user expands a skill folder in the panel
- **THEN** hierarchical file tree displayed with icons
- **AND** directories can be expanded/collapsed
- **AND** executable scripts visually distinguished

#### Scenario: Read file contents
- **WHEN** user clicks a non-directory file in the tree
- **THEN** file contents displayed in a read-only viewer
- **AND** path traversal attempts blocked (path must stay within folder)

### Requirement: Skill folder script execution

The system SHALL support executing scripts within skill folders:
- Supported extensions: `.py`, `.js`, `.ts`, `.sh`, `.bash`, `.zsh`, `.rb`, `.pl`
- Platform-specific: `.ps1`, `.bat`, `.cmd` (Windows only)
- Scripts run with user permissions in skill folder as working directory
- Human-in-the-loop: confirmation dialog required before execution
- Path traversal protection: validates script path stays within `~/.meridian/skills/<folder>/`
- Script output (stdout) returned on success; stderr on failure

#### Scenario: Execute script with confirmation
- **WHEN** user clicks "Run" on an executable file
- **THEN** confirmation dialog shown with script path and warning
- **WHEN** user confirms
- **THEN** script executed with appropriate interpreter
- **AND** output displayed to user

#### Scenario: User declines execution
- **WHEN** user declines the confirmation dialog
- **THEN** no script executed
- **AND** no error shown

#### Scenario: Script fails
- **WHEN** executed script exits with non-zero status
- **THEN** error shown with exit code and stderr content

#### Scenario: Path traversal blocked
- **WHEN** script path attempts to escape folder boundary (e.g., `../../etc/passwd`)
- **THEN** error: "Invalid script path (path traversal attempt)"
- **AND** no execution attempted

### Requirement: Delete skill folder

The system SHALL allow deleting installed skill folders:
- Removes entire folder from `~/.meridian/skills/`
- Path validation ensures deletion stays within skills directory
- No confirmation for deletion (immediate)

#### Scenario: Delete folder
- **WHEN** user clicks delete on a skill folder
- **THEN** entire folder removed from disk
- **AND** folder disappears from panel immediately

#### Scenario: Folder not found
- **WHEN** attempting to delete a non-existent folder
- **THEN** error: "Skill folder '{name}' not found"

### Requirement: Platform-specific folder picker command

The system SHALL use a Tauri command (`pick_folder_dialog`) for native folder selection:
- macOS: Uses `osascript -e "choose folder"` for reliable NSOpenPanel behavior
- Windows/Linux: Uses `rfd::FileDialog::new().pick_folder()`
- Returns `Option<String>` — path on selection, null on cancel
- Runs on blocking thread pool (`tokio::task::spawn_blocking`)

#### Scenario: macOS folder picker
- **WHEN** `pick_folder_dialog` invoked on macOS
- **THEN** AppleScript `choose folder` dialog opens
- **AND** returns POSIX path of selected folder (trimmed)

#### Scenario: Non-macOS folder picker
- **WHEN** `pick_folder_dialog` invoked on Windows/Linux
- **THEN** `rfd::FileDialog` folder picker opens
- **AND** returns selected folder path as string

## Implementation Notes

**Key files:**
- `src-tauri/src/skills/folders.rs` — All filesystem operations (list, install, validate, delete, read, execute)
- `src-tauri/src/commands/skills.rs` — `pick_folder_dialog` and folder-related Tauri commands
- `src/components/skills/SkillFoldersPanel.tsx` — File tree UI and upload handler
- `src/components/skills/SkillsList.tsx` — "Upload Skill" button (folder-based, not file-based)
- `src/components/skills/SkillsPage.tsx` — Conditional folders panel rendering
- `src/lib/tauri.ts` — `pickFolderDialog`, `installSkillFolder`, `listSkillFolders`, etc.

**Why AppleScript on macOS:**
The `@tauri-apps/plugin-dialog` `open({ directory: true })` and `rfd::FileDialog::pick_folder()` both have issues on macOS where the "Open" button remains disabled for folder selection. This is due to how NSOpenPanel is attached as a sheet to the Tauri window. AppleScript's `choose folder` invokes a standalone folder picker that reliably works.

**Dependencies:**
- `rfd = "0.16"` in Cargo.toml (for Windows/Linux fallback)
- Platform-conditional compilation: `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]`
