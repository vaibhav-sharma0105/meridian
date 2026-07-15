import YAML from "yaml";
import type { Skill, CreateSkillInput } from "./tauri";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface SkillFrontmatter {
  name: string;
  description?: string;
  trigger: {
    type: "schedule" | "event" | "manual";
    cron?: string;
    timezone?: string;
    event_type?: string;
    filter?: Record<string, unknown>;
  };
  context?: {
    scope?: "project" | "global";
    variables?: string[];
    max_tokens?: number;
    include_documents?: boolean;
    document_filter?: string;
    max_documents?: number;
  };
  action: {
    type: "summarize" | "draft_message" | "create_tasks" | "analyze" | "custom";
    format?: string;
    channel?: string;
    template?: string;
    max_length?: number;
  };
  settings?: {
    approval_mode?: "auto" | "notify" | "approve_first" | "approve_always";
    category?: string;
    tags?: string[];
    enabled?: boolean;
    shared?: boolean;
  };
}

export interface SkillMarkdownBody {
  role: string;
  context: string;
  instructions: string;
  output_format: string;
  examples: string;
}

export interface ParsedSkillFile {
  frontmatter: SkillFrontmatter;
  body: SkillMarkdownBody;
  raw: string;
}

// ─── Constants ────────────────────────────────────────────────────────────────

const BODY_SECTIONS = ["role", "context", "instructions", "output_format", "examples"] as const;

const SECTION_HEADINGS: Record<string, keyof SkillMarkdownBody> = {
  "role": "role",
  "context": "context",
  "instructions": "instructions",
  "output format": "output_format",
  "output_format": "output_format",
  "examples": "examples",
};

export const EMPTY_BODY: SkillMarkdownBody = {
  role: "",
  context: "",
  instructions: "",
  output_format: "",
  examples: "",
};

export const AVAILABLE_VARIABLES = [
  { name: "{{tasks}}", description: "All tasks in context" },
  { name: "{{meetings}}", description: "Recent meetings" },
  { name: "{{project_name}}", description: "Current project name" },
  { name: "{{date}}", description: "Current date" },
  { name: "{{overdue_count}}", description: "Number of overdue tasks" },
  { name: "{{completed_today}}", description: "Tasks completed today" },
];

// ─── Parsing ──────────────────────────────────────────────────────────────────

export function parseSkillFile(raw: string): ParsedSkillFile {
  const { frontmatter, body } = splitFrontmatter(raw);
  const parsedFm = YAML.parse(frontmatter) as SkillFrontmatter;
  const parsedBody = parseMarkdownBody(body);

  return { frontmatter: parsedFm, body: parsedBody, raw };
}

function splitFrontmatter(raw: string): { frontmatter: string; body: string } {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("---")) {
    return { frontmatter: "", body: trimmed };
  }

  const endIdx = trimmed.indexOf("---", 3);
  if (endIdx === -1) {
    return { frontmatter: "", body: trimmed };
  }

  const frontmatter = trimmed.slice(3, endIdx).trim();
  const body = trimmed.slice(endIdx + 3).trim();
  return { frontmatter, body };
}

