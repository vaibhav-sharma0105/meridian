import { useState } from "react";
import {
  Clock,
  Check,
  X,
  AlertTriangle,
  ChevronRight,
  CheckSquare,
  Square,
} from "lucide-react";
import {
  usePendingApprovals,
  useApproveAction,
  useRejectAction,
  useBulkApprove,
  useBulkReject,
} from "@/hooks/useGovernance";
import type { PendingApproval, RiskLevel } from "@/lib/tauri";

const RISK_COLORS: Record<RiskLevel, string> = {
  low: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
  medium: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
  high: "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400",
  critical: "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
};

function RiskBadge({ level }: { level: string }) {
  const riskLevel = level as RiskLevel;
  return (
    <span
      className={`px-1.5 py-0.5 text-[10px] font-medium uppercase rounded ${
        RISK_COLORS[riskLevel] || RISK_COLORS.medium
      }`}
    >
      {level}
    </span>
  );
}

function TimeRemaining({ timeoutAt }: { timeoutAt: string | null }) {
  if (!timeoutAt) return null;

  const timeout = new Date(timeoutAt);
  const now = new Date();
  const diffMs = timeout.getTime() - now.getTime();

  if (diffMs <= 0) return <span className="text-red-500 text-xs">Expired</span>;

  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);

  if (diffHours > 0) {
    return (
      <span className="text-zinc-500 text-xs flex items-center gap-1">
        <Clock className="w-3 h-3" />
        {diffHours}h {diffMins % 60}m
      </span>
    );
  }

  return (
    <span
      className={`text-xs flex items-center gap-1 ${
        diffMins < 10 ? "text-orange-500" : "text-zinc-500"
      }`}
    >
      <Clock className="w-3 h-3" />
      {diffMins}m
    </span>
  );
}

interface ApprovalItemProps {
  approval: PendingApproval;
  selected: boolean;
  onSelect: () => void;
  onApprove: () => void;
  onReject: () => void;
  expanded: boolean;
  onToggleExpand: () => void;
}

function ApprovalItem({
  approval,
  selected,
  onSelect,
  onApprove,
  onReject,
  expanded,
  onToggleExpand,
}: ApprovalItemProps) {
  const [rejectReason, setRejectReason] = useState("");
  const [showRejectInput, setShowRejectInput] = useState(false);

  let actionConfig: Record<string, unknown> = {};
  try {
    actionConfig = JSON.parse(approval.action_config);
  } catch {
    // ignore
  }

  const handleReject = () => {
    if (showRejectInput && rejectReason.trim()) {
      onReject();
      setShowRejectInput(false);
      setRejectReason("");
    } else {
      setShowRejectInput(true);
    }
  };

  return (
    <div
      className={`border rounded-lg transition-colors ${
        selected
          ? "border-indigo-300 dark:border-indigo-600 bg-indigo-50/50 dark:bg-indigo-900/10"
          : "border-zinc-200 dark:border-zinc-700"
      }`}
    >
      <div className="flex items-center gap-3 p-3">
        <button
          onClick={onSelect}
          className="flex-shrink-0 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
        >
          {selected ? (
            <CheckSquare className="w-4 h-4 text-indigo-500" />
          ) : (
            <Square className="w-4 h-4" />
          )}
        </button>

        <button
          onClick={onToggleExpand}
          className="flex-1 flex items-center gap-2 text-left"
        >
          <ChevronRight
            className={`w-4 h-4 text-zinc-400 transition-transform ${
              expanded ? "rotate-90" : ""
            }`}
          />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100 truncate">
                {approval.action_type.replace(/_/g, " ")}
              </span>
              <RiskBadge level={approval.risk_level} />
            </div>
            <div className="flex items-center gap-2 text-xs text-zinc-500 mt-0.5">
              {approval.source_type && (
                <span>
                  {approval.source_type}: {approval.source_id?.slice(0, 8)}
                </span>
              )}
              <TimeRemaining timeoutAt={approval.timeout_at} />
            </div>
          </div>
        </button>

        <div className="flex items-center gap-1.5">
          <button
            onClick={onApprove}
            className="p-1.5 rounded-md bg-green-100 hover:bg-green-200 dark:bg-green-900/30 dark:hover:bg-green-900/50 text-green-600 dark:text-green-400 transition-colors"
            title="Approve"
          >
            <Check className="w-4 h-4" />
          </button>
          <button
            onClick={handleReject}
            className="p-1.5 rounded-md bg-red-100 hover:bg-red-200 dark:bg-red-900/30 dark:hover:bg-red-900/50 text-red-600 dark:text-red-400 transition-colors"
            title="Reject"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {expanded && (
        <div className="px-3 pb-3 pt-0 border-t border-zinc-100 dark:border-zinc-800">
          <div className="mt-2">
            <div className="text-xs font-medium text-zinc-500 mb-1">Action Config</div>
            <pre className="text-xs bg-zinc-50 dark:bg-zinc-800 p-2 rounded overflow-x-auto">
              {JSON.stringify(actionConfig, null, 2)}
            </pre>
          </div>
          {approval.context && (
            <div className="mt-2">
              <div className="text-xs font-medium text-zinc-500 mb-1">Context</div>
              <div className="text-xs text-zinc-600 dark:text-zinc-400">
                {approval.context}
              </div>
            </div>
          )}
        </div>
      )}

      {showRejectInput && (
        <div className="px-3 pb-3 border-t border-zinc-100 dark:border-zinc-800">
          <input
            type="text"
            value={rejectReason}
            onChange={(e) => setRejectReason(e.target.value)}
            placeholder="Rejection reason (optional)"
            className="w-full mt-2 px-2 py-1.5 text-sm border border-zinc-200 dark:border-zinc-700 rounded bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") handleReject();
              if (e.key === "Escape") setShowRejectInput(false);
            }}
          />
        </div>
      )}
    </div>
  );
}

