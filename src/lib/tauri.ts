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
}

export interface Task {
  id: string;
  project_id: string;
  meeting_id: string | null;
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
}

export interface CreateTaskInput {
  project_id: string;
  meeting_id?: string;
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
}

export interface TaskFilters {
  assignee?: string;
  status?: string;
  tags?: string[];
  search_query?: string;
  date_from?: string;
  date_to?: string;
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
export const getMeetingsForProject = (projectId: string) =>
  invoke<Meeting[]>("get_meetings_for_project", { projectId });
export const getMeeting = (id: string) =>
  invoke<Meeting | null>("get_meeting", { id });
export const deleteMeeting = (id: string) =>
  invoke<void>("delete_meeting", { id });

// ─── Tasks ───────────────────────────────────────────────────────────────────

export const getTasksForProject = (
  projectId: string,
  filters?: TaskFilters
) =>
  invoke<Task[]>("get_tasks_for_project", {
    projectId,
    filters: filters || {},
  });
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
