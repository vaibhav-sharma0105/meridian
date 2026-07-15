import type { Skill, CreateSkillInput } from "./tauri";

export interface SkillPromptSections {
  role: string;
  context: string;
  instructions: string;
  output_format: string;
  examples: string;
}

export const SECTION_META: Record<
  keyof SkillPromptSections,
  { label: string; description: string; placeholder: string; budget: number; required: boolean }
> = {
  role: {
    label: "Role",
    description: "Who the AI is when executing this skill",
    placeholder: "You are a concise project status reporter...",
    budget: 100,
    required: true,
  },
  context: {
    label: "Context",
    description: "Data to inject — use {{variables}} for dynamic content",
    placeholder: "{{tasks}} {{meetings}} {{project_name}}",
    budget: 200,
    required: false,
  },
  instructions: {
    label: "Instructions",
    description: "Step-by-step what to do with the context",
    placeholder: "1. Summarize completed tasks\n2. Highlight blockers\n3. Note upcoming due dates",
    budget: 500,
    required: true,
  },
  output_format: {
    label: "Output Format",
    description: "Structure and constraints for the output",
    placeholder: "Use markdown. Keep under 300 words. Use headers for sections.",
    budget: 150,
    required: false,
  },
  examples: {
    label: "Examples",
    description: "Input/output pairs showing expected behavior",
    placeholder: "## Week of 2026-07-07\n**Completed:** 5 tasks across 3 assignees...",
    budget: 300,
    required: false,
  },
};

export const EMPTY_SECTIONS: SkillPromptSections = {
  role: "",
  context: "",
  instructions: "",
  output_format: "",
  examples: "",
};

const SECTION_KEYS: (keyof SkillPromptSections)[] = [
  "role",
  "context",
  "instructions",
  "output_format",
  "examples",
];

export function estimateTokens(text: string): number {
  if (!text || !text.trim()) return 0;
  return Math.ceil(text.length / 4);
}

export function getTokenStatus(tokens: number, budget: number): "ok" | "warn" | "over" {
  const ratio = tokens / budget;
  if (ratio > 1) return "over";
  if (ratio > 0.8) return "warn";
  return "ok";
}

export function parsePromptSections(raw: string | null | undefined): SkillPromptSections {
  if (!raw || !raw.trim()) return { ...EMPTY_SECTIONS };

  const sections: SkillPromptSections = { ...EMPTY_SECTIONS };
  let hasAnyTag = false;

  for (const key of SECTION_KEYS) {
    const regex = new RegExp(`<${key}>\\s*([\\s\\S]*?)\\s*</${key}>`, "i");
    const match = raw.match(regex);
    if (match) {
      sections[key] = match[1].trim();
      hasAnyTag = true;
    }
  }

  if (!hasAnyTag) {
    sections.instructions = raw.trim();
  }

  return sections;
}

export function serializePromptSections(sections: SkillPromptSections): string {
  const parts: string[] = [];
  for (const key of SECTION_KEYS) {
    const value = sections[key]?.trim();
    if (value) {
      parts.push(`<${key}>\n${value}\n</${key}>`);
    }
  }
  return parts.join("\n\n");
}

export function getTotalTokens(sections: SkillPromptSections): number {
  return SECTION_KEYS.reduce((sum, key) => sum + estimateTokens(sections[key]), 0);
}

export function getTotalBudget(): number {
  return Object.values(SECTION_META).reduce((sum, m) => sum + m.budget, 0);
}

// ─── Export/Import v2 ──────────────────────────────────────────────────────────

export interface SkillExportV2 {
  version: "2.0";
  name: string;
  description: string | null;
  trigger: {
    type: string;
    cron?: string;
    timezone?: string;
    event_type?: string;
    filter?: Record<string, unknown>;
  };
  action: {
    type?: string;
    format?: string;
    template?: string;
    max_length?: number;
  };
  prompt: Partial<SkillPromptSections>;
  settings: {
    approval_mode: string;
    category: string | null;
    tags: string[] | null;
  };
  exported_at: string;
  token_budget: Record<string, number>;
}

export function exportSkillV2(skill: Skill): SkillExportV2 {
  const triggerConfig = skill.trigger_config ? safeJsonParse(skill.trigger_config) : {};
  const actionConfig = skill.action_config ? safeJsonParse(skill.action_config) : {};
  const contextConfig = skill.context_config ? safeJsonParse(skill.context_config) : {};
  const tags = skill.tags ? safeJsonParse(skill.tags) : null;

  const sections = parsePromptSections(contextConfig?.system_prompt as string | undefined);

  const prompt: Partial<SkillPromptSections> = {};
  for (const key of SECTION_KEYS) {
    if (sections[key]) prompt[key] = sections[key];
  }

  return {
    version: "2.0",
    name: skill.name,
    description: skill.description,
    trigger: {
      type: skill.trigger_type,
      ...triggerConfig,
    },
    action: actionConfig,
    prompt,
    settings: {
      approval_mode: skill.approval_mode,
      category: skill.category,
      tags: Array.isArray(tags) ? tags : null,
    },
    exported_at: new Date().toISOString(),
    token_budget: Object.fromEntries(
      SECTION_KEYS.map((k) => [k, SECTION_META[k].budget])
    ),
  };
}

export function importSkillFromV2(json: SkillExportV2): CreateSkillInput {
  const sections: SkillPromptSections = { ...EMPTY_SECTIONS };
  if (json.prompt) {
    for (const key of SECTION_KEYS) {
      if (json.prompt[key]) sections[key] = json.prompt[key]!;
    }
  }
  const systemPrompt = serializePromptSections(sections);

  const { type: triggerType, ...triggerRest } = json.trigger || { type: "manual" };

  return {
    name: json.name,
    description: json.description || undefined,
    trigger_type: triggerType as "schedule" | "event" | "manual",
    trigger_config: Object.keys(triggerRest).length > 0 ? triggerRest : undefined,
    context_config: systemPrompt ? { system_prompt: systemPrompt } : undefined,
    action_config: json.action && Object.keys(json.action).length > 0
      ? json.action as CreateSkillInput["action_config"]
      : undefined,
    approval_mode: (json.settings?.approval_mode as CreateSkillInput["approval_mode"]) || "notify",
    category: json.settings?.category || undefined,
    tags: json.settings?.tags || undefined,
  };
}

export function isV2Export(json: unknown): json is SkillExportV2 {
  return (
    typeof json === "object" &&
    json !== null &&
    (json as Record<string, unknown>).version === "2.0" &&
    typeof (json as Record<string, unknown>).prompt === "object"
  );
}

function safeJsonParse(str: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(str);
    return typeof parsed === "object" && parsed !== null ? parsed : {};
  } catch {
    return {};
  }
}
