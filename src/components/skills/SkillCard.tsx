import { useState } from "react";
import {
  Play,
  MoreHorizontal,
  Clock,
  Zap,
  Hand,
  Copy,
  Trash2,
  Edit,
  BarChart2,
  Download,
} from "lucide-react";
import type { Skill } from "@/lib/tauri";
import { exportSkillToDirectory } from "@/lib/tauri";
import { skillToSkillFile } from "@/lib/skill-format";
import { useToggleSkillEnabled, useRunSkillManually, useDeleteSkill, useCloneSkill } from "@/hooks/useSkills";

interface SkillCardProps {
  skill: Skill;
  onEdit: (skill: Skill) => void;
  onViewHistory: (skill: Skill) => void;
}

const triggerIcons: Record<string, typeof Clock> = {
  schedule: Clock,
  event: Zap,
  manual: Hand,
};

const triggerLabels: Record<string, string> = {
  schedule: "Scheduled",
  event: "Event-triggered",
  manual: "Manual",
};

export function SkillCard({ skill, onEdit, onViewHistory }: SkillCardProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const toggleEnabled = useToggleSkillEnabled();
  const runManually = useRunSkillManually();
  const deleteSkill = useDeleteSkill();
  const cloneSkill = useCloneSkill();

  const TriggerIcon = triggerIcons[skill.trigger_type] || Hand;
  const triggerLabel = triggerLabels[skill.trigger_type] || skill.trigger_type;

  const handleToggle = () => {
    toggleEnabled.mutate({ id: skill.id, enabled: !skill.enabled });
  };

  const handleRun = () => {
    runManually.mutate(skill.id);
  };

  const handleDelete = () => {
    if (confirm(`Delete skill "${skill.name}"?`)) {
      deleteSkill.mutate(skill.id);
    }
  };

  const handleClone = () => {
    cloneSkill.mutate({ skillId: skill.id });
    setMenuOpen(false);
  };

  const handleExport = async () => {
    try {
      const skillMd = skillToSkillFile(skill);
      await exportSkillToDirectory(skillMd, skill.name);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg !== "Export cancelled") {
        console.error("Failed to export skill:", err);
        alert(`Export failed: ${msg}`);
      }
    }
    setMenuOpen(false);
  };

  return (
    <div className="group relative border border-zinc-200 dark:border-zinc-800 rounded-lg p-4 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            {skill.icon && <span className="text-lg">{skill.icon}</span>}
            <h3 className="font-medium text-zinc-900 dark:text-zinc-100 truncate">
              {skill.name}
            </h3>
            {skill.is_builtin && (
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400">
                Built-in
              </span>
            )}
            {skill.shared && (
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400">
                Shared
              </span>
            )}
          </div>

          {skill.description && (
            <p className="text-[13px] text-zinc-500 dark:text-zinc-400 line-clamp-2 mb-2">
              {skill.description}
            </p>
          )}

          <div className="flex items-center gap-3 text-[11px] text-zinc-400">
            <span className="flex items-center gap-1">
              <TriggerIcon className="w-3 h-3" />
              {triggerLabel}
            </span>
            {skill.category && (
              <>
                <span>·</span>
                <span className="capitalize">{skill.category}</span>
              </>
            )}
            {skill.next_run_at && (
              <>
                <span>·</span>
                <span>
                  Next: {new Date(skill.next_run_at).toLocaleDateString()}
                </span>
              </>
            )}
          </div>
        </div>

        <div className="flex items-center gap-2">
          {skill.trigger_type === "manual" && (
            <button
              onClick={handleRun}
              disabled={runManually.isPending}
              className="p-1.5 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-500 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors disabled:opacity-50"
              title="Run now"
            >
              <Play className="w-4 h-4" />
            </button>
          )}

          <label className="relative inline-flex items-center cursor-pointer gap-1.5" title={skill.enabled ? "Disable skill" : "Enable skill"}>
            <input
              type="checkbox"
              checked={skill.enabled}
              onChange={handleToggle}
              className="sr-only peer"
            />
            <div className="w-9 h-5 bg-zinc-200 dark:bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-indigo-500" />
            <span className={`text-[11px] ${skill.enabled ? "text-indigo-600 dark:text-indigo-400" : "text-zinc-400"}`}>
              {skill.enabled ? "On" : "Off"}
            </span>
          </label>

          <div className="relative">
            <button
              onClick={() => setMenuOpen(!menuOpen)}
              className="p-1.5 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-400 transition-colors"
            >
              <MoreHorizontal className="w-4 h-4" />
            </button>

            {menuOpen && (
              <>
                <div
                  className="fixed inset-0 z-10"
                  onClick={() => setMenuOpen(false)}
                />
                <div className="absolute right-0 top-full mt-1 z-20 w-40 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl py-1">
                  <button
                    onClick={() => {
                      onEdit(skill);
                      setMenuOpen(false);
                    }}
                    className="w-full px-3 py-1.5 text-left text-[13px] hover:bg-zinc-100 dark:hover:bg-zinc-800 flex items-center gap-2"
                  >
                    <Edit className="w-3.5 h-3.5" />
                    Edit
                  </button>
                  <button
                    onClick={() => {
                      onViewHistory(skill);
                      setMenuOpen(false);
                    }}
                    className="w-full px-3 py-1.5 text-left text-[13px] hover:bg-zinc-100 dark:hover:bg-zinc-800 flex items-center gap-2"
                  >
                    <BarChart2 className="w-3.5 h-3.5" />
                    History
                  </button>
                  <button
                    onClick={handleClone}
                    className="w-full px-3 py-1.5 text-left text-[13px] hover:bg-zinc-100 dark:hover:bg-zinc-800 flex items-center gap-2"
                  >
                    <Copy className="w-3.5 h-3.5" />
                    Clone
                  </button>
                  <button
                    onClick={handleExport}
                    className="w-full px-3 py-1.5 text-left text-[13px] hover:bg-zinc-100 dark:hover:bg-zinc-800 flex items-center gap-2"
                  >
                    <Download className="w-3.5 h-3.5" />
                    Export
                  </button>
                  {!skill.is_builtin && (
                    <>
                      <hr className="my-1 border-zinc-200 dark:border-zinc-700" />
                      <button
                        onClick={() => {
                          handleDelete();
                          setMenuOpen(false);
                        }}
                        className="w-full px-3 py-1.5 text-left text-[13px] text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                        Delete
                      </button>
                    </>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
