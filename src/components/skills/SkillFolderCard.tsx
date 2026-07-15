import { useState } from "react";
import { Folder, MoreHorizontal, Trash2, Eye, Play, AlertTriangle, X } from "lucide-react";
import type { SkillFolder } from "@/lib/tauri";
import { deleteSkillFolder, readSkillFile, executeSkillScript, toggleFolderSkillEnabled } from "@/lib/tauri";
import { useQueryClient } from "@tanstack/react-query";

interface SkillFolderCardProps {
  folder: SkillFolder;
}

export function SkillFolderCard({ folder }: SkillFolderCardProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [viewFile, setViewFile] = useState<{ path: string; content: string } | null>(null);
  const [executeConfirm, setExecuteConfirm] = useState<string | null>(null);
  const [executeResult, setExecuteResult] = useState<{ output?: string; error?: string } | null>(null);
  const queryClient = useQueryClient();

  const handleDelete = async () => {
    if (!confirm(`Delete skill folder "${folder.name}"? This cannot be undone.`)) return;
    try {
      await deleteSkillFolder(folder.name);
      queryClient.invalidateQueries({ queryKey: ["skill-folders"] });
    } catch (err) {
      alert(`Delete failed: ${err instanceof Error ? err.message : String(err)}`);
    }
    setMenuOpen(false);
  };

  const handleViewSkillMd = async () => {
    try {
      const content = await readSkillFile(folder.name, "skill.md");
      setViewFile({ path: "skill.md", content });
    } catch (err) {
      alert(`Failed to read skill.md: ${err instanceof Error ? err.message : String(err)}`);
    }
    setMenuOpen(false);
  };

  const handleExecuteScript = async () => {
    if (!executeConfirm) return;
    try {
      const output = await executeSkillScript(folder.name, executeConfirm);
      setExecuteResult({ output });
    } catch (err) {
      setExecuteResult({ error: err instanceof Error ? err.message : String(err) });
    }
  };

  const handleToggleEnabled = async () => {
    try {
      await toggleFolderSkillEnabled(folder.name, !folder.enabled);
      queryClient.invalidateQueries({ queryKey: ["skill-folders"] });
    } catch (err) {
      alert(`Failed to toggle: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const executableFiles = folder.files.flatMap(function findExecutables(entry): string[] {
    if (entry.is_directory && entry.children) {
      return entry.children.flatMap(findExecutables);
    }
    return entry.is_executable ? [entry.path] : [];
  });

  return (
    <>
      <div className="group relative border border-zinc-200 dark:border-zinc-800 rounded-lg p-4 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <Folder className="w-4 h-4 text-amber-500" />
              <h3 className="font-medium text-zinc-900 dark:text-zinc-100 truncate">
                {folder.name}
              </h3>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400">
                Package
              </span>
              {folder.has_executables && (
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400">
                  Scripts
                </span>
              )}
            </div>

            {folder.description && (
              <p className="text-[13px] text-zinc-500 dark:text-zinc-400 line-clamp-2 mb-2">
                {folder.description}
              </p>
            )}

            <div className="flex items-center gap-3 text-[11px] text-zinc-400">
              <span>{folder.files.length} files</span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {executableFiles.length > 0 && (
              <button
                onClick={() => setExecuteConfirm(executableFiles[0])}
                className="p-1.5 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-500 hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
                title="Run script"
              >
                <Play className="w-4 h-4" />
              </button>
            )}

            <label className="relative inline-flex items-center cursor-pointer gap-1.5" title={folder.enabled ? "Disable skill" : "Enable skill"}>
              <input
                type="checkbox"
                checked={folder.enabled}
                onChange={handleToggleEnabled}
                className="sr-only peer"
              />
              <div className="w-9 h-5 bg-zinc-200 dark:bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-indigo-500" />
              <span className={`text-[11px] ${folder.enabled ? "text-indigo-600 dark:text-indigo-400" : "text-zinc-400"}`}>
                {folder.enabled ? "On" : "Off"}
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
                      onClick={handleViewSkillMd}
                      className="w-full px-3 py-1.5 text-left text-[13px] hover:bg-zinc-100 dark:hover:bg-zinc-800 flex items-center gap-2"
                    >
                      <Eye className="w-3.5 h-3.5" />
                      View skill.md
                    </button>
                    <hr className="my-1 border-zinc-200 dark:border-zinc-700" />
                    <button
                      onClick={handleDelete}
                      className="w-full px-3 py-1.5 text-left text-[13px] text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                      Delete
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* File Viewer Modal */}
      {viewFile && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[600px] max-h-[80vh] flex flex-col border border-zinc-200 dark:border-zinc-700">
            <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
              <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">
                {folder.name}/{viewFile.path}
              </span>
              <button
                onClick={() => setViewFile(null)}
                className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700"
              >
                <X className="w-4 h-4 text-zinc-500" />
              </button>
            </div>
            <pre className="flex-1 overflow-auto p-4 text-[12px] text-zinc-700 dark:text-zinc-300 font-mono whitespace-pre-wrap">
              {viewFile.content}
            </pre>
          </div>
        </div>
      )}

      {/* Execute Confirmation Dialog */}
      {executeConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[480px] border border-zinc-200 dark:border-zinc-700">
            <div className="flex items-center gap-3 px-5 py-4 border-b border-zinc-200 dark:border-zinc-800">
              <div className="w-8 h-8 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
                <AlertTriangle className="w-4 h-4 text-amber-600" />
              </div>
              <div>
                <h3 className="text-[14px] font-semibold text-zinc-900 dark:text-zinc-100">
                  Execute Script?
                </h3>
                <p className="text-[12px] text-zinc-500">
                  This will run code on your machine
                </p>
              </div>
            </div>
            <div className="px-5 py-4">
              <div className="bg-zinc-100 dark:bg-zinc-800 rounded-md px-3 py-2 mb-3">
                <p className="text-[12px] text-zinc-600 dark:text-zinc-400 font-mono">
                  {folder.name}/{executeConfirm}
                </p>
              </div>
              <p className="text-[13px] text-zinc-600 dark:text-zinc-400">
                Only run scripts you trust. This script will execute with your user permissions.
              </p>
              {executeResult && (
                <div className={`mt-3 p-3 rounded-md text-[12px] font-mono whitespace-pre-wrap max-h-48 overflow-auto ${
                  executeResult.error
                    ? "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400"
                    : "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400"
                }`}>
                  {executeResult.error || executeResult.output || "(no output)"}
                </div>
              )}
            </div>
            <div className="flex justify-end gap-2 px-5 py-3 border-t border-zinc-200 dark:border-zinc-800">
              <button
                onClick={() => {
                  setExecuteConfirm(null);
                  setExecuteResult(null);
                }}
                className="px-4 py-2 text-[13px] text-zinc-600 dark:text-zinc-400 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
              >
                {executeResult ? "Close" : "Cancel"}
              </button>
              {!executeResult && (
                <button
                  onClick={handleExecuteScript}
                  className="px-4 py-2 text-[13px] bg-amber-500 hover:bg-amber-600 text-white rounded-lg transition-colors"
                >
                  Run Script
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}
