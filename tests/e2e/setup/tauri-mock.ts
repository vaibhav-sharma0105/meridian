/**
 * Tauri API mock for Playwright tests.
 *
 * Inject this as a page init script so that @tauri-apps/api calls resolve
 * against mock data rather than a live Rust backend.
 *
 * Usage in a test:
 *   await page.addInitScript({ path: "tests/e2e/setup/tauri-mock.ts" });
 *   // or inline:
 *   await page.addInitScript(tauriMockScript, mockOverrides);
 */

export const MOCK_PROJECTS = [
  {
    id: "proj-1",
    name: "Alpha Project",
    color: "#6366f1",
    archived_at: null,
    open_task_count: 3,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  },
  {
    id: "proj-2",
    name: "Beta Project",
    color: "#10b981",
    archived_at: null,
    open_task_count: 1,
    created_at: "2025-01-02T00:00:00Z",
    updated_at: "2025-01-02T00:00:00Z",
  },
];

export const MOCK_TASKS = [
  {
    id: "task-1",
    project_id: "proj-1",
    meeting_id: "mtg-1",
    title: "Fix the login bug",
    description: "Users cannot log in when 2FA is enabled and the session token expires",
    assignee: "Alice",
    assignee_confidence: "committed",
    assignee_source_quote: null,
    due_date: "2026-05-01",
    due_confidence: "committed",
    due_source_quote: null,
    status: "open",
    priority: "critical",
    confidence_score: 0.95,
    tags: '["bug","auth"]',
    kanban_column: "open",
    kanban_order: 0,
    notes: null,
    is_duplicate: false,
    duplicate_of_id: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    completed_at: null,
  },
  {
    id: "task-2",
    project_id: "proj-1",
    meeting_id: null,
    title: "Write API documentation",
    description: "Document all public endpoints with examples and edge cases",
    assignee: "Bob",
    assignee_confidence: "inferred",
    assignee_source_quote: null,
    due_date: null,
    due_confidence: "none",
    due_source_quote: null,
    status: "in_progress",
    priority: "medium",
    confidence_score: 0.7,
    tags: '["docs"]',
    kanban_column: "in_progress",
    kanban_order: 0,
    notes: null,
    is_duplicate: false,
    duplicate_of_id: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    completed_at: null,
  },
  {
    id: "task-3",
    project_id: "proj-1",
    meeting_id: null,
    title: "Deploy to staging",
    description: null,
    assignee: null,
    assignee_confidence: "unassigned",
    assignee_source_quote: null,
    due_date: null,
    due_confidence: "none",
    due_source_quote: null,
    status: "done",
    priority: "low",
    confidence_score: null,
    tags: "[]",
    kanban_column: "done",
    kanban_order: 0,
    notes: null,
    is_duplicate: false,
    duplicate_of_id: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    completed_at: new Date().toISOString(),
  },
];

export const MOCK_MEETINGS = [
  {
    id: "mtg-1",
    project_id: "proj-1",
    title: "Sprint Planning Q1",
    platform: "zoom",
    raw_transcript: null,
    ai_summary: "Team discussed sprint goals and assigned tasks.",
    summary: "Team discussed sprint goals and assigned tasks.",
    decisions: "Move to two-week sprints starting next cycle.",
    health_score: 82,
    health_breakdown: null,
    attendees: "Alice,Bob,Carol",
    duration_minutes: 45,
    meeting_at: "2026-04-01T10:00:00Z",
    ingested_at: "2026-04-01T11:00:00Z",
    created_at: "2026-04-01T11:00:00Z",
    updated_at: "2026-04-01T11:00:00Z",
  },
];

/**
 * The init script string to inject into page context.
 * Mocks window.__TAURI_INTERNALS__.invoke with configurable overrides.
 */
export function buildTauriMockScript(overrides: Record<string, unknown> = {}) {
  const data = {
    get_projects: MOCK_PROJECTS,
    get_tasks_for_project: MOCK_TASKS,
    get_all_tasks: MOCK_TASKS,
    get_meetings_for_project: MOCK_MEETINGS,
    get_meeting: MOCK_MEETINGS[0],
    get_notifications: [],
    get_pending_imports: [],
    count_pending_imports: 0,
    get_connection: null,
    get_connection_zoom: null,
    get_connection_gmail: null,
    get_app_setting: null,
    get_app_settings: { onboarding_complete: "true", theme: "light", language: "en" },
    sync_connections: { new_imports: 0, skipped_duplicates: 0, errors: [] },
    create_task: MOCK_TASKS[0],
    update_task: MOCK_TASKS[0],
    delete_task: null,
    rename_meeting: null,
    delete_meeting: null,
    ...overrides,
  };

  return `
    (function() {
      const mockData = ${JSON.stringify(data)};
      let _cbId = 0;
      const _callbacks = {};

      // Tauri v2 needs transformCallback for event listeners
      function transformCallback(callback, once) {
        const id = ++_cbId;
        _callbacks[id] = { callback, once };
        return id;
      }

      window.__TAURI_INTERNALS__ = {
        invoke: function(cmd, args) {
          // Handle Tauri plugin event commands silently
          if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') {
            return Promise.resolve(_cbId++);
          }
          // Strip "plugin:X|" prefix for lookup
          const key = cmd.replace(/^plugin:[^|]+\\|/, '');
          if (cmd in mockData) return Promise.resolve(mockData[cmd]);
          if (key in mockData) return Promise.resolve(mockData[key]);
          return Promise.resolve(null);
        },
        transformCallback,
        convertFileSrc: function(path) { return path; },
        metadata: {
          currentWindow: { label: 'main' },
          windows: [{ label: 'main' }],
        },
        postMessage: function() {},
      };
    })();
  `;
}
