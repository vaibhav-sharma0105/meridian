import { useEffect, useCallback, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import toast from "react-hot-toast";

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
          `${result.new_imports} new meeting${result.new_imports > 1 ? "s" : ""} found`,
          { icon: "📥" }
        );
        qc.invalidateQueries({ queryKey: ["pending-imports"] });
        qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
        qc.invalidateQueries({ queryKey: ["notifications"] });
      }
      if (result.errors.length > 0) {
        console.warn("Sync errors:", result.errors);
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
