import { useState, useEffect, useRef, useCallback } from "react";
import { X, Play, Loader2, AlertCircle, Variable, Code, Settings2 } from "lucide-react";
import type { Skill, CreateSkillInput } from "@/lib/tauri";
import { useCreateSkill, useUpdateSkill, useTestRunSkill } from "@/hooks/useSkills";
import { useUIStore } from "@/stores/uiStore";
import {
  parseSkillFile,
  skillToSkillFile,
  skillFileToCreateInput,
  serializeSkillFile,
  AVAILABLE_VARIABLES,
  type SkillFrontmatter,
  type SkillMarkdownBody,
  EMPTY_BODY,
} from "@/lib/skill-format";

interface SkillEditorModalProps {
  skill: Skill | null;
  onClose: () => void;
}

const TRIGGER_TYPES = [
  { value: "manual", label: "Manual", description: "Run on demand" },
  { value: "schedule", label: "Scheduled", description: "Run on a cron schedule" },
  { value: "event", label: "Event-triggered", description: "Run when an event occurs" },
];

const EVENT_TYPES = [
  { value: "task_created", label: "Task Created" },
  { value: "task_completed", label: "Task Completed" },
  { value: "meeting_imported", label: "Meeting Imported" },
];

const CRON_PRESETS = [
  { value: "0 9 * * 1-5", label: "Weekdays at 9am" },
  { value: "0 9 * * 1", label: "Mondays at 9am" },
  { value: "0 17 * * *", label: "Daily at 5pm" },
  { value: "0 9 * * *", label: "Daily at 9am" },
];

const ACTION_TYPES = [
  { value: "summarize", label: "Summarize", description: "Generate a summary" },
  { value: "draft_message", label: "Draft Message", description: "Draft an email or message" },
  { value: "create_tasks", label: "Create Tasks", description: "Suggest tasks to create" },
  { value: "analyze", label: "Analyze", description: "Provide analysis insights" },
  { value: "custom", label: "Custom", description: "Use custom instructions" },
];

const APPROVAL_MODES = [
  { value: "auto", label: "Auto", description: "Execute without notification" },
  { value: "notify", label: "Notify", description: "Execute and notify of results" },
  { value: "approve_first", label: "Approve for side effects", description: "Require approval for actions with side effects" },
  { value: "approve_always", label: "Always approve", description: "Always require approval before execution" },
];

const CATEGORIES = [
  { value: "custom", label: "Custom" },
  { value: "productivity", label: "Productivity" },
  { value: "communication", label: "Communication" },
  { value: "reporting", label: "Reporting" },
];

const fieldCls =
  "w-full px-3 py-2 text-[13px] rounded-lg border border-zinc-200 dark:border-zinc-700 " +
  "bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 " +
  "focus:border-indigo-400 transition-colors";

const labelCls = "block text-[11px] font-medium text-zinc-500 dark:text-zinc-400 mb-1.5";

