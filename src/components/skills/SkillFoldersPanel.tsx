import { useState } from "react";
import {
  FolderOpen,
  Upload,
  Trash2,
  ChevronRight,
  ChevronDown,
  File,
  Folder,
  Play,
  AlertTriangle,
  X,
} from "lucide-react";
import type { SkillFolder, SkillFileEntry } from "@/lib/tauri";
import {
  pickFolderDialog,
  listSkillFolders,
  installSkillFolder,
  deleteSkillFolder,
  readSkillFile,
  executeSkillScript,
} from "@/lib/tauri";
import { useQuery, useQueryClient } from "@tanstack/react-query";

function FileTreeNode({
  entry,
  folderName,
  depth = 0,
  onExecute,
  onViewFile,
}: {
  entry: SkillFileEntry;
  folderName: string;
  depth?: number;
  onExecute: (path: string) => void;
  onViewFile: (path: string) => void;
}) {
  const [expanded, setExpanded] = useState(depth < 1);

  if (entry.is_directory) {
    return (
      <div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="w-full flex items-center gap-1.5 px-2 py-1 text-[13px] text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
        >
          {expanded ? (
            <ChevronDown className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
          )}
          <Folder className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
          <span className="truncate">{entry.name}</span>
        </button>
        {expanded && entry.children && (
          <div>
            {entry.children.map((child) => (
              <FileTreeNode
                key={child.path}
                entry={child}
                folderName={folderName}
                depth={depth + 1}
                onExecute={onExecute}
                onViewFile={onViewFile}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      className="flex items-center gap-1.5 px-2 py-1 group hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
      style={{ paddingLeft: `${depth * 16 + 8}px` }}
    >
      <File
        className={`w-3.5 h-3.5 flex-shrink-0 ${
          entry.is_executable
            ? "text-amber-500"
            : "text-zinc-400"
        }`}
      />
      <button
        onClick={() => onViewFile(entry.path)}
        className="flex-1 text-left text-[13px] text-zinc-700 dark:text-zinc-300 truncate hover:text-indigo-600 dark:hover:text-indigo-400"
      >
        {entry.name}
      </button>
      {entry.is_executable && (
        <button
          onClick={() => onExecute(entry.path)}
          className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-500 hover:text-indigo-600 transition-all"
          title="Execute script"
        >
          <Play className="w-3 h-3" />
        </button>
      )}
      <span className="text-[10px] text-zinc-400 tabular-nums">
        {formatSize(entry.size)}
      </span>
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}

export function SkillFoldersPanel() {
  const queryClient = useQueryClient();
  const [expandedFolder, setExpandedFolder] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<{ path: string; content: string } | null>(null);
  const [executeConfirm, setExecuteConfirm] = useState<{ folder: string; path: string } | null>(null);
  const [executeResult, setExecuteResult] = useState<{ output?: string; error?: string } | null>(null);
  const [installing, setInstalling] = useState(false);

  const { data: folders = [], isLoading } = useQuery({
    queryKey: ["skill-folders"],
    queryFn: listSkillFolders,
  });

  const handleUpload = async () => {
    const path = await pickFolderDialog();
    if (!path) return;

    setInstalling(true);
    try {
      await installSkillFolder(path);
      queryClient.invalidateQueries({ queryKey: ["skill-folders"] });
    } catch (err) {
      alert(`Install failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setInstalling(false);
    }
  };

  const handleDelete = async (folderName: string) => {
    if (!confirm(`Delete skill folder "${folderName}"? This cannot be undone.`)) return;
    try {
      await deleteSkillFolder(folderName);
      queryClient.invalidateQueries({ queryKey: ["skill-folders"] });
      if (expandedFolder === folderName) setExpandedFolder(null);
    } catch (err) {
      alert(`Delete failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleViewFile = async (folderName: string, filePath: string) => {
    try {
      const content = await readSkillFile(folderName, filePath);
      setFileContent({ path: filePath, content });
    } catch (err) {
      alert(`Failed to read file: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const handleExecuteRequest = (folderName: string, scriptPath: string) => {
    setExecuteConfirm({ folder: folderName, path: scriptPath });
    setExecuteResult(null);
  };

  const handleExecuteConfirm = async () => {
    if (!executeConfirm) return;
    try {
      const output = await executeSkillScript(executeConfirm.folder, executeConfirm.path);
      setExecuteResult({ output });
    } catch (err) {
      setExecuteResult({ error: err instanceof Error ? err.message : String(err) });
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center gap-2">
          <FolderOpen className="w-4 h-4 text-zinc-500" />
          <h3 className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">
            Installed Packages
          </h3>
          <span className="text-[11px] text-zinc-400">
            {folders.length}
          </span>
        </div>
        <button
          onClick={handleUpload}
          disabled={installing}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-[12px] border border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors disabled:opacity-50"
        >
          <Upload className="w-3.5 h-3.5" />
          {installing ? "Installing..." : "Add Folder"}
        </button>
      </div>

      <div className="flex-1 overflow-y-auto relative">
        {installing && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-white/80 dark:bg-zinc-900/80 backdrop-blur-sm">
            <div className="flex items-center gap-2 px-4 py-2 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg">
              <div className="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
              <span className="text-[13px] text-zinc-600 dark:text-zinc-300">Installing package...</span>
            </div>
          </div>
        )}
        {isLoading ? (
          <div className="flex items-center justify-center py-8 text-[13px] text-zinc-400">
            Loading...
          </div>
        ) : folders.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
            <FolderOpen className="w-8 h-8 text-zinc-300 dark:text-zinc-600 mb-3" />
            <p className="text-[13px] text-zinc-500 mb-1">No skill packages installed</p>
            <p className="text-[12px] text-zinc-400">
              Upload a folder containing scripts, configs, or skill definitions
            </p>
          </div>
        ) : (
          <div className="p-2 space-y-1">
            {folders.map((folder) => (
              <div
                key={folder.name}
                className="border border-zinc-200 dark:border-zinc-800 rounded-lg overflow-hidden"
              >
                <div className="flex items-center gap-2 px-3 py-2 bg-zinc-50 dark:bg-zinc-800/50">
                  <button
                    onClick={() =>
                      setExpandedFolder(expandedFolder === folder.name ? null : folder.name)
                    }
                    className="flex items-center gap-2 flex-1 min-w-0"
                  >
                    {expandedFolder === folder.name ? (
                      <ChevronDown className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
                    )}
                    <Folder className="w-4 h-4 text-zinc-500 flex-shrink-0" />
                    <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300 truncate">
                      {folder.name}
                    </span>
                    {folder.has_executables && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 flex-shrink-0">
                        Executable
                      </span>
                    )}
                  </button>
                  <button
                    onClick={() => handleDelete(folder.name)}
                    className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-400 hover:text-red-500 transition-colors"
                    title="Delete folder"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>

                {expandedFolder === folder.name && (
                  <div className="border-t border-zinc-200 dark:border-zinc-800 py-1">
                    {folder.description && (
                      <p className="px-3 py-1 text-[12px] text-zinc-500 italic">
                        {folder.description}
                      </p>
                    )}
                    {folder.files.map((entry) => (
                      <FileTreeNode
                        key={entry.path}
                        entry={entry}
                        folderName={folder.name}
                        onExecute={(path) => handleExecuteRequest(folder.name, path)}
                        onViewFile={(path) => handleViewFile(folder.name, path)}
                      />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* File Viewer Modal */}
      {fileContent && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[600px] max-h-[80vh] flex flex-col border border-zinc-200 dark:border-zinc-700">
            <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
              <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300 truncate">
                {fileContent.path}
              </span>
              <button
                onClick={() => setFileContent(null)}
                className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700"
              >
                <X className="w-4 h-4 text-zinc-500" />
              </button>
            </div>
            <pre className="flex-1 overflow-auto p-4 text-[12px] text-zinc-700 dark:text-zinc-300 font-mono whitespace-pre-wrap">
              {fileContent.content}
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
                  {executeConfirm.folder}/{executeConfirm.path}
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
                  onClick={handleExecuteConfirm}
                  className="px-4 py-2 text-[13px] bg-amber-500 hover:bg-amber-600 text-white rounded-lg transition-colors"
                >
                  Run Script
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
