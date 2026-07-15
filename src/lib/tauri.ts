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
  is_read: boolean;
  created_at: string;
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
}

export interface PatternSummary {
  pattern_type: string;
  confidence: number;
  observation_count: number;
  last_updated: string;
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
  enabled: boolean;
  shared: boolean;
  owner_id: string | null;
  category: string | null;
  icon: string | null;
  tags: string | null;
  next_run_at: string | null;
  cloned_from_id: string | null;
  is_builtin: boolean;
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
