import { useState, useEffect } from "react";
import {
  RefreshCw,
  CheckCircle2,
  Clock,
  AlertTriangle,
  Loader2,
  ChevronDown,
  ChevronRight,
  Activity,
} from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { getBackgroundJobs, getRecentBackgroundJobs, type BackgroundJob } from "@/lib/tauri";

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);

  if (diffSec < 60) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  return date.toLocaleDateString();
}

function JobStatusIcon({ status }: { status: string }) {
  switch (status) {
    case "running":
      return <Loader2 className="w-3.5 h-3.5 text-blue-500 animate-spin" />;
    case "completed":
      return <CheckCircle2 className="w-3.5 h-3.5 text-green-500" />;
    case "failed":
      return <AlertTriangle className="w-3.5 h-3.5 text-red-500" />;
    case "pending":
    default:
      return <Clock className="w-3.5 h-3.5 text-zinc-400" />;
  }
}

function JobRow({ job }: { job: BackgroundJob }) {
  return (
    <div className="flex items-center justify-between px-3 py-2 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 rounded-md transition-colors">
      <div className="flex items-center gap-2 min-w-0">
        <JobStatusIcon status={job.status} />
        <div className="min-w-0">
          <div className="text-sm text-zinc-900 dark:text-zinc-100 truncate">
            {job.description}
          </div>
          <div className="text-xs text-zinc-500 truncate">
            {job.job_type}
          </div>
        </div>
      </div>
      <div className="flex items-center gap-2 flex-shrink-0 ml-2">
        <span
          className={`px-1.5 py-0.5 text-xs font-medium rounded ${
            job.status === "running"
              ? "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
              : job.status === "completed"
              ? "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300"
              : job.status === "failed"
              ? "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300"
              : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400"
          }`}
        >
          {job.status}
        </span>
        <span className="text-xs text-zinc-400">
          {formatRelativeTime(job.started_at || job.created_at)}
        </span>
      </div>
    </div>
  );
}

export function BackgroundJobsPanel() {
  const [expanded, setExpanded] = useState(true);
  const [showRecent, setShowRecent] = useState(false);

  // Active jobs (pending + running)
  const { data: activeJobs = [], isLoading: loadingActive, refetch: refetchActive } = useQuery({
    queryKey: ["background-jobs", "active"],
    queryFn: () => getBackgroundJobs(20),
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  // Recent jobs (including completed/failed)
  const { data: recentJobs = [], isLoading: loadingRecent, refetch: refetchRecent } = useQuery({
    queryKey: ["background-jobs", "recent"],
    queryFn: () => getRecentBackgroundJobs(20),
    enabled: showRecent,
    refetchInterval: showRecent ? 10000 : false,
  });

  const handleRefresh = () => {
    refetchActive();
    if (showRecent) refetchRecent();
  };

  const runningCount = activeJobs.filter((j) => j.status === "running").length;
  const pendingCount = activeJobs.filter((j) => j.status === "pending").length;

  return (
    <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between p-4 bg-zinc-50 dark:bg-zinc-800/50 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
      >
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 flex items-center justify-center">
            <Activity className="w-5 h-5 text-indigo-500" />
          </div>
          <div className="text-left">
            <div className="font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
              Background Processes
              {runningCount > 0 && (
                <span className="px-1.5 py-0.5 text-xs font-medium bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded flex items-center gap-1">
                  <Loader2 className="w-3 h-3 animate-spin" />
                  {runningCount} running
                </span>
              )}
              {pendingCount > 0 && (
                <span className="px-1.5 py-0.5 text-xs font-medium bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 rounded">
                  {pendingCount} pending
                </span>
              )}
            </div>
            <div className="text-xs text-zinc-500 mt-0.5">
              Integration syncs, embedding jobs, skill executions
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleRefresh();
            }}
            className="p-1.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded transition-colors"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${loadingActive ? "animate-spin" : ""}`} />
          </button>
          {expanded ? (
            <ChevronDown className="w-5 h-5 text-zinc-400" />
          ) : (
            <ChevronRight className="w-5 h-5 text-zinc-400" />
          )}
        </div>
      </button>

      {/* Content */}
      {expanded && (
        <div className="p-4 border-t border-zinc-200 dark:border-zinc-700">
          {/* Toggle between active and recent */}
          <div className="flex items-center gap-2 mb-3">
            <button
              onClick={() => setShowRecent(false)}
              className={`px-2.5 py-1 text-xs font-medium rounded transition-colors ${
                !showRecent
                  ? "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300"
                  : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
              }`}
            >
              Active ({activeJobs.length})
            </button>
            <button
              onClick={() => setShowRecent(true)}
              className={`px-2.5 py-1 text-xs font-medium rounded transition-colors ${
                showRecent
                  ? "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300"
                  : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
              }`}
            >
              Recent
            </button>
          </div>

          {/* Job list */}
          <div className="space-y-1 max-h-64 overflow-y-auto">
            {loadingActive && !showRecent ? (
              <div className="flex items-center justify-center py-6 text-zinc-400">
                <Loader2 className="w-5 h-5 animate-spin mr-2" />
                Loading...
              </div>
            ) : loadingRecent && showRecent ? (
              <div className="flex items-center justify-center py-6 text-zinc-400">
                <Loader2 className="w-5 h-5 animate-spin mr-2" />
                Loading...
              </div>
            ) : !showRecent && activeJobs.length === 0 ? (
              <div className="text-center py-6 text-zinc-500 text-sm">
                No active background processes
              </div>
            ) : showRecent && recentJobs.length === 0 ? (
              <div className="text-center py-6 text-zinc-500 text-sm">
                No recent jobs
              </div>
            ) : (
              (showRecent ? recentJobs : activeJobs).map((job) => (
                <JobRow key={job.id} job={job} />
              ))
            )}
          </div>

          {/* Info text */}
          <div className="mt-3 pt-3 border-t border-zinc-100 dark:border-zinc-800">
            <p className="text-xs text-zinc-400">
              Jobs include: document embedding, integration syncs, skill executions, pattern analysis, and suggestion generation.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
