## Why

With Phases 8-13 complete, Meridian excels at per-project intelligence. However, users working across multiple projects lack visibility into cross-project dependencies, velocity trends, and system usage. New users also need better onboarding to discover agentic features. This phase adds cross-project intelligence and comprehensive analytics.

## What Changes

- **Cross-Project Blockers**: Detect tasks in one project blocking work in another; surface in My Activity
- **Meeting Consolidation**: Identify duplicate or related meetings across projects that could be combined
- **Velocity Trends**: Analyze task completion rates across projects; identify slowing projects
- **Interactive Onboarding**: Step-by-step tour of autonomy settings, skills, and integrations
- **Demo Mode**: Sample data showing Meridian in action for new users
- **Progressive Tooltips**: Contextual hints for agentic features as user encounters them
- **Activity Metrics**: Dashboard showing task completion, meeting extraction rates, suggestion acceptance
- **AI Token Usage**: Track and display token consumption across AI features
- **Storage Breakdown**: Show database size, embedding storage, cache usage
- **Productivity Insights**: Derived metrics (velocity, estimation accuracy, response times)
- **Exportable Reports**: PDF/CSV export of analytics data

## Capabilities

### New Capabilities
- `cross-project-intelligence`: Blocker detection, meeting consolidation, velocity analysis across projects
- `onboarding-tour`: Interactive walkthrough of Meridian's agentic capabilities
- `demo-mode`: Pre-populated sample data for feature exploration
- `usage-analytics`: Comprehensive metrics dashboard with export

### Modified Capabilities
- `my-activity-dashboard`: Include cross-project insights
- `suggestions`: Generate cross-project blocker suggestions
- `pattern-observation`: Compute cross-project velocity metrics

## Technical Specifications

### Cross-Project Query Performance

**Graph Structure:**
- `cross_project_links` table with indexes on `(source_type, source_id)` and `(target_type, target_id)`
- Materialized relationships: Links computed by daemon, not on-demand
- Incremental updates: When task updated, only recompute its links (not full graph)

**Detection Heuristics:**
| Link Type | Detection Method |
|-----------|------------------|
| Blocks | Task mentions another project's task by ID or exact title |
| Related | Embedding similarity > 0.8 between tasks in different projects |
| Duplicate Meeting | Same attendees + similar title + within 7 days |

**Computation Schedule:**
- Full analysis: Nightly at 2am
- Incremental: On task/meeting create/update, queue link analysis job
- Job processing: Within 5 minutes of change

### Demo Mode Implementation

**Data Isolation:**
- Separate SQLite database: `~/.meridian/demo.db`
- Demo flag in app state; all queries route to demo DB when active
- No mixing: Real data never visible in demo mode; demo data never persists to real DB

**Sample Data:**
- 2 projects: "Mobile App" and "Backend API"
- 15 tasks per project with varied statuses, priorities, assignees
- 5 meetings with extracted tasks
- 3 sample skills (already executed with results)
- Pre-configured integrations with mock cache data
- Cross-project blocker example: "Mobile depends on Backend API auth fix"

**Entry/Exit:**
- Settings → "Try Demo Mode" button
- Banner when in demo: "Demo Mode — using sample data. [Exit Demo]"
- Exit: Returns to real database; no data migration

### Onboarding Timing

**Progressive Disclosure:**
- First launch: Basic tour only (navigation, task creation)
- After 5 tasks: "Discover AI features" prompt
- After first meeting import: "Set up integrations" prompt
- After 1 week active use: "Explore automation with skills" prompt

**Feature-Triggered Hints:**
| User Action | Tooltip Shown |
|-------------|---------------|
| Opens task with linked GitHub issue | "You can view GitHub details inline" |
| Receives first AI suggestion | "Meridian learns from your patterns" |
| Creates 3rd skill | "Skills can run on schedules too" |
| Ignores attention section 3 times | "You can customize attention thresholds" |

**Skip Option:**
- "Don't show tips" toggle in settings
- Individual tips have "Got it, don't show again" option

### Analytics for Single User

**Personal Productivity Focus:**

| Metric | What It Shows | Why Single User Cares |
|--------|---------------|----------------------|
| Tasks completed/week | Personal velocity trend | "Am I getting more done?" |
| Avg task duration | Time management | "How long do things actually take?" |
| Suggestion acceptance rate | AI utility | "Is Meridian helping me?" |
| Meeting → Task ratio | Meeting efficiency | "Am I capturing action items?" |
| Overdue rate | Planning accuracy | "Am I overcommitting?" |

**Not Shown (Team Metrics):**
- Team velocity comparisons
- Individual contributor rankings
- Workload distribution charts

**Exportable Reports:**
- Weekly summary: PDF with charts, exportable
- Raw data: CSV export of tasks, meetings, patterns
- Date range selector: Last 7/30/90 days or custom

### Token Usage Tracking

**Granularity:**
- Track per feature: AI chat, skill execution, suggestion generation, draft creation
- Track per day: Daily totals stored in `usage_metrics`
- Running total: Month-to-date visible in settings

**Display:**
- Settings → Usage: Bar chart of daily token usage
- Breakdown: Pie chart by feature
- Trend: "You've used 50% more tokens this week" (if significant change)

**No Billing Integration:**
- Informational only; user manages API billing separately
- "Learn more about API pricing" link to Anthropic/OpenAI docs

### Storage Breakdown

**Components Tracked:**
- Database: `meridian.db` size
- Embeddings: `vectors/` directory size (if local embeddings used)
- Integration cache: Derived from `integration_cache` table
- Created files: `created_files/` directory size
- Skills: `skills/` directory size

**Display:**
- Horizontal bar chart showing relative sizes
- Total at top: "Meridian is using 1.2 GB"
- Per-component: "Integration cache: 400 MB [Clear]"

**Actions:**
- Clear integration cache (respects archive settings)
- Clear old created files (date selector)
- Compact database (VACUUM)

## Impact

- **Database**: New `cross_project_links` table with indexes; `usage_metrics` table for analytics; `onboarding_state` for tour progress; `demo.db` for demo mode
- **Backend**: Cross-project analyzer daemon job with incremental updates; metrics aggregation; report generation (PDF via wkhtmltopdf or similar); demo database initialization
- **Frontend**: AnalyticsDashboard view with charts (recharts); OnboardingTour overlay with step tracking; DemoModeBanner; ExportReportDialog; StorageBreakdown component with actions; FeatureTooltips system
- **Performance**: Cross-project queries use pre-computed links; metrics pre-aggregated daily; demo mode routes to separate DB file
