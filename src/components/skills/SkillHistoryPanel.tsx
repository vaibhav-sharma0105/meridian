import { useState } from "react";
import {
  X,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Loader2,
  ChevronLeft,
  ChevronRight,
  Filter,
} from "lucide-react";
import type { Skill, SkillRun } from "@/lib/tauri";
import { useSkillRuns, useSkillStats } from "@/hooks/useSkills";

interface SkillHistoryPanelProps {
  skill: Skill;
  onClose: () => void;
}

const statusIcons: Record<string, typeof CheckCircle> = {
  completed: CheckCircle,
  failed: XCircle,
  pending: Clock,
  running: Loader2,
  approval_pending: AlertTriangle,
  cancelled: XCircle,
};

const statusColors: Record<string, string> = {
  completed: "text-green-500",
  failed: "text-red-500",
  pending: "text-zinc-400",
  running: "text-indigo-500",
  approval_pending: "text-amber-500",
  cancelled: "text-zinc-400",
};

const STATUS_OPTIONS = [
  { value: "", label: "All" },
  { value: "completed", label: "Completed" },
  { value: "failed", label: "Failed" },
  { value: "running", label: "Running" },
  { value: "pending", label: "Pending" },
  { value: "approval_pending", label: "Awaiting Approval" },
  { value: "cancelled", label: "Cancelled" },
];

const PAGE_SIZE = 10;

function formatDuration(ms: number | null): string {
  if (!ms) return "-";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
  return d.toLocaleDateString();
}

function SkillRunCard({ run }: { run: SkillRun }) {
  const StatusIcon = statusIcons[run.status] || Clock;
  const statusColor = statusColors[run.status] || "text-zinc-400";

  return (
    <div className="border border-zinc-200 dark:border-zinc-800 rounded-lg p-3">
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2">
          <StatusIcon
            className={`w-4 h-4 ${statusColor} ${
              run.status === "running" ? "animate-spin" : ""
            }`}
          />
          <span className="text-[13px] font-medium text-zinc-900 dark:text-zinc-100 capitalize">
            {run.status.replace("_", " ")}
          </span>
        </div>
        <span className="text-[11px] text-zinc-400">
          {formatTime(run.created_at)}
        </span>
      </div>

      <div className="mt-2 flex items-center gap-3 text-[11px] text-zinc-500">
        <span className="capitalize">{run.trigger_type}</span>
        {run.duration_ms && (
          <>
            <span>·</span>
            <span>{formatDuration(run.duration_ms)}</span>
          </>
        )}
      </div>

      {run.error && (
        <div className="mt-2 p-2 bg-red-50 dark:bg-red-900/20 rounded text-[12px] text-red-600 dark:text-red-400">
          {run.error}
        </div>
      )}

      {run.output && run.status === "completed" && (
        <div className="mt-2 p-2 bg-zinc-50 dark:bg-zinc-800 rounded text-[12px] text-zinc-600 dark:text-zinc-400 line-clamp-3">
          {run.output.substring(0, 200)}
          {run.output.length > 200 && "..."}
        </div>
      )}
    </div>
  );
}

export function SkillHistoryPanel({ skill, onClose }: SkillHistoryPanelProps) {
  const [statusFilter, setStatusFilter] = useState("");
  const [page, setPage] = useState(0);

  const { data: runs = [], isLoading } = useSkillRuns(
    skill.id,
    statusFilter || undefined
  );
  const { data: stats } = useSkillStats(skill.id);

  const totalPages = Math.ceil(runs.length / PAGE_SIZE);
  const paginatedRuns = runs.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  const handleStatusChange = (status: string) => {
    setStatusFilter(status);
    setPage(0);
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-zinc-900">
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
        <div>
          <h3 className="font-medium text-zinc-900 dark:text-zinc-100">
            {skill.name}
          </h3>
          <p className="text-[12px] text-zinc-500">Run History</p>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800"
        >
          <X className="w-4 h-4 text-zinc-400" />
        </button>
      </div>

      {stats && (
        <div className="px-4 py-3 border-b border-zinc-200 dark:border-zinc-800">
          <div className="grid grid-cols-3 gap-4 text-center">
            <div>
              <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                {stats.total_runs}
              </div>
              <div className="text-[11px] text-zinc-500">Total Runs</div>
            </div>
            <div>
              <div className="text-lg font-semibold text-green-600">
                {Math.round(stats.success_rate * 100)}%
              </div>
              <div className="text-[11px] text-zinc-500">Success Rate</div>
            </div>
            <div>
              <div className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                {formatDuration(stats.avg_duration_ms)}
              </div>
              <div className="text-[11px] text-zinc-500">Avg Duration</div>
            </div>
          </div>
        </div>
      )}

      {/* Status Filter */}
      <div className="px-4 py-2 border-b border-zinc-200 dark:border-zinc-800 flex items-center gap-2">
        <Filter className="w-3.5 h-3.5 text-zinc-400" />
        <select
          value={statusFilter}
          onChange={(e) => handleStatusChange(e.target.value)}
          className="flex-1 text-[12px] bg-transparent border-none outline-none text-zinc-700 dark:text-zinc-300"
        >
          {STATUS_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        {statusFilter && (
          <button
            onClick={() => handleStatusChange("")}
            className="text-[11px] text-indigo-500 hover:text-indigo-700"
          >
            Clear
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-8 text-zinc-400">
            <Loader2 className="w-5 h-5 animate-spin" />
          </div>
        ) : paginatedRuns.length === 0 ? (
          <div className="text-center py-8 text-zinc-500 text-[13px]">
            {statusFilter ? "No runs matching filter" : "No runs yet"}
          </div>
        ) : (
          <div className="space-y-3">
            {paginatedRuns.map((run) => (
              <SkillRunCard key={run.id} run={run} />
            ))}
          </div>
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between px-4 py-2 border-t border-zinc-200 dark:border-zinc-800">
          <span className="text-[11px] text-zinc-500">
            Page {page + 1} of {totalPages}
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={page === 0}
              className="p-1 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-30"
            >
              <ChevronLeft className="w-4 h-4 text-zinc-500" />
            </button>
            <button
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              className="p-1 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-30"
            >
              <ChevronRight className="w-4 h-4 text-zinc-500" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
