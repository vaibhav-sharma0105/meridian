import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface Project {
  id: string;
  name: string;
  description: string | null;
  color: string;
  created_at: string;
  updated_at: string;
  archived_at: string | null;
  open_task_count?: number;
}

export interface CreateProjectInput {
  name: string;
  description?: string;
  color?: string;
}

export interface UpdateProjectInput {
  id: string;
  name?: string;
  description?: string;
  color?: string;
}

export interface Meeting {
  id: string;
  project_id: string;
  title: string;
  platform: string;
  raw_transcript: string | null;
  ai_summary: string | null;
  summary: string | null; // alias for ai_summary
  decisions: string | null;
  health_score: number | null;
  health_breakdown: string | null;
  attendees: string | null;
  duration_minutes: number | null;
  meeting_at: string | null;
  ingested_at: string;
  created_at: string; // alias for ingested_at
  updated_at: string;
  archived_at: string | null;
}

export interface Task {
  id: string;
  project_id: string;
  meeting_id: string | null;
  parent_task_id: string | null;
  title: string;
  description: string | null;
  assignee: string | null;
  assignee_confidence: "committed" | "inferred" | "unassigned";
  assignee_source_quote: string | null;
  due_date: string | null;
  due_confidence: "committed" | "inferred" | "none";
  due_source_quote: string | null;
  status: "open" | "in_progress" | "done" | "cancelled";
  priority: "low" | "medium" | "high" | "critical";
  confidence_score: number | null;
  tags: string; // JSON array string
  kanban_column: string;
  kanban_order: number;
  is_duplicate: boolean;
  duplicate_of_id: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  archived_at: string | null;
}

export interface CreateTaskInput {
  project_id: string;
  meeting_id?: string;
  parent_task_id?: string;
  title: string;
  description?: string;
  assignee?: string;
  assignee_confidence?: string;
  due_date?: string;
  due_confidence?: string;
  priority?: string;
  tags?: string[];
  notes?: string;
  kanban_column?: string;
}

export interface UpdateTaskInput {
  id: string;
  title?: string;
  description?: string;
  assignee?: string;
  assignee_confidence?: string;
  due_date?: string;
  due_confidence?: string;
  status?: string;
  priority?: string;
  tags?: string[];
  kanban_column?: string;
  kanban_order?: number;
  notes?: string;
  meeting_id?: string | null;
}

export interface TaskFilters {
  assignee?: string;
  status?: string;
  priority?: string;
  project_id?: string;    // client-side only — not sent to backend
  meeting_ids?: string[]; // client-side only — multi-select meeting filter
  tags?: string[];
  search_query?: string;
  date_from?: string;
  date_to?: string;
  show_archived?: boolean;
}

export interface Document {
  id: string;
  project_id: string;
  title: string | null;
  filename: string;
  file_path: string;
  file_type: string;
  source_url: string | null;
  content_text: string | null;
  chunks: string | null;
  embeddings_ready: boolean;
  embedding_model: string | null;
  file_size_bytes: number | null;
  uploaded_at: string;
  created_at: string; // alias for uploaded_at
}

export interface AiSettings {
  id: string;
  label: string;
  provider: string;
  base_url: string | null;
  model_id: string | null;
  ollama_base_url: string;
  ollama_model: string;
  embedding_provider: string;
  is_active: boolean;
  created_at: string;
}

export interface AiSettingsInput {
  id?: string;
  label: string;
  provider: string;
  base_url?: string;
  api_key?: string;
  model_id?: string;
  ollama_base_url?: string;
  ollama_model?: string;
  embedding_provider?: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  context_window?: number;
}

export interface PromptTemplate {
  id: string;
  name: string;
  description: string | null;
  system_prompt: string;
  user_prompt_template: string;
  output_format: string;
  is_default: boolean;
  is_builtin: boolean;
  created_at: string;
  updated_at: string;
}

export interface AppNotification {
  id: string;
  type: string;
  title: string;
  body: string;
  task_id: string | null;
  project_id: string | null;
  skill_run_id: string | null;
  integration_id: string | null;
  severity: "info" | "warning" | "critical";
  desktop: boolean;
  is_read: boolean;
  created_at: string;
  /** Message Center entry holding the full result; drives "View full result". */
  message_id: string | null;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  project_id?: string;
  meeting_id?: string;
  template_id?: string;
  created_at?: string;
}

export interface SearchResult {
  document_id: string;
  document_title: string;
  filename: string;
  chunk_text: string;
  content: string; // alias for chunk_text
  score: number;
  search_type: string;
}

export interface IngestMeetingResult {
  meeting: Meeting;
  tasks: Task[];
}

export interface VerifyConnectionResult {
  success: boolean;
  error: string | null;
  latency_ms: number;
}

export interface ExportResult {
  file_path: string;
  size_bytes: number;
}

export interface ImportResult {
  projects_imported: number;
  meetings_imported: number;
  tasks_imported: number;
}

export interface UpdateCheckResult {
  update_available: boolean;
  version?: string;
  release_notes?: string;
}

export interface OllamaStatus {
  running: boolean;
  models: string[];
}

// ─── Message Center Types ─────────────────────────────────────────────────────

