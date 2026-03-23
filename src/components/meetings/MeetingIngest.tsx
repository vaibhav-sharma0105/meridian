import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Loader, CheckCircle, Upload, FileText } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import { ingestMeeting, ingestMeetingFromFile } from "@/lib/tauri";
import { useMeetings } from "@/hooks/useMeetings";
import { PLATFORMS } from "@/lib/constants";

type Tab = "paste" | "file";

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
  const [selectedFilePath, setSelectedFilePath] = useState<string | null>(null);
  const [selectedFileName, setSelectedFileName] = useState<string | null>(null);

  const [processing, setProcessing] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState("");

  if (!ingestModalOpen) return null;

  const handleClose = () => {
    setIngestModalOpen(false);
    setTab("paste");
    setTitle("");
    setTranscript("");
    setAttendees("");
    setDuration("");
    setSelectedFilePath(null);
    setSelectedFileName(null);
    setDone(false);
    setError("");
  };

  const handlePickFile = async () => {
    const result = await open({
      multiple: false,
      filters: [
        {
          name: "Transcript files",
          extensions: ["txt", "vtt", "srt", "md", "docx"],
        },
      ],
    });
    if (result && typeof result === "string") {
      setSelectedFilePath(result);
      // Extract just the filename for display
      const parts = result.replace(/\\/g, "/").split("/");
      setSelectedFileName(parts[parts.length - 1] ?? result);
      setError("");
    }
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
    if (!selectedFilePath || !activeProjectId) return;
    setProcessing(true);
    setError("");
    try {
      await ingestMeetingFromFile({
        projectId: activeProjectId,
        filePath: selectedFilePath,
        title: title || undefined,
        platform,
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

  const canSubmitPaste = !processing && !!transcript.trim();
  const canSubmitFile = !processing && !!selectedFilePath;

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
            <CheckCircle className="w-12 h-12 text-green-500" />
            <p className="text-zinc-700 dark:text-zinc-300 font-medium">{t("meetings.ingestSuccess")}</p>
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
                      <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">{t("meetings.title")} <span className="text-zinc-400">(optional)</span></label>
                      <input
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder="Derived from filename if blank"
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
                      Transcript file *
                    </label>
                    <button
                      onClick={handlePickFile}
                      className="flex items-center gap-2 w-full px-4 py-3 rounded-lg border-2 border-dashed border-zinc-300 dark:border-zinc-600 hover:border-indigo-400 dark:hover:border-indigo-500 text-zinc-500 dark:text-zinc-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors text-sm"
                    >
                      <FileText className="w-4 h-4 flex-shrink-0" />
                      {selectedFileName ? (
                        <span className="truncate text-zinc-800 dark:text-zinc-200 font-medium">{selectedFileName}</span>
                      ) : (
                        <span>Click to choose a file&hellip;</span>
                      )}
                    </button>
                    <p className="text-xs text-zinc-400 mt-1">Supported formats: TXT, VTT, SRT, MD, DOCX</p>
                  </div>
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
                {processing ? t("meetings.processing") : t("meetings.analyzeAndIngest")}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
