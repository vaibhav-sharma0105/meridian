---
name: task-cleanup
description: Review and suggest cleanup for stale or duplicate tasks
trigger: schedule
cron: "0 9 * * 1"  # Every Monday at 9 AM
approval: approve_first
---

# Task Cleanup Review

Analyze the task list and identify items that may need attention:

## Analysis Criteria
1. **Stale Tasks**: Open tasks with no updates in 30+ days
2. **Potential Duplicates**: Tasks with very similar titles or descriptions
3. **Missing Details**: Tasks lacking descriptions or assignments
4. **Priority Mismatches**: High priority tasks that haven't been touched
5. **Completed but Open**: Tasks that appear done but aren't marked complete

## Output Format
For each identified issue, provide:
- Task ID and title
- Issue type
- Recommended action (archive, merge, update, complete)
- Confidence level (high/medium/low)

## Actions
Suggest specific changes but require approval before:
- Archiving any tasks
- Merging duplicate tasks
- Changing task status

## Tone
- Helpful assistant suggesting improvements
- Focus on maintaining a clean, actionable task list
