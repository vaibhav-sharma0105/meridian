import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Upload, CheckCircle, XCircle, Loader } from "lucide-react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ingestDocument } from "@/lib/tauri";

interface Props {
  projectId: string;
  onUploaded: () => void;
}

interface BulkUploadProgress {
  total: number;
  completed: number;
  current: string;
  results: { path: string; success: boolean; error?: string }[];
}

const SUPPORTED_EXTENSIONS = ["pdf", "docx", "pptx", "txt", "md", "csv", "xlsx", "xls"];

export default function DocUpload({ projectId, onUploaded }: Props) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<"file" | "url" | "text">("file");
  const [url, setUrl] = useState("");
  const [text, setText] = useState("");
  const [title, setTitle] = useState("");
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState("");
  const [dragging, setDragging] = useState(false);
  const [bulkProgress, setBulkProgress] = useState<BulkUploadProgress | null>(null);

  const isFileSupported = (fileName: string): boolean => {
    const ext = fileName.split(".").pop()?.toLowerCase();
    return ext ? SUPPORTED_EXTENSIONS.includes(ext) : false;
  };

  const handleFileUpload = async (filePaths?: string[]) => {
    let paths = filePaths;
    if (!paths || paths.length === 0) {
      const selected = await openFileDialog({
        filters: [{ name: "Documents", extensions: SUPPORTED_EXTENSIONS }],
        multiple: true,
      });
      if (!selected) return;
      paths = Array.isArray(selected) ? selected : [selected];
    }

    if (paths.length === 0) return;

    setUploading(true);
    setError("");

    if (paths.length === 1) {
      try {
        await ingestDocument({ projectId, filePath: paths[0] });
        onUploaded();
      } catch (e) {
        setError(String(e));
      } finally {
        setUploading(false);
      }
    } else {
      const progress: BulkUploadProgress = {
        total: paths.length,
        completed: 0,
        current: "",
        results: [],
      };
      setBulkProgress(progress);

      for (const path of paths) {
        const fileName = path.split("/").pop() ?? path;
        setBulkProgress(p => p ? { ...p, current: fileName } : null);

        if (!isFileSupported(fileName)) {
          progress.results.push({ path, success: false, error: "Unsupported file type" });
          progress.completed++;
          setBulkProgress({ ...progress });
          continue;
        }

        try {
          await ingestDocument({ projectId, filePath: path });
          progress.results.push({ path, success: true });
        } catch (e) {
          progress.results.push({ path, success: false, error: String(e) });
        }
        progress.completed++;
        setBulkProgress({ ...progress });
      }

      setUploading(false);
      onUploaded();
    }
  };

  const handleUrlIngest = async () => {
    if (!url.trim()) return;
    setUploading(true);
    setError("");
    try {
      await ingestDocument({ projectId, url: url.trim(), title: title || undefined });
      onUploaded();
      setUrl("");
      setTitle("");
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
    }
  };

  const handleTextIngest = async () => {
    if (!text.trim() || !title.trim()) return;
    setUploading(true);
    setError("");
    try {
      await ingestDocument({ projectId, content: text.trim(), title: title.trim() });
      onUploaded();
      setText("");
      setTitle("");
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="space-y-3">
      {/* Mode tabs */}
      <div className="flex gap-1 bg-zinc-100 dark:bg-zinc-800 rounded-lg p-1">
        {(["file", "url", "text"] as const).map((m) => (
          <button
            key={m}
            onClick={() => setMode(m)}
            className={`flex-1 py-1 text-xs rounded-md font-medium transition-colors capitalize ${mode === m ? "bg-white dark:bg-zinc-700 shadow-sm text-zinc-900 dark:text-zinc-50" : "text-zinc-500"}`}
          >
            {m === "file" ? t("documents.file") : m === "url" ? "URL" : t("documents.text")}
          </button>
        ))}
      </div>

      {mode === "file" && (
        <>
          <div
            onDragOver={(e) => { e.preventDefault(); setDragging(true); }}
            onDragLeave={() => setDragging(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragging(false);
              const files = Array.from(e.dataTransfer.files);
              const paths = files.map(f => (f as any).path).filter(Boolean);
              if (paths.length > 0) {
                handleFileUpload(paths);
              }
            }}
            className={`border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors ${dragging ? "border-indigo-400 bg-indigo-50 dark:bg-indigo-900/10" : "border-zinc-200 dark:border-zinc-700 hover:border-indigo-300"}`}
            onClick={() => !uploading && handleFileUpload()}
          >
            <Upload className="w-8 h-8 text-zinc-400 mx-auto mb-2" />
            <p className="text-sm text-zinc-600 dark:text-zinc-400">{uploading ? t("common.loading") : t("documents.dropOrClick")}</p>
            <p className="text-xs text-zinc-400 mt-1">PDF, DOCX, PPTX, TXT, MD, CSV, XLSX · max 50MB each</p>
            <p className="text-xs text-zinc-400">Supports multiple files</p>
          </div>

          {bulkProgress && (
            <div className="space-y-2 mt-3">
              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-600 dark:text-zinc-400">
                  Uploading {bulkProgress.completed}/{bulkProgress.total}
                </span>
                {bulkProgress.current && (
                  <span className="text-xs text-zinc-400 truncate max-w-[200px]">
                    {bulkProgress.current}
                  </span>
                )}
              </div>
              <div className="h-2 w-full bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-indigo-500 rounded-full transition-all duration-200"
                  style={{ width: `${(bulkProgress.completed / bulkProgress.total) * 100}%` }}
                />
              </div>
              {bulkProgress.completed === bulkProgress.total && (
                <div className="space-y-1 mt-2">
                  {bulkProgress.results.map((result, i) => (
                    <div key={i} className="flex items-center gap-2 text-xs">
                      {result.success ? (
                        <CheckCircle className="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" />
                      ) : (
                        <XCircle className="w-3.5 h-3.5 text-red-500 flex-shrink-0" />
                      )}
                      <span className={`truncate ${result.success ? "text-zinc-600 dark:text-zinc-400" : "text-red-600 dark:text-red-400"}`}>
                        {result.path.split("/").pop()}
                        {result.error && `: ${result.error}`}
                      </span>
                    </div>
                  ))}
                  <button
                    onClick={() => setBulkProgress(null)}
                    className="mt-2 text-xs text-indigo-500 hover:text-indigo-600"
                  >
                    Clear results
                  </button>
                </div>
              )}
            </div>
          )}
        </>
      )}

      {mode === "url" && (
        <div className="space-y-2">
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t("documents.titlePlaceholder")}
            className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          />
          <div className="flex gap-2">
            <input
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://..."
              className="flex-1 px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
            />
            <button
              onClick={handleUrlIngest}
              disabled={uploading || !url.trim()}
              className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
            >
              {uploading ? "..." : t("documents.fetch")}
            </button>
          </div>
        </div>
      )}

      {mode === "text" && (
        <div className="space-y-2">
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t("documents.titlePlaceholder")}
            className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          />
          <textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t("documents.pastePlaceholder")}
            rows={6}
            className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 resize-none"
          />
          <button
            onClick={handleTextIngest}
            disabled={uploading || !text.trim() || !title.trim()}
            className="w-full px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {uploading ? t("common.loading") : t("documents.save")}
          </button>
        </div>
      )}

      {error && <p className="text-sm text-red-600">{error}</p>}
    </div>
  );
}
