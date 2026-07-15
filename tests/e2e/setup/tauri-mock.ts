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
    parent_task_id: null,
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
    parent_task_id: null,
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
    parent_task_id: null,
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
    archive_task: null,
    unarchive_task: null,
    move_task_to_project: null,
    rename_meeting: null,
    delete_meeting: null,
    // Archived projects
    get_archived_projects: [],
    unarchive_project: null,
    // Encryption
    get_encryption_status: { initialized: true, mode: "device", version: 1 },
    check_password_strength: { score: 5, strength: "good", label: "Good", suggestions: [] },
    // Daemon
    get_daemon_status: { running: false, pid: null, jobs_processed: null, uptime_seconds: null, last_error: null },
    start_daemon: { running: true, pid: 12345, jobs_processed: 0, uptime_seconds: 0, last_error: null },
    stop_daemon: null,
    daemon_health_check: true,
    // Audit
    get_audit_log: { entries: [], total: 0, has_more: false },
    export_audit_log: "/tmp/audit-export.json",
    prune_old_audit_logs: 0,
    get_audit_log_stats: { total_entries: 0, entries_by_action: {}, entries_by_entity: {} },
    // Migration (defaults to no migration needed)
    get_migration_status: { needs_migration: false, database_exists: true, is_encrypted: true, backup_exists: false, backup_path: null, database_size_mb: 1.5 },
    migrate_database: { success: true, backup_path: "/tmp/backup.db", safe_backup_path: "~/Documents/Meridian Backups/meridian-backup.db", tables_migrated: 10, error: null },
    get_safe_backup_dir_path: "~/Documents/Meridian Backups",
    list_safe_backups_cmd: [],
    restore_safe_backup: null,
    list_backups: [],
    cleanup_old_backups: 0,
    restore_from_backup: null,
    // Scheduler
    get_scheduler_status: { enabled: false, platform: "macos", service_name: "com.meridian.daemon", error: null },
    enable_system_scheduler: null,
    disable_system_scheduler: null,
    // AI & Embeddings
    get_ai_settings: {
      id: "ai-1",
      label: "openai-main",
      provider: "openai",
      base_url: null,
      model_id: "gpt-4o-mini",
      ollama_base_url: "http://localhost:11434",
      ollama_model: "nomic-embed-text",
      embedding_provider: "bundled",
      is_active: true,
      created_at: "2025-01-01T00:00:00Z",
    },
    check_ollama_status: { running: false, models: [] },
    hybrid_search_documents: [],
    search_documents: [],
    // Documents
    get_documents_for_project: [],
    upload_document: { id: "doc-1", project_id: "proj-1", filename: "test.pdf", file_path: "/path/test.pdf", file_type: "pdf", embeddings_ready: false, uploaded_at: new Date().toISOString(), created_at: new Date().toISOString() },
    delete_document: null,
    get_document_embedding_status: { document_id: "doc-1", embeddings_ready: false, embedding_model: null, job_status: null, job_error: null },
    retry_document_embedding: null,
    get_embedding_migration_status: { documents_needing_embedding: 0, jobs_queued: 0 },
    queue_embedding_migration: { documents_needing_embedding: 0, jobs_queued: 0 },
    start_embedding_worker: null,
    stop_embedding_worker: null,
    get_indexing_status: { worker_running: false, jobs_processed: 0, pending_jobs: 0, running_jobs: 0 },
    process_pending_embeddings: { worker_running: false, jobs_processed: 0, pending_jobs: 0, running_jobs: 0 },
    // Pattern Learning
    get_pattern_summaries: [],
    get_pattern_model: { id: "pm-1", pattern_type: "workflow_sequence", project_id: null, model_data: "{}", confidence: 0.5, observation_count: 10, last_updated: new Date().toISOString() },
    get_workflow_suggestions: [],
    dismiss_workflow_suggestion: null,
    get_smart_defaults: { suggested_priority: null, priority_confidence: 0, suggested_assignee: null, assignee_confidence: 0, source: "none" },
    get_communication_style: null,
    record_draft_edit: null,
    export_learning_data: { version: "1.0", exported_at: new Date().toISOString(), pattern_models: [] },
    import_learning_data: 0,
    reset_pattern_category: true,
    reset_all_learning: 0,
    // Suggestions
    get_pending_suggestions: [],
    accept_suggestion: null,
    dismiss_suggestion: null,
    stop_suggesting: null,
    create_suggestion: { id: "sug-1", type: "overdue_task", title: "Test suggestion", description: null, reasoning: null, action_config: null, severity: "info", status: "pending", project_id: "proj-1", created_at: new Date().toISOString(), acted_at: null },
    get_suggestion_count_today: 0,
    // Drafts
    get_drafts_for_task: [],
    generate_draft: { id: "draft-1", task_id: "task-1", channel: "email", recipient: null, subject: "Test", body: "Test draft", ai_signature: true, status: "draft", sensitive_warnings: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString(), sent_at: null },
    update_draft: { id: "draft-1", task_id: "task-1", channel: "email", recipient: null, subject: "Test", body: "Updated draft", ai_signature: true, status: "draft", sensitive_warnings: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString(), sent_at: null },
    delete_draft: null,
    scan_draft: [],
    // Plans
    evaluate_task_plan: { complexity: "medium", reasoning: "This task has multiple steps", suggested_subtasks: ["Step 1", "Step 2"], suggested_action: null },
    get_task_plan: null,
    accept_plan: [],
    record_plan_correction: null,
    // Skills
    list_skills: [],
    get_skill: { id: "skill-1", name: "Test Skill", description: "A test skill", trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: true, shared: false, owner_id: null, category: "custom", icon: null, tags: null, next_run_at: null, cloned_from_id: null, is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    create_skill: { id: "skill-new", name: "New Skill", description: null, trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: false, shared: false, owner_id: null, category: null, icon: null, tags: null, next_run_at: null, cloned_from_id: null, is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    update_skill: { id: "skill-1", name: "Updated Skill", description: null, trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: true, shared: false, owner_id: null, category: null, icon: null, tags: null, next_run_at: null, cloned_from_id: null, is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    delete_skill: null,
    toggle_skill_enabled: { id: "skill-1", name: "Test Skill", description: null, trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: false, shared: false, owner_id: null, category: null, icon: null, tags: null, next_run_at: null, cloned_from_id: null, is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    run_skill_manually: { id: "run-1", skill_id: "skill-1", status: "completed", trigger_type: "manual", trigger_context: null, output: "Skill completed", error: null, pending_changes: null, started_at: new Date().toISOString(), completed_at: new Date().toISOString(), duration_ms: 150, approval_decision: null, approval_reason: null, created_at: new Date().toISOString() },
    test_run_skill: { skill_id: "skill-1", skill_name: "Test Skill", context: { tasks: [], meetings: [], documents: [], project: null, truncated: false }, context_tasks_count: 0, context_meetings_count: 0, context_truncated: false, action_type: "summarize", approval_mode: "notify" },
    get_skill_runs: [],
    get_skill_run: { id: "run-1", skill_id: "skill-1", status: "completed", trigger_type: "manual", trigger_context: null, output: "Skill completed", error: null, pending_changes: null, started_at: new Date().toISOString(), completed_at: new Date().toISOString(), duration_ms: 150, approval_decision: null, approval_reason: null, created_at: new Date().toISOString() },
    approve_skill_run: { type: "tasks_created", count: 0, task_ids: [] },
    reject_skill_run: null,
    clone_skill: { id: "skill-cloned", name: "Test Skill (Copy)", description: null, trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: false, shared: false, owner_id: null, category: null, icon: null, tags: null, next_run_at: null, cloned_from_id: "skill-1", is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    export_skill: { name: "Test Skill", trigger_type: "manual", approval_mode: "notify", version: "1.0" },
    import_skill: { id: "skill-imported", name: "Imported Skill", description: null, trigger_type: "manual", trigger_config: null, context_config: null, action_config: null, approval_mode: "notify", enabled: false, shared: false, owner_id: null, category: null, icon: null, tags: null, next_run_at: null, cloned_from_id: null, is_builtin: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    get_skill_stats: { total_runs: 10, successful_runs: 8, failed_runs: 2, avg_duration_ms: 200, last_run_at: new Date().toISOString(), success_rate: 0.8 },
    record_skill_output_edit: null,
    extract_skill_from_chat: { name: "weekly-digest", description: "Send a weekly digest every Monday", trigger_type: "schedule", trigger_config: { cron: "0 9 * * 1" }, action_type: "summarize", system_prompt: "Summarize the week's progress", approval_mode: "notify" },
    initialize_builtin_skills: [],
    reset_builtin_skills: ["skill-b1", "skill-b2", "skill-b3"],
    // Skill folders
    pick_folder_dialog: "/tmp/test-skill-folder",
    export_skill_to_directory: "/tmp/exported/test-skill",
    list_skill_folders: [],
    get_skill_folder: { name: "test-folder", path: "/tmp/skills/test-folder", description: null, files: [], has_executables: false, created_at: new Date().toISOString(), enabled: true },
    install_skill_folder: { name: "test-folder", path: "/tmp/skills/test-folder", description: null, files: [], has_executables: false, created_at: new Date().toISOString(), enabled: true },
    delete_skill_folder: null,
    read_skill_file: "file content",
    execute_skill_script: "script output",
    toggle_folder_skill_enabled: true,
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
