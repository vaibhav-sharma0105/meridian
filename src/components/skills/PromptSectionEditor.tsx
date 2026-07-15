import { useRef, useState } from "react";
import { Variable, ChevronDown, ChevronRight } from "lucide-react";
import { estimateTokens, getTokenStatus, SECTION_META } from "@/lib/skill-prompt";
import type { SkillPromptSections } from "@/lib/skill-prompt";

const VARIABLES = [
  { name: "{{tasks}}", description: "All tasks in context" },
  { name: "{{meetings}}", description: "Recent meetings" },
  { name: "{{project_name}}", description: "Current project name" },
  { name: "{{date}}", description: "Current date" },
  { name: "{{overdue_count}}", description: "Number of overdue tasks" },
  { name: "{{completed_today}}", description: "Tasks completed today" },
];

interface Props {
  sectionKey: keyof SkillPromptSections;
  value: string;
  onChange: (value: string) => void;
  defaultExpanded?: boolean;
}

export function PromptSectionEditor({ sectionKey, value, onChange, defaultExpanded = false }: Props) {
  const [expanded, setExpanded] = useState(defaultExpanded || !!value);
  const [showVars, setShowVars] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const meta = SECTION_META[sectionKey];
  const tokens = estimateTokens(value);
  const status = getTokenStatus(tokens, meta.budget);
  const showVariableHelper = sectionKey === "context" || sectionKey === "instructions";

  const insertVariable = (variable: string) => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const newValue = value.slice(0, start) + variable + value.slice(end);
    onChange(newValue);
    setShowVars(false);
    setTimeout(() => {
      textarea.focus();
      textarea.selectionStart = textarea.selectionEnd = start + variable.length;
    }, 0);
  };

  const badgeColor =
    status === "over"
      ? "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400"
      : status === "warn"
      ? "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
      : "bg-zinc-100 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400";

  return (
    <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-3 py-2 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
      >
        <div className="flex items-center gap-2">
          {expanded ? (
            <ChevronDown className="w-3.5 h-3.5 text-zinc-400" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5 text-zinc-400" />
          )}
          <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">
            {meta.label}
          </span>
          {meta.required && (
            <span className="text-[10px] text-red-400 font-medium">*</span>
          )}
          {!expanded && value && (
            <span className="text-[11px] text-zinc-400 truncate max-w-[200px]">
              {value.slice(0, 60)}{value.length > 60 ? "..." : ""}
            </span>
          )}
        </div>
        <span className={`text-[11px] font-mono px-1.5 py-0.5 rounded ${badgeColor}`}>
          {tokens}/{meta.budget}
        </span>
      </button>

      {expanded && (
        <div className="px-3 pb-3 space-y-1.5">
          <p className="text-[11px] text-zinc-400">{meta.description}</p>

          <div className="relative">
            {showVariableHelper && (
              <div className="absolute right-2 top-2 z-10">
                <button
                  type="button"
                  onClick={() => setShowVars(!showVars)}
                  className="flex items-center gap-1 text-[11px] text-indigo-500 hover:text-indigo-700 dark:hover:text-indigo-300 bg-white dark:bg-zinc-800 px-1.5 py-0.5 rounded border border-zinc-200 dark:border-zinc-700"
                >
                  <Variable className="w-3 h-3" />
                  var
                </button>
                {showVars && (
                  <div className="absolute right-0 top-full mt-1 w-52 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl py-1 animate-fade-in">
                    {VARIABLES.map((v) => (
                      <button
                        key={v.name}
                        type="button"
                        onClick={() => insertVariable(v.name)}
                        className="w-full px-3 py-1.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors"
                      >
                        <div className="text-[11px] font-mono text-indigo-600 dark:text-indigo-400">
                          {v.name}
                        </div>
                        <div className="text-[10px] text-zinc-500">{v.description}</div>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
            <textarea
              ref={textareaRef}
              value={value}
              onChange={(e) => onChange(e.target.value)}
              placeholder={meta.placeholder}
              rows={sectionKey === "instructions" || sectionKey === "examples" ? 4 : 2}
              className="w-full px-3 py-2 text-[12.5px] border border-zinc-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-800 focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none font-mono leading-relaxed"
            />
          </div>

          {status === "over" && (
            <p className="text-[11px] text-red-500">
              Over budget by {tokens - meta.budget} tokens — consider being more concise.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
