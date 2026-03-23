import { z } from "zod";

export const ProjectSchema = z.object({
  id: z.string(),
  name: z.string().min(1, "Project name is required"),
  description: z.string().nullable(),
  color: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
  archived_at: z.string().nullable(),
  open_task_count: z.number().optional(),
});

export const CreateProjectSchema = z.object({
  name: z.string().min(1, "Project name is required").max(100),
  description: z.string().max(500).optional(),
  color: z.string().optional(),
});

export const TaskSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  meeting_id: z.string().nullable(),
  title: z.string().min(1),
  description: z.string().nullable(),
  assignee: z.string().nullable(),
  assignee_confidence: z.enum(["committed", "inferred", "unassigned"]),
  due_date: z.string().nullable(),
  due_confidence: z.enum(["committed", "inferred", "none"]),
  status: z.enum(["open", "in_progress", "done", "cancelled"]),
  tags: z.string(),
  kanban_column: z.string(),
  kanban_order: z.number(),
  is_duplicate: z.boolean(),
  notes: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
  completed_at: z.string().nullable(),
});

export const AiSettingsInputSchema = z.object({
  label: z.string().min(1),
  provider: z.string().min(1),
  base_url: z.string().url().optional().or(z.literal("")),
  api_key: z.string().optional(),
  model_id: z.string().optional(),
  ollama_base_url: z.string().optional(),
  ollama_model: z.string().optional(),
});

export const MeetingIngestSchema = z.object({
  title: z.string().min(1, "Title is required"),
  platform: z.string(),
  raw_transcript: z.string().min(50, "Transcript must be at least 50 characters"),
  meeting_at: z.string().optional(),
});

export const parseTags = (tagsJson: string): string[] => {
  try {
    const parsed = JSON.parse(tagsJson);
    if (Array.isArray(parsed)) return parsed;
    return [];
  } catch {
    return [];
  }
};
