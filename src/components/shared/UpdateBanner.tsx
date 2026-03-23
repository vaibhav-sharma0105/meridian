import { useState, useEffect } from "react";
import { X, Download } from "lucide-react";
import { checkForUpdates } from "@/lib/tauri";

export default function UpdateBanner() {
  const [update, setUpdate] = useState<{ version: string } | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    checkForUpdates()
      .then((result) => {
        if (result.update_available && result.version) {
          setUpdate({ version: result.version });
        }
      })
      .catch(() => {/* silently ignore */});
  }, []);

  if (!update || dismissed) return null;

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-indigo-500 text-white text-sm flex-shrink-0">
      <Download className="w-4 h-4 flex-shrink-0" />
      <span className="flex-1">
        A new version <strong>{update.version}</strong> is available.
      </span>
      <button
        onClick={() => window.location.reload()}
        className="px-3 py-1 bg-white text-indigo-600 rounded font-medium hover:bg-indigo-50 transition-colors text-xs"
      >
        Install now
      </button>
      <button
        onClick={() => setDismissed(true)}
        className="p-1 hover:bg-indigo-600 rounded transition-colors"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}