export interface Message {
  id: string;
  project_id: string | null;
  /** Mirrors the Rust `MessageType` enum — extend both together. */
  message_type: "skill_result" | "digest" | "pinned_chat" | "integration_sync";
  title: string;
  content: string | null;
  source_id: string | null;
  source_type: string | null;
  auto_pinned: boolean;
  pinned_reason: string | null;
  file_refs: string[] | null;
  ai_visible_until: string | null;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateMessageInput {
  project_id?: string;
  message_type: string;
  title: string;
  content?: string;
  source_id?: string;
  source_type?: string;
  auto_pinned?: boolean;
  pinned_reason?: string;
  file_refs?: string[];
}

export interface MessageFilters {
  project_id?: string;
  message_type?: string;
  search?: string;
  include_deleted?: boolean;
}

export interface PaginatedMessages {
  messages: Message[];
  total: number;
  page: number;
  per_page: number;
}

export interface StorageStats {
  total_messages: number;
  total_files: number;
  storage_bytes: number;
  oldest_message: string | null;
  newest_message: string | null;
}

export interface CleanupStats {
  soft_deleted: number;
  hard_deleted: number;
  files_removed: number;
}

// ─── Role Types ───────────────────────────────────────────────────────────────

export interface RoleScores {
  tech_lead: number;
  ic: number;
  pm: number;
  manager: number;
}

export interface RoleClassification {
  primary: string;
  secondary: string | null;
  confidence: number;
}

export type InferenceStatus =
  | { type: "Learning"; message: string; progress: number }
  | { type: "PendingConfirmation"; inferred: string; confidence: number }
  | { type: "Confirmed"; role: string; secondary: string | null };

export interface UserProfile {
  id: string;
  inferred_role: string | null;
  secondary_role: string | null;
  custom_role_description: string | null;
  role_confirmed: boolean;
  role_confirmed_at: string | null;
  role_scores: RoleScores | null;
  last_inference_at: string | null;
  productivity_patterns: ProductivityPatterns | null;
  productivity_tracking_enabled: boolean;
  ai_context_days: number;
  message_retention: string;
  archive_old_files: boolean;
  archive_after_days: number;
  display_name: string | null;
  user_email: string | null;
  user_aliases: string[];
  created_at: string;
  updated_at: string;
}

// ─── Productivity Types ───────────────────────────────────────────────────────

export interface ProductivityPatterns {
  task_completions_by_hour: Record<string, number[]>;
  peak_hours: Record<string, number[]>;
  low_productivity_hours: number[];
  total_completions: number;
  last_aggregation: string | null;
  tracking_enabled: boolean;
}

export type ProductivityStatus =
  | { type: "Ready" }
  | { type: "Learning"; completions_needed: number }
  | { type: "Disabled" };

export interface ProductivityInsights {
  patterns: ProductivityPatterns;
  status: ProductivityStatus;
  storage_warning: string | null;
}

/** Mirrors the Rust `TimeSuggestion` struct exactly — `confidence` is an enum, not a number. */
export interface TimeSuggestion {
  suggested_hour: number;
  reason: string;
  confidence: TimeSuggestionConfidence;
}

export type TimeSuggestionConfidence = "High" | "Default" | "Low";

export interface BatchingSuggestion {
  message: string;
  /** Contiguous hours the meetings could collapse into, when one is found. */
  suggested_block: number[] | null;
  freed_hours: number;
}

export interface ProductivityExport {
  peak_hours: Record<string, number[]>;
  total_data_points: number;
  tracking_since: string;
}

/** Mirrors the Rust `ProductivitySettings` struct taken by `update_productivity_settings`. */
export interface ProductivitySettings {
  tracking_enabled: boolean;
  show_suggestions: boolean;
  data_retention_days: number;
}

// ─── Projects ────────────────────────────────────────────────────────────────

export const getProjects = () => invoke<Project[]>("get_projects");
export const createProject = (input: CreateProjectInput) =>
  invoke<Project>("create_project", { input });
export const updateProject = (input: UpdateProjectInput) =>
  invoke<Project>("update_project", { input });
export const archiveProject = (id: string) =>
  invoke<void>("archive_project", { id });
export const getArchivedProjects = () =>
  invoke<Project[]>("get_archived_projects");
export const unarchiveProject = (id: string) =>
  invoke<void>("unarchive_project", { id });

// ─── Meetings ────────────────────────────────────────────────────────────────

export const ingestMeeting = (args: {
  projectId: string;
  title: string;
  platform: string;
  rawTranscript: string;
  attendees?: string;
  durationMinutes?: number;
  meetingAt?: string;
}) =>
  invoke<IngestMeetingResult>("ingest_meeting", {
    projectId: args.projectId,
    title: args.title,
    platform: args.platform,
    attendees: args.attendees,
    durationMinutes: args.durationMinutes,
    rawTranscript: args.rawTranscript,
    meetingAt: args.meetingAt,
  });
export const ingestMeetingFromFile = (args: {
  projectId: string;
  filePath: string;
  title?: string;
  platform?: string;
}) =>
  invoke<IngestMeetingResult>("ingest_meeting_from_file", {
    projectId: args.projectId,
    filePath: args.filePath,
    title: args.title,
    platform: args.platform,
  });
export const getMeetingsForProject = (projectId: string, showArchived = false) =>
  invoke<Meeting[]>("get_meetings_for_project", { projectId, showArchived });
export const getMeeting = (id: string) =>
  invoke<Meeting | null>("get_meeting", { id });
export const deleteMeeting = (id: string) =>
  invoke<void>("delete_meeting", { id });
export const forceDeleteMeeting = (id: string) =>
  invoke<void>("force_delete_meeting", { id });
export const unarchiveMeeting = (id: string) =>
  invoke<void>("unarchive_meeting", { id });
export const renameMeeting = (id: string, title: string) =>
  invoke<void>("rename_meeting", { id, title });

export interface MoveMeetingResult {
  old_project_id: string;
  new_project_id: string;
  tasks_moved: number;
}

/** Count open/in-progress tasks that would follow the meeting on a move. */
export const countMoveableTasks = (meetingId: string) =>
  invoke<number>("count_moveable_tasks", { meetingId });

/** Move a meeting and its eligible tasks to a new project. */
export const moveMeetingToProject = (meetingId: string, newProjectId: string) =>
  invoke<MoveMeetingResult>("move_meeting_to_project", { meetingId, newProjectId });

// ─── Tasks ───────────────────────────────────────────────────────────────────

export const getTasksForProject = (
  projectId: string,
  filters?: TaskFilters
) =>
  invoke<Task[]>("get_tasks_for_project", {
    projectId,
    filters: filters || {},
  });

export const getAllTasks = (filters?: TaskFilters) =>
  invoke<Task[]>("get_all_tasks", { filters: filters || {} });
export const createTask = (input: CreateTaskInput) =>
  invoke<Task>("create_task", { input });
export const updateTask = (input: UpdateTaskInput) =>
  invoke<Task>("update_task", { input });
export const bulkUpdateTasks = (
  taskIds: string[],
  updates: Partial<Task>
) => invoke<void>("bulk_update_tasks", { taskIds, updates });
export const reorderTask = (
  taskId: string,
  newColumn: string,
  newOrder: number
) => invoke<void>("reorder_tasks", { taskId, newColumn, newOrder });
export const deleteTask = (id: string) =>
  invoke<void>("delete_task", { id });
export const archiveTask = (id: string) =>
  invoke<void>("archive_task", { id });
export const unarchiveTask = (id: string) =>
  invoke<void>("unarchive_task", { id });
export const moveTaskToProject = (taskId: string, newProjectId: string) =>
  invoke<void>("move_task_to_project", { taskId, newProjectId });

// ─── Documents ───────────────────────────────────────────────────────────────

export const uploadDocument = (projectId: string, filePath: string) =>
  invoke<Document>("upload_document", { projectId, filePath });
export const uploadUrl = (projectId: string, url: string) =>
  invoke<Document>("upload_document", { projectId, url });
export const getDocumentsForProject = (projectId: string) =>
  invoke<Document[]>("get_documents_for_project", { projectId });
export const deleteDocument = (id: string) =>
  invoke<void>("delete_document", { id });
export const searchDocuments = (args: {
  projectId: string;
  query: string;
  limit?: number;
  useSemantic?: boolean;
}) =>
  invoke<SearchResult[]>("search_documents", {
    projectId: args.projectId,
    query: args.query,
    useSemantic: args.useSemantic ?? true,
  });

export const hybridSearchDocuments = (args: {
  projectId: string;
  query: string;
  limit?: number;
}) =>
  invoke<SearchResult[]>("hybrid_search_documents", {
    projectId: args.projectId,
    query: args.query,
    limit: args.limit ?? 10,
  });

export interface DocumentEmbeddingStatus {
  document_id: string;
  embeddings_ready: boolean;
  embedding_model: string | null;
  job_status: string | null;
  job_error: string | null;
}

export const getDocumentEmbeddingStatus = (documentId: string) =>
  invoke<DocumentEmbeddingStatus>("get_document_embedding_status", { documentId });

export const retryDocumentEmbedding = (documentId: string) =>
  invoke<void>("retry_document_embedding", { documentId });

export interface EmbeddingMigrationStatus {
  documents_needing_embedding: number;
  jobs_queued: number;
}

export const getEmbeddingMigrationStatus = () =>
  invoke<EmbeddingMigrationStatus>("get_embedding_migration_status");

export const queueEmbeddingMigration = () =>
  invoke<EmbeddingMigrationStatus>("queue_embedding_migration");

export interface IndexingStatus {
  worker_running: boolean;
  jobs_processed: number;
  pending_jobs: number;
  running_jobs: number;
}

export const startEmbeddingWorker = () =>
  invoke<void>("start_embedding_worker");

export const stopEmbeddingWorker = () =>
  invoke<void>("stop_embedding_worker");

export const getIndexingStatus = () =>
  invoke<IndexingStatus>("get_indexing_status");

export const processPendingEmbeddings = () =>
  invoke<IndexingStatus>("process_pending_embeddings");

// Convenience wrapper for uploading documents by various methods
export const ingestDocument = (args: {
  projectId: string;
  filePath?: string;
  url?: string;
  content?: string;
  title?: string;
}) => {
  if (args.filePath) return invoke<Document>("upload_document", { projectId: args.projectId, filePath: args.filePath });
  if (args.url) return invoke<Document>("upload_document", { projectId: args.projectId, url: args.url });
  return invoke<Document>("upload_text", { projectId: args.projectId, content: args.content, title: args.title });
};

// Document recovery
export interface OrphanedDocument {
  folder_id: string;
  filename: string;
  file_path: string;
  file_size_bytes: number;
}

export const findOrphanedDocuments = () =>
  invoke<OrphanedDocument[]>("find_orphaned_documents");

export const recoverOrphanedDocument = (projectId: string, filePath: string) =>
  invoke<Document>("recover_orphaned_document", { projectId, filePath });

// ─── AI ──────────────────────────────────────────────────────────────────────

export const verifyAiConnection = (args: {
  provider: string;
  baseUrl?: string;
  apiKey: string;
  modelId?: string;
}) => invoke<VerifyConnectionResult>("verify_ai_connection", args);

export const fetchAvailableModels = (args: {
  provider: string;
  baseUrl?: string;
  apiKeyLabel: string;
  apiKey?: string;
}) => invoke<ModelInfo[]>("fetch_available_models", args);

export const saveAiSettings = (settings: AiSettingsInput) =>
  invoke<AiSettings>("save_ai_settings", { settings });
export const getAiSettings = () =>
  invoke<AiSettings | null>("get_ai_settings");

export const extractTasksFromTranscript = (args: {
  meetingId: string;
  transcript: string;
  projectId: string;
}) => invoke<Task[]>("extract_tasks_from_transcript", args);

export const chatWithProject = (args: {
  projectId: string;
  meetingId?: string;
  message: string;
  templateId?: string;
  conversationHistory?: Array<{ role: string; content: string }>;
  skillContext?: string;
}) => invoke<ChatMessage>("chat_with_project", args);

export const checkOllamaStatus = () =>
  invoke<OllamaStatus>("check_ollama_status");

export const embedDocumentChunks = (documentId: string) =>
  invoke<{ chunks_embedded: number }>("embed_document_chunks", {
    documentId,
  });

// ─── Settings ────────────────────────────────────────────────────────────────

export const getAppSettings = () =>
  invoke<Record<string, string>>("get_app_settings");
export const setAppSetting = (key: string, value: string) =>
  invoke<void>("set_app_setting", { key, value });
export const getPromptTemplates = () =>
  invoke<PromptTemplate[]>("get_prompt_templates");
export const savePromptTemplate = (template: PromptTemplate) =>
  invoke<PromptTemplate>("save_prompt_template", { template });

// ─── Export / Import ─────────────────────────────────────────────────────────

export const exportProject = (
  projectId: string,
  format: string,
  includeDocs: boolean
) =>
  invoke<ExportResult>("export_project", { projectId, format, includeDocs });
export const exportAll = () => invoke<ExportResult>("export_all");
export const importProject = (filePath: string) =>
  invoke<ImportResult>("import_project", { filePath });
export const exportData = (args: { format: string; projectId?: string }) =>
  args.projectId
    ? exportProject(args.projectId, args.format, true)
    : exportAll();
export const importData = (args: { filePath: string }) =>
  importProject(args.filePath);

// ─── Output Templates ────────────────────────────────────────────────────────

export const generateOutput = (args: { projectId: string; templateId: string }) =>
  invoke<string>("generate_output", { projectId: args.projectId, templateId: args.templateId });

// ─── Notifications ───────────────────────────────────────────────────────────

export const getNotifications = () =>
  invoke<AppNotification[]>("get_notifications");
export const markNotificationRead = (id: string) =>
  invoke<void>("mark_notification_read", { id });
export const markAllRead = () => invoke<void>("mark_all_read");
export const createNotification = (args: {
  notificationType: string;
  title: string;
  body: string;
  taskId?: string;
  projectId?: string;
}) => invoke<AppNotification>("create_notification", args);

// ─── Updater ─────────────────────────────────────────────────────────────────

export const checkForUpdates = () =>
  invoke<UpdateCheckResult>("check_for_updates");
export const backupDatabase = () => invoke<string>("backup_database");

// ─── Event Listeners ─────────────────────────────────────────────────────────

export const onChatChunk = (
  callback: (data: { content: string; done: boolean }) => void
) => {
  return listen<{ content: string; done: boolean }>(
    "chat_chunk",
    (event) => callback(event.payload)
  );
};

export const onEmbedProgress = (
  callback: (data: { document_id: string; progress: number; total: number }) => void
) => {
  return listen<{ document_id: string; progress: number; total: number }>(
    "embed_progress",
    (event) => callback(event.payload)
  );
};

export interface SyncProgress {
  step: string;
  current: number;
  total: number;
}

export const onExportProgress = (callback: (data: SyncProgress) => void) => {
  return listen<SyncProgress>("export_progress", (event) => callback(event.payload));
};

export const onImportProgress = (callback: (data: SyncProgress) => void) => {
  return listen<SyncProgress>("import_progress", (event) => callback(event.payload));
};

// ─── Connections ──────────────────────────────────────────────────────────────

export interface Connection {
  id: string;
  provider: "zoom" | "gmail" | "sheets_relay";
  account_email: string | null;
  scopes: string | null;
  token_expires_at: string | null;
  last_sync_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface PendingImport {
  id: string;
  provider: "zoom" | "gmail" | "sheets_relay" | "manual";
  external_meeting_id: string | null;
  title: string;
  meeting_date: string | null;
  duration_minutes: number | null;
  attendees: string | null;
  summary_preview: string | null;
  summary_full: string | null;
  transcript_available: boolean;
  transcript_content: string | null;
  zoom_join_url: string | null;
  source_email_id: string | null;
  status: "pending" | "imported" | "dismissed";
  imported_meeting_id: string | null;
  project_id: string | null;
  created_at: string;
}

export interface ImportApproval {
  pending_import_id: string;
  project_id: string;
  import_type: "summary" | "transcript";
}

export interface SyncResult {
  new_imports: number;
  skipped_duplicates: number;
  errors: string[];
}

export const connectZoom = () => invoke<Connection>("connect_zoom");
export const connectGmail = () => invoke<Connection>("connect_gmail");
export const getConnection = (provider: string) =>
  invoke<Connection | null>("get_connection", { provider });
export const disconnectProvider = (provider: string) =>
  invoke<void>("disconnect_provider", { provider });
export const syncConnections = () => invoke<SyncResult>("sync_connections");
export const getPendingImports = () =>
  invoke<PendingImport[]>("get_pending_imports");
export const countPendingImports = () =>
  invoke<number>("count_pending_imports");
export const approveImport = (input: ImportApproval) =>
  invoke<IngestMeetingResult>("approve_import", { input });
export const dismissImport = (pendingImportId: string) =>
  invoke<void>("dismiss_import", { pendingImportId });

export const onSyncComplete = (callback: (data: SyncResult) => void) =>
  listen<SyncResult>("sync_complete", (event) => callback(event.payload));

export const openUrl = (url: string) => invoke<void>("open_url", { url });

// ─── Sheets Relay ─────────────────────────────────────────────────────────────

export const saveSheetRelayConfig = (scriptUrl: string, secretKey: string) =>
  invoke<Connection>("save_sheets_relay_config", { scriptUrl, secretKey });

export const testSheetsRelay = () =>
  invoke<string>("test_sheets_relay");

export const resetSheetsRelaySync = () =>
  invoke<void>("reset_sheets_relay_sync");

// ─── Encryption ───────────────────────────────────────────────────────────────

export interface EncryptionStatus {
  initialized: boolean;
  mode: "password" | "device" | null;
  version: number | null;
}

export interface PasswordStrength {
  score: number;
  strength: "weak" | "fair" | "good" | "strong";
  label: string;
  suggestions: string[];
}

export const getEncryptionStatus = () =>
  invoke<EncryptionStatus>("get_encryption_status");

export const checkPasswordStrength = (password: string) =>
  invoke<PasswordStrength>("check_password_strength", { password });

// ─── Daemon ───────────────────────────────────────────────────────────────────

export interface DaemonStatus {
  running: boolean;
  pid: number | null;
  jobs_processed: number | null;
  uptime_seconds: number | null;
  last_error: string | null;
}

export const getDaemonStatus = () =>
  invoke<DaemonStatus>("get_daemon_status");

export const startDaemon = () =>
  invoke<DaemonStatus>("start_daemon");

export const stopDaemon = () =>
  invoke<void>("stop_daemon");

export const daemonHealthCheck = () =>
  invoke<boolean>("daemon_health_check");

export interface BackgroundJob {
  id: string;
  job_type: string;
  status: string;
  priority: number;
  scheduled_at: string;
  started_at: string | null;
  created_at: string;
  description: string;
}

export const getBackgroundJobs = (limit?: number) =>
  invoke<BackgroundJob[]>("get_background_jobs", { limit });

export const getRecentBackgroundJobs = (limit?: number) =>
  invoke<BackgroundJob[]>("get_recent_background_jobs", { limit });

// ─── Migration ────────────────────────────────────────────────────────────────

export interface MigrationStatus {
  needs_migration: boolean;
  database_exists: boolean;
  is_encrypted: boolean;
  backup_exists: boolean;
  backup_path: string | null;
  database_size_mb: number;
}

export interface MigrationResult {
  success: boolean;
  backup_path: string;
  safe_backup_path: string;
  tables_migrated: number;
  error: string | null;
}

export interface BackupInfo {
  path: string;
  size_mb: number;
  created_at: string;
  age_days: number;
}

export const getMigrationStatus = () =>
  invoke<MigrationStatus>("get_migration_status");

export const migrateDatabase = (password?: string) =>
  invoke<MigrationResult>("migrate_database", { password });

export const listBackups = () =>
  invoke<BackupInfo[]>("list_backups");

export const cleanupOldBackups = (maxAgeDays: number) =>
  invoke<number>("cleanup_old_backups", { maxAgeDays });

export const restoreFromBackup = (backupPath: string) =>
  invoke<void>("restore_from_backup", { backupPath });

export const getSafeBackupDirPath = () =>
  invoke<string>("get_safe_backup_dir_path");

export const listSafeBackups = () =>
  invoke<BackupInfo[]>("list_safe_backups_cmd");

export const restoreSafeBackup = (backupPath: string) =>
  invoke<void>("restore_safe_backup", { backupPath });

// ─── System Scheduler ─────────────────────────────────────────────────────────

export interface SchedulerStatus {
  enabled: boolean;
  platform: string;
  service_name: string;
  error: string | null;
}

export const getSchedulerStatus = () =>
  invoke<SchedulerStatus>("get_scheduler_status");

export const enableSystemScheduler = () =>
  invoke<void>("enable_system_scheduler");

export const disableSystemScheduler = () =>
  invoke<void>("disable_system_scheduler");

// ─── Pattern Learning ─────────────────────────────────────────────────────────

export interface PatternObservation {
  id: string;
  observation_type: string;
  entity_type: string | null;
  entity_id: string | null;
  project_id: string | null;
  context_data: string;
  created_at: string;
  processed_at: string | null;
}

export interface PatternModel {
  id: string;
  pattern_type: string;
  project_id: string | null;
  model_data: string;
  confidence: number;
  observation_count: number;
  last_updated: string;
  scope: string;
  contributor_count: number;
}

export interface PatternSummary {
  pattern_type: string;
  confidence: number;
  observation_count: number;
  last_updated: string;
  contributor_count?: number;
}

export interface WorkflowSequence {
  trigger_action: string;
  follow_action: string;
  occurrence_count: number;
  avg_delay_minutes: number;
}

export interface WorkflowSequenceModelData {
  sequences: WorkflowSequence[];
  negative_sequences: string[];
}

export interface PriorityPattern {
  keyword: string;
  priority: string;
  occurrence_count: number;
}

export interface AssigneePattern {
  keyword: string;
  assignee: string;
  occurrence_count: number;
}

export interface ProjectDefault {
  default_priority: string | null;
  default_assignee: string | null;
}

export interface SmartDefaultsModelData {
  priority_patterns: PriorityPattern[];
  assignee_patterns: AssigneePattern[];
  project_defaults: Record<string, ProjectDefault>;
}

export interface CommunicationStyleModelData {
  length_preference: "concise" | "verbose" | "neutral";
  formality_level: "formal" | "casual" | "neutral";
  common_additions: [string, number][];
  common_removals: [string, number][];
  signature_patterns: string[];
}

export interface WorkflowSuggestion {
  trigger_task_id: string;
  suggested_action: string;
  confidence: number;
  sequence_id: string;
}

export interface SmartDefaults {
  suggested_priority: string | null;
  priority_confidence: number;
  suggested_assignee: string | null;
  assignee_confidence: number;
  source: string;
}

export interface LearningExport {
  version: string;
  exported_at: string;
  pattern_models: PatternModel[];
}

export interface LearningImport {
  version: string;
  pattern_models: PatternModel[];
}

export const getPatternSummaries = (projectId?: string) =>
  invoke<PatternSummary[]>("get_pattern_summaries", { projectId });

export const getPatternModel = (patternType: string, projectId?: string) =>
  invoke<PatternModel>("get_pattern_model", { patternType, projectId });

export const getWorkflowSuggestions = (completedTaskId: string, projectId: string) =>
  invoke<WorkflowSuggestion[]>("get_workflow_suggestions", { completedTaskId, projectId });

export const dismissWorkflowSuggestion = (sequenceId: string, projectId: string) =>
  invoke<void>("dismiss_workflow_suggestion", { sequenceId, projectId });

export const getSmartDefaults = (taskTitle: string, projectId: string) =>
  invoke<SmartDefaults>("get_smart_defaults", { taskTitle, projectId });

export const getCommunicationStyle = (context?: string, projectId?: string) =>
  invoke<CommunicationStyleModelData | null>("get_communication_style", { context, projectId });

export const recordDraftEdit = (
  originalText: string,
  editedText: string,
  contextType?: string,
  projectId?: string
) =>
  invoke<void>("record_draft_edit", { originalText, editedText, contextType, projectId });

export const exportLearningData = () =>
  invoke<LearningExport>("export_learning_data");

export const importLearningData = (data: LearningImport) =>
  invoke<number>("import_learning_data", { data });

export const resetPatternCategory = (patternType: string, projectId?: string) =>
  invoke<boolean>("reset_pattern_category", { patternType, projectId });

export const resetAllLearning = () =>
  invoke<number>("reset_all_learning");

// ─── Shared Patterns ────────────────────────────────────────────────────────

export const getPatternContributionEnabled = () =>
  invoke<boolean>("get_pattern_contribution_enabled");

export const setPatternContributionEnabled = (enabled: boolean) =>
  invoke<void>("set_pattern_contribution_enabled", { enabled });

export const getUseTeamPatternsEnabled = () =>
  invoke<boolean>("get_use_team_patterns_enabled");

export const setUseTeamPatternsEnabled = (enabled: boolean) =>
  invoke<void>("set_use_team_patterns_enabled", { enabled });

export const getTeamPatternSummaries = () =>
  invoke<PatternSummary[]>("get_team_pattern_summaries");

// ─── Suggestions ──────────────────────────────────────────────────────────────

export interface Suggestion {
  id: string;
  type: string;
  title: string;
  description: string | null;
  reasoning: string | null;
  action_config: string | null;
  severity: "info" | "warning" | "critical";
  status: "pending" | "accepted" | "dismissed" | "expired";
  project_id: string | null;
  created_at: string;
  acted_at: string | null;
}

export interface CreateSuggestionInput {
  suggestion_type: string;
  title: string;
  description?: string;
  reasoning?: string;
  action_config?: string;
  severity?: string;
  project_id?: string;
}

export const getPendingSuggestions = (projectId?: string) =>
  invoke<Suggestion[]>("get_pending_suggestions", { projectId });

export const acceptSuggestion = (id: string) =>
  invoke<void>("accept_suggestion", { id });

export const dismissSuggestion = (id: string) =>
  invoke<void>("dismiss_suggestion", { id });

export const stopSuggesting = (id: string, suggestionType: string) =>
  invoke<void>("stop_suggesting", { id, suggestionType });

export const createSuggestion = (input: CreateSuggestionInput) =>
  invoke<Suggestion>("create_suggestion", { input });

export const getSuggestionCountToday = () =>
  invoke<number>("get_suggestion_count_today");

// ─── Drafts ───────────────────────────────────────────────────────────────────

export interface DraftMessage {
  id: string;
  task_id: string | null;
  channel: string;
  recipient: string | null;
  subject: string | null;
  body: string;
  ai_signature: boolean;
  status: "draft" | "sent" | "archived";
  sensitive_warnings: string | null;
  created_at: string;
  updated_at: string;
  sent_at: string | null;
}

export interface CreateDraftInput {
  task_id?: string;
  channel: string;
  recipient?: string;
  subject?: string;
  body: string;
  ai_signature?: boolean;
}

export interface UpdateDraftInput {
  recipient?: string;
  subject?: string;
  body?: string;
  ai_signature?: boolean;
  sensitive_warnings?: string;
  status?: string;
}

export interface SensitiveWarning {
  warning_type: "pii" | "credentials" | "financial";
  severity: "info" | "warning" | "critical";
  message: string;
  pattern_name: string;
  start_pos: number;
  end_pos: number;
}

export const getDraftsForTask = (taskId: string) =>
  invoke<DraftMessage[]>("get_drafts_for_task", { taskId });

export const generateDraft = (taskId: string, channel: string) =>
  invoke<DraftMessage>("generate_draft", { taskId, channel });

export const updateDraft = (id: string, input: UpdateDraftInput) =>
  invoke<DraftMessage>("update_draft", { id, input });

export const deleteDraft = (id: string) =>
  invoke<void>("delete_draft", { id });

export const scanDraft = (content: string, draftId?: string) =>
  invoke<SensitiveWarning[]>("scan_draft", { content, draftId });

// ─── Task Plans ───────────────────────────────────────────────────────────────

export interface TaskPlan {
  complexity: "simple" | "medium" | "complex";
  reasoning: string;
  suggested_subtasks: string[];
  suggested_action?: string;
}

export const evaluateTaskPlan = (taskId: string) =>
  invoke<TaskPlan>("evaluate_task_plan", { taskId });

export const getTaskPlan = (taskId: string) =>
  invoke<TaskPlan | null>("get_task_plan", { taskId });

export const acceptPlan = (taskId: string, subtaskTitles: string[]) =>
  invoke<Task[]>("accept_plan", { taskId, subtaskTitles });

export const recordPlanCorrection = (
  taskId: string,
  originalSubtasks: string[],
  editedSubtasks: string[],
  action: string
) =>
  invoke<void>("record_plan_correction", { taskId, originalSubtasks, editedSubtasks, action });

// ─── Skills ──────────────────────────────────────────────────────────────────

export type TriggerType = "schedule" | "event" | "manual";
export type ApprovalMode = "auto" | "notify" | "approve_first" | "approve_always";
export type SkillRunStatus = "pending" | "running" | "completed" | "failed" | "partial_failure" | "cancelled" | "approval_pending";

export interface TriggerConfig {
  cron?: string;
  timezone?: string;
  event_type?: string;
  filter?: Record<string, unknown>;
}

export interface ContextConfig {
  scope?: "global" | "project";
  project_id?: string;
  include_documents?: boolean;
  document_filter?: string;
  max_documents?: number;
  max_tokens?: number;
  include_archived?: boolean;
  priority_order?: string[];
  system_prompt?: string;
  output_instructions?: string;
}

export interface ActionConfig {
  action_type?: "summarize" | "draft_message" | "create_tasks" | "analyze" | "custom";
  format?: "markdown" | "plain" | "html";
  channel?: string;
  template?: string;
  max_length?: number;
  has_side_effects?: boolean;
}

export interface Skill {
  id: string;
  name: string;
  description: string | null;
  trigger_type: TriggerType;
  trigger_config: string | null;
  context_config: string | null;
  action_config: string | null;
  approval_mode: string;
  autonomy_mode: string | null;
  enabled: boolean;
  shared: boolean;
  owner_id: string | null;
  category: string | null;
  icon: string | null;
  tags: string | null;
  next_run_at: string | null;
  cloned_from_id: string | null;
  is_builtin: boolean;
  // Phase 9: Sync fields
  sync_source: string | null;
  sync_path: string | null;
  sync_commit: string | null;
  last_sync_check: string | null;
  content_hash: string | null;
  // Phase 9: Trust fields
  trust_state: string | null;
  trust_granted_at: string | null;
  network_mode: string | null;
  network_allowlist: string | null;
  // Timestamps
  created_at: string;
  updated_at: string;
}

export interface CreateSkillInput {
  name: string;
  description?: string;
  trigger_type: TriggerType;
  trigger_config?: TriggerConfig;
  context_config?: ContextConfig;
  action_config?: ActionConfig;
  approval_mode?: ApprovalMode;
  category?: string;
  icon?: string;
  tags?: string[];
  shared?: boolean;
}

export interface UpdateSkillInput {
  id: string;
  name?: string;
  description?: string;
  trigger_type?: TriggerType;
  trigger_config?: TriggerConfig;
  context_config?: ContextConfig;
  action_config?: ActionConfig;
  approval_mode?: ApprovalMode;
  enabled?: boolean;
  shared?: boolean;
  category?: string;
  icon?: string;
  tags?: string[];
  autonomy_mode?: string | null;
}

export interface SkillRun {
  id: string;
  skill_id: string;
  status: SkillRunStatus;
  trigger_type: string;
  trigger_context: string | null;
  output: string | null;
  error: string | null;
  pending_changes: string | null;
  started_at: string | null;
  completed_at: string | null;
  duration_ms: number | null;
  approval_decision: string | null;
  approval_reason: string | null;
  created_at: string;
}

export interface SkillStats {
  total_runs: number;
  successful_runs: number;
  failed_runs: number;
  avg_duration_ms: number | null;
  last_run_at: string | null;
  success_rate: number;
}

export interface SkillTestResult {
  skill_id: string;
  skill_name: string;
  context: {
    tasks: unknown[];
    meetings: unknown[];
    documents: unknown[];
    project: unknown | null;
    truncated: boolean;
  };
  context_tasks_count: number;
  context_meetings_count: number;
  context_truncated: boolean;
  action_type: string | null;
  approval_mode: string;
}

// Skill CRUD
export const createSkill = (input: CreateSkillInput) =>
  invoke<Skill>("create_skill", { input });

export const getSkill = (id: string) =>
  invoke<Skill>("get_skill", { id });

export const listSkills = (args?: {
  shared?: boolean;
  category?: string;
  enabled?: boolean;
}) => invoke<Skill[]>("list_skills", args ?? {});

export const updateSkill = (input: UpdateSkillInput) =>
  invoke<Skill>("update_skill", { input });

export const deleteSkill = (id: string) =>
  invoke<void>("delete_skill", { id });

export const toggleSkillEnabled = (id: string, enabled: boolean) =>
  invoke<Skill>("toggle_skill_enabled", { id, enabled });

// Skill execution
export const runSkillManually = (skillId: string) =>
  invoke<SkillRun>("run_skill_manually", { skillId });

export const testRunSkill = (skillId: string) =>
  invoke<SkillTestResult>("test_run_skill", { skillId });

// Skill runs
export const getSkillRuns = (args: {
  skillId: string;
  status?: string;
  limit?: number;
  offset?: number;
}) => invoke<SkillRun[]>("get_skill_runs", args);

export const getSkillRun = (id: string) =>
  invoke<SkillRun>("get_skill_run", { id });

// Approval
export const approveSkillRun = (runId: string, projectId?: string) =>
  invoke<unknown>("approve_skill_run", { runId, projectId });

export const rejectSkillRun = (runId: string, reason?: string) =>
  invoke<void>("reject_skill_run", { runId, reason });

// Skill utilities
export const cloneSkill = (skillId: string, newName?: string) =>
  invoke<Skill>("clone_skill", { skillId, newName });

export const exportSkill = (skillId: string) =>
  invoke<unknown>("export_skill", { skillId });

export const exportSkillToDirectory = (skillMdContent: string, skillName: string) =>
  invoke<string>("export_skill_to_directory", { skillMdContent, skillName });

export const importSkill = (skillJson: unknown) =>
  invoke<Skill>("import_skill", { skillJson });

export const getSkillStats = (skillId: string) =>
  invoke<SkillStats>("get_skill_stats", { skillId });

export const recordSkillOutputEdit = (skillId: string, runId: string, originalOutput: string, editedOutput: string) =>
  invoke<void>("record_skill_output_edit", { skillId, runId, originalOutput, editedOutput });

export interface ExtractedSkillDefinition {
  name: string;
  description: string;
  trigger_type: "schedule" | "event" | "manual";
  trigger_config: Record<string, unknown>;
  action_type: string;
  system_prompt?: string;
  approval_mode?: string;
}

export const extractSkillFromChat = (description: string) =>
  invoke<ExtractedSkillDefinition>("extract_skill_from_chat", { description });

export const initializeBuiltinSkills = () =>
  invoke<string[]>("initialize_builtin_skills");

export const resetBuiltinSkills = () =>
  invoke<string[]>("reset_builtin_skills");

// ─── Skill Folders ────────────────────────────────────────────────────────────

export interface SkillFileEntry {
  name: string;
  path: string;
  is_directory: boolean;
  is_executable: boolean;
  size: number;
  children: SkillFileEntry[] | null;
}

export interface SkillFolder {
  name: string;
  path: string;
  description: string | null;
  files: SkillFileEntry[];
  has_executables: boolean;
  created_at: string;
  enabled: boolean;
}

export const pickFolderDialog = () =>
  invoke<string | null>("pick_folder_dialog");

export const listSkillFolders = () =>
  invoke<SkillFolder[]>("list_skill_folders");

export const getSkillFolder = (folderName: string) =>
  invoke<SkillFolder>("get_skill_folder", { folderName });

export const installSkillFolder = (sourcePath: string) =>
  invoke<SkillFolder>("install_skill_folder", { sourcePath });

export const deleteSkillFolder = (folderName: string) =>
  invoke<void>("delete_skill_folder", { folderName });

export const readSkillFile = (folderName: string, filePath: string) =>
  invoke<string>("read_skill_file", { folderName, filePath });

export const toggleFolderSkillEnabled = (folderName: string, enabled: boolean) =>
  invoke<boolean>("toggle_folder_skill_enabled", { folderName, enabled });

export const executeSkillScript = (folderName: string, scriptPath: string) =>
  invoke<string>("execute_skill_script", { folderName, scriptPath });

// ─── Skills: GitHub Sync & Trust (Phase 9) ────────────────────────────────────

export interface ImportableSkill {
  name: string;
  path: string;
  description: string | null;
}

export interface SyncResult {
  skill_id: string;
  status: string;
  new_content: string | null;
  diff_preview: string | null;
}

export interface SkillTrustState {
  skill_id: string;
  trust_state: string;
  trust_granted_at: string | null;
  network_mode: string;
  network_allowlist: string[];
}

export interface SandboxedExecutionResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code: number | null;
  run_id: string;
}

export const listImportableSkills = (integrationId: string, owner: string, repo: string) =>
  invoke<ImportableSkill[]>("list_importable_skills", { integrationId, owner, repo });

export const importSkillFromRepo = (
  integrationId: string,
  skillPath: string,
  localName?: string
) => invoke<Skill>("import_skill_from_repo", { integrationId, skillPath, localName });

export const checkSkillUpdates = (skillId: string) =>
  invoke<{ status: string; new_commit?: string }>("check_skill_updates", { skillId });

export const syncSkill = (
  skillId: string,
  strategy: "keep_local" | "use_remote" | "manual"
) => invoke<SyncResult>("sync_skill", { skillId, strategy });

export const grantSkillTrust = (
  skillId: string,
  networkMode: "none" | "allowlist" | "full",
  allowlist?: string[]
) => invoke<void>("grant_skill_trust", { skillId, networkMode, allowlist });

export const revokeSkillTrust = (skillId: string) =>
  invoke<void>("revoke_skill_trust", { skillId });

export const getSkillTrustState = (skillId: string) =>
  invoke<SkillTrustState>("get_skill_trust_state", { skillId });

export const executeSkillSandboxed = (skillId: string, scriptPath: string, inputs?: Record<string, unknown>) =>
  invoke<SandboxedExecutionResult>("execute_skill_sandboxed", { skillId, scriptPath, inputs });

// ─── Integrations ─────────────────────────────────────────────────────────────

export interface IntegrationConfig {
  access_token?: string;
  refresh_token?: string;
  expires_at?: string;
  client_id?: string;
  client_secret?: string;
  api_token?: string;
  base_url?: string;
  scopes?: string[];
  repositories?: string[];
  projects?: string[];
  channels?: ChannelConfig[];
  bot_token?: string;
  user_token?: string;
  app_token?: string;
  socket_mode_enabled?: boolean;
}

export interface ChannelConfig {
  id: string;
  name: string;
  autonomy_mode: string;
  is_external: boolean;
}

export interface IntegrationPermissions {
  read: boolean;
  write: boolean;
  delete: boolean;
  admin: boolean;
}

export interface Integration {
  id: string;
  type: string;
  name: string;
  config: IntegrationConfig;
  permissions?: IntegrationPermissions;
  autonomy_mode: string;
  linking_workflow: "lazy" | "ai_suggested" | "manual";
  status: string;
  last_sync?: string;
  sync_interval_minutes: number;
  webhook_token?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface IntegrationCache {
  id: string;
  integration_id: string;
  external_type: string;
  external_id: string;
  external_url?: string;
  data: unknown;
  synced_at: string;
  attention_score?: number;
  attention_reason?: string;
  evaluated_at?: string;
  archived_at?: string;
  expires_at?: string;
}

export interface IntegrationLink {
  id: string;
  integration_id: string;
  local_type: string;
  local_id: string;
  external_type: string;
  external_id: string;
  external_url?: string;
  sync_enabled: boolean;
  created_at: string;
}

export interface SyncState {
  integration_id: string;
  status: string;
  last_sync?: string;
  items_synced: number;
  items_new: number;
  items_updated: number;
  errors: string[];
}

export interface CreateIntegrationInput {
  integration_type: string;
  name: string;
  config: IntegrationConfig;
  permissions?: IntegrationPermissions;
  autonomy_mode?: string;
  linking_workflow?: "lazy" | "ai_suggested" | "manual";
  sync_interval_minutes?: number;
}

export interface UpdateIntegrationInput {
  id: string;
  name?: string;
  config?: IntegrationConfig;
  permissions?: IntegrationPermissions;
  autonomy_mode?: string;
  linking_workflow?: "lazy" | "ai_suggested" | "manual";
  status?: string;
  sync_interval_minutes?: number;
  error_message?: string;
}

export interface CreateLinkInput {
  integration_id: string;
  local_type: string;
  local_id: string;
  external_type: string;
  external_id: string;
  external_url?: string;
  sync_enabled?: boolean;
}

export interface AvailableIntegration {
  type: string;
  name: string;
  description: string;
  icon: string;
  capabilities: string[];
}

export const listIntegrations = () =>
  invoke<Integration[]>("list_integrations");

export const getIntegration = (id: string) =>
  invoke<Integration | null>("get_integration", { id });

export const getAvailableIntegrations = () =>
  invoke<AvailableIntegration[]>("get_available_integrations");

export const createIntegration = (input: CreateIntegrationInput) =>
  invoke<Integration>("create_integration", { input });

export const updateIntegration = (input: UpdateIntegrationInput) =>
  invoke<Integration>("update_integration", { input });

export const deleteIntegration = (id: string) =>
  invoke<void>("delete_integration", { id });

export const startOAuthFlow = (
  integrationType: string,
  redirectUri: string,
  clientId?: string,
  clientSecret?: string,
) =>
  invoke<string>("start_oauth_flow", { integrationType, redirectUri, clientId, clientSecret });

export const handleOAuthCallback = (oauthState: string, code: string) =>
  invoke<Integration>("handle_oauth_callback", { oauthState, code });

export type OAuthCallbackPayload =
  | { success: true; code: string; state: string }
  | { success: false; error: string };

export const onOAuthCallbackReceived = (
  callback: (payload: OAuthCallbackPayload) => void
) => listen<OAuthCallbackPayload>("oauth_callback_received", (e) => callback(e.payload));

export const refreshIntegrationToken = (id: string) =>
  invoke<Integration>("refresh_integration_token", { id });

export const syncIntegration = (id: string) =>
  invoke<SyncState>("sync_integration", { id });

export const getSyncStatus = (id: string) =>
  invoke<SyncState | null>("get_sync_status", { id });

export const clearIntegrationCache = (id: string) =>
  invoke<void>("clear_integration_cache", { id });

export const getCachedItems = (integrationId: string, externalType?: string) =>
  invoke<IntegrationCache[]>("get_cached_items", { integrationId, externalType });

export const createIntegrationLink = (input: CreateLinkInput) =>
  invoke<IntegrationLink>("create_integration_link", { input });

export const getLinksForTask = (taskId: string) =>
  invoke<IntegrationLink[]>("get_links_for_task", { taskId });

export const getLinksForMeeting = (meetingId: string) =>
  invoke<IntegrationLink[]>("get_links_for_meeting", { meetingId });

export const unlinkIntegrationItem = (linkId: string) =>
  invoke<void>("unlink_integration_item", { linkId });

// Slack Socket Mode
export interface SocketModeStatus {
  connected: boolean;
  app_token_configured: boolean;
  last_event_at: string | null;
  reconnect_count: number;
}

export const getSlackSocketStatus = () =>
  invoke<SocketModeStatus>("get_slack_socket_status");

export const detectSlackActionItems = (text: string, botUserId?: string) =>
  invoke<string[]>("detect_slack_action_items", { text, botUserId });

export interface IntegrationWriteResult {
  success: boolean;
  queued_for_approval: boolean;
  approval_id: string | null;
  result: unknown | null;
}

export const agentIntegrationWrite = (
  integrationId: string,
  actionType: string,
  actionConfig: string
) =>
  invoke<IntegrationWriteResult>("agent_integration_write", {
    integrationId,
    actionType,
    actionConfig,
  });

// Notification enhancements
export const createNotificationWithOptions = (
  notificationType: string,
  title: string,
  body: string,
  options?: {
    taskId?: string;
    projectId?: string;
    skillRunId?: string;
    integrationId?: string;
    severity?: "info" | "warning" | "critical";
    desktop?: boolean;
  }
) =>
  invoke<AppNotification>("create_notification_with_options", {
    notificationType,
    title,
    body,
    taskId: options?.taskId,
    projectId: options?.projectId,
    skillRunId: options?.skillRunId,
    integrationId: options?.integrationId,
    severity: options?.severity ?? "info",
    desktop: options?.desktop ?? false,
  });

export const checkNotificationPermission = () =>
  invoke<boolean>("check_notification_permission");

export const requestNotificationPermission = () =>
  invoke<boolean>("request_notification_permission");

// MCP Permissions
export interface McpPermissions {
  read_tasks: boolean;
  read_meetings: boolean;
  read_projects: boolean;
  create_task: boolean;
  update_task: boolean;
  delete_task: boolean;
  create_meeting_note: boolean;
  run_skill: boolean;
  create_report: boolean;
  draft_message: boolean;
  rate_limit_per_minute: number;
}

export const getMcpPermissions = () =>
  invoke<McpPermissions>("get_mcp_permissions");

export const setMcpPermissions = (permissions: McpPermissions) =>
  invoke<void>("set_mcp_permissions", { permissions });

// ─── Governance ──────────────────────────────────────────────────────────────

export type RiskLevel = "low" | "medium" | "high" | "critical";
export type AutonomyMode = "manual" | "supervised" | "autonomous";
export type AutonomySource = "global" | "integration" | "skill";
export type ApprovalStatusType = "pending" | "approved" | "rejected" | "archived" | "executed";

export interface ApprovalDecision {
  requires_approval: boolean;
  risk_level: RiskLevel;
  autonomy_mode: AutonomyMode;
  autonomy_source: AutonomySource;
  reason: string;
}

export interface PendingApproval {
  id: string;
  action_type: string;
  action_config: string;
  source_type: string | null;
  source_id: string | null;
  risk_level: string;
  autonomy_mode: string;
  context: string | null;
  timeout_at: string | null;
  status: ApprovalStatusType;
  resolved_by: string | null;
  resolution_reason: string | null;
  created_at: string;
  resolved_at: string | null;
}

export interface ActionHistory {
  id: string;
  action_type: string;
  entity_type: string;
  entity_id: string;
  before_state: string | null;
  after_state: string | null;
  undoable: boolean;
  undo_action_id: string | null;
  audit_log_id: string | null;
  created_at: string;
}

export interface GovernanceMetrics {
  date: string;
  metric_type: string;
  breakdown_key: string | null;
  value: number;
}

export interface RiskAdjustment {
  id: string;
  adjustment_type: string;
  target_type: string;
  target_id: string;
  risk_delta: number;
  reason: string | null;
  created_at: string;
}

export interface UndoResult {
  success: boolean;
  undo_action_id: string | null;
  message: string;
  reversal_type: string | null;
}

export interface EvaluateActionInput {
  action_type: string;
  destination: string;
  content?: string;
  integration_id?: string;
  skill_id?: string;
}

export interface CreateApprovalInput {
  action_type: string;
  action_config: string;
  source_type?: string;
  source_id?: string;
  risk_level: RiskLevel;
  autonomy_mode: AutonomyMode;
  context?: string;
  timeout_minutes?: number;
}

export const evaluateAction = (input: EvaluateActionInput) =>
  invoke<ApprovalDecision>("evaluate_action", { input });

export const getAutonomySetting = (key: string) =>
  invoke<string | null>("get_autonomy_setting", { key });

export const setAutonomySetting = (key: string, value?: string) =>
  invoke<void>("set_autonomy_setting", { key, value });

export const getPendingApprovals = (status?: string, limit?: number) =>
  invoke<PendingApproval[]>("get_pending_approvals", { status, limit });

export const getPendingApproval = (id: string) =>
  invoke<PendingApproval | null>("get_pending_approval", { id });

export const approvePendingAction = (id: string) =>
  invoke<PendingApproval>("approve_pending_action", { id });

export const rejectPendingAction = (id: string, reason?: string) =>
  invoke<PendingApproval>("reject_pending_action", { id, reason });

export const bulkApproveActions = (ids: string[]) =>
  invoke<string[]>("bulk_approve_actions", { ids });

export const bulkRejectActions = (ids: string[], reason?: string) =>
  invoke<string[]>("bulk_reject_actions", { ids, reason });

export const getPendingApprovalCount = () =>
  invoke<number>("get_pending_approval_count");

export const createPendingApproval = (input: CreateApprovalInput) =>
  invoke<string>("create_pending_approval", { input });

export const getActionHistory = (entityType?: string, entityId?: string, limit?: number) =>
  invoke<ActionHistory[]>("get_action_history", { entityType, entityId, limit });

export const getUndoableActions = (limit?: number) =>
  invoke<ActionHistory[]>("get_undoable_actions", { limit });

export const undoAction = (actionId: string) =>
  invoke<UndoResult>("undo_action", { actionId });

export const captureActionState = (
  actionType: string,
  entityType: string,
  entityId: string,
  beforeState?: string,
  afterState?: string,
  auditLogId?: string
) =>
  invoke<string>("capture_action_state", {
    actionType,
    entityType,
    entityId,
    beforeState,
    afterState,
    auditLogId,
  });

export const getGovernanceMetrics = (startDate: string, endDate: string, metricType?: string) =>
  invoke<GovernanceMetrics[]>("get_governance_metrics", { startDate, endDate, metricType });

export const createRiskAdjustment = (
  adjustmentType: string,
  targetType: string,
  targetId: string,
  riskDelta: number,
  reason?: string
) =>
  invoke<string>("create_risk_adjustment", {
    adjustmentType,
    targetType,
    targetId,
    riskDelta,
    reason,
  });

export const getRiskAdjustment = (targetType: string, targetId: string) =>
  invoke<RiskAdjustment | null>("get_risk_adjustment", { targetType, targetId });

export const deleteRiskAdjustment = (targetType: string, targetId: string) =>
  invoke<void>("delete_risk_adjustment", { targetType, targetId });

export const calculateRiskLevel = (actionStr: string, destinationStr: string, content?: string) =>
  invoke<RiskLevel>("calculate_risk_level", { actionStr, destinationStr, content });

// ─── Team ─────────────────────────────────────────────────────────────────────

export interface TeamMember {
  id: string;
  name: string;
  email?: string;
  avatar_url?: string;
  source: string;
  source_id?: string;
  role: string;
  expertise?: string[];
  workload_score?: number;
  metadata?: Record<string, unknown>;
  last_synced_at?: string;
  created_at: string;
}

export interface CreateTeamMemberInput {
  name: string;
  email?: string;
  avatar_url?: string;
  source: string;
  source_id?: string;
  role?: string;
  expertise?: string[];
  metadata?: Record<string, unknown>;
}

export interface UpdateTeamMemberInput {
  id: string;
  name?: string;
  email?: string;
  avatar_url?: string;
  role?: string;
  expertise?: string[];
  metadata?: Record<string, unknown>;
}

export interface TeamSyncResult {
  added: number;
  updated: number;
  total: number;
}

export interface AssigneeFactors {
  pattern_score: number;
  workload_score: number;
  expertise_score: number;
  recency_score: number;
}

export interface AssigneeSuggestion {
  member: TeamMember;
  score: number;
  confidence: string;
  reason: string;
  factors: AssigneeFactors;
}

export const getTeamMembers = () => invoke<TeamMember[]>("get_team_members");

export const getTeamMember = (id: string) =>
  invoke<TeamMember | null>("get_team_member", { id });

export const createTeamMember = (input: CreateTeamMemberInput) =>
  invoke<TeamMember>("create_team_member", { input });

export const updateTeamMember = (input: UpdateTeamMemberInput) =>
  invoke<TeamMember>("update_team_member", { input });

export const deleteTeamMember = (id: string) =>
  invoke<void>("delete_team_member", { id });

export const computeTeamWorkloads = () =>
  invoke<[string, number][]>("compute_team_workloads");

export const syncTeamFromSlack = () =>
  invoke<TeamSyncResult>("sync_team_from_slack");

export const syncTeamFromGoogle = () =>
  invoke<TeamSyncResult>("sync_team_from_google");

export const getAssigneeSuggestions = (
  taskTitle: string,
  taskDescription?: string,
  projectId?: string
) =>
  invoke<AssigneeSuggestion[]>("get_assignee_suggestions", {
    taskTitle,
    taskDescription,
    projectId,
  });

export const recordAssigneeSelection = (
  selectedName: string,
  suggestions: AssigneeSuggestion[],
  wasOverride: boolean
) =>
  invoke<void>("record_assignee_selection", {
    selectedName,
    suggestions,
    wasOverride,
  });

// ─── Sync (Export/Import) ─────────────────────────────────────────────────────

export interface ExportOptions {
  include_projects: boolean;
  include_tasks: boolean;
  include_meetings: boolean;
  include_skills: boolean;
  include_patterns: boolean;
  include_team: boolean;
  include_documents: boolean;
  include_vectors: boolean;
  project_ids?: string[];
  password?: string;
  description?: string;
}

export interface ExportContents {
  projects: boolean;
  tasks: boolean;
  meetings: boolean;
  skills: boolean;
  patterns: boolean;
  documents: boolean;
  team_members: boolean;
  settings: boolean;
  vectors: boolean;
  project_count: number;
  task_count: number;
  meeting_count: number;
  skill_count: number;
  pattern_count: number;
  document_count: number;
  team_member_count: number;
}

export interface ExportManifest {
  format_version: string;
  app_version: string;
  created_at: string;
  created_by?: string;
  description?: string;
  contents: ExportContents;
}

export interface ExportResult {
  file_path: string;
  file_size: number;
  manifest: ExportManifest;
}

export interface ImportOptions {
  mode: "Merge" | "Replace";
  password?: string;
  conflict_resolution: "Skip" | "Overwrite" | "Ask";
  create_backup: boolean;
}

export interface ImportConflict {
  entity_type: string;
  entity_id: string;
  local_name: string;
  import_name: string;
  local_updated?: string;
  import_updated?: string;
}

export interface ImportPreview {
  manifest: ExportManifest;
  conflicts: ImportConflict[];
  new_items: Record<string, number>;
}

export interface ImportResult {
  success: boolean;
  imported_count: number;
  skipped_count: number;
  conflict_count: number;
  errors: string[];
  conflicts: ImportConflict[];
  backup_path?: string;
}

export const exportAllData = (outputPath: string, options: ExportOptions) =>
  invoke<ExportResult>("export_all_data", { outputPath, options });

export const previewImportData = (archivePath: string, options: ImportOptions) =>
  invoke<ImportPreview>("preview_import_data", { archivePath, options });

export const importAllData = (
  archivePath: string,
  options: ImportOptions,
  conflictResolutions: Record<string, string>
) =>
  invoke<ImportResult>("import_all_data", { archivePath, options, conflictResolutions });

export const pickExportSavePath = (defaultName: string) =>
  invoke<string | null>("pick_export_save_path", { defaultName });

export const pickImportFilePath = () =>
  invoke<string | null>("pick_import_file_path");

// ─── Attention Items ─────────────────────────────────────────────────────────

export interface AttentionItem {
  id: string;
  source_type: string;
  source_id: string;
  severity: "critical" | "warning" | "info";
  category: string;
  reason_text: string | null;
  matched_skill_id: string | null;
  computed_at: string;
  dismissed_at: string | null;
}

export interface AttentionFilters {
  severity?: string;
  source_type?: string;
  category?: string;
  include_dismissed?: boolean;
}

export const getAttentionItems = (filters?: AttentionFilters) =>
  invoke<AttentionItem[]>("get_attention_items", { filters });

export const getAttentionCount = () =>
  invoke<[number, number]>("get_attention_count", {});

export const dismissAttentionItem = (id: string) =>
  invoke<void>("dismiss_attention_item", { id });

// ─── Integration Browser ─────────────────────────────────────────────────────

export const getIntegrationCacheForProject = (
  projectId: string,
  integrationType?: string,
  itemType?: string,
  limit?: number
) =>
  invoke<IntegrationCache[]>("get_integration_cache_for_project", {
    project_id: projectId,
    integration_type: integrationType,
    item_type: itemType,
    limit,
  });

export const searchCachedIntegrationItems = (
  query: string,
  projectId?: string,
  limit?: number
) =>
  invoke<IntegrationCache[]>("search_cached_integration_items", {
    query,
    project_id: projectId,
    limit,
  });

// ─── Message Center ───────────────────────────────────────────────────────────

export const createMessage = (input: CreateMessageInput) =>
  invoke<Message>("create_message", { input });

export const getMessage = (id: string) =>
  invoke<Message>("get_message", { id });

export const getMessages = (
  filters?: MessageFilters,
  page?: number,
  perPage?: number
) =>
  invoke<PaginatedMessages>("get_messages", {
    filters,
    page: page ?? 1,
    per_page: perPage ?? 20,
  });

export const getMessagesForAiContext = (
  projectId?: string,
  aiContextDays?: number
) =>
  invoke<Message[]>("get_messages_for_ai_context", {
    project_id: projectId,
    ai_context_days: aiContextDays ?? 30,
  });

export const softDeleteMessage = (id: string) =>
  invoke<void>("delete_message", { id });

export const restoreMessage = (id: string) =>
  invoke<void>("restore_message", { id });

export const getDeletedMessages = (limit?: number) =>
  invoke<Message[]>("get_deleted_messages", { limit });

export const pinFromSource = (
  sourceType: string,
  sourceId: string,
  title: string,
  content?: string,
  projectId?: string
) =>
  invoke<Message>("pin_message", {
    source_type: sourceType,
    source_id: sourceId,
    title,
    content,
    project_id: projectId,
  });

export const getStorageStats = () =>
  invoke<StorageStats>("get_storage_stats", {});

export const runMessageCleanup = () =>
  invoke<CleanupStats>("cleanup_messages", {});

// ─── User Role ────────────────────────────────────────────────────────────────

export const getUserProfile = () =>
  invoke<UserProfile>("get_user_profile", {});

export const getInferenceStatus = () =>
  invoke<InferenceStatus>("get_role_inference_status", {});

export const confirmRole = (role: string, customDescription?: string) =>
  invoke<UserProfile>("confirm_role", {
    role,
    custom_description: customDescription,
  });

export const changeRole = (role: string, customDescription?: string) =>
  invoke<UserProfile>("change_role", {
    role,
    custom_description: customDescription,
  });

export const dismissDriftAlert = () =>
  invoke<void>("dismiss_role_drift_alert", {});

export const runRoleInference = () =>
  invoke<void>("run_role_inference", {});

/**
 * Retention + AI-context-window settings live on `user_profile`, not in the
 * ProductivitySettings struct. Separate command, separate wrapper.
 */
export const updateRetentionSettings = (opts: {
  aiContextDays?: number;
  messageRetention?: string;
  productivityTrackingEnabled?: boolean;
  archiveOldFiles?: boolean;
  archiveAfterDays?: number;
}) =>
  invoke<UserProfile>("update_retention_settings", {
    ai_context_days: opts.aiContextDays,
    message_retention: opts.messageRetention,
    productivity_tracking_enabled: opts.productivityTrackingEnabled,
    archive_old_files: opts.archiveOldFiles,
    archive_after_days: opts.archiveAfterDays,
  });

/**
 * Identifies who "me" is. Role-based My Activity ordering needs this to tell
 * the user's own items apart from their team's; without it, ordering falls
 * back to severity + recency. Omitted fields are left unchanged.
 */
export const updateUserIdentity = (opts: {
  displayName?: string;
  userEmail?: string;
  userAliases?: string[];
}) =>
  invoke<UserProfile>("update_user_identity", {
    display_name: opts.displayName,
    user_email: opts.userEmail,
    user_aliases: opts.userAliases,
  });

// ─── Productivity ─────────────────────────────────────────────────────────────

export const getProductivityInsights = () =>
  invoke<ProductivityInsights>("get_productivity_insights", {});

/** Suggestion for a task category ("focus_work" | "meetings" | "quick_tasks"). */
export const getTimeSuggestion = (category: string) =>
  invoke<TimeSuggestion | null>("get_time_suggestion_for_category", { category });

/** Suggestion for a specific task — backend derives the category from the task row. */
export const getTimeSuggestionForTask = (taskId: string) =>
  invoke<TimeSuggestion | null>("get_time_suggestion", { task_id: taskId });

/**
 * Batching suggestion for a fragmented meeting day. `date` defaults to today
 * (local) — the backend reads meeting hours from `meetings.meeting_at` itself.
 */
export const getMeetingBatchingSuggestion = (date?: string) =>
  invoke<BatchingSuggestion | null>("get_meeting_batching_suggestion", { date });

export const updateProductivitySettings = (settings: ProductivitySettings) =>
  invoke<void>("update_productivity_settings", { settings });

export const getProductivitySettings = () =>
  invoke<ProductivitySettings>("get_productivity_settings", {});

export const clearProductivityData = () =>
  invoke<void>("clear_productivity_data", {});

export const exportProductivityData = () =>
  invoke<ProductivityExport>("export_productivity_data", {});

export const aggregateProductivityPatterns = () =>
  invoke<void>("aggregate_productivity_patterns", {});

export interface RoleDriftAlert {
  previous_role: string;
  suggested_role: string;
  confidence: number;
}

/**
 * Polled, not subscribed: drift is computed by the daemon worker, which has no
 * AppHandle and therefore cannot emit Tauri events.
 */
export const getRoleDriftAlert = () =>
  invoke<RoleDriftAlert | null>("get_role_drift_alert", {});
