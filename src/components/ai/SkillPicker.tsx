import { useState, useEffect, useRef } from "react";
import { Search, Zap, Clock, Hand, X, Folder } from "lucide-react";
import type { Skill } from "@/lib/tauri";
import type { UnifiedSkill } from "@/hooks/useAI";

interface SkillPickerProps {
  skills: UnifiedSkill[];
  onSelect: (skill: UnifiedSkill) => void;
  onClose: () => void;
  position: { bottom: number; left: number };
}

const triggerIcons: Record<string, typeof Clock> = {
  schedule: Clock,
  event: Zap,
  manual: Hand,
};

export function SkillPicker({ skills, onSelect, onClose, position }: SkillPickerProps) {
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const filteredSkills = search
    ? skills.filter(
        (s) =>
          s.name.toLowerCase().includes(search.toLowerCase()) ||
          s.description?.toLowerCase().includes(search.toLowerCase())
      )
    : skills;

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  useEffect(() => {
    searchRef.current?.focus();
  }, []);

  return (
    <div
      ref={containerRef}
      style={{ bottom: position.bottom, left: position.left }}
      className="absolute z-50 w-72 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-xl shadow-xl overflow-hidden animate-fade-in"
    >
      <div className="flex items-center justify-between px-3 py-2 border-b border-zinc-100 dark:border-zinc-800">
        <div className="flex items-center gap-2">
          <Zap className="w-4 h-4 text-indigo-500" />
          <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">Select a Skill</span>
        </div>
        <button
          onClick={onClose}
          className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="max-h-[200px] overflow-y-auto">
        {filteredSkills.length === 0 ? (
          <div className="py-6 text-center text-[12px] text-zinc-400">
            {search ? "No matching skills" : "No enabled skills"}
          </div>
        ) : (
          filteredSkills.map((skill) => {
            const TriggerIcon = skill.type === "folder" ? Folder : (triggerIcons[skill.trigger_type] || Hand);
            const isPackage = skill.type === "folder";

            return (
              <button
                key={skill.id}
                onClick={() => onSelect(skill)}
                className="w-full px-3 py-2.5 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors border-b border-zinc-50 dark:border-zinc-800 last:border-0"
              >
                <div className="flex items-center gap-2">
                  <span className="text-base flex-shrink-0">
                    {isPackage ? "📦" : "⚡"}
                  </span>
                  <span className="text-[13px] font-medium text-zinc-800 dark:text-zinc-200 truncate flex-1">
                    {skill.name}
                  </span>
                  <TriggerIcon className={`w-3 h-3 flex-shrink-0 ${isPackage ? "text-amber-500" : "text-zinc-400"}`} />
                </div>
                {skill.description && (
                  <p className="text-[11px] text-zinc-500 dark:text-zinc-400 truncate mt-0.5 ml-6">
                    {skill.description}
                  </p>
                )}
              </button>
            );
          })
        )}
      </div>

      <div className="px-3 py-2 border-t border-zinc-100 dark:border-zinc-800">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-400" />
          <input
            ref={searchRef}
            type="text"
            placeholder="Search skills..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-8 pr-3 py-1.5 text-[12px] bg-zinc-100 dark:bg-zinc-800 border-0 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>
      </div>
    </div>
  );
}

interface SkillBadgeProps {
  skill: UnifiedSkill | Skill;
  onRemove: () => void;
}

export function SkillBadge({ skill, onRemove }: SkillBadgeProps) {
  const isPackage = "type" in skill && skill.type === "folder";
  const icon = isPackage ? "📦" : ("icon" in skill && skill.icon ? skill.icon : "⚡");

  return (
    <div className="inline-flex items-center gap-1.5 px-2.5 py-1 bg-indigo-100 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 rounded-lg text-[12px] font-medium">
      <span className="text-sm">{icon}</span>
      <span>{skill.name}</span>
      <button
        onClick={onRemove}
        className="ml-0.5 p-0.5 hover:bg-indigo-200 dark:hover:bg-indigo-800 rounded transition-colors"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
}
