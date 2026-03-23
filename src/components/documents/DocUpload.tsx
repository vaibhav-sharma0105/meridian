import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Upload } from "lucide-react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ingestDocument } from "@/lib/tauri";

interface Props {
  projectId: string;
  onUploaded: () => void;
}

export default function DocUpload({ projectId, onUploaded }: Props) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<"file" | "url" | "text">("file");
  const [url, setUrl] = useState("");
  const [text, setText] = useState("");
  const [title, setTitle] = useState("");
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState("");
  const [dragging, setDragging] = useState(false);

  const handleFileUpload = async (filePath?: string) => {
    let path = filePath;
    if (!path) {
      const selected = await openFileDialog({
        filters: [{ name: "Documents", extensions: ["pdf", "docx", "pptx", "txt", "md", "csv"] }],
        multiple: false,
      });
      if (!selected || typeof selected !== "string") return;
      path = selected;
    }
    setUploading(true);
    setError("");
    try {
      await ingestDocument({ projectId, filePath: path });
      onUploaded();
    } catch (e) {
      setError(String(e));
    } finally {
      setUploading(false);
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
        <div
          onDragOver={(e) => { e.preventDefault(); setDragging(true); }}
          onDragLeave={() => setDragging(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragging(false);
            // Note: file.path is available via Tauri's webview, not standard browser
            const file = e.dataTransfer.files[0];
            if (file) {
              // Cast to any to access Tauri-specific path property
              const tauriFile = file as any;
              handleFileUpload(tauriFile.path ?? undefined);
            }
          }}
          className={`border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors ${dragging ? "border-indigo-400 bg-indigo-50 dark:bg-indigo-900/10" : "border-zinc-200 dark:border-zinc-700 hover:border-indigo-300"}`}
          onClick={() => !uploading && handleFileUpload()}
        >
          <Upload className="w-8 h-8 text-zinc-400 mx-auto mb-2" />
          <p className="text-sm text-zinc-600 dark:text-zinc-400">{uploading ? t("common.loading") : t("documents.dropOrClick")}</p>
          <p className="text-xs text-zinc-400 mt-1">PDF, DOCX, PPTX, TXT, MD, CSV · max 50MB</p>
        </div>
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
