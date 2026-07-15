import { useState, useEffect } from "react";
import { Zap, RotateCcw } from "lucide-react";
import type { Skill } from "@/lib/tauri";
import { useUIStore } from "@/stores/uiStore";
import { useResetBuiltinSkills } from "@/hooks/useSkills";
import { SkillsList } from "./SkillsList";
import { SkillEditorModal } from "./SkillEditorModal";
import { SkillHistoryPanel } from "./SkillHistoryPanel";

export function SkillsPage() {
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingSkill, setEditingSkill] = useState<Skill | null>(null);
  const [historySkill, setHistorySkill] = useState<Skill | null>(null);

  const skillEditorData = useUIStore((s) => s.skillEditorData);
  const resetBuiltin = useResetBuiltinSkills();

  useEffect(() => {
    if (skillEditorData) {
      setEditingSkill(null);
      setEditorOpen(true);
    }
  }, [skillEditorData]);

  const handleCreateSkill = () => {
    setEditingSkill(null);
    setEditorOpen(true);
  };

  const handleEditSkill = (skill: Skill) => {
    setEditingSkill(skill);
    setEditorOpen(true);
  };

  const handleViewHistory = (skill: Skill) => {
    setHistorySkill(skill);
  };

  const handleCloseEditor = () => {
    setEditorOpen(false);
    setEditingSkill(null);
  };

  const handleResetDefaults = () => {
    if (
      window.confirm(
        "This will re-create the built-in skill templates. Your custom skills won't be affected. Continue?"
      )
    ) {
      resetBuiltin.mutate();
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
            <Zap className="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              Skills
            </h1>
            <p className="text-[13px] text-zinc-500">
              Automate tasks with scheduled or event-triggered workflows
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleResetDefaults}
            disabled={resetBuiltin.isPending}
            className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors disabled:opacity-50"
            title="Re-create built-in skill templates"
          >
            <RotateCcw className={`w-3.5 h-3.5 ${resetBuiltin.isPending ? "animate-spin" : ""}`} />
            Reset defaults
          </button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        <div className={`flex-1 ${historySkill ? "border-r border-zinc-200 dark:border-zinc-800" : ""}`}>
          <SkillsList
            onCreateSkill={handleCreateSkill}
            onEditSkill={handleEditSkill}
            onViewHistory={handleViewHistory}
          />
        </div>

        {historySkill && (
          <div className="w-96 flex-shrink-0">
            <SkillHistoryPanel
              skill={historySkill}
              onClose={() => setHistorySkill(null)}
            />
          </div>
        )}
      </div>

      {editorOpen && (
        <SkillEditorModal
          skill={editingSkill}
          onClose={handleCloseEditor}
        />
      )}
    </div>
  );
}
