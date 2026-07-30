import { useState, useEffect } from "react";
import {
  X,
  Download,
  FolderOpen,
  Check,
  AlertCircle,
  Loader2,
  Lock,
  FileArchive,
} from "lucide-react";
import * as api from "@/lib/tauri";

interface ExportDialogProps {
  onClose: () => void;
}

interface ExportOptions {
  include_projects: boolean;
  include_tasks: boolean;
  include_meetings: boolean;
  include_skills: boolean;
  include_patterns: boolean;
  include_team: boolean;
  include_documents: boolean;
  include_vectors: boolean;
  project_ids?: string[];
  password?: string;
  description?: string;
}

export function ExportDialog({ onClose }: ExportDialogProps) {
  const [step, setStep] = useState<"options" | "exporting" | "complete" | "error">("options");
  const [options, setOptions] = useState<ExportOptions>({
    include_projects: true,
    include_tasks: true,
    include_meetings: true,
    include_skills: false, // not implemented yet — see disabled checkbox below
    include_patterns: true,
    include_team: true,
    include_documents: false, // not implemented yet — see disabled checkbox below
    include_vectors: false, // off by default: requires Qdrant to be running
  });
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [description, setDescription] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [result, setResult] = useState<api.ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<api.SyncProgress | null>(null);

  useEffect(() => {
    if (step !== "exporting") return;
    const unlisten = api.onExportProgress(setProgress);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [step]);

  const handleSelectPath = async () => {
    try {
      const timestamp = new Date().toISOString().slice(0, 10);
      const defaultName = `meridian-export-${timestamp}.zip`;
      const selected = await api.pickExportSavePath(defaultName);
      if (selected) {
        setOutputPath(selected);
      }
    } catch (e) {
      console.error("Failed to select path:", e);
      setError("Failed to open save dialog");
    }
  };

  const handleExport = async () => {
    if (!outputPath) {
      setError("Please select an export location");
      return;
    }

    if (password && password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    setStep("exporting");
    setError(null);
    setProgress(null);

    try {
      const exportOptions: ExportOptions = {
        ...options,
        password: password || undefined,
        description: description || undefined,
      };

      const exportResult = await api.exportAllData(outputPath, exportOptions);
      setResult(exportResult);
      setStep("complete");
    } catch (e) {
      setError(String(e));
      setStep("error");
    }
  };

  const toggleOption = (key: keyof ExportOptions) => {
    setOptions((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg">
              <Download className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              Export Data
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
        <div className="p-6">
          {step === "options" && (
            <div className="space-y-6">
              {/* Content Selection */}
              <div>
                <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-3">
                  What to export
                </h3>
                <div className="grid grid-cols-2 gap-2">
                  {[
                    { key: "include_projects", label: "Projects" },
                    { key: "include_tasks", label: "Tasks" },
                    { key: "include_meetings", label: "Meetings" },
                    { key: "include_team", label: "Team Members" },
                    { key: "include_patterns", label: "Learning Patterns", hint: "anonymized" },
                    { key: "include_vectors", label: "Vector Embeddings", hint: "requires Qdrant running" },
                  ].map(({ key, label, hint }) => (
                    <label
                      key={key}
                      className="flex items-center gap-2 p-2 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-800 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={options[key as keyof ExportOptions] as boolean}
                        onChange={() => toggleOption(key as keyof ExportOptions)}
                        className="rounded border-zinc-300 text-indigo-600 focus:ring-indigo-500"
                      />
                      <span className="text-sm text-zinc-700 dark:text-zinc-300">
                        {label}
                        {hint && <span className="text-zinc-400"> ({hint})</span>}
                      </span>
                    </label>
                  ))}
                  {[
                    { key: "include_skills", label: "Skills" },
                    { key: "include_documents", label: "Document metadata" },
                    { key: "include_audit", label: "Audit log" },
                  ].map(({ key, label }) => (
                    <label
                      key={key}
                      title="Not implemented yet — this content is never included in the export"
                      className="flex items-center gap-2 p-2 rounded-lg opacity-50 cursor-not-allowed"
                    >
                      <input type="checkbox" checked={false} disabled className="rounded border-zinc-300" />
                      <span className="text-sm text-zinc-500">
                        {label} <span className="text-zinc-400">(coming soon)</span>
                      </span>
                    </label>
                  ))}
                </div>
              </div>

              {/* Password */}
              <div>
                <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-3">
                  <Lock className="w-4 h-4 inline mr-1" />
                  Password Protection (optional)
                </h3>
                <div className="space-y-2">
                  <input
                    type="password"
                    placeholder="Password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                             border border-zinc-200 dark:border-zinc-700 rounded-lg
                             focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                  />
                  {password && (
                    <input
                      type="password"
                      placeholder="Confirm password"
                      value={confirmPassword}
                      onChange={(e) => setConfirmPassword(e.target.value)}
                      className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                               border border-zinc-200 dark:border-zinc-700 rounded-lg
                               focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                    />
                  )}
                </div>
              </div>

              {/* Description */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  Description (optional)
                </label>
                <input
                  type="text"
                  placeholder="e.g., Monthly backup"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                           border border-zinc-200 dark:border-zinc-700 rounded-lg
                           focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                />
              </div>

              {/* Output Location */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  Save to
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={outputPath}
                    onChange={(e) => setOutputPath(e.target.value)}
                    placeholder="Select location..."
                    className="flex-1 px-3 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                             border border-zinc-200 dark:border-zinc-700 rounded-lg
                             focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                  />
                  <button
                    onClick={handleSelectPath}
                    className="px-3 py-2 text-sm text-zinc-600 dark:text-zinc-400
                             border border-zinc-200 dark:border-zinc-700 rounded-lg
                             hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                  >
                    <FolderOpen className="w-4 h-4" />
                  </button>
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

          {step === "exporting" && (
            <div className="flex flex-col items-center justify-center py-8">
              <Loader2 className="w-8 h-8 text-indigo-600 animate-spin mb-4" />
              <p className="text-sm text-zinc-600 dark:text-zinc-400 mb-3">
                {progress ? progress.step : "Exporting data..."}
              </p>
              <div className="w-full max-w-xs h-1.5 bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-indigo-600 transition-all duration-300"
                  style={{
                    width: progress ? `${Math.round((progress.current / progress.total) * 100)}%` : "5%",
                  }}
                />
              </div>
              {progress && (
                <p className="text-xs text-zinc-400 mt-2">
                  Step {progress.current} of {progress.total}
                </p>
              )}
            </div>
          )}

          {step === "complete" && result && (
            <div className="text-center py-4">
              <div className="w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                <Check className="w-6 h-6 text-green-600 dark:text-green-400" />
              </div>
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100 mb-2">
                Export Complete
              </h3>
              <div className="text-sm text-zinc-500 space-y-1 mb-4">
                <div className="flex items-center justify-center gap-2">
                  <FileArchive className="w-4 h-4" />
                  <span>{formatFileSize(result.file_size)}</span>
                </div>
                <p className="truncate max-w-xs mx-auto">{result.file_path}</p>
              </div>
              <div className="text-xs text-zinc-400">
                Includes: {result.manifest.contents.project_count} projects,{" "}
                {result.manifest.contents.task_count} tasks,{" "}
                {result.manifest.contents.meeting_count} meetings,{" "}
                {result.manifest.contents.team_member_count} team members
                {result.manifest.contents.patterns && `, ${result.manifest.contents.pattern_count} pattern contributions`}
                {result.manifest.contents.vectors && ", vector embeddings"}
              </div>
            </div>
          )}

          {step === "error" && (
            <div className="text-center py-4">
              <div className="w-12 h-12 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                <AlertCircle className="w-6 h-6 text-red-600 dark:text-red-400" />
              </div>
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100 mb-2">
                Export Failed
              </h3>
              <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400
                     hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            {step === "complete" || step === "error" ? "Close" : "Cancel"}
          </button>
          {step === "options" && (
            <button
              onClick={handleExport}
              disabled={!outputPath}
              className="px-4 py-2 text-sm font-medium text-white bg-indigo-600
                       hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed
                       rounded-lg transition-colors"
            >
              Export
            </button>
          )}
          {step === "error" && (
            <button
              onClick={() => setStep("options")}
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
