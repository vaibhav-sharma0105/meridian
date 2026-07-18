import { useState } from "react";
import {
  RefreshCw,
  MoreHorizontal,
  Trash2,
  Settings,
  CheckCircle2,
  AlertCircle,
  Clock,
} from "lucide-react";
import type { Integration } from "@/lib/tauri";
import { useSyncIntegration, useDeleteIntegration } from "@/hooks/useIntegrations";

interface IntegrationCardProps {
  integration: Integration;
}

export function IntegrationCard({ integration }: IntegrationCardProps) {
  const [showMenu, setShowMenu] = useState(false);
  const syncMutation = useSyncIntegration();
  const deleteMutation = useDeleteIntegration();

  const isSyncing = integration.status === "syncing" || syncMutation.isPending;

  const statusIcon = {
    connected: <CheckCircle2 className="w-4 h-4 text-green-500" />,
    syncing: <RefreshCw className="w-4 h-4 text-indigo-500 animate-spin" />,
    error: <AlertCircle className="w-4 h-4 text-red-500" />,
    disconnected: <Clock className="w-4 h-4 text-zinc-400" />,
  }[integration.status] || <Clock className="w-4 h-4 text-zinc-400" />;

  const handleSync = () => {
    syncMutation.mutate(integration.id);
  };

  const handleDelete = () => {
    if (confirm(`Disconnect ${integration.name}? This will remove all cached data.`)) {
      deleteMutation.mutate(integration.id);
    }
  };

  const formatLastSync = (dateStr?: string) => {
    if (!dateStr) return "Never synced";
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="flex items-center gap-4 p-4 border border-zinc-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-900">
      <div className="w-10 h-10 flex items-center justify-center bg-zinc-100 dark:bg-zinc-800 rounded-lg">
        <Settings className="w-5 h-5 text-zinc-600 dark:text-zinc-400" />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-zinc-900 dark:text-zinc-100">
            {integration.name}
          </span>
          {statusIcon}
        </div>
        <div className="text-xs text-zinc-500 mt-0.5">
          {integration.status === "error" && integration.error_message
            ? integration.error_message
            : formatLastSync(integration.last_sync ?? undefined)}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={handleSync}
          disabled={isSyncing}
          className="p-2 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors disabled:opacity-50"
          title="Sync now"
        >
          <RefreshCw className={`w-4 h-4 ${isSyncing ? "animate-spin" : ""}`} />
        </button>

        <div className="relative">
          <button
            onClick={() => setShowMenu(!showMenu)}
            className="p-2 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors"
          >
            <MoreHorizontal className="w-4 h-4" />
          </button>

          {showMenu && (
            <>
              <div
                className="fixed inset-0 z-10"
                onClick={() => setShowMenu(false)}
              />
              <div className="absolute right-0 top-full mt-1 w-40 py-1 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg z-20">
                <button
                  onClick={() => {
                    setShowMenu(false);
                    handleDelete();
                  }}
                  className="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                >
                  <Trash2 className="w-4 h-4" />
                  Disconnect
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
