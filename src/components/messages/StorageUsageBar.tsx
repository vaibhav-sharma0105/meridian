import { HardDrive, AlertTriangle } from "lucide-react";
import { useStorageStats } from "@/hooks/useMessages";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function StorageUsageBar() {
  const { data: stats, isLoading } = useStorageStats();

  if (isLoading || !stats) {
    return null;
  }

  const maxBytes = 1024 * 1024 * 1024; // 1 GB threshold
  const percentage = Math.min((stats.storage_bytes / maxBytes) * 100, 100);
  const isWarning = stats.storage_bytes > 500 * 1024 * 1024; // 500 MB
  const isCritical = stats.storage_bytes > maxBytes;

  return (
    <div className="px-4 py-3 border-t border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800/50">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
          <HardDrive className="w-4 h-4" />
          <span>Storage</span>
        </div>
        <div className="flex items-center gap-2">
          {(isWarning || isCritical) && (
            <AlertTriangle
              className={`w-4 h-4 ${
                isCritical ? "text-red-500" : "text-amber-500"
              }`}
            />
          )}
          <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {formatBytes(stats.storage_bytes)}
          </span>
        </div>
      </div>

      <div className="h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all ${
            isCritical
              ? "bg-red-500"
              : isWarning
              ? "bg-amber-500"
              : "bg-indigo-500"
          }`}
          style={{ width: `${percentage}%` }}
        />
      </div>

      <div className="flex items-center justify-between mt-2 text-xs text-zinc-500">
        <span>{stats.total_messages} messages</span>
        <span>{stats.total_files} files</span>
      </div>
    </div>
  );
}
