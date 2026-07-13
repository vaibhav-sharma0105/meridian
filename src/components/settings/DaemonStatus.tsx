import { useState, useEffect, Component, ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Play, Square, RefreshCw, CheckCircle, XCircle, Clock, Loader, Power, AlertTriangle } from "lucide-react";
import { getDaemonStatus, startDaemon, stopDaemon, getSchedulerStatus, enableSystemScheduler, disableSystemScheduler } from "@/lib/tauri";
import type { DaemonStatus as DaemonStatusType, SchedulerStatus as SchedulerStatusType } from "@/lib/tauri";

class DaemonStatusErrorBoundary extends Component<{ children: ReactNode }, { hasError: boolean; error: string }> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: "" };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-start gap-2 px-3 py-2 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
          <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-xs text-amber-700 dark:text-amber-300 font-medium">Background Service unavailable</p>
            <p className="text-xs text-amber-600 dark:text-amber-400 mt-0.5">{this.state.error}</p>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

function DaemonStatusInner() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<DaemonStatusType | null>(null);
  const [schedulerStatus, setSchedulerStatus] = useState<SchedulerStatusType | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [schedulerLoading, setSchedulerLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = async () => {
    try {
      const [daemonStatus, schedStatus] = await Promise.all([
        getDaemonStatus(),
        getSchedulerStatus(),
      ]);
      setStatus(daemonStatus);
      setSchedulerStatus(schedStatus);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleSchedulerToggle = async () => {
    if (!schedulerStatus) return;
    setSchedulerLoading(true);
    try {
      if (schedulerStatus.enabled) {
        await disableSystemScheduler();
      } else {
        await enableSystemScheduler();
      }
      await fetchStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setSchedulerLoading(false);
    }
  };

  const handleStart = async () => {
    setActionLoading(true);
    try {
      const s = await startDaemon();
      setStatus(s);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setActionLoading(false);
    }
  };

  const handleStop = async () => {
    setActionLoading(true);
    try {
      await stopDaemon();
      await fetchStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setActionLoading(false);
    }
  };

  const formatUptime = (seconds: number | null) => {
    if (seconds === null) return "—";
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
    return `${Math.floor(seconds / 86400)}d ${Math.floor((seconds % 86400) / 3600)}h`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader className="w-5 h-5 animate-spin text-zinc-400" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`w-2.5 h-2.5 rounded-full ${status?.running ? "bg-green-500" : "bg-zinc-300 dark:bg-zinc-600"}`} />
          <div>
            <div className="text-sm font-medium text-zinc-900 dark:text-zinc-50">
              Background Service
            </div>
            <div className="text-xs text-zinc-500">
              {status?.running ? "Running" : "Stopped"}
              {status?.pid && ` (PID ${status.pid})`}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={fetchStatus}
            disabled={actionLoading}
            className="p-2 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
            title="Refresh status"
          >
            <RefreshCw className={`w-4 h-4 ${actionLoading ? "animate-spin" : ""}`} />
          </button>

          {status?.running ? (
            <button
              onClick={handleStop}
              disabled={actionLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50"
            >
              <Square className="w-3.5 h-3.5" />
              Stop
            </button>
          ) : (
            <button
              onClick={handleStart}
              disabled={actionLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-green-600 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors disabled:opacity-50"
            >
              <Play className="w-3.5 h-3.5" />
              Start
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="flex items-start gap-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <XCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
          <p className="text-xs text-red-700 dark:text-red-300">{error}</p>
        </div>
      )}

      {status?.running && (
        <div className="grid grid-cols-2 gap-3">
          <div className="px-3 py-2 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
            <div className="flex items-center gap-1.5 text-xs text-zinc-500 mb-0.5">
              <Clock className="w-3 h-3" />
              Uptime
            </div>
            <div className="text-sm font-medium text-zinc-900 dark:text-zinc-50">
              {formatUptime(status.uptime_seconds)}
            </div>
          </div>

          <div className="px-3 py-2 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
            <div className="flex items-center gap-1.5 text-xs text-zinc-500 mb-0.5">
              <CheckCircle className="w-3 h-3" />
              Jobs Processed
            </div>
            <div className="text-sm font-medium text-zinc-900 dark:text-zinc-50">
              {status.jobs_processed ?? 0}
            </div>
          </div>
        </div>
      )}

      {status?.last_error && (
        <div className="flex items-start gap-2 px-3 py-2 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
          <XCircle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-xs font-medium text-amber-700 dark:text-amber-300">Last Error</p>
            <p className="text-xs text-amber-600 dark:text-amber-400">{status.last_error}</p>
          </div>
        </div>
      )}

      {/* Start at login toggle */}
      {schedulerStatus && schedulerStatus.platform !== "unsupported" && (
        <div className="flex items-center justify-between py-2 border-t border-zinc-100 dark:border-zinc-800">
          <div className="flex items-center gap-2">
            <Power className="w-4 h-4 text-zinc-400" />
            <div>
              <div className="text-sm text-zinc-700 dark:text-zinc-300">Start at login</div>
              <div className="text-xs text-zinc-400">
                Run background service automatically
              </div>
            </div>
          </div>
          <button
            onClick={handleSchedulerToggle}
            disabled={schedulerLoading}
            className={`w-10 h-5 rounded-full transition-colors relative ${
              schedulerStatus.enabled ? "bg-indigo-500" : "bg-zinc-300 dark:bg-zinc-600"
            } ${schedulerLoading ? "opacity-50" : ""}`}
          >
            <div className={`w-4 h-4 bg-white rounded-full shadow absolute top-0.5 transition-transform ${
              schedulerStatus.enabled ? "translate-x-5" : "translate-x-0.5"
            }`} />
          </button>
        </div>
      )}

      <div className="pt-2 border-t border-zinc-100 dark:border-zinc-800">
        <p className="text-xs text-zinc-400">
          The background service handles scheduled syncs, audit log cleanup, and other maintenance tasks.
          It runs independently of the main application.
        </p>
      </div>
    </div>
  );
}

export default function DaemonStatus() {
  return (
    <DaemonStatusErrorBoundary>
      <DaemonStatusInner />
    </DaemonStatusErrorBoundary>
  );
}