function parseMarkdownBody(body: string): SkillMarkdownBody {
  const sections: SkillMarkdownBody = { ...EMPTY_BODY };

  if (!body.trim()) return sections;

  const lines = body.split("\n");
  let currentSection: keyof SkillMarkdownBody | null = null;
  const sectionLines: Record<keyof SkillMarkdownBody, string[]> = {
    role: [],
    context: [],
    instructions: [],
    output_format: [],
    examples: [],
  };

  for (const line of lines) {
    const headingMatch = line.match(/^#\s+(.+)$/);
    if (headingMatch) {
      const heading = headingMatch[1].trim().toLowerCase();
      const mapped = SECTION_HEADINGS[heading];
      if (mapped) {
        currentSection = mapped;
        continue;
      }
    }

    if (currentSection) {
      sectionLines[currentSection].push(line);
    }
  }

  for (const key of BODY_SECTIONS) {
    const content = sectionLines[key].join("\n").trim();
    sections[key] = content;
  }

  return sections;
}

// ─── Serialization ────────────────────────────────────────────────────────────

export function serializeSkillFile(frontmatter: SkillFrontmatter, body: SkillMarkdownBody): string {
  const yamlStr = YAML.stringify(frontmatter, { lineWidth: 0 }).trim();
  const bodyStr = serializeMarkdownBody(body);

  return `---\n${yamlStr}\n---\n\n${bodyStr}\n`;
}

function serializeMarkdownBody(body: SkillMarkdownBody): string {
  const parts: string[] = [];

  const sectionOrder: { key: keyof SkillMarkdownBody; heading: string }[] = [
    { key: "role", heading: "Role" },
    { key: "context", heading: "Context" },
    { key: "instructions", heading: "Instructions" },
    { key: "output_format", heading: "Output Format" },
    { key: "examples", heading: "Examples" },
  ];

  for (const { key, heading } of sectionOrder) {
    const content = body[key]?.trim();
    if (content) {
      parts.push(`# ${heading}\n\n${content}`);
    }
  }

  return parts.join("\n\n");
}

// ─── Conversion: Skill (DB) → skill.md format ────────────────────────────────

export function skillToSkillFile(skill: Skill): string {
  const triggerConfig = skill.trigger_config ? safeJsonParse(skill.trigger_config) : {};
  const contextConfig = skill.context_config ? safeJsonParse(skill.context_config) : {};
  const actionConfig = skill.action_config ? safeJsonParse(skill.action_config) : {};
  const tags = skill.tags ? safeJsonParse(skill.tags) : null;

  const frontmatter: SkillFrontmatter = {
    name: skill.name,
    description: skill.description || undefined,
    trigger: {
      type: skill.trigger_type as SkillFrontmatter["trigger"]["type"],
      ...triggerConfig,
    },
    action: {
      type: (actionConfig.action_type || "summarize") as SkillFrontmatter["action"]["type"],
      format: actionConfig.format as string | undefined,
      channel: actionConfig.channel as string | undefined,
    },
  };

  if (contextConfig.scope || contextConfig.max_tokens || contextConfig.include_documents) {
    frontmatter.context = {
      scope: contextConfig.scope as "project" | "global" | undefined,
      max_tokens: contextConfig.max_tokens as number | undefined,
      include_documents: contextConfig.include_documents as boolean | undefined,
      document_filter: contextConfig.document_filter as string | undefined,
      max_documents: contextConfig.max_documents as number | undefined,
    };
  }

  const settings: NonNullable<SkillFrontmatter["settings"]> = {};
  if (skill.approval_mode && skill.approval_mode !== "notify") {
    settings.approval_mode = skill.approval_mode as NonNullable<SkillFrontmatter["settings"]>["approval_mode"];
  }
  if (skill.category) settings.category = skill.category;
  if (Array.isArray(tags)) settings.tags = tags as string[];
  if (skill.shared) settings.shared = skill.shared;
  if (Object.keys(settings).length > 0) {
    frontmatter.settings = settings;
  }

  // Remove undefined values from action/trigger
  cleanUndefined(frontmatter.trigger);
  cleanUndefined(frontmatter.action);

  const body = parseBodyFromSystemPrompt(contextConfig.system_prompt as string | undefined);

  return serializeSkillFile(frontmatter, body);
}

function parseBodyFromSystemPrompt(systemPrompt: string | null | undefined): SkillMarkdownBody {
  if (!systemPrompt || !systemPrompt.trim()) return { ...EMPTY_BODY };

  // Try XML format first (existing skills)
  const xmlSections = tryParseXmlSections(systemPrompt);
  if (xmlSections) return xmlSections;

  // Try markdown heading format
  const mdSections = parseMarkdownBody(systemPrompt);
  const hasContent = BODY_SECTIONS.some((k) => mdSections[k].trim());
  if (hasContent) return mdSections;

  // Fallback: everything goes in instructions
  return { ...EMPTY_BODY, instructions: systemPrompt.trim() };
}

function tryParseXmlSections(raw: string): SkillMarkdownBody | null {
  const sections: SkillMarkdownBody = { ...EMPTY_BODY };
  let hasAnyTag = false;

  for (const key of BODY_SECTIONS) {
    const regex = new RegExp(`<${key}>\\s*([\\s\\S]*?)\\s*</${key}>`, "i");
    const match = raw.match(regex);
    if (match) {
      sections[key] = match[1].trim();
      hasAnyTag = true;
    }
  }

  return hasAnyTag ? sections : null;
}

// ─── Conversion: skill.md format → CreateSkillInput (for import/save) ─────────

export function skillFileToCreateInput(parsed: ParsedSkillFile): CreateSkillInput {
  const { frontmatter: fm, body } = parsed;

  const systemPrompt = serializeMarkdownBody(body);

  const triggerConfig: Record<string, unknown> = {};
  if (fm.trigger.cron) triggerConfig.cron = fm.trigger.cron;
  if (fm.trigger.timezone) triggerConfig.timezone = fm.trigger.timezone;
  if (fm.trigger.event_type) triggerConfig.event_type = fm.trigger.event_type;
  if (fm.trigger.filter) triggerConfig.filter = fm.trigger.filter;

  const contextConfig: Record<string, unknown> = {};
  if (fm.context?.scope) contextConfig.scope = fm.context.scope;
  if (fm.context?.max_tokens) contextConfig.max_tokens = fm.context.max_tokens;
  if (fm.context?.include_documents) contextConfig.include_documents = fm.context.include_documents;
  if (fm.context?.document_filter) contextConfig.document_filter = fm.context.document_filter;
  if (fm.context?.max_documents) contextConfig.max_documents = fm.context.max_documents;
  if (systemPrompt) contextConfig.system_prompt = systemPrompt;

  const actionConfig: Record<string, unknown> = {};
  if (fm.action.type) actionConfig.action_type = fm.action.type;
  if (fm.action.format) actionConfig.format = fm.action.format;
  if (fm.action.channel) actionConfig.channel = fm.action.channel;
  if (fm.action.template) actionConfig.template = fm.action.template;
  if (fm.action.max_length) actionConfig.max_length = fm.action.max_length;

  return {
    name: fm.name,
    description: fm.description,
    trigger_type: fm.trigger.type,
    trigger_config: Object.keys(triggerConfig).length > 0 ? triggerConfig : undefined,
    context_config: Object.keys(contextConfig).length > 0 ? contextConfig : undefined,
    action_config: Object.keys(actionConfig).length > 0 ? (actionConfig as CreateSkillInput["action_config"]) : undefined,
    approval_mode: fm.settings?.approval_mode || "notify",
    category: fm.settings?.category,
    tags: fm.settings?.tags,
    shared: fm.settings?.shared,
  };
}

// ─── Export (DB → directory format) ───────────────────────────────────────────

export function exportSkillAsDirectory(skill: Skill): { filename: string; content: string } {
  const skillMd = skillToSkillFile(skill);
  const dirName = skill.name.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
  return { filename: `${dirName}/skill.md`, content: skillMd };
}

// ─── Import detection ─────────────────────────────────────────────────────────

export function isSkillMdFormat(content: string): boolean {
  return content.trim().startsWith("---");
}

export function isV2JsonFormat(json: unknown): boolean {
  return (
    typeof json === "object" &&
    json !== null &&
    (json as Record<string, unknown>).version === "2.0" &&
    typeof (json as Record<string, unknown>).prompt === "object"
  );
}

// ─── Token estimation (kept from old module) ──────────────────────────────────

export function estimateTokens(text: string): number {
  if (!text || !text.trim()) return 0;
  return Math.ceil(text.length / 4);
}

// ─── Utilities ────────────────────────────────────────────────────────────────

function safeJsonParse(str: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(str);
    return typeof parsed === "object" && parsed !== null ? parsed : {};
  } catch {
    return {};
  }
}

function cleanUndefined(obj: Record<string, unknown>): void {
  for (const key of Object.keys(obj)) {
    if (obj[key] === undefined) delete obj[key];
  }
}
