# Multi-Skill Execution Specification

## Overview

AI autonomously selects and chains multiple skills based on user intent and skill frontmatter.

## Skill Selection Algorithm

### Step 1: Intent Extraction
Parse user message to extract:
- Primary intent (what they want to accomplish)
- Secondary context (project, time range, etc.)

### Step 2: Skill Matching

For each enabled skill:
1. Read `name` and `description` from frontmatter
2. Compute embedding similarity with user intent
3. Score = semantic_similarity + keyword_bonus + context_bonus

```rust
pub struct SkillMatch {
    skill_id: String,
    score: f64,
    match_reason: String,
}

pub fn match_skills(
    intent: &str,
    skills: &[Skill],
    context: &ChatContext,
) -> Vec<SkillMatch> {
    skills.iter()
        .map(|s| compute_match(intent, s, context))
        .filter(|m| m.score > 0.5)
        .sorted_by(|a, b| b.score.partial_cmp(&a.score))
        .collect()
}
```

### Step 3: Confidence Thresholds

| Score Range | Action |
|-------------|--------|
| ≥ 0.85 | Auto-execute skill |
| 0.7 - 0.85 | Execute with brief confirmation |
| 0.5 - 0.7 | Ask user to choose from options |
| < 0.5 | No skill match; regular AI response |

### Step 4: Dependency Resolution

If skill A declares `depends_on: B` in frontmatter:
1. Check if B is in matched skills
2. If not, add B to execution list
3. Execute B before A

```yaml
# skill.md frontmatter
---
name: weekly-report
depends_on:
  - gather-tasks
  - summarize-meetings
---
```

### Step 5: Execution Order

```rust
pub fn resolve_execution_order(matches: &[SkillMatch]) -> Vec<&Skill> {
    // Topological sort based on depends_on
    let mut sorted = Vec::new();
    let mut visited = HashSet::new();
    
    for m in matches {
        visit(m.skill, &mut sorted, &mut visited);
    }
    
    sorted
}
```

## Chaining Behavior

### Output Piping
Previous skill's output available as `{{previous_output}}` in next skill.

### Error Handling
If a skill fails:
1. Log error
2. Ask user: "Skill X failed. Continue with remaining skills?"
3. If yes, skip failed skill and continue
4. If no, abort chain

### Approval Modes in Chains

| Approval Mode | Chain Behavior |
|---------------|----------------|
| `auto` | Execute without prompts |
| `notify` | Execute, notify after completion |
| `approve_first` | Approve first execution; auto after |
| `approve_always` | Approve each skill in chain |

## Fallback Behavior

When no skill matches confidently (all < 0.5):
1. AI responds normally without skills
2. Optionally suggest: "I found some skills that might help: [list top 3]"

## UI Indicators

### Skill Selection Indicator
Show in chat which skills were selected:
```
🔧 Using skills: gather-tasks → summarize-meetings → weekly-report
```

### Progress Indicator
During multi-skill execution:
```
⏳ Running gather-tasks (1/3)...
✓ gather-tasks complete
⏳ Running summarize-meetings (2/3)...
```

## MCP Integration

New MCP tools for external clients:

```rust
// Queue a skill for execution
pub async fn queue_skill(skill_id: String, inputs: Value) -> Result<String, String>

// Get result of queued skill
pub async fn get_skill_result(execution_id: String) -> Result<SkillResult, String>
```
