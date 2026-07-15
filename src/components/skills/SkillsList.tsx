import { useState } from "react";
import { Plus, Search, Upload, AlertTriangle, Loader2 } from "lucide-react";
import type { Skill, SkillFolder } from "@/lib/tauri";
import { pickFolderDialog, installSkillFolder, listSkillFolders } from "@/lib/tauri";
import { useSkills } from "@/hooks/useSkills";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { SkillCard } from "./SkillCard";
import { SkillFolderCard } from "./SkillFolderCard";
import EmptyState from "@/components/shared/EmptyState";

const CATEGORIES = [
  { id: null, label: "All" },
  { id: "productivity", label: "Productivity" },
  { id: "communication", label: "Communication" },
  { id: "reporting", label: "Reporting" },
  { id: "custom", label: "Custom" },
] as const;

interface SkillsListProps {
  onCreateSkill: () => void;
  onEditSkill: (skill: Skill) => void;
  onViewHistory: (skill: Skill) => void;
}

export function SkillsList({
  onCreateSkill,
  onEditSkill,
  onViewHistory,
}: SkillsListProps) {
  const [category, setCategory] = useState<string | null>(null);
  const [showBuiltIn, setShowBuiltIn] = useState(true);
  const [search, setSearch] = useState("");
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [scriptWarning, setScriptWarning] = useState<{ path: string; hasScripts: boolean } | null>(null);
  const queryClient = useQueryClient();

  const { data: skills = [], isLoading } = useSkills({
    category: category || undefined,
  });

  const { data: folders = [] } = useQuery({
    queryKey: ["skill-folders"],
    queryFn: listSkillFolders,
  });

  const handleUploadSkill = async () => {
    setUploadError(null);
    const path = await pickFolderDialog();
    if (!path) return;

    setUploading(true);
    try {
      const result = await installSkillFolder(path);
      // Check if folder has executable scripts
      if (result.has_executables) {
        setScriptWarning({ path: result.name, hasScripts: true });
      }
      queryClient.invalidateQueries({ queryKey: ["skill-folders"] });
    } catch (err) {
      setUploadError(err instanceof Error ? err.message : String(err));
    } finally {
      setUploading(false);
    }
  };

  // Filter skills: built-in toggle controls visibility
  const visibleSkills = showBuiltIn
    ? skills
    : skills.filter((s) => !s.is_builtin);

  // When built-in is OFF and category is "custom" or null, include folder packages
  const showFolders = !showBuiltIn && (category === null || category === "custom");
  const filteredFolders = showFolders
    ? folders.filter((f) =>
        search
          ? f.name.toLowerCase().includes(search.toLowerCase()) ||
            f.description?.toLowerCase().includes(search.toLowerCase())
          : true
      )
    : [];

  const filteredSkills = search
    ? visibleSkills.filter(
        (s) =>
          s.name.toLowerCase().includes(search.toLowerCase()) ||
          s.description?.toLowerCase().includes(search.toLowerCase())
      )
    : visibleSkills;

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center gap-1">
          {CATEGORIES.map((cat) => (
            <button
              key={cat.id ?? "all"}
              onClick={() => setCategory(cat.id)}
              className={`px-3 py-1.5 text-[13px] rounded-md transition-colors ${
                category === cat.id
                  ? "bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900"
                  : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800"
              }`}
            >
              {cat.label}
            </button>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={showBuiltIn}
              onChange={() => setShowBuiltIn(!showBuiltIn)}
              className="sr-only peer"
            />
            <div className="w-7 h-4 bg-zinc-200 dark:bg-zinc-700 rounded-full peer peer-checked:bg-indigo-500 peer-checked:after:translate-x-3 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3 after:w-3 after:transition-all relative" />
            <span className="text-[12px] text-zinc-500 dark:text-zinc-400">Built-in</span>
          </label>

          <button
            onClick={handleUploadSkill}
            disabled={uploading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-[13px] border border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors disabled:opacity-50"
          >
            {uploading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Upload className="w-4 h-4" />
            )}
            {uploading ? "Uploading..." : "Upload Skill"}
          </button>

          <button
            onClick={onCreateSkill}
            className="flex items-center gap-1.5 px-3 py-1.5 text-[13px] bg-indigo-500 hover:bg-indigo-600 text-white rounded-md transition-colors"
          >
            <Plus className="w-4 h-4" />
            New Skill
          </button>
        </div>
      </div>

      <div className="px-4 py-2 border-b border-zinc-200 dark:border-zinc-800">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
          <input
            type="text"
            placeholder="Search skills..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 text-[13px] bg-zinc-100 dark:bg-zinc-800 border-0 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        </div>
      </div>

      {/* Upload error banner */}
      {uploadError && (
        <div className="mx-4 mt-2 flex items-start gap-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <AlertTriangle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="text-[12px] text-red-700 dark:text-red-300">{uploadError}</p>
          </div>
          <button
            onClick={() => setUploadError(null)}
            className="text-red-400 hover:text-red-600"
          >
            ×
          </button>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-12 text-zinc-400">
            Loading skills...
          </div>
        ) : filteredSkills.length === 0 && filteredFolders.length === 0 ? (
          <EmptyState
            icon="zap"
            title={search ? "No skills found" : "No skills yet"}
            description={
              search
                ? "Try a different search term"
                : "Create your first skill to automate repetitive tasks"
            }
            action={
              !search
                ? {
                    label: "Create Skill",
                    onClick: onCreateSkill,
                  }
                : undefined
            }
          />
        ) : (
          <div className="space-y-3">
            {filteredSkills.map((skill) => (
              <SkillCard
                key={skill.id}
                skill={skill}
                onEdit={onEditSkill}
                onViewHistory={onViewHistory}
              />
            ))}
            {filteredFolders.map((folder) => (
              <SkillFolderCard key={folder.name} folder={folder} />
            ))}
          </div>
        )}
      </div>

      {/* Script warning dialog */}
      {scriptWarning && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[400px] border border-zinc-200 dark:border-zinc-700">
            <div className="flex items-center gap-3 px-5 py-4 border-b border-zinc-200 dark:border-zinc-800">
              <div className="w-8 h-8 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
                <AlertTriangle className="w-4 h-4 text-amber-600" />
              </div>
              <div>
                <h3 className="text-[14px] font-semibold text-zinc-900 dark:text-zinc-100">
                  Scripts Detected
                </h3>
                <p className="text-[12px] text-zinc-500">
                  This package contains executable scripts
                </p>
              </div>
            </div>
            <div className="px-5 py-4">
              <p className="text-[13px] text-zinc-600 dark:text-zinc-400">
                The skill package <strong>"{scriptWarning.path}"</strong> contains scripts that can be executed.
                Please review the scripts before running them to ensure they are safe.
              </p>
            </div>
            <div className="flex justify-end px-5 py-3 border-t border-zinc-200 dark:border-zinc-800">
              <button
                onClick={() => setScriptWarning(null)}
                className="px-4 py-2 text-[13px] bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg transition-colors"
              >
                Got it
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
