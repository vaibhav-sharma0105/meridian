import { useState, useEffect } from "react";
import {
  X,
  RefreshCw,
  Check,
  Trash2,
  Search,
  Github,
  AlertCircle,
  Clock,
  GitPullRequest,
  CircleDot,
} from "lucide-react";
import {
  useUpdateIntegration,
  useDeleteIntegration,
  useSyncIntegration,
  useCachedItems,
} from "@/hooks/useIntegrations";
import type { Integration } from "@/lib/tauri";
import toast from "react-hot-toast";

interface GitHubSettingsProps {
  integration?: Integration;
  onClose: () => void;
}

export function GitHubSettings({ integration, onClose }: GitHubSettingsProps) {
  const [repoFilter, setRepoFilter] = useState("");
  const [selectedRepos, setSelectedRepos] = useState<string[]>(
    integration?.config.repositories ?? []
  );
  const [syncInterval, setSyncInterval] = useState(
    integration?.sync_interval_minutes ?? 15
  );
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const updateMutation = useUpdateIntegration();
  const deleteMutation = useDeleteIntegration();
  const syncMutation = useSyncIntegration();
  const { data: cachedIssues = [] } = useCachedItems(integration?.id ?? "", "issue");
  const { data: cachedPRs = [] } = useCachedItems(integration?.id ?? "", "pr");

  const handleSave = async () => {
    if (!integration) return;
    try {
      await updateMutation.mutateAsync({
        id: integration.id,
        config: {
          ...integration.config,
          repositories: selectedRepos,
        },
        sync_interval_minutes: syncInterval,
      });
      toast.success("GitHub settings saved");
    } catch (e) {
      toast.error("Failed to save settings");
    }
  };

  const handleSync = () => {
    if (integration) {
      syncMutation.mutate(integration.id);
      toast.success("Syncing GitHub data...");
    }
  };

  const handleDelete = async () => {
    if (!integration) return;
    try {
      await deleteMutation.mutateAsync(integration.id);
      toast.success("GitHub disconnected");
      onClose();
    } catch (e) {
      toast.error("Failed to disconnect");
    }
  };

  const toggleRepo = (repo: string) => {
    setSelectedRepos((prev) =>
      prev.includes(repo) ? prev.filter((r) => r !== repo) : [...prev, repo]
    );
  };

  if (!integration) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="bg-white dark:bg-zinc-900 rounded-xl p-6">
          <p className="text-zinc-500">GitHub is not connected</p>
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
            <Github className="w-6 h-6" />
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                GitHub Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Configure repository sync and permissions
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

        <div className="p-6 space-y-6">
          {/* Connection Status */}
          <div className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <div className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <span className="text-sm text-green-700 dark:text-green-300">
                Connected
              </span>
            </div>
            {integration.last_sync && (
              <div className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
                <Clock className="w-3 h-3" />
                Last sync: {new Date(integration.last_sync).toLocaleString()}
              </div>
            )}
          </div>

          {/* Sync Interval */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
              Sync Interval
            </label>
            <select
              value={syncInterval}
              onChange={(e) => setSyncInterval(Number(e.target.value))}
              className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
            >
              <option value={5}>Every 5 minutes</option>
              <option value={15}>Every 15 minutes</option>
              <option value={30}>Every 30 minutes</option>
              <option value={60}>Every hour</option>
              <option value={0}>Manual only</option>
            </select>
          </div>

          {/* Repository Selection */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
              Repositories to Sync
            </label>
            <div className="relative mb-3">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
              <input
                type="text"
                placeholder="Filter repositories..."
                value={repoFilter}
                onChange={(e) => setRepoFilter(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
              />
            </div>
            <div className="max-h-48 overflow-y-auto border border-zinc-200 dark:border-zinc-700 rounded-lg">
              {selectedRepos.length === 0 ? (
                <div className="p-4 text-center text-sm text-zinc-500">
                  No repositories selected. Sync to fetch available repos.
                </div>
              ) : (
                selectedRepos
                  .filter((r) =>
                    r.toLowerCase().includes(repoFilter.toLowerCase())
                  )
                  .map((repo) => (
                    <label
                      key={repo}
                      className="flex items-center gap-3 px-3 py-2 hover:bg-zinc-50 dark:hover:bg-zinc-800 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={selectedRepos.includes(repo)}
                        onChange={() => toggleRepo(repo)}
                        className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600 focus:ring-indigo-500"
                      />
                      <span className="text-sm text-zinc-700 dark:text-zinc-300 font-mono">
                        {repo}
                      </span>
                    </label>
                  ))
              )}
            </div>
          </div>

          {/* Cached Data Stats */}
          <div className="grid grid-cols-2 gap-3">
            <div className="p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <CircleDot className="w-3 h-3" />
                Issues Cached
              </div>
              <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                {cachedIssues.length}
              </div>
            </div>
            <div className="p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <GitPullRequest className="w-3 h-3" />
                PRs Cached
              </div>
              <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                {cachedPRs.length}
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          {showDeleteConfirm ? (
            <div className="flex items-center gap-2">
              <span className="text-sm text-red-600">Disconnect GitHub?</span>
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
          <div className="flex items-center gap-2">
            <button
              onClick={handleSync}
              disabled={syncMutation.isPending}
              className="flex items-center gap-1 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded"
            >
              <RefreshCw
                className={`w-4 h-4 ${syncMutation.isPending ? "animate-spin" : ""}`}
              />
              Sync Now
            </button>
            <button
              onClick={handleSave}
              disabled={updateMutation.isPending}
              className="px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50"
            >
              Save Changes
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
