## ADDED Requirements

### Requirement: Global autonomy mode
The system SHALL maintain a global autonomy mode setting that determines the default behavior for all agent actions.

#### Scenario: Set global autonomy to Manual
- **WHEN** user sets global autonomy mode to "Manual"
- **THEN** all agent actions require explicit user approval before execution

#### Scenario: Set global autonomy to Supervised
- **WHEN** user sets global autonomy mode to "Supervised"
- **THEN** low-risk actions execute automatically
- **AND** medium-risk actions request approval
- **AND** high-risk actions always request approval

#### Scenario: Set global autonomy to Autonomous
- **WHEN** user sets global autonomy mode to "Autonomous"
- **THEN** low and medium-risk actions execute automatically
- **AND** high-risk actions request approval
- **AND** critical-risk actions always request approval

### Requirement: Per-integration autonomy override
The system SHALL allow users to override the global autonomy mode for specific integrations.

#### Scenario: Override integration to Manual
- **WHEN** user sets GitHub integration autonomy to "Manual"
- **AND** global autonomy is "Supervised"
- **THEN** GitHub actions require explicit approval regardless of risk level
- **AND** other integrations follow global "Supervised" behavior

#### Scenario: Inherit global setting
- **WHEN** user sets integration autonomy to "Inherit"
- **THEN** integration uses the global autonomy mode setting

### Requirement: Per-skill autonomy override
The system SHALL allow users to override autonomy mode for individual skills.

#### Scenario: Override skill to Autonomous
- **WHEN** user sets "Weekly Summary" skill autonomy to "Autonomous"
- **AND** global autonomy is "Manual"
- **THEN** "Weekly Summary" skill can execute low/medium-risk actions automatically
- **AND** other skills follow global "Manual" behavior

#### Scenario: Skill inherits integration autonomy
- **WHEN** skill action targets a specific integration
- **AND** skill autonomy is "Inherit"
- **THEN** skill uses the integration's autonomy setting (or global if integration inherits)

### Requirement: Autonomy mode inheritance chain
The system SHALL resolve autonomy mode through inheritance: skill → integration → global.

#### Scenario: Resolve autonomy for skill action
- **WHEN** skill with autonomy "Inherit" executes action on integration with autonomy "Inherit"
- **THEN** system uses global autonomy mode

#### Scenario: Explicit override breaks inheritance
- **WHEN** skill has explicit autonomy mode set (not "Inherit")
- **THEN** system uses skill's autonomy mode regardless of integration or global settings

### Requirement: Autonomy settings persistence
The system SHALL persist autonomy settings in the database and load them on app startup.

#### Scenario: Persist global autonomy
- **WHEN** user changes global autonomy mode
- **THEN** setting is saved to app_settings table
- **AND** survives app restart

#### Scenario: Persist integration autonomy
- **WHEN** user changes integration autonomy mode
- **THEN** setting is saved to integrations table autonomy_mode column

### Requirement: Autonomy mode UI
The system SHALL provide a visual settings UI for configuring autonomy modes.

#### Scenario: View autonomy settings
- **WHEN** user opens Autonomy settings panel
- **THEN** system displays global mode with radio buttons (Manual/Supervised/Autonomous)
- **AND** system displays list of integrations with their override settings
- **AND** system displays list of skills with their override settings

#### Scenario: Visual autonomy indicator
- **WHEN** user is on any screen with agent-controlled elements
- **THEN** system displays current effective autonomy mode indicator
- **AND** indicator is color-coded (green=Autonomous, yellow=Supervised, red=Manual)
