## ADDED Requirements

### Requirement: Agentic features tour
The system SHALL provide an optional interactive tour of agentic features.

#### Scenario: Offer tour after basic onboarding
- **WHEN** user completes basic onboarding (AI provider, first project)
- **THEN** system shows: "Meridian has powerful automation features. Take a 5-minute tour?"
- **AND** user can choose: "Start Tour", "Maybe Later", "Skip Forever"

#### Scenario: Tour covers autonomy settings
- **WHEN** user is on autonomy step of tour
- **THEN** system highlights: autonomy mode selector, explains Manual/Supervised/Autonomous
- **AND** system shows interactive example: "What happens when Meridian suggests an action?"

#### Scenario: Tour covers skills
- **WHEN** user is on skills step of tour
- **THEN** system highlights: Skills nav item, built-in skills list
- **AND** system explains: "Skills automate recurring tasks. Try running one!"
- **AND** user can trigger a demo skill run

#### Scenario: Tour covers suggestions and governance
- **WHEN** user is on governance step of tour
- **THEN** system highlights: notification badge, approval queue
- **AND** system explains: "You control what Meridian does automatically"

#### Scenario: Tour progress persistence
- **WHEN** user exits tour mid-way
- **THEN** system saves progress
- **AND** system offers to resume from where user left off

### Requirement: Demo mode with synthetic data
The system SHALL provide a demo mode with pre-populated sample data.

#### Scenario: Enter demo mode
- **WHEN** user clicks "Try Demo Mode" in tour or Settings
- **THEN** system creates temporary demo project with: sample tasks (various statuses, priorities, assignees), sample meetings with transcripts, sample skills and suggestions
- **AND** system clearly marks UI as "Demo Mode"

#### Scenario: Demo mode isolation
- **WHEN** in demo mode
- **THEN** all changes affect only demo data
- **AND** real user data is unchanged
- **AND** demo data is stored separately (demo_* tables or in-memory)

#### Scenario: Exit demo mode
- **WHEN** user clicks "Exit Demo Mode"
- **THEN** system removes demo data
- **AND** system returns to normal view
- **AND** system shows summary: "You tried X features in demo mode"

#### Scenario: Demo mode timeout
- **WHEN** user is in demo mode for 30+ minutes
- **THEN** system prompts: "Still exploring? Continue or exit demo mode?"

### Requirement: Progressive feature tooltips
The system SHALL show contextual tooltips on first encounter of advanced features.

#### Scenario: First-time feature tooltip
- **WHEN** user encounters an advanced feature for first time (e.g., opens Skills view)
- **AND** user has not completed tour for this feature
- **THEN** system shows tooltip: brief explanation, "Got it" to dismiss, "Learn more" to see details

#### Scenario: Tooltip dismissal persistence
- **WHEN** user dismisses a tooltip with "Got it"
- **THEN** system records dismissal
- **AND** system does not show that tooltip again

#### Scenario: Reset tooltips option
- **WHEN** user clicks "Reset Tooltips" in Settings
- **THEN** system clears tooltip dismissal records
- **AND** tooltips will appear again on next feature encounter

### Requirement: Onboarding progress tracking
The system SHALL track and display onboarding progress.

#### Scenario: Track completion milestones
- **WHEN** user completes onboarding steps
- **THEN** system records: basic_onboarding_complete, agentic_tour_complete, demo_mode_tried, tooltips_seen

#### Scenario: Display onboarding checklist
- **WHEN** user opens Settings > Getting Started
- **THEN** system shows: completion checklist, progress percentage, "Continue where you left off" button

#### Scenario: Celebrate completion
- **WHEN** user completes all onboarding milestones
- **THEN** system shows celebratory message: "You've explored all of Meridian's features!"
- **AND** system hides onboarding prompts

### Requirement: Re-accessible tour
The system SHALL allow users to re-access the tour anytime.

#### Scenario: Access tour from settings
- **WHEN** user clicks "Retake Agentic Tour" in Settings
- **THEN** system starts tour from beginning
- **AND** user can skip steps they already know

#### Scenario: Access tour from help
- **WHEN** user clicks help icon and selects "Feature Tour"
- **THEN** system starts tour
- **AND** current context is preserved for after tour completion
