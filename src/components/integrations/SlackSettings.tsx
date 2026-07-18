import { useState, useEffect } from "react";
import {
  X,
  RefreshCw,
  Check,
  Trash2,
  Hash,
  Clock,
  AlertCircle,
  Shield,
  MessageSquare,
  Key,
  Eye,
  EyeOff,
  HelpCircle,
} from "lucide-react";
import {
  useUpdateIntegration,
  useDeleteIntegration,
  useSyncIntegration,
} from "@/hooks/useIntegrations";
import type { Integration, ChannelConfig } from "@/lib/tauri";
import toast from "react-hot-toast";

interface SlackSettingsProps {
  integration?: Integration;
  onClose: () => void;
}

const AUTONOMY_MODES = [
  { value: "approve_first", label: "Require Approval", description: "Review drafts before sending" },
  { value: "auto", label: "Auto Send", description: "Send drafts automatically after delay" },
  { value: "notify", label: "Notify Only", description: "Send and notify about sent messages" },
];

export function SlackSettings({ integration, onClose }: SlackSettingsProps) {
  const [channels, setChannels] = useState<ChannelConfig[]>(
    integration?.config.channels ?? []
  );
  const [syncInterval, setSyncInterval] = useState(
    integration?.sync_interval_minutes ?? 15
  );
  const [draftDelay, setDraftDelay] = useState(10); // minutes
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [appToken, setAppToken] = useState(integration?.config.app_token ?? "");
  const [showAppToken, setShowAppToken] = useState(false);
  const [socketModeEnabled, setSocketModeEnabled] = useState(
    integration?.config.socket_mode_enabled ?? false
  );
  const [showAppTokenHelp, setShowAppTokenHelp] = useState(false);

  const updateMutation = useUpdateIntegration();
  const deleteMutation = useDeleteIntegration();
  const syncMutation = useSyncIntegration();

  const handleSave = async () => {
    if (!integration) return;
    try {
      await updateMutation.mutateAsync({
        id: integration.id,
        config: {
          ...integration.config,
          channels,
          app_token: appToken || undefined,
          socket_mode_enabled: socketModeEnabled && !!appToken,
        },
        sync_interval_minutes: syncInterval,
      });
      toast.success("Slack settings saved");
    } catch (e) {
      toast.error("Failed to save settings");
    }
  };

  const handleSync = () => {
    if (integration) {
      syncMutation.mutate(integration.id);
      toast.success("Syncing Slack channels...");
    }
  };

  const handleDelete = async () => {
    if (!integration) return;
    try {
      await deleteMutation.mutateAsync(integration.id);
      toast.success("Slack disconnected");
      onClose();
    } catch (e) {
      toast.error("Failed to disconnect");
    }
  };

  const updateChannelAutonomy = (channelId: string, autonomyMode: string) => {
    setChannels((prev) =>
      prev.map((ch) =>
        ch.id === channelId ? { ...ch, autonomy_mode: autonomyMode } : ch
      )
    );
  };

  if (!integration) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="bg-white dark:bg-zinc-900 rounded-xl p-6">
          <p className="text-zinc-500">Slack is not connected</p>
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
            <span className="text-2xl">💬</span>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Slack Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Configure channels and message autonomy
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
                Connected to Slack workspace
              </span>
            </div>
            {integration.last_sync && (
              <div className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
                <Clock className="w-3 h-3" />
                Last sync: {new Date(integration.last_sync).toLocaleString()}
              </div>
            )}
          </div>

          {/* Socket Mode / App Token */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <label className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                  Socket Mode (Real-time Events)
                </label>
                <button
                  onClick={() => setShowAppTokenHelp(!showAppTokenHelp)}
                  className="p-0.5 text-zinc-400 hover:text-zinc-600"
                >
                  <HelpCircle className="w-3.5 h-3.5" />
                </button>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={socketModeEnabled}
                  onChange={(e) => setSocketModeEnabled(e.target.checked)}
                  disabled={!appToken}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-zinc-200 peer-focus:outline-none rounded-full peer dark:bg-zinc-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:border-zinc-600 peer-checked:bg-indigo-600 peer-disabled:opacity-50"></div>
              </label>
            </div>

            {showAppTokenHelp && (
              <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg text-xs text-blue-700 dark:text-blue-300">
                <strong>Socket Mode</strong> enables real-time event monitoring (mentions, requests).
                It requires an <strong>App Token</strong> (starts with <code className="bg-blue-100 dark:bg-blue-800 px-1 rounded">xapp-</code>).
                <ol className="mt-2 list-decimal list-inside space-y-1">
                  <li>Go to <a href="https://api.slack.com/apps" target="_blank" rel="noopener noreferrer" className="underline">api.slack.com/apps</a></li>
                  <li>Select your Meridian app</li>
                  <li>Go to <strong>Socket Mode</strong> → Enable</li>
                  <li>Generate token with <code className="bg-blue-100 dark:bg-blue-800 px-1 rounded">connections:write</code> scope</li>
                  <li>Paste the <code className="bg-blue-100 dark:bg-blue-800 px-1 rounded">xapp-...</code> token below</li>
                </ol>
              </div>
            )}

            <div>
              <label className="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5">
                App Token
              </label>
              <div className="relative">
                <Key className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
                <input
                  type={showAppToken ? "text" : "password"}
                  value={appToken}
                  onChange={(e) => setAppToken(e.target.value)}
                  placeholder="xapp-1-..."
                  className="w-full pl-9 pr-10 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 placeholder:text-zinc-400"
                />
                <button
                  type="button"
                  onClick={() => setShowAppToken(!showAppToken)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600"
                >
                  {showAppToken ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
              {appToken && !appToken.startsWith("xapp-") && (
                <p className="mt-1 text-xs text-amber-600 dark:text-amber-400">
                  App tokens should start with "xapp-"
                </p>
              )}
            </div>
          </div>

          {/* Draft Queue Delay */}
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
              Draft Send Delay (minutes)
            </label>
            <div className="flex items-center gap-3">
              <input
                type="range"
                min={1}
                max={30}
                value={draftDelay}
                onChange={(e) => setDraftDelay(Number(e.target.value))}
                className="flex-1"
              />
              <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300 w-12 text-center">
                {draftDelay} min
              </span>
            </div>
            <p className="text-xs text-zinc-500 mt-1">
              Time to review or cancel auto-send drafts
            </p>
          </div>

          {/* Channel List with Per-Channel Autonomy */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                Channel Settings
              </label>
              <button
                onClick={handleSync}
                disabled={syncMutation.isPending}
                className="flex items-center gap-1 text-xs text-indigo-600 hover:text-indigo-700"
              >
                <RefreshCw
                  className={`w-3 h-3 ${syncMutation.isPending ? "animate-spin" : ""}`}
                />
                Refresh Channels
              </button>
            </div>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {channels.length === 0 ? (
                <div className="p-4 text-center text-sm text-zinc-500 border border-zinc-200 dark:border-zinc-700 rounded-lg">
                  No channels synced. Click "Refresh Channels" to fetch.
                </div>
              ) : (
                channels.map((channel) => (
                  <div
                    key={channel.id}
                    className="p-3 border border-zinc-200 dark:border-zinc-700 rounded-lg"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Hash className="w-4 h-4 text-zinc-400" />
                        <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                          {channel.name}
                        </span>
                        {channel.is_external && (
                          <span className="px-1.5 py-0.5 text-[10px] font-medium bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 rounded">
                            External
                          </span>
                        )}
                      </div>
                    </div>
                    <select
                      value={channel.autonomy_mode}
                      onChange={(e) =>
                        updateChannelAutonomy(channel.id, e.target.value)
                      }
                      className="w-full px-2 py-1.5 text-xs border border-zinc-300 dark:border-zinc-600 rounded bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                    >
                      {AUTONOMY_MODES.map((mode) => (
                        <option key={mode.value} value={mode.value}>
                          {mode.label} — {mode.description}
                        </option>
                      ))}
                    </select>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Autonomy Help */}
          <div className="flex items-start gap-2 p-3 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
            <Shield className="w-4 h-4 text-purple-500 mt-0.5 flex-shrink-0" />
            <div className="text-xs text-purple-700 dark:text-purple-300">
              <strong>Autonomy levels:</strong>
              <ul className="mt-1 space-y-0.5 list-disc list-inside">
                <li><strong>Require Approval:</strong> Drafts wait for your OK</li>
                <li><strong>Auto Send:</strong> Send after delay (cancellable)</li>
                <li><strong>Notify Only:</strong> Send immediately, notify you</li>
              </ul>
            </div>
          </div>

          {/* Draft Queue Preview (placeholder) */}
          <div className="p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
            <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
              <MessageSquare className="w-3 h-3" />
              Pending Drafts
            </div>
            <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
              0
            </div>
            <p className="text-xs text-zinc-500 mt-1">
              No drafts waiting to be sent
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          {showDeleteConfirm ? (
            <div className="flex items-center gap-2">
              <span className="text-sm text-red-600">Disconnect Slack?</span>
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
  );
}
