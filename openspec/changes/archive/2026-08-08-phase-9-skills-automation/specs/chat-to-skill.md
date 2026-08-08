# Chat-to-Skill Specification

## Overview

Convert multi-turn conversations into reusable skills with AI-guided clarification.

## Trigger Conditions

### Minimum Requirements
- At least 3 conversation turns
- User expressed a repeatable pattern (detected via intent analysis)
- Not a one-off question/answer

### Proactive Suggestion
AI suggests skill creation when detecting:
- "Can you do this again..."
- "Every time I need to..."
- "Make this a template..."
- Similar requests across conversations (cross-session pattern)

## Extraction Flow

### Step 1: Pattern Detection
```rust
pub struct PatternDetection {
    confidence: f64,           // 0.0-1.0
    detected_trigger: String,  // What initiates the workflow
    detected_inputs: Vec<String>,  // Required user inputs
    detected_output: String,   // Expected output format
}
```

### Step 2: Clarification Questions

AI asks (in order):
1. **Trigger**: "When should this skill run? (e.g., manually, on schedule, on event)"
2. **Scope**: "Should this apply to all projects or specific ones?"
3. **Inputs**: "What information do you need to provide each time?"
4. **Output**: "What format should the output be? (text, file, tasks)"

### Step 3: Skill Generation

Generate Anthropic-standard skill format:

```markdown
---
name: weekly-status-report
description: Generate a weekly status report from tasks and meetings
trigger: manual
inputs:
  - name: week_ending
    type: date
    description: The Friday date for the report
output:
  format: markdown
  destination: chat
---

# Weekly Status Report

Generate a status report for the week ending {{week_ending}}.

## Instructions

1. Gather all completed tasks from the past 7 days
2. Summarize key meetings and decisions
3. List upcoming tasks and blockers
4. Format as a professional status update

## Output Format

```markdown
# Status Report - Week Ending {{week_ending}}

## Completed This Week
- ...

## Key Decisions
- ...

## Next Week
- ...

## Blockers
- ...
```
```

### Step 4: Preview & Edit

Always show full skill preview before saving:
- Editable text area with syntax highlighting
- "Save" and "Cancel" buttons
- Option to test-run before saving

## UI Components

### ChatToSkillDialog

```tsx
interface ChatToSkillDialogProps {
  conversationId: string;
  suggestedPattern: PatternDetection;
  onSave: (skill: CreateSkillInput) => void;
  onCancel: () => void;
}
```

Wizard steps:
1. Pattern confirmation
2. Clarification questions
3. Generated skill preview
4. Edit & save

### Inline Suggestion

In chat, show subtle prompt:
```
💡 This looks like a reusable pattern. Create a skill?  [Yes] [Not now]
```

## Storage

Created skills saved to:
- Database: `skills` table with `source: "chat"`, `source_conversation_id`
- Local files: `~/.meridian/skills/{name}/skill.md`

## Quality Guardrails

- Minimum 3 turns required
- Must detect clear input/output pattern
- Generated skill must be syntactically valid
- User must review before saving (no auto-save)