interface ApprovalQueueProps {
  className?: string;
}

export function ApprovalQueue({ className }: ApprovalQueueProps) {
  const { data: approvals = [], isLoading } = usePendingApprovals("pending");
  const approveAction = useApproveAction();
  const rejectAction = useRejectAction();
  const bulkApprove = useBulkApprove();
  const bulkReject = useBulkReject();

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const toggleSelect = (id: string) => {
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    setSelectedIds(newSet);
  };

  const selectAll = () => {
    if (selectedIds.size === approvals.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(approvals.map((a) => a.id)));
    }
  };

  const handleBulkApprove = () => {
    bulkApprove.mutate(Array.from(selectedIds));
    setSelectedIds(new Set());
  };

  const handleBulkReject = () => {
    bulkReject.mutate({ ids: Array.from(selectedIds) });
    setSelectedIds(new Set());
  };

  if (isLoading) {
    return (
      <div className={`${className} flex items-center justify-center py-8`}>
        <div className="text-sm text-zinc-500">Loading approvals...</div>
      </div>
    );
  }

  if (approvals.length === 0) {
    return (
      <div className={`${className} flex flex-col items-center justify-center py-12`}>
        <Check className="w-8 h-8 text-green-500 mb-2" />
        <div className="text-sm text-zinc-500">No pending approvals</div>
        <div className="text-xs text-zinc-400 mt-1">
          Actions that need your review will appear here
        </div>
      </div>
    );
  }

  return (
    <div className={className}>
      {selectedIds.size > 0 && (
        <div className="flex items-center justify-between mb-3 p-2 bg-zinc-50 dark:bg-zinc-800 rounded-lg">
          <span className="text-sm text-zinc-600 dark:text-zinc-400">
            {selectedIds.size} selected
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={handleBulkApprove}
              disabled={bulkApprove.isPending}
              className="px-2.5 py-1 text-xs font-medium bg-green-500 hover:bg-green-600 text-white rounded transition-colors disabled:opacity-50"
            >
              Approve all
            </button>
            <button
              onClick={handleBulkReject}
              disabled={bulkReject.isPending}
              className="px-2.5 py-1 text-xs font-medium bg-red-500 hover:bg-red-600 text-white rounded transition-colors disabled:opacity-50"
            >
              Reject all
            </button>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between mb-2">
        <button
          onClick={selectAll}
          className="text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
        >
          {selectedIds.size === approvals.length ? "Deselect all" : "Select all"}
        </button>
        <span className="text-xs text-zinc-400">
          {approvals.length} pending
        </span>
      </div>

      <div className="space-y-2">
        {approvals.map((approval) => (
          <ApprovalItem
            key={approval.id}
            approval={approval}
            selected={selectedIds.has(approval.id)}
            onSelect={() => toggleSelect(approval.id)}
            onApprove={() => approveAction.mutate(approval.id)}
            onReject={() => rejectAction.mutate({ id: approval.id })}
            expanded={expandedId === approval.id}
            onToggleExpand={() =>
              setExpandedId(expandedId === approval.id ? null : approval.id)
            }
          />
        ))}
      </div>

      {approvals.some((a) => a.risk_level === "critical") && (
        <div className="mt-4 flex items-start gap-2 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-100 dark:border-red-900/50">
          <AlertTriangle className="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
          <p className="text-xs text-red-600 dark:text-red-400">
            Some actions are marked as critical risk. Please review carefully before
            approving.
          </p>
        </div>
      )}
    </div>
  );
}
