import { useState } from "react";
import {
  X,
  RefreshCw,
  Link2,
  Unlink,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";
import { useConnections } from "@/hooks/useConnections";
import { format, isValid } from "date-fns";

interface Props {
  open: boolean;
  onClose: () => void;
  runSync: () => Promise<void>;
  isSyncing: boolean;
}

export default function ConnectionsSettings({ open, onClose, runSync, isSyncing }: Props) {
  const {
    zoom,
    gmail,
    connectZoom,
    isConnectingZoom,
    connectGmail,
    isConnectingGmail,
    disconnect,
    zoomError,
    gmailError,
  } = useConnections();
  const [disconnectConfirm, setDisconnectConfirm] = useState<string | null>(
    null
  );

  if (!open) return null;

  const handleDisconnect = async (provider: string) => {
    if (disconnectConfirm !== provider) {
      setDisconnectConfirm(provider);
      return;
    }
    await disconnect(provider);
    setDisconnectConfirm(null);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-md bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-zinc-200 dark:border-zinc-800 p-6"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">
            Connections
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="space-y-4">
          {/* ── Zoom ── */}
          <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-lg">🎥</span>
              <span className="font-medium text-zinc-900 dark:text-zinc-50">
                Zoom
              </span>
              {zoom && (
                <CheckCircle2 className="w-4 h-4 text-green-500 ml-auto" />
              )}
            </div>

            {zoom ? (
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-zinc-600 dark:text-zinc-400">
                    Connected as{" "}
                    <span className="font-medium text-zinc-900 dark:text-zinc-50">
                      {zoom.account_email}
                    </span>
                  </p>
                  <p className="text-xs text-zinc-400 mt-0.5">
                    {zoom.last_sync_at && (() => { const d = new Date(zoom.last_sync_at!); return isValid(d) ? `Last synced: ${format(d, "MMM d, h:mm a")}` : null; })() || "Never synced"}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={runSync}
                    disabled={isSyncing}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-indigo-50 text-indigo-700 hover:bg-indigo-100 dark:bg-indigo-900/30 dark:text-indigo-300 dark:hover:bg-indigo-900/50 disabled:opacity-50 transition-colors"
                  >
                    <RefreshCw
                      className={`w-3 h-3 ${isSyncing ? "animate-spin" : ""}`}
                    />
                    {isSyncing ? "Syncing..." : "Sync Now"}
                  </button>
                  <button
                    onClick={() => handleDisconnect("zoom")}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                  >
                    <Unlink className="w-3 h-3" />
                    {disconnectConfirm === "zoom"
                      ? "Click again to confirm"
                      : "Disconnect"}
                  </button>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  Import meeting summaries and transcripts from your Zoom
                  meetings automatically.
                </p>
                {zoomError && (
                  <div className="flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/20 rounded-md">
                    <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                    <p className="text-xs text-red-600 dark:text-red-400">
                      {String(zoomError)}
                    </p>
                  </div>
                )}
                <button
                  onClick={() => connectZoom()}
                  disabled={isConnectingZoom}
                  className="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                >
                  <Link2 className="w-4 h-4" />
                  {isConnectingZoom
                    ? "Connecting… (check your browser)"
                    : "Connect Zoom"}
                </button>
              </div>
            )}
          </div>

          {/* ── Gmail ── */}
          <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-lg">✉️</span>
              <span className="font-medium text-zinc-900 dark:text-zinc-50">
                Gmail
              </span>
              {gmail && (
                <CheckCircle2 className="w-4 h-4 text-green-500 ml-auto" />
              )}
              {!gmail && (
                <span className="ml-auto text-xs text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-2 py-0.5 rounded-full">
                  Coming soon
                </span>
              )}
            </div>

            {gmail ? (
              <div className="space-y-3">
                <p className="text-sm text-zinc-600 dark:text-zinc-400">
                  Connected as{" "}
                  <span className="font-medium">{gmail.account_email}</span>
                </p>
                <button
                  onClick={() => handleDisconnect("gmail")}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                >
                  <Unlink className="w-3 h-3" />
                  {disconnectConfirm === "gmail"
                    ? "Click again to confirm"
                    : "Disconnect"}
                </button>
              </div>
            ) : (
              <div className="space-y-2">
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  Import Zoom meeting summaries from Gmail — covers meetings you
                  attend but don't host.
                </p>
                {gmailError && (
                  <div className="flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/20 rounded-md">
                    <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                    <p className="text-xs text-red-600 dark:text-red-400">
                      {String(gmailError)}
                    </p>
                  </div>
                )}
                <button
                  disabled
                  className="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-md bg-zinc-100 text-zinc-400 dark:bg-zinc-800 dark:text-zinc-500 cursor-not-allowed"
                >
                  <Link2 className="w-4 h-4" />
                  Connect Gmail
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
