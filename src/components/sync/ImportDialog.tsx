import { useState } from "react";
import {
  X,
  Upload,
  FolderOpen,
  Check,
  AlertCircle,
  Loader2,
  Lock,
  FileArchive,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import * as api from "@/lib/tauri";

interface ImportDialogProps {
  onClose: () => void;
  onSuccess?: () => void;
}

export function ImportDialog({ onClose, onSuccess }: ImportDialogProps) {
  const [step, setStep] = useState<"select" | "preview" | "conflicts" | "importing" | "complete" | "error">("select");
  const [archivePath, setArchivePath] = useState("");
  const [password, setPassword] = useState("");
  const [mode, setMode] = useState<"Merge" | "Replace">("Merge");
  const [preview, setPreview] = useState<api.ImportPreview | null>(null);
  const [conflictResolutions, setConflictResolutions] = useState<Record<string, string>>({});
  const [expandedTypes, setExpandedTypes] = useState<Record<string, boolean>>({});
  const [result, setResult] = useState<api.ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSelectFile = async () => {
    // In real implementation, would use file dialog
    // For now, just allow manual path entry
  };

  const handlePreview = async () => {
    if (!archivePath) {
      setError("Please select an archive file");
      return;
    }

    setError(null);

    try {
      const importPreview = await api.previewImportData(archivePath, {
        mode,
        password: password || undefined,
        conflict_resolution: "Ask",
        create_backup: true,
      });
      setPreview(importPreview);

      if (importPreview.conflicts.length > 0) {
        setStep("conflicts");
      } else {
        setStep("preview");
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const handleImport = async () => {
    setStep("importing");
    setError(null);

    try {
      const importResult = await api.importAllData(archivePath, {
        mode,
        password: password || undefined,
        conflict_resolution: mode === "Replace" ? "Overwrite" : "Ask",
        create_backup: true,
      }, conflictResolutions);

      setResult(importResult);
      setStep("complete");
      onSuccess?.();
    } catch (e) {
      setError(String(e));
      setStep("error");
    }
  };

  const resolveConflict = (conflictKey: string, resolution: string) => {
    setConflictResolutions((prev) => ({ ...prev, [conflictKey]: resolution }));
  };

  const resolveAllAs = (entityType: string, resolution: string) => {
    const updates: Record<string, string> = {};
    preview?.conflicts
      .filter((c) => c.entity_type === entityType)
      .forEach((c) => {
        updates[`${c.entity_type}:${c.entity_id}`] = resolution;
      });
    setConflictResolutions((prev) => ({ ...prev, ...updates }));
  };

  const toggleType = (type: string) => {
    setExpandedTypes((prev) => ({ ...prev, [type]: !prev[type] }));
  };

  const groupedConflicts = preview?.conflicts.reduce((acc, c) => {
    const type = c.entity_type;
    if (!acc[type]) acc[type] = [];
    acc[type].push(c);
    return acc;
  }, {} as Record<string, api.ImportConflict[]>) || {};

  const allConflictsResolved = preview?.conflicts.every(
    (c) => conflictResolutions[`${c.entity_type}:${c.entity_id}`]
  );

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-xl max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex-shrink-0 flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg">
              <Upload className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              Import Data
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {step === "select" && (
            <div className="space-y-6">
              {/* File Selection */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  Export Archive
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={archivePath}
                    onChange={(e) => setArchivePath(e.target.value)}
                    placeholder="Select .zip file..."
                    className="flex-1 px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                             border border-zinc-200 dark:border-zinc-700 rounded-lg
                             focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                  />
                  <button
                    onClick={handleSelectFile}
                    className="px-3 py-2 text-sm text-zinc-600 dark:text-zinc-400
                             border border-zinc-200 dark:border-zinc-700 rounded-lg
                             hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                  >
                    <FolderOpen className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {/* Password */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  <Lock className="w-4 h-4 inline mr-1" />
                  Password (if encrypted)
                </label>
                <input
                  type="password"
                  placeholder="Enter password..."
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                           border border-zinc-200 dark:border-zinc-700 rounded-lg
                           focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                />
              </div>

              {/* Mode Selection */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  Import Mode
                </label>
                <div className="flex gap-3">
                  <label className="flex-1 flex items-center gap-2 p-3 rounded-lg border cursor-pointer
                                  border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800
                                  has-[:checked]:border-indigo-500 has-[:checked]:bg-indigo-50 dark:has-[:checked]:bg-indigo-900/20">
                    <input
                      type="radio"
                      name="mode"
                      checked={mode === "Merge"}
                      onChange={() => setMode("Merge")}
                      className="text-indigo-600 focus:ring-indigo-500"
                    />
                    <div>
                      <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">Merge</div>
                      <div className="text-xs text-zinc-500">Add new items, keep existing</div>
                    </div>
                  </label>
                  <label className="flex-1 flex items-center gap-2 p-3 rounded-lg border cursor-pointer
                                  border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800
                                  has-[:checked]:border-indigo-500 has-[:checked]:bg-indigo-50 dark:has-[:checked]:bg-indigo-900/20">
                    <input
                      type="radio"
                      name="mode"
                      checked={mode === "Replace"}
                      onChange={() => setMode("Replace")}
                      className="text-indigo-600 focus:ring-indigo-500"
                    />
                    <div>
                      <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">Replace</div>
                      <div className="text-xs text-zinc-500">Overwrite all existing data</div>
                    </div>
                  </label>
                </div>
              </div>

              {error && (
                <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
                  <AlertCircle className="w-4 h-4" />
                  {error}
                </div>
              )}
            </div>
          )}

          {step === "preview" && preview && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
                <FileArchive className="w-4 h-4" />
                <span>Version {preview.manifest.format_version}</span>
                <span>•</span>
                <span>{preview.manifest.created_at.slice(0, 10)}</span>
              </div>

              <div className="bg-zinc-50 dark:bg-zinc-800 rounded-lg p-4">
                <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-2">
                  Items to Import
                </h3>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  {Object.entries(preview.new_items).map(([type, count]) => (
                    <div key={type} className="flex justify-between">
                      <span className="text-zinc-500 capitalize">{type}</span>
                      <span className="text-zinc-900 dark:text-zinc-100">{count}</span>
                    </div>
                  ))}
                </div>
              </div>

              {preview.conflicts.length > 0 && (
                <div className="flex items-center gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg text-sm text-amber-700 dark:text-amber-400">
                  <AlertTriangle className="w-4 h-4" />
                  <span>{preview.conflicts.length} conflicts need resolution</span>
                </div>
              )}
            </div>
          )}

          {step === "conflicts" && preview && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg text-sm text-amber-700 dark:text-amber-400">
                <AlertTriangle className="w-4 h-4" />
                <span>{preview.conflicts.length} items already exist. Choose what to do.</span>
              </div>

              {Object.entries(groupedConflicts).map(([type, conflicts]) => (
                <div key={type} className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
                  <button
                    onClick={() => toggleType(type)}
                    className="w-full flex items-center justify-between px-4 py-2 bg-zinc-50 dark:bg-zinc-800
                             hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors"
                  >
                    <div className="flex items-center gap-2">
                      {expandedTypes[type] ? (
                        <ChevronDown className="w-4 h-4" />
                      ) : (
                        <ChevronRight className="w-4 h-4" />
                      )}
                      <span className="font-medium capitalize">{type}s</span>
                      <span className="text-xs text-zinc-500">({conflicts.length})</span>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          resolveAllAs(type, "skip");
                        }}
                        className="px-2 py-1 text-xs text-zinc-600 hover:bg-zinc-200 dark:hover:bg-zinc-600 rounded"
                      >
                        Skip All
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          resolveAllAs(type, "overwrite");
                        }}
                        className="px-2 py-1 text-xs text-zinc-600 hover:bg-zinc-200 dark:hover:bg-zinc-600 rounded"
                      >
                        Overwrite All
                      </button>
                    </div>
                  </button>

                  {expandedTypes[type] && (
                    <div className="divide-y divide-zinc-200 dark:divide-zinc-700">
                      {conflicts.map((conflict) => {
                        const key = `${conflict.entity_type}:${conflict.entity_id}`;
                        const resolution = conflictResolutions[key];
                        return (
                          <div key={key} className="px-4 py-2 flex items-center justify-between">
                            <div className="flex-1 min-w-0">
                              <div className="text-sm text-zinc-900 dark:text-zinc-100 truncate">
                                {conflict.import_name}
                              </div>
                              <div className="text-xs text-zinc-500">
                                Local: {conflict.local_name}
                              </div>
                            </div>
                            <div className="flex gap-1">
                              <button
                                onClick={() => resolveConflict(key, "skip")}
                                className={`px-2 py-1 text-xs rounded ${
                                  resolution === "skip"
                                    ? "bg-zinc-200 dark:bg-zinc-600 text-zinc-900 dark:text-zinc-100"
                                    : "text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-700"
                                }`}
                              >
                                Skip
                              </button>
                              <button
                                onClick={() => resolveConflict(key, "overwrite")}
                                className={`px-2 py-1 text-xs rounded ${
                                  resolution === "overwrite"
                                    ? "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300"
                                    : "text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-700"
                                }`}
                              >
                                Overwrite
                              </button>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {step === "importing" && (
            <div className="flex flex-col items-center justify-center py-8">
              <Loader2 className="w-8 h-8 text-indigo-600 animate-spin mb-4" />
              <p className="text-sm text-zinc-600 dark:text-zinc-400">Importing data...</p>
            </div>
          )}

          {step === "complete" && result && (
            <div className="text-center py-4">
              <div className="w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                <Check className="w-6 h-6 text-green-600 dark:text-green-400" />
              </div>
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100 mb-2">
                Import Complete
              </h3>
              <div className="text-sm text-zinc-500 space-y-1">
                <p>{result.imported_count} items imported</p>
                {result.skipped_count > 0 && <p>{result.skipped_count} items skipped</p>}
                {result.errors.length > 0 && (
                  <p className="text-amber-600">{result.errors.length} errors</p>
                )}
                {result.backup_path && (
                  <p className="text-xs text-zinc-400 truncate" title={result.backup_path}>
                    Backup saved to {result.backup_path}
                  </p>
                )}
              </div>
            </div>
          )}

          {step === "error" && (
            <div className="text-center py-4">
              <div className="w-12 h-12 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                <AlertCircle className="w-6 h-6 text-red-600 dark:text-red-400" />
              </div>
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100 mb-2">
                Import Failed
              </h3>
              <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex-shrink-0 flex justify-end gap-3 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400
                     hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            {step === "complete" || step === "error" ? "Close" : "Cancel"}
          </button>

          {step === "select" && (
            <button
              onClick={handlePreview}
              disabled={!archivePath}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed
                       rounded-lg transition-colors"
            >
              Continue
            </button>
          )}

          {step === "preview" && (
            <button
              onClick={handleImport}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 rounded-lg transition-colors"
            >
              Import
            </button>
          )}

          {step === "conflicts" && (
            <button
              onClick={handleImport}
              disabled={!allConflictsResolved}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed
                       rounded-lg transition-colors"
            >
              Import ({Object.keys(conflictResolutions).length}/{preview?.conflicts.length} resolved)
            </button>
          )}

          {step === "error" && (
            <button
              onClick={() => setStep("select")}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 rounded-lg transition-colors"
            >
              Try Again
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
