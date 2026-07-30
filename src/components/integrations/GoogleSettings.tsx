import { useState } from "react";
import { X, RefreshCw, Check, Trash2, Clock, AlertCircle } from "lucide-react";
import { useDeleteIntegration } from "@/hooks/useIntegrations";
import { useSyncTeamFromGoogle } from "@/hooks/useTeam";
import type { Integration } from "@/lib/tauri";
import toast from "react-hot-toast";

interface GoogleSettingsProps {
  integration?: Integration;
  onClose: () => void;
}

export function GoogleSettings({ integration, onClose }: GoogleSettingsProps) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const deleteMutation = useDeleteIntegration();
  const syncTeam = useSyncTeamFromGoogle();

  const handleSync = () => {
    syncTeam.mutate();
  };

  const handleDelete = async () => {
    if (!integration) return;
    try {
      await deleteMutation.mutateAsync(integration.id);
      toast.success("Google disconnected");
      onClose();
    } catch (e) {
      toast.error("Failed to disconnect");
    }
  };

  if (!integration) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="bg-white dark:bg-zinc-900 rounded-xl p-6">
          <p className="text-zinc-500">Google is not connected</p>
          <button
            onClick={onClose}
            className="mt-4 px-4 py-2 bg-zinc-200 dark:bg-zinc-700 rounded"
          >
            Close
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🔵</span>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Google Workspace Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Syncs your team roster from Google Workspace Directory
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          {/* Connection Status */}
          <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg space-y-1">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span className="text-sm text-green-700 dark:text-green-300">
                  Connected to Google Workspace
                </span>
              </div>
              {integration.last_sync && (
                <div className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
                  <Clock className="w-3 h-3" />
                  Last roster sync: {new Date(integration.last_sync).toLocaleString()}
                </div>
              )}
            </div>
            <p className="text-xs text-green-600/80 dark:text-green-400/70 pl-6">
              {integration.last_sync
                ? "Roster syncs automatically once a day, or use the button below any time."
                : "Not synced yet — click \"Sync Team Roster Now\" below, or wait for tomorrow's automatic sync."}
            </p>
          </div>

          {/* Requires domain admin note */}
          <div className="flex items-start gap-2 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
            <AlertCircle className="w-4 h-4 text-zinc-400 mt-0.5 flex-shrink-0" />
            <p className="text-xs text-zinc-500">
              Reading the Workspace directory requires the{" "}
              <code className="text-[11px]">admin.directory.user.readonly</code> scope,
              which only a Workspace domain admin can approve. If sync fails with a
              permission error, ask your admin to approve this app in the Google
              Workspace Admin Console.
            </p>
          </div>

          {/* Sync Now */}
          <button
            onClick={handleSync}
            disabled={syncTeam.isPending}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium
                     text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800
                     hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${syncTeam.isPending ? "animate-spin" : ""}`} />
            Sync Team Roster Now
          </button>

          {syncTeam.isSuccess && (
            <p className="text-xs text-green-600 dark:text-green-400">
              Synced {syncTeam.data?.added ?? 0} new members, updated {syncTeam.data?.updated ?? 0}
            </p>
          )}
          {syncTeam.isError && (
            <p className="text-xs text-red-600 dark:text-red-400">
              {(syncTeam.error as Error)?.message || "Sync failed"}
            </p>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          {showDeleteConfirm ? (
            <div className="flex items-center gap-2">
              <span className="text-sm text-red-600">Disconnect Google?</span>
              <button
                onClick={handleDelete}
                className="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
              >
                Confirm
              </button>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="px-3 py-1.5 text-sm text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded"
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              onClick={() => setShowDeleteConfirm(true)}
              className="flex items-center gap-1 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
            >
              <Trash2 className="w-4 h-4" />
              Disconnect
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
