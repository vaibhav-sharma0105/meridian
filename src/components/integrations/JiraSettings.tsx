import { useState } from "react";
import {
  X,
  RefreshCw,
  Check,
  Trash2,
  Search,
  AlertCircle,
  Clock,
  Folder,
  ListTodo,
} from "lucide-react";
import {
  useUpdateIntegration,
  useDeleteIntegration,
  useSyncIntegration,
  useCachedItems,
} from "@/hooks/useIntegrations";
import type { Integration } from "@/lib/tauri";
import toast from "react-hot-toast";

interface JiraSettingsProps {
  integration?: Integration;
  onClose: () => void;
}

export function JiraSettings({ integration, onClose }: JiraSettingsProps) {
  const [projectFilter, setProjectFilter] = useState("");
  const [selectedProjects, setSelectedProjects] = useState<string[]>(
    integration?.config.projects ?? []
  );
  const [syncInterval, setSyncInterval] = useState(
    integration?.sync_interval_minutes ?? 15
  );
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const updateMutation = useUpdateIntegration();
  const deleteMutation = useDeleteIntegration();
  const syncMutation = useSyncIntegration();
  const { data: cachedIssues = [] } = useCachedItems(
    integration?.id ?? "",
    "jira_issue"
  );

  const handleSave = async () => {
    if (!integration) return;
    try {
      await updateMutation.mutateAsync({
        id: integration.id,
        config: {
          ...integration.config,
          projects: selectedProjects,
        },
        sync_interval_minutes: syncInterval,
      });
      toast.success("Jira settings saved");
    } catch (e) {
      toast.error("Failed to save settings");
    }
  };

  const handleSync = () => {
    if (integration) {
      syncMutation.mutate(integration.id);
      toast.success("Syncing Jira data...");
    }
  };

  const handleDelete = async () => {
    if (!integration) return;
    try {
      await deleteMutation.mutateAsync(integration.id);
      toast.success("Jira disconnected");
      onClose();
    } catch (e) {
      toast.error("Failed to disconnect");
    }
  };

  const toggleProject = (project: string) => {
    setSelectedProjects((prev) =>
      prev.includes(project)
        ? prev.filter((p) => p !== project)
        : [...prev, project]
    );
  };

  if (!integration) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="bg-white dark:bg-zinc-900 rounded-xl p-6">
          <p className="text-zinc-500">Jira is not connected</p>
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
            <span className="text-2xl">🔷</span>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Jira Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Configure project sync and permissions
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
                Connected to Jira Cloud
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

          {/* Project Selection */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
              Projects to Sync
            </label>
            <div className="relative mb-3">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
              <input
                type="text"
                placeholder="Filter projects..."
                value={projectFilter}
                onChange={(e) => setProjectFilter(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
              />
            </div>
            <div className="max-h-48 overflow-y-auto border border-zinc-200 dark:border-zinc-700 rounded-lg">
              {selectedProjects.length === 0 ? (
                <div className="p-4 text-center text-sm text-zinc-500">
                  No projects selected. Sync to fetch available projects.
                </div>
              ) : (
                selectedProjects
                  .filter((p) =>
                    p.toLowerCase().includes(projectFilter.toLowerCase())
                  )
                  .map((project) => (
                    <label
                      key={project}
                      className="flex items-center gap-3 px-3 py-2 hover:bg-zinc-50 dark:hover:bg-zinc-800 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={selectedProjects.includes(project)}
                        onChange={() => toggleProject(project)}
                        className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600 focus:ring-indigo-500"
                      />
                      <Folder className="w-4 h-4 text-blue-500" />
                      <span className="text-sm text-zinc-700 dark:text-zinc-300">
                        {project}
                      </span>
                    </label>
                  ))
              )}
            </div>
          </div>

          {/* Cached Data Stats */}
          <div className="p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
            <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
              <ListTodo className="w-3 h-3" />
              Issues Cached
            </div>
            <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              {cachedIssues.length}
            </div>
          </div>

          {/* Issue Sync Info */}
          <div className="flex items-start gap-2 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <AlertCircle className="w-4 h-4 text-blue-500 mt-0.5 flex-shrink-0" />
            <div className="text-xs text-blue-700 dark:text-blue-300">
              Meridian syncs issues assigned to you from selected projects.
              Issues can be linked to tasks for bidirectional updates.
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          {showDeleteConfirm ? (
            <div className="flex items-center gap-2">
              <span className="text-sm text-red-600">Disconnect Jira?</span>
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