export function SkillEditorModal({ skill, onClose }: SkillEditorModalProps) {
  const [mode, setMode] = useState<"basic" | "advanced">("basic");
  const [rawContent, setRawContent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [showVars, setShowVars] = useState(false);

  // Basic mode fields
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [triggerType, setTriggerType] = useState<"manual" | "schedule" | "event">("manual");
  const [cronExpr, setCronExpr] = useState("");
  const [eventType, setEventType] = useState("task_created");
  const [actionType, setActionType] = useState("summarize");
  const [approvalMode, setApprovalMode] = useState("notify");
  const [category, setCategory] = useState("custom");
  const [instructions, setInstructions] = useState("");
  const [shared, setShared] = useState(false);
  const [includeDocuments, setIncludeDocuments] = useState(false);
  const [documentFilter, setDocumentFilter] = useState("");
  const [maxDocuments, setMaxDocuments] = useState(10);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const skillEditorData = useUIStore((s) => s.skillEditorData);
  const setSkillEditorData = useUIStore((s) => s.setSkillEditorData);

  const createSkill = useCreateSkill();
  const updateSkill = useUpdateSkill();
  const testRun = useTestRunSkill();

  // Parse skill into basic fields
  const parseIntoFields = useCallback((content: string) => {
    try {
      const parsed = parseSkillFile(content);
      setName(parsed.frontmatter.name || "");
      setDescription(parsed.frontmatter.description || "");
      setTriggerType((parsed.frontmatter.trigger?.type as "manual" | "schedule" | "event") || "manual");
      setCronExpr(parsed.frontmatter.trigger?.cron || "");
      setEventType(parsed.frontmatter.trigger?.event_type || "task_created");
      setActionType(parsed.frontmatter.action?.type || "summarize");
      setApprovalMode(parsed.frontmatter.settings?.approval_mode || "notify");
      setCategory(parsed.frontmatter.settings?.category || "custom");
      setInstructions(parsed.body.instructions || "");
    } catch {
      // If parsing fails, keep existing values
    }
  }, []);

  // Build content from basic fields
  const buildFromFields = useCallback((): string => {
    const fm: SkillFrontmatter = {
      name,
      description: description || undefined,
      trigger: {
        type: triggerType,
        ...(triggerType === "schedule" && cronExpr ? { cron: cronExpr } : {}),
        ...(triggerType === "event" ? { event_type: eventType } : {}),
      },
      action: {
        type: actionType as SkillFrontmatter["action"]["type"],
      },
      settings: {
        approval_mode: approvalMode as NonNullable<SkillFrontmatter["settings"]>["approval_mode"],
        category,
      },
    };
    const body: SkillMarkdownBody = {
      ...EMPTY_BODY,
      instructions,
    };
    return serializeSkillFile(fm, body);
  }, [name, description, triggerType, cronExpr, eventType, actionType, approvalMode, category, instructions]);

  useEffect(() => {
    if (skill) {
      const content = skillToSkillFile(skill);
      setRawContent(content);
      parseIntoFields(content);
      // Set shared from skill directly (not in YAML)
      setShared(skill.shared ?? false);
      // Set document options from context_config
      try {
        const ctxConfig = skill.context_config ? JSON.parse(skill.context_config) : {};
        setIncludeDocuments(ctxConfig.include_documents ?? false);
        setDocumentFilter(ctxConfig.document_filter ?? "");
        setMaxDocuments(ctxConfig.max_documents ?? 10);
      } catch {
        setIncludeDocuments(false);
        setDocumentFilter("");
        setMaxDocuments(10);
      }
    } else if (skillEditorData) {
      const data = skillEditorData as Record<string, unknown>;
      setName((data.name as string) || "");
      setDescription((data.description as string) || "");
      setTriggerType(((data.trigger_type as string) || "manual") as "manual" | "schedule" | "event");
      setActionType((data.action_type as string) || "summarize");
      setApprovalMode((data.approval_mode as string) || "notify");
      setCategory("custom");
      setInstructions((data.system_prompt as string) || "");
      setSkillEditorData(null);
    } else {
      setName("");
      setDescription("");
      setTriggerType("manual");
      setActionType("summarize");
      setApprovalMode("notify");
      setCategory("custom");
      setInstructions("");
      setShared(false);
      setIncludeDocuments(false);
      setDocumentFilter("");
      setMaxDocuments(10);
    }
  }, [skill, skillEditorData, setSkillEditorData, parseIntoFields]);

  // Sync between modes
  useEffect(() => {
    if (mode === "advanced") {
      setRawContent(buildFromFields());
    }
  }, [mode, buildFromFields]);

  const insertVariable = useCallback((variable: string) => {
    if (mode === "basic") {
      setInstructions((prev) => prev + variable);
    } else {
      const textarea = textareaRef.current;
      if (!textarea) return;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newValue = rawContent.slice(0, start) + variable + rawContent.slice(end);
      setRawContent(newValue);
      setTimeout(() => {
        textarea.focus();
        textarea.selectionStart = textarea.selectionEnd = start + variable.length;
      }, 0);
    }
    setShowVars(false);
  }, [mode, rawContent]);

  const handleTestRun = async () => {
    if (!skill) return;
    setTestResult(null);
    setError(null);
    try {
      const result = await testRun.mutateAsync(skill.id);
      setTestResult(JSON.stringify(result, null, 2));
    } catch (err) {
      setError(`Test run failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const validate = (): CreateSkillInput | null => {
    const content = mode === "basic" ? buildFromFields() : rawContent;
    try {
      const parsed = parseSkillFile(content);

      if (!parsed.frontmatter.name?.trim()) {
        setError("Name is required");
        return null;
      }

      if (!parsed.body.instructions?.trim()) {
        setError("Instructions are required");
        return null;
      }

      if (parsed.frontmatter.trigger?.type === "schedule" && !parsed.frontmatter.trigger.cron) {
        setError("Cron expression is required for scheduled triggers");
        return null;
      }

      return skillFileToCreateInput(parsed);
    } catch (err) {
      setError(`Invalid skill format: ${err instanceof Error ? err.message : String(err)}`);
      return null;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const input = validate();
    if (!input) return;

    // Add document options to context_config
    const contextConfig = {
      ...(input.context_config || {}),
      include_documents: includeDocuments,
      ...(includeDocuments && documentFilter ? { document_filter: documentFilter } : {}),
      ...(includeDocuments ? { max_documents: maxDocuments } : {}),
    };

    try {
      if (skill) {
        // For updates, include shared and updated context_config
        await updateSkill.mutateAsync({
          id: skill.id,
          ...input,
          context_config: contextConfig,
          shared,
        });
      } else {
        // For creates, include updated context_config (shared defaults to false on create)
        await createSkill.mutateAsync({
          ...input,
          context_config: contextConfig,
        });
      }
      onClose();
    } catch (err) {
      setError(`Failed to save skill: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const isPending = createSkill.isPending || updateSkill.isPending;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-4">
            <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              {skill ? "Edit Skill" : "Create Skill"}
            </h2>
            <div className="flex items-center gap-1 bg-zinc-100 dark:bg-zinc-800 p-0.5 rounded-lg">
              <button
                type="button"
                onClick={() => setMode("basic")}
                className={`flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium rounded-md transition-colors ${
                  mode === "basic"
                    ? "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100 shadow-sm"
                    : "text-zinc-500 dark:text-zinc-400"
                }`}
              >
                <Settings2 className="w-3 h-3" />
                Basic
              </button>
              <button
                type="button"
                onClick={() => setMode("advanced")}
                className={`flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium rounded-md transition-colors ${
                  mode === "advanced"
                    ? "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100 shadow-sm"
                    : "text-zinc-500 dark:text-zinc-400"
                }`}
              >
                <Code className="w-3 h-3" />
                YAML
              </button>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {skill && (
              <button
                type="button"
                onClick={handleTestRun}
                disabled={testRun.isPending}
                className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] text-indigo-600 dark:text-indigo-400 border border-indigo-200 dark:border-indigo-800 rounded-lg hover:bg-indigo-50 dark:hover:bg-indigo-900/20 transition-colors disabled:opacity-50"
              >
                {testRun.isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
                Test
              </button>
            )}
            <button onClick={onClose} className="p-1 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800">
              <X className="w-5 h-5 text-zinc-400" />
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto flex flex-col min-h-0">
          {/* Error banner */}
          {error && (
            <div className="flex items-start gap-2 mx-6 mt-4 px-3 py-2.5 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
              <p className="text-[12.5px] text-red-700 dark:text-red-300">{error}</p>
            </div>
          )}

          {mode === "basic" ? (
            /* Basic Mode */
            <div className="flex-1 p-6 space-y-5 overflow-y-auto">
              {/* Name & Description */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className={labelCls}>Name *</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="My Skill"
                    className={fieldCls}
                  />
                </div>
                <div>
                  <label className={labelCls}>Category</label>
                  <select value={category} onChange={(e) => setCategory(e.target.value)} className={fieldCls}>
                    {CATEGORIES.map((c) => (
                      <option key={c.value} value={c.value}>{c.label}</option>
                    ))}
                  </select>
                </div>
              </div>

              <div>
                <label className={labelCls}>Description</label>
                <input
                  type="text"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="What this skill does..."
                  className={fieldCls}
                />
              </div>

              {/* Trigger */}
              <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4 space-y-3">
                <label className="block text-[12px] font-semibold text-zinc-700 dark:text-zinc-300">Trigger</label>
                <div className="grid grid-cols-3 gap-2">
                  {TRIGGER_TYPES.map((t) => (
                    <button
                      key={t.value}
                      type="button"
                      onClick={() => setTriggerType(t.value as "manual" | "schedule" | "event")}
                      className={`p-3 rounded-lg border text-left transition-colors ${
                        triggerType === t.value
                          ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/30"
                          : "border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800"
                      }`}
                    >
                      <div className="text-[12px] font-medium text-zinc-900 dark:text-zinc-100">{t.label}</div>
                      <div className="text-[10px] text-zinc-500">{t.description}</div>
                    </button>
                  ))}
                </div>

                {triggerType === "schedule" && (
                  <div className="space-y-2 pt-2">
                    <label className={labelCls}>Cron Expression</label>
                    <input
                      type="text"
                      value={cronExpr}
                      onChange={(e) => setCronExpr(e.target.value)}
                      placeholder="0 9 * * 1-5"
                      className={fieldCls + " font-mono"}
                    />
                    <div className="flex flex-wrap gap-1.5">
                      {CRON_PRESETS.map((p) => (
                        <button
                          key={p.value}
                          type="button"
                          onClick={() => setCronExpr(p.value)}
                          className="px-2 py-1 text-[10px] bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700"
                        >
                          {p.label}
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {triggerType === "event" && (
                  <div className="pt-2">
                    <label className={labelCls}>Event Type</label>
                    <select value={eventType} onChange={(e) => setEventType(e.target.value)} className={fieldCls}>
                      {EVENT_TYPES.map((e) => (
                        <option key={e.value} value={e.value}>{e.label}</option>
                      ))}
                    </select>
                  </div>
                )}
              </div>

              {/* Action & Approval */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className={labelCls}>Action Type</label>
                  <select value={actionType} onChange={(e) => setActionType(e.target.value)} className={fieldCls}>
                    {ACTION_TYPES.map((a) => (
                      <option key={a.value} value={a.value}>{a.label}</option>
                    ))}
                  </select>
                  <p className="text-[10px] text-zinc-400 mt-1">
                    {ACTION_TYPES.find((a) => a.value === actionType)?.description}
                  </p>
                </div>
                <div>
                  <label className={labelCls}>Approval Mode</label>
                  <select value={approvalMode} onChange={(e) => setApprovalMode(e.target.value)} className={fieldCls}>
                    {APPROVAL_MODES.map((a) => (
                      <option key={a.value} value={a.value}>{a.label}</option>
                    ))}
                  </select>
                  <p className="text-[10px] text-zinc-400 mt-1">
                    {APPROVAL_MODES.find((a) => a.value === approvalMode)?.description}
                  </p>
                </div>
              </div>

              {/* Options */}
              <div className="flex items-center gap-6">
                <label className="flex items-center gap-2 text-sm text-zinc-700 dark:text-zinc-300">
                  <input
                    type="checkbox"
                    checked={shared}
                    onChange={(e) => setShared(e.target.checked)}
                    className="rounded border-zinc-300 dark:border-zinc-600 text-indigo-500 focus:ring-indigo-500"
                  />
                  <span className="text-[12px]">Share with team</span>
                </label>
                <label className="flex items-center gap-2 text-sm text-zinc-700 dark:text-zinc-300">
                  <input
                    type="checkbox"
                    checked={includeDocuments}
                    onChange={(e) => setIncludeDocuments(e.target.checked)}
                    className="rounded border-zinc-300 dark:border-zinc-600 text-indigo-500 focus:ring-indigo-500"
                  />
                  <span className="text-[12px]">Include project documents in context</span>
                </label>
              </div>

              {/* Document options - shown when includeDocuments is checked */}
              {includeDocuments && (
                <div className="grid grid-cols-2 gap-3 pl-6 border-l-2 border-zinc-200 dark:border-zinc-700">
                  <div>
                    <label className={labelCls}>Filter pattern</label>
                    <input
                      type="text"
                      value={documentFilter}
                      onChange={(e) => setDocumentFilter(e.target.value)}
                      placeholder="e.g. .*\.md$"
                      className={fieldCls + " font-mono text-[11px]"}
                    />
                    <p className="text-[10px] text-zinc-400 mt-0.5">Regex to filter by filename</p>
                  </div>
                  <div>
                    <label className={labelCls}>Max documents</label>
                    <input
                      type="number"
                      min={1}
                      max={50}
                      value={maxDocuments}
                      onChange={(e) => setMaxDocuments(Math.max(1, Math.min(50, parseInt(e.target.value) || 10)))}
                      className={fieldCls}
                    />
                    <p className="text-[10px] text-zinc-400 mt-0.5">Limit (1-50)</p>
                  </div>
                </div>
              )}

              {/* Instructions */}
              <div>
                <div className="flex items-center justify-between mb-1.5">
                  <label className={labelCls + " mb-0"}>Instructions *</label>
                  <div className="relative">
                    <button
                      type="button"
                      onClick={() => setShowVars(!showVars)}
                      className="flex items-center gap-1 text-[10px] text-indigo-500 hover:text-indigo-700"
                    >
                      <Variable className="w-3 h-3" />
                      Insert variable
                    </button>
                    {showVars && (
                      <div className="absolute right-0 top-full mt-1 w-56 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl py-1 z-20">
                        {AVAILABLE_VARIABLES.map((v) => (
                          <button
                            key={v.name}
                            type="button"
                            onClick={() => insertVariable(v.name)}
                            className="w-full px-3 py-1.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-700"
                          >
                            <div className="text-[11px] font-mono text-indigo-600 dark:text-indigo-400">{v.name}</div>
                            <div className="text-[10px] text-zinc-500">{v.description}</div>
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
                <textarea
                  value={instructions}
                  onChange={(e) => setInstructions(e.target.value)}
                  placeholder="Describe what this skill should do..."
                  rows={6}
                  className={fieldCls + " resize-none font-mono text-[12px]"}
                />
              </div>
            </div>
          ) : (
            /* Advanced Mode - Raw YAML+MD */
            <div className="flex-1 flex flex-col min-h-0 p-6 pb-2 space-y-2">
              <div className="flex items-center justify-between">
                <p className="text-[12px] text-zinc-500">YAML frontmatter + Markdown body</p>
                <div className="relative">
                  <button
                    type="button"
                    onClick={() => setShowVars(!showVars)}
                    className="flex items-center gap-1 text-[11px] text-indigo-500 hover:text-indigo-700"
                  >
                    <Variable className="w-3 h-3" />
                    Insert variable
                  </button>
                  {showVars && (
                    <div className="absolute right-0 top-full mt-1 w-56 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl py-1 z-20">
                      {AVAILABLE_VARIABLES.map((v) => (
                        <button
                          key={v.name}
                          type="button"
                          onClick={() => insertVariable(v.name)}
                          className="w-full px-3 py-1.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-700"
                        >
                          <div className="text-[11px] font-mono text-indigo-600 dark:text-indigo-400">{v.name}</div>
                          <div className="text-[10px] text-zinc-500">{v.description}</div>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
              <textarea
                ref={textareaRef}
                value={rawContent}
                onChange={(e) => setRawContent(e.target.value)}
                className="flex-1 min-h-[320px] w-full px-4 py-3 text-[12.5px] border border-zinc-200 dark:border-zinc-700 rounded-lg bg-zinc-50 dark:bg-zinc-950 resize-none font-mono leading-relaxed text-zinc-800 dark:text-zinc-200"
                spellCheck={false}
              />
            </div>
          )}

          {/* Test Run Result */}
          {testResult && (
            <div className="mx-6 mb-4 border border-zinc-200 dark:border-zinc-800 rounded-lg overflow-hidden">
              <div className="px-3 py-2 bg-zinc-50 dark:bg-zinc-800 border-b border-zinc-200 dark:border-zinc-700">
                <span className="text-[12px] font-medium text-zinc-600 dark:text-zinc-400">Test Run Result</span>
              </div>
              <pre className="p-3 text-[12px] text-zinc-700 dark:text-zinc-300 overflow-auto max-h-48 font-mono">
                {testResult}
              </pre>
            </div>
          )}
        </form>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-[13px] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isPending}
            className="px-4 py-2 text-[13px] bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isPending ? "Saving..." : skill ? "Save Changes" : "Create Skill"}
          </button>
        </div>
      </div>
    </div>
  );
}
