import { useState, useEffect } from "react";
import { Loader, CheckCircle, X, Brain } from "lucide-react";
import { getIndexingStatus, processPendingEmbeddings, type IndexingStatus } from "@/lib/tauri";

export default function IndexingBanner() {
  const [status, setStatus] = useState<IndexingStatus | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const s = await getIndexingStatus();
        setStatus(s);
      } catch {
        // Ignore errors
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleStartIndexing = async () => {
    setStarting(true);
    try {
      const s = await processPendingEmbeddings();
      setStatus(s);
    } finally {
      setStarting(false);
    }
  };

  // Don't show if dismissed or no pending jobs
  if (dismissed) return null;
  if (!status) return null;
  if (status.pending_jobs === 0 && status.running_jobs === 0) return null;

  const isActive = status.worker_running && (status.running_jobs > 0 || status.pending_jobs > 0);
  const totalJobs = status.pending_jobs + status.running_jobs;

  return (
    <div className="bg-indigo-50 dark:bg-indigo-900/20 border-b border-indigo-100 dark:border-indigo-800 px-4 py-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {isActive ? (
            <div className="flex items-center gap-2">
              <Loader className="w-4 h-4 text-indigo-500 animate-spin" />
              <span className="text-sm text-indigo-700 dark:text-indigo-300">
                Indexing documents... ({status.running_jobs} active, {status.pending_jobs} queued)
              </span>
            </div>
          ) : status.pending_jobs > 0 ? (
            <div className="flex items-center gap-2">
              <Brain className="w-4 h-4 text-indigo-500" />
              <span className="text-sm text-indigo-700 dark:text-indigo-300">
                {totalJobs} document{totalJobs !== 1 ? "s" : ""} waiting to be indexed
              </span>
              <button
                onClick={handleStartIndexing}
                disabled={starting}
                className="ml-2 px-2 py-0.5 text-xs bg-indigo-500 hover:bg-indigo-600 text-white rounded disabled:opacity-50"
              >
                {starting ? "Starting..." : "Start Indexing"}
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <CheckCircle className="w-4 h-4 text-emerald-500" />
              <span className="text-sm text-emerald-700 dark:text-emerald-300">
                All documents indexed
              </span>
            </div>
          )}
        </div>

        <button
          onClick={() => setDismissed(true)}
          className="p-1 text-indigo-400 hover:text-indigo-600 dark:hover:text-indigo-200 rounded"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
