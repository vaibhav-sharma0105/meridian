import { useState } from "react";
import { useTranslation } from "react-i18next";
import { exportData } from "@/lib/tauri";
import { X, Download } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
}

export default function ExportDialog({ open, onClose }: Props) {
  const { t } = useTranslation();
  const [format, setFormat] = useState<"json" | "csv">("json");
  const [exporting, setExporting] = useState(false);

  if (!open) return null;

  const handleExport = async () => {
    setExporting(true);
    try {
      await exportData({ format });
      onClose();
    } catch (e) {
      console.error(e);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 border border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("export.title")}</h2>
          <button onClick={onClose} className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded">
            <X className="w-4 h-4 text-zinc-500" />
          </button>
        </div>

        <div className="space-y-3 mb-6">
          <p className="text-sm text-zinc-500">{t("export.description")}</p>
          <div className="flex gap-3">
            {(["json", "csv"] as const).map((f) => (
              <label key={f} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="format"
                  value={f}
                  checked={format === f}
                  onChange={() => setFormat(f)}
                  className="text-indigo-500"
                />
                <span className="text-sm text-zinc-700 dark:text-zinc-300 uppercase font-mono">{f}</span>
              </label>
            ))}
          </div>
        </div>

        <div className="flex gap-3 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleExport}
            disabled={exporting}
            className="flex items-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            <Download className="w-4 h-4" />
            {exporting ? t("common.loading") : t("export.export")}
          </button>
        </div>
      </div>
    </div>
  );
}
