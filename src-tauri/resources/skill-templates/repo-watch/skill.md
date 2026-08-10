---
name: repo-watch
description: Monitor GitHub repository activity and surface relevant updates
trigger: schedule
cron: "0 8 * * *"  # Daily at 8 AM
approval: auto
network: allowlist
network_hosts:
  - api.github.com
---

# Repository Activity Monitor

Watch connected GitHub repositories for relevant activity and generate daily summaries.

## What to Track
1. **New Issues**: Issues opened in the past 24 hours
2. **PR Activity**: Pull requests opened, merged, or needing review
3. **Mentions**: Activity where team members are mentioned
4. **Deployments**: Recent deployment or release activity
5. **CI Failures**: Build or test failures that need attention

## Filtering
- Focus on repositories mapped to active projects
- Prioritize items assigned to or mentioning current user
- Highlight items blocking other work

## Output Format
Daily digest format:
- **Needs Attention**: Items requiring immediate action
- **Updates**: General activity summary
- **Upcoming**: Scheduled items or milestones

## Integration
Uses cached GitHub data from integrations.
Does not make additional API calls unless cache is stale.
