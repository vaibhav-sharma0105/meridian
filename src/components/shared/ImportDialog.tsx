import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Upload } from "lucide-react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { importData } from "@/lib/tauri";

interface Props {
  open: boolean;
  onClose: () => void;
}

export default function ImportDialog({ open: isOpen, onClose }: Props) {
  const { t } = useTranslation();
  const [filePath, setFilePath] = useState("");
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState("");

  if (!isOpen) return null;

  const handleBrowse = async () => {
    const selected = await openFileDialog({
      filters: [{ name: "JSON", extensions: ["json"] }],
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      setFilePath(selected);
    }
  };

  const handleImport = async () => {
    if (!filePath) return;
    setImporting(true);
    setError("");
    try {
      await importData({ filePath });
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 border border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("import.title")}</h2>
          <button onClick={onClose} className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded">
            <X className="w-4 h-4 text-zinc-500" />
          </button>
        </div>

        <div className="space-y-3 mb-6">
          <p className="text-sm text-zinc-500">{t("import.description")}</p>
          <div className="flex gap-2">
            <input
              type="text"
              value={filePath}
              readOnly
              placeholder={t("import.noFile")}
              className="flex-1 px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm"
            />
            <button
              onClick={handleBrowse}
              className="px-3 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
            >
              {t("import.browse")}
            </button>
          </div>
          {error && <p className="text-sm text-red-600">{error}</p>}
        </div>

        <div className="flex gap-3 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleImport}
            disabled={importing || !filePath}
            className="flex items-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            <Upload className="w-4 h-4" />
            {importing ? t("common.loading") : t("import.import")}
          </button>
        </div>
      </div>
    </div>
  );
}
