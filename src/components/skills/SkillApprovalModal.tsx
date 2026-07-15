import { useState } from "react";
import { X, CheckCircle, XCircle, AlertTriangle, ListTodo } from "lucide-react";
import type { SkillRun } from "@/lib/tauri";
import { useSkill, useApproveSkillRun, useRejectSkillRun } from "@/hooks/useSkills";

interface SkillApprovalModalProps {
  run: SkillRun;
  onClose: () => void;
}

interface PendingChanges {
  type: string;
  tasks?: Array<{ title: string; priority?: string; assignee?: string }>;
  message?: string;
}

export function SkillApprovalModal({ run, onClose }: SkillApprovalModalProps) {
  const [showRejectReason, setShowRejectReason] = useState(false);
  const [rejectReason, setRejectReason] = useState("");

  const { data: skill } = useSkill(run.skill_id);
  const approveRun = useApproveSkillRun();
  const rejectRun = useRejectSkillRun();

  const pendingChanges: PendingChanges | null = run.pending_changes
    ? JSON.parse(run.pending_changes)
    : null;

  const handleApprove = async () => {
    try {
      await approveRun.mutateAsync({ runId: run.id });
      onClose();
    } catch (err) {
      console.error("Failed to approve:", err);
    }
  };

  const handleReject = async () => {
    try {
      await rejectRun.mutateAsync({
        runId: run.id,
        reason: rejectReason || undefined,
      });
      onClose();
    } catch (err) {
      console.error("Failed to reject:", err);
    }
  };

  const isPending = approveRun.isPending || rejectRun.isPending;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
              <AlertTriangle className="w-4 h-4 text-amber-600 dark:text-amber-400" />
            </div>
            <div>
              <h2 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Approval Required
              </h2>
              <p className="text-[12px] text-zinc-500">
                {skill?.name || "Skill"} wants to make changes
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-5 h-5 text-zinc-400" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {pendingChanges?.type === "create_tasks" && pendingChanges.tasks && (
            <div>
              <div className="flex items-center gap-2 mb-3">
                <ListTodo className="w-4 h-4 text-zinc-500" />
                <span className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">
                  {pendingChanges.tasks.length} task(s) will be created
                </span>
              </div>
              <div className="space-y-2">
                {pendingChanges.tasks.map((task, i) => (
                  <div
                    key={i}
                    className="p-3 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700"
                  >
                    <div className="font-medium text-[13px] text-zinc-900 dark:text-zinc-100">
                      {task.title}
                    </div>
                    {(task.priority || task.assignee) && (
                      <div className="flex items-center gap-2 mt-1 text-[11px] text-zinc-500">
                        {task.priority && (
                          <span className="capitalize">{task.priority}</span>
                        )}
                        {task.priority && task.assignee && <span>·</span>}
                        {task.assignee && <span>{task.assignee}</span>}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {pendingChanges?.type === "send_message" && (
            <div>
              <div className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                Message to send
              </div>
              <div className="p-3 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 text-[13px] text-zinc-600 dark:text-zinc-400 whitespace-pre-wrap">
                {pendingChanges.message || "No preview available"}
              </div>
            </div>
          )}

          {!pendingChanges && (
            <div className="text-center py-8 text-zinc-500">
              No pending changes to preview
            </div>
          )}

          {showRejectReason && (
            <div className="mt-4 pt-4 border-t border-zinc-200 dark:border-zinc-800">
              <label className="block text-[13px] font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                Rejection reason (optional)
              </label>
              <textarea
                value={rejectReason}
                onChange={(e) => setRejectReason(e.target.value)}
                placeholder="Why are you rejecting this?"
                rows={3}
                className="w-full px-3 py-2 text-[13px] border border-zinc-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-800 focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none"
              />
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          {!showRejectReason ? (
            <>
              <button
                onClick={() => setShowRejectReason(true)}
                disabled={isPending}
                className="flex items-center gap-1.5 px-4 py-2 text-[13px] text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50"
              >
                <XCircle className="w-4 h-4" />
                Reject
              </button>
              <button
                onClick={handleApprove}
                disabled={isPending}
                className="flex items-center gap-1.5 px-4 py-2 text-[13px] bg-green-500 hover:bg-green-600 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                <CheckCircle className="w-4 h-4" />
                {isPending ? "Approving..." : "Approve"}
              </button>
            </>
          ) : (
            <>
              <button
                onClick={() => setShowRejectReason(false)}
                disabled={isPending}
                className="px-4 py-2 text-[13px] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
              >
                Back
              </button>
              <button
                onClick={handleReject}
                disabled={isPending}
                className="flex items-center gap-1.5 px-4 py-2 text-[13px] bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                <XCircle className="w-4 h-4" />
                {isPending ? "Rejecting..." : "Confirm Reject"}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
