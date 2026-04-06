import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FileText, ChevronRight } from "lucide-react";
import { getPromptTemplates, generateOutput } from "@/lib/tauri";
import type { PromptTemplate } from "@/lib/tauri";

interface Props {
  projectId: string | null;
  onOutput: (text: string) => void;
}

export default function OutputTemplates({ projectId, onOutput }: Props) {
  const { t } = useTranslation();
  const [templates, setTemplates] = useState<PromptTemplate[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [generating, setGenerating] = useState<string | null>(null);
  const [genError, setGenError] = useState<string | null>(null);

  const load = async () => {
    if (templates.length > 0) { setExpanded(!expanded); return; }
    const tpls = await getPromptTemplates();
    setTemplates(tpls);
    setExpanded(true);
  };

  const handleGenerate = async (template: PromptTemplate) => {
    if (!projectId) return;
    setGenerating(template.id);
    setGenError(null);
    try {
      const result = await generateOutput({ projectId, templateId: template.id });
      onOutput(result);
    } catch (e) {
      setGenError(e instanceof Error ? e.message : String(e));
    } finally {
      setGenerating(null);
    }
  };

  return (
    <div className="border-t border-zinc-100 dark:border-zinc-800">
      <button
        onClick={load}
        className="flex items-center gap-2 w-full px-3 py-2 text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors"
      >
        <FileText className="w-3.5 h-3.5" />
        {t("ai.outputTemplates")}
        <ChevronRight className={`w-3 h-3 ml-auto transition-transform ${expanded ? "rotate-90" : ""}`} />
      </button>

      {expanded && templates.length > 0 && (
        <div className="px-3 pb-2 space-y-1">
          {templates.map((tpl) => (
            <button
              key={tpl.id}
              onClick={() => handleGenerate(tpl)}
              disabled={generating === tpl.id || !projectId}
              className="flex items-center justify-between w-full px-2 py-1.5 text-xs text-left rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-700 dark:text-zinc-300 disabled:opacity-50 transition-colors"
            >
              <span className="font-medium">{tpl.name}</span>
              {generating === tpl.id && (
                <span className="text-zinc-400 animate-pulse">Generating...</span>
              )}
            </button>
          ))}
          {genError && (
            <p className="text-xs text-red-500 px-2 py-1">{genError}</p>
          )}
        </div>
      )}
    </div>
  );
}
