import { useEffect, useCallback, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import toast from "react-hot-toast";
import { X } from "lucide-react";

export function useSync() {
  const qc = useQueryClient();
  const hasSynced = useRef(false);
  const [isSyncing, setIsSyncing] = useState(false);

  const runSync = useCallback(async () => {
    setIsSyncing(true);
    try {
      const result = await api.syncConnections();
      if (result.new_imports > 0) {
        toast(
          (t) => (
            <div className="flex items-center gap-2">
              <span>📥 {result.new_imports} new meeting{result.new_imports > 1 ? "s" : ""} found</span>
              <button
                onClick={() => toast.dismiss(t.id)}
                className="p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10"
              >
                <X className="w-3.5 h-3.5 text-zinc-400" />
              </button>
            </div>
          ),
          { duration: 4000 }
        );
        qc.invalidateQueries({ queryKey: ["pending-imports"] });
        qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
        qc.invalidateQueries({ queryKey: ["notifications"] });
      }
      // Only show duplicate message if there are NO new imports
      if (result.skipped_duplicates > 0 && result.new_imports === 0) {
        toast(
          (t) => (
            <div className="flex items-center gap-2">
              <span>✓ {result.skipped_duplicates} meeting{result.skipped_duplicates > 1 ? "s" : ""} already imported</span>
              <button
                onClick={() => toast.dismiss(t.id)}
                className="p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10"
              >
                <X className="w-3.5 h-3.5 text-zinc-400" />
              </button>
            </div>
          ),
          { duration: 4000 }
        );
      }
      if (result.errors.length > 0) {
        console.warn("Sync errors:", result.errors);
        result.errors.forEach((err) => toast.error(err, { duration: 8000 }));
      }
    } catch (e) {
      console.error("Sync failed:", e);
    } finally {
      setIsSyncing(false);
    }
  }, [qc]);

  // Auto-sync once on app launch
  useEffect(() => {
    if (!hasSynced.current) {
      hasSynced.current = true;
      runSync();
    }
  }, [runSync]);

  return { runSync, isSyncing };
}
