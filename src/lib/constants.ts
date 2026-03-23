export const PLATFORMS = [
  { value: "manual", label: "Manual" },
  { value: "zoom", label: "Zoom" },
  { value: "google_meet", label: "Google Meet" },
  { value: "teams", label: "Microsoft Teams" },
  { value: "slack", label: "Slack" },
  { value: "webex", label: "Webex" },
] as const;

export const PROJECT_COLORS = [
  "#6366f1", // Indigo
  "#8b5cf6", // Violet
  "#ec4899", // Pink
  "#ef4444", // Red
  "#f97316", // Orange
  "#eab308", // Yellow
  "#22c55e", // Green
  "#06b6d4", // Cyan
] as const;

export const TAG_COLORS: Record<string, string> = {
  blocker: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
  decision: "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400",
  deliverable: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400",
  "follow-up": "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400",
  dependency: "bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400",
  research: "bg-teal-100 text-teal-800 dark:bg-teal-900/30 dark:text-teal-400",
  review: "bg-slate-100 text-slate-800 dark:bg-slate-900/30 dark:text-slate-400",
  approval: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400",
};

export const HEALTH_COLORS = {
  red: { bg: "bg-red-100", text: "text-red-700", border: "border-red-300" },
  amber: { bg: "bg-amber-100", text: "text-amber-700", border: "border-amber-300" },
  green: { bg: "bg-green-100", text: "text-green-700", border: "border-green-300" },
};

export const STATUS_STYLES: Record<string, string> = {
  open: "bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300",
  in_progress: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
  done: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
  cancelled: "bg-zinc-100 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-500 line-through",
};

export const TEMPLATE_IDS = {
  "2x2": "tpl_2x2",
  jira: "tpl_jira",
  agenda: "tpl_agenda",
  status: "tpl_status",
  freeform: "tpl_freeform",
} as const;

export const AI_PROVIDERS = [
  { value: "openai", label: "OpenAI", defaultUrl: "https://api.openai.com/v1" },
  { value: "anthropic", label: "Anthropic", defaultUrl: "https://api.anthropic.com/v1" },
  { value: "gemini", label: "Google Gemini", defaultUrl: "https://generativelanguage.googleapis.com/v1beta/openai" },
  { value: "groq", label: "Groq", defaultUrl: "https://api.groq.com/openai/v1" },
  { value: "litellm", label: "LiteLLM (self-hosted)", defaultUrl: "http://localhost:4000" },
  { value: "ollama", label: "Ollama (local only)", defaultUrl: "http://localhost:11434" },
  { value: "custom", label: "Custom", defaultUrl: "" },
] as const;

export const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "hi", label: "हिंदी (Hindi)" },
  { value: "gu", label: "ગુજરાતી (Gujarati)" },
] as const;

export const KANBAN_COLUMNS = [
  { id: "open", label: "Open" },
  { id: "in_progress", label: "In Progress" },
  { id: "done", label: "Done" },
] as const;

export const MAX_CHAT_CHARS = 4000;
export const CHAT_WARN_AT = 2000;
export const AUTO_SAVE_DEBOUNCE_MS = 300;
export const SAVED_INDICATOR_MS = 1500;
