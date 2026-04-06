import { useState } from "react";
import { ExternalLink, Download, FileText, X, Loader2 } from "lucide-react";
import { format, isValid } from "date-fns";
import type { PendingImport, Project } from "@/lib/tauri";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  import: PendingImport;
  projects: Project[];
  onApprove: (
    pendingImportId: string,
    projectId: string,
    importType: "summary" | "transcript"
  ) => Promise<void>;
  onDismiss: (pendingImportId: string) => Promise<void>;
}

export default function PendingImportCard({
  import: imp,
  projects,
  onApprove,
  onDismiss,
}: Props) {
  const [selectedProjectId, setSelectedProjectId] = useState("");
  const [importingType, setImportingType] = useState<
    "summary" | "transcript" | null
  >(null);
  const [isDismissing, setIsDismissing] = useState(false);

  const handleApprove = async (type: "summary" | "transcript") => {
    if (!selectedProjectId) return;
    setImportingType(type);
    try {
      await onApprove(imp.id, selectedProjectId, type);
    } finally {
      setImportingType(null);
    }
  };

  const handleDismiss = async () => {
    setIsDismissing(true);
    try {
      await onDismiss(imp.id);
    } finally {
      setIsDismissing(false);
    }
  };

  const handleAudit = () => {
    if (imp.zoom_join_url) {
      invoke("open_url", { url: imp.zoom_join_url }).catch(console.error);
    }
  };

  const canImport = selectedProjectId !== "";
  const isLoading = importingType !== null || isDismissing;

  return (
    <div className="px-4 py-3 border-b border-zinc-100 dark:border-zinc-800 last:border-0">
      {/* Header */}
      <div className="flex items-start gap-2 mb-2">
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50 truncate">
            {imp.title}
          </p>
          <p className="text-xs text-zinc-400 mt-0.5">
            via {imp.provider === "zoom" ? "Zoom" : "Gmail"}
            {imp.meeting_date && (() => { const d = new Date(imp.meeting_date); return isValid(d) ? ` · ${format(d, "MMM d, h:mm a")}` : null; })()}
            {imp.duration_minutes && ` · ${imp.duration_minutes} min`}
          </p>
        </div>
        <button
          onClick={handleDismiss}
          disabled={isLoading}
          className="flex-shrink-0 p-1 text-zinc-300 hover:text-zinc-500 dark:hover:text-zinc-400 disabled:opacity-50 transition-colors"
          title="Dismiss"
        >
          {isDismissing ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <X className="w-3 h-3" />
          )}
        </button>
      </div>

      {/* Summary preview */}
      {imp.summary_preview && (
        <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-3 line-clamp-2 italic">
          "{imp.summary_preview}"
        </p>
      )}

      {/* Actions */}
      <div className="space-y-2">
        <select
          value={selectedProjectId}
          onChange={(e) => setSelectedProjectId(e.target.value)}
          disabled={isLoading}
          className="w-full text-xs rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-indigo-500 disabled:opacity-50"
        >
          <option value="">Select project…</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>

        <div className="flex items-center gap-1.5 flex-wrap">
          {imp.zoom_join_url && (
            <button
              onClick={handleAudit}
              title="Open meeting link in Zoom (join URL)"
              className="flex items-center gap-1 px-2 py-1 text-xs rounded-md text-zinc-500 hover:text-zinc-700 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
            >
              <ExternalLink className="w-3 h-3" />
              Open in Zoom
            </button>
          )}
          <button
            onClick={() => handleApprove("summary")}
            disabled={!canImport || isLoading}
            className="flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            {importingType === "summary" ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Download className="w-3 h-3" />
            )}
            Import Summary
          </button>
          <button
            onClick={() => handleApprove("transcript")}
            disabled={!canImport || !imp.transcript_available || isLoading}
            title={
              !imp.transcript_available ? "No transcript available" : undefined
            }
            className="flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-md bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            {importingType === "transcript" ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <FileText className="w-3 h-3" />
            )}
            Import Transcript
          </button>
        </div>
      </div>
    </div>
  );
}
