---
name: weekly-status-report
description: Generate a weekly status report summarizing completed tasks and upcoming work
trigger: schedule
cron: "0 17 * * 5"  # Every Friday at 5 PM
approval: notify
---

# Weekly Status Report

Generate a comprehensive weekly status report that summarizes:
- Tasks completed this week
- Tasks in progress
- Blockers and challenges
- Upcoming priorities for next week

## Context
- Include tasks from the past 7 days
- Group by project if multiple projects exist
- Highlight overdue or high-priority items

## Output Format
Format as a professional status update suitable for sharing with stakeholders.
Use clear sections with bullet points for easy scanning.

## Tone
- Professional but concise
- Focus on accomplishments and forward momentum
- Be transparent about blockers
