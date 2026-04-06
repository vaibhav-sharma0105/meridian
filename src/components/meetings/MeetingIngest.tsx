import { useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { X, Loader, CheckCircle, Upload, FileText, AlertCircle, Trash2 } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import { ingestMeeting, ingestMeetingFromFile } from "@/lib/tauri";
import { useMeetings } from "@/hooks/useMeetings";
import { PLATFORMS } from "@/lib/constants";

type Tab = "paste" | "file";

type FileStatus = "pending" | "processing" | "done" | "error";

interface SelectedFile {
  path: string;
  name: string;
  status: FileStatus;
  error?: string;
}

function extractFileName(filePath: string): string {
  const parts = filePath.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] ?? filePath;
}

export default function MeetingIngest() {
  const { t } = useTranslation();
  const { ingestModalOpen, setIngestModalOpen } = useUIStore();
  const { activeProjectId } = useProjectStore();
  const { refetch } = useMeetings(activeProjectId);

  const [tab, setTab] = useState<Tab>("paste");

  // Shared fields
  const [title, setTitle] = useState("");
  const [platform, setPlatform] = useState("zoom");

  // Paste-tab fields
  const [transcript, setTranscript] = useState("");
  const [attendees, setAttendees] = useState("");
  const [duration, setDuration] = useState("");

  // File-tab fields
  const [selectedFiles, setSelectedFiles] = useState<SelectedFile[]>([]);

  const [processing, setProcessing] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState("");

  // Track batch progress for display
  const [batchIndex, setBatchIndex] = useState(0);

  // Track batch results for summary
  const [batchSummary, setBatchSummary] = useState<{ total: number; succeeded: number } | null>(null);

  // Ref to allow cancellation-awareness (not true cancellation, just stop-after-current)
  const cancelledRef = useRef(false);

  if (!ingestModalOpen) return null;

  const handleClose = () => {
    setIngestModalOpen(false);
    setTab("paste");
    setTitle("");
    setTranscript("");
    setAttendees("");
    setDuration("");
    setSelectedFiles([]);
    setDone(false);
    setError("");
    setBatchIndex(0);
    setBatchSummary(null);
    cancelledRef.current = false;
  };

  const handlePickFile = async () => {
    const result = await open({
      multiple: true,
      filters: [
        {
          name: "Transcript files",
          extensions: ["txt", "vtt", "srt", "md", "docx"],
        },
      ],
    });
    if (result) {
      // open() with multiple: true returns string[] | null
      const paths = Array.isArray(result) ? result : [result];
      // Deduplicate against already-selected files
      const existingPaths = new Set(selectedFiles.map((f) => f.path));
      const newFiles: SelectedFile[] = paths
        .filter((p) => !existingPaths.has(p))
        .map((p) => ({
          path: p,
          name: extractFileName(p),
          status: "pending" as FileStatus,
        }));
      if (newFiles.length > 0) {
        setSelectedFiles((prev) => [...prev, ...newFiles]);
        setError("");
      }
    }
  };

  const handleRemoveFile = (path: string) => {
    setSelectedFiles((prev) => prev.filter((f) => f.path !== path));
  };

  const handleIngestPaste = async () => {
    if (!transcript.trim() || !activeProjectId) return;
    setProcessing(true);
    setError("");
    try {
      await ingestMeeting({
        projectId: activeProjectId,
        platform,
        rawTranscript: transcript,
        title: title || "Untitled Meeting",
        attendees: attendees || undefined,
        durationMinutes: duration ? parseInt(duration) : undefined,
      });
      await refetch();
      setDone(true);
      setTimeout(handleClose, 1500);
    } catch (e) {
      setError(String(e));
    } finally {
      setProcessing(false);
    }
  };

  const handleIngestFile = async () => {
    if (selectedFiles.length === 0 || !activeProjectId) return;
    setProcessing(true);
    setError("");
    cancelledRef.current = false;

    let succeeded = 0;
    const total = selectedFiles.length;

    // Reset all files to pending before starting
    setSelectedFiles((prev) =>
      prev.map((f) => ({ ...f, status: "pending" as FileStatus, error: undefined }))
    );

    for (let i = 0; i < selectedFiles.length; i++) {
      if (cancelledRef.current) break;

      setBatchIndex(i);
      const file = selectedFiles[i];

      // Mark current file as processing
      setSelectedFiles((prev) =>
        prev.map((f, idx) =>
          idx === i ? { ...f, status: "processing" as FileStatus } : f
        )
      );

      // Build the title for this file
      let fileTitle: string | undefined;
      if (title.trim()) {
        // Use shared title as prefix: "Title — filename"
        if (selectedFiles.length > 1) {
          fileTitle = `${title.trim()} \u2014 ${file.name}`;
        } else {
          fileTitle = title.trim();
        }
      }

      try {
        await ingestMeetingFromFile({
          projectId: activeProjectId,
          filePath: file.path,
          title: fileTitle,
          platform,
        });

        // Mark as done
        setSelectedFiles((prev) =>
          prev.map((f, idx) =>
            idx === i ? { ...f, status: "done" as FileStatus } : f
          )
        );
        succeeded++;
      } catch (e) {
        // Mark as error but continue with remaining files
        setSelectedFiles((prev) =>
          prev.map((f, idx) =>
            idx === i
              ? { ...f, status: "error" as FileStatus, error: String(e) }
              : f
          )
        );
      }
    }

    await refetch();
    setProcessing(false);
    setBatchSummary({ total, succeeded });
    setDone(true);

    // Auto-close only if all files succeeded
    if (succeeded === total) {
      setTimeout(handleClose, 1500);
    }
  };

  const canSubmitPaste = !processing && !!transcript.trim();
  const canSubmitFile = !processing && selectedFiles.length > 0;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={handleClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-full max-w-2xl mx-4 border border-zinc-200 dark:border-zinc-700 flex flex-col max-h-[90vh]">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-100 dark:border-zinc-800">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("meetings.ingest")}</h2>
          <button onClick={handleClose} className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded">
            <X className="w-4 h-4 text-zinc-500" />
          </button>
        </div>

        {done ? (
          <div className="flex flex-col items-center justify-center py-16 gap-4">
            {batchSummary && batchSummary.succeeded < batchSummary.total ? (
              <>
                <AlertCircle className="w-12 h-12 text-amber-500" />
                <p className="text-zinc-700 dark:text-zinc-300 font-medium">
                  {batchSummary.succeeded} of {batchSummary.total} files processed successfully
                </p>
                {/* Show per-file errors */}
                <div className="w-full max-w-md space-y-1 px-4">
                  {selectedFiles
                    .filter((f) => f.status === "error")
                    .map((f) => (
                      <div key={f.path} className="flex items-start gap-2 text-sm text-red-600">
                        <AlertCircle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
                        <span className="truncate">{f.name}: {f.error}</span>
                      </div>
                    ))}
                </div>
                <button
                  onClick={handleClose}
                  className="mt-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg text-sm font-medium"
                >
                  Close
                </button>
              </>
            ) : (
              <>
                <CheckCircle className="w-12 h-12 text-green-500" />
                <p className="text-zinc-700 dark:text-zinc-300 font-medium">
                  {batchSummary && batchSummary.total > 1
                    ? `${batchSummary.total} files processed successfully`
                    : t("meetings.ingestSuccess")}
                </p>
              </>
            )}
          </div>
        ) : (
          <>
            {/* Tab bar */}
            <div className="flex border-b border-zinc-100 dark:border-zinc-800 px-6">
              <button
                onClick={() => setTab("paste")}
                className={`py-3 px-4 text-sm font-medium border-b-2 -mb-px transition-colors ${
                  tab === "paste"
                    ? "border-indigo-500 text-indigo-600 dark:text-indigo-400"
                    : "border-transparent text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                }`}
              >
                Paste text
              </button>
              <button
                onClick={() => setTab("file")}
                className={`py-3 px-4 text-sm font-medium border-b-2 -mb-px transition-colors ${
                  tab === "file"
                    ? "border-indigo-500 text-indigo-600 dark:text-indigo-400"
                    : "border-transparent text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                }`}
              >
                Upload file
              </button>
            </div>

            <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
              {tab === "paste" ? (
                <>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.title")}</label>
                      <input
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder={t("meetings.titlePlaceholder")}
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.platform")}</label>
                      <select
                        value={platform}
                        onChange={(e) => setPlatform(e.target.value)}
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      >
                        {PLATFORMS.map((p) => (
                          <option key={p.value} value={p.value}>{p.label}</option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.attendees")}</label>
                      <input
                        type="text"
                        value={attendees}
                        onChange={(e) => setAttendees(e.target.value)}
                        placeholder="Alice, Bob, Carol"
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.duration")}</label>
                      <input
                        type="number"
                        value={duration}
                        onChange={(e) => setDuration(e.target.value)}
                        placeholder="60"
                        min="1"
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      />
                    </div>
                  </div>

                  <div>
                    <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                      {t("meetings.transcript")} *
                    </label>
                    <textarea
                      value={transcript}
                      onChange={(e) => setTranscript(e.target.value)}
                      placeholder={t("meetings.transcriptPlaceholder")}
                      rows={12}
                      className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 resize-none font-mono text-xs"
                    />
                    <p className="text-xs text-zinc-400 mt-1">{t("meetings.transcriptHint")}</p>
                  </div>
                </>
              ) : (
                <>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                        {t("meetings.title")} <span className="text-zinc-400">
                          {selectedFiles.length > 1 ? "(prefix, optional)" : "(optional)"}
                        </span>
                      </label>
                      <input
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder={selectedFiles.length > 1 ? "Used as prefix for each meeting" : "Derived from filename if blank"}
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      />
                    </div>
                    <div>
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.platform")}</label>
                      <select
                        value={platform}
                        onChange={(e) => setPlatform(e.target.value)}
                        className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                      >
                        {PLATFORMS.map((p) => (
                          <option key={p.value} value={p.value}>{p.label}</option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div>
                    <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                      Transcript files *
                    </label>
                    <button
                      onClick={handlePickFile}
                      disabled={processing}
                      className="flex items-center gap-2 w-full px-4 py-3 rounded-lg border-2 border-dashed border-zinc-300 dark:border-zinc-600 hover:border-indigo-400 dark:hover:border-indigo-500 text-zinc-500 dark:text-zinc-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors text-sm disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <Upload className="w-4 h-4 flex-shrink-0" />
                      <span>
                        {selectedFiles.length === 0
                          ? "Click to choose files\u2026"
                          : "Add more files\u2026"}
                      </span>
                    </button>
                    <p className="text-xs text-zinc-400 mt-1">Supported formats: TXT, VTT, SRT, MD, DOCX. You can select multiple files.</p>
                  </div>

                  {/* File list with status */}
                  {selectedFiles.length > 0 && (
                    <div className="space-y-1.5">
                      {processing && (
                        <p className="text-xs font-medium text-indigo-600 dark:text-indigo-400 mb-1">
                          Processing {batchIndex + 1} of {selectedFiles.length} files...
                        </p>
                      )}
                      {selectedFiles.map((file) => (
                        <div
                          key={file.path}
                          className="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-50 dark:bg-zinc-800/50 border border-zinc-100 dark:border-zinc-700/50"
                        >
                          {/* Status icon */}
                          {file.status === "pending" && (
                            <FileText className="w-4 h-4 text-zinc-400 flex-shrink-0" />
                          )}
                          {file.status === "processing" && (
                            <Loader className="w-4 h-4 text-indigo-500 animate-spin flex-shrink-0" />
                          )}
                          {file.status === "done" && (
                            <CheckCircle className="w-4 h-4 text-green-500 flex-shrink-0" />
                          )}
                          {file.status === "error" && (
                            <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                          )}

                          {/* Filename */}
                          <span
                            className={`text-sm truncate flex-1 ${
                              file.status === "done"
                                ? "text-green-700 dark:text-green-400"
                                : file.status === "error"
                                ? "text-red-700 dark:text-red-400"
                                : file.status === "processing"
                                ? "text-indigo-700 dark:text-indigo-300 font-medium"
                                : "text-zinc-700 dark:text-zinc-300"
                            }`}
                            title={file.path}
                          >
                            {file.name}
                          </span>

                          {/* Error message */}
                          {file.status === "error" && file.error && (
                            <span className="text-xs text-red-500 truncate max-w-[200px]" title={file.error}>
                              {file.error}
                            </span>
                          )}

                          {/* Remove button (only when not processing) */}
                          {!processing && (
                            <button
                              onClick={() => handleRemoveFile(file.path)}
                              className="p-0.5 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded flex-shrink-0"
                              title="Remove file"
                            >
                              <Trash2 className="w-3.5 h-3.5 text-zinc-400 hover:text-red-500" />
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </>
              )}

              {error && <p className="text-sm text-red-600">{error}</p>}
            </div>

            <div className="flex justify-end gap-3 px-6 py-4 border-t border-zinc-100 dark:border-zinc-800">
              <button
                onClick={handleClose}
                className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={tab === "paste" ? handleIngestPaste : handleIngestFile}
                disabled={tab === "paste" ? !canSubmitPaste : !canSubmitFile}
                className="flex items-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
              >
                {processing ? <Loader className="w-4 h-4 animate-spin" /> : <Upload className="w-4 h-4" />}
                {processing
                  ? (tab === "file" && selectedFiles.length > 1
                      ? `Processing ${batchIndex + 1} of ${selectedFiles.length}...`
                      : t("meetings.processing"))
                  : (tab === "file" && selectedFiles.length > 1
                      ? `Analyze ${selectedFiles.length} files`
                      : t("meetings.analyzeAndIngest"))}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
