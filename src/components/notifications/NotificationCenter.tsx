import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Bell, CheckCheck, AlertTriangle } from "lucide-react";
import { useNotificationStore } from "@/stores/notificationStore";
import { useProjectStore } from "@/stores/projectStore";
import { usePendingImports } from "@/hooks/usePendingImports";
import PendingImportCard from "./PendingImportCard";
import { SuggestionsList } from "@/components/suggestions/SuggestionsList";
import { SkillApprovalModal } from "@/components/skills/SkillApprovalModal";
import { format } from "date-fns";
import { getSkillRun } from "@/lib/tauri";
import type { SkillRun } from "@/lib/tauri";

interface Props {
  open: boolean;
  onClose: () => void;
}

export default function NotificationCenter({ open, onClose }: Props) {
  const { t } = useTranslation();
  const { notifications, markAllRead, dismiss, markRead } = useNotificationStore();
  const { pendingImports, approveImport, dismissImport } = usePendingImports();
  const { projects } = useProjectStore();
  const [approvalRun, setApprovalRun] = useState<SkillRun | null>(null);

  const handleNotificationClick = async (notif: typeof notifications[0]) => {
    if (notif.type === "skill_approval_needed" && notif.task_id) {
      try {
        const run = await getSkillRun(notif.task_id);
        if (run.status === "approval_pending") {
          setApprovalRun(run);
          markRead(notif.id);
        }
      } catch (err) {
        console.error("Failed to load skill run:", err);
      }
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-40 flex justify-end" onClick={onClose}>
      <div
        className="relative w-80 h-full bg-white dark:bg-zinc-900 shadow-2xl border-l border-zinc-200 dark:border-zinc-800 flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-100 dark:border-zinc-800">
          <div className="flex items-center gap-2">
            <Bell className="w-4 h-4 text-zinc-500" />
            <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-50">{t("notifications.title")}</span>
          </div>
          <div className="flex items-center gap-1">
            {notifications.some((n) => !n.is_read) && (
              <button
                onClick={markAllRead}
                className="p-1.5 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300"
                title={t("notifications.markAllRead")}
              >
                <CheckCheck className="w-4 h-4" />
              </button>
            )}
            <button onClick={onClose} className="p-1.5 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300">
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* ── Pending imports section ── */}
        {pendingImports.length > 0 && (
          <div className={`border-b border-zinc-200 dark:border-zinc-800 overflow-y-auto ${notifications.length === 0 ? "flex-1" : "flex-shrink-0 max-h-[40vh]"}`}>
            <div className="px-4 py-2 bg-indigo-50 dark:bg-indigo-900/20 sticky top-0 z-10">
              <span className="text-xs font-semibold text-indigo-600 dark:text-indigo-400 uppercase tracking-wide">
                {pendingImports.length} New Meeting
                {pendingImports.length > 1 ? "s" : ""} Found
              </span>
            </div>
            {pendingImports.map((pi) => (
              <PendingImportCard
                key={pi.id}
                import={pi}
                projects={projects}
                onApprove={async (id, projectId, type) => {
                  await approveImport({
                    pending_import_id: id,
                    project_id: projectId,
                    import_type: type,
                  });
                }}
                onDismiss={async (id) => {
                  await dismissImport(id);
                }}
              />
            ))}
          </div>
        )}

        {/* ── Suggestions section ── */}
        <div className="border-b border-zinc-200 dark:border-zinc-800">
          <SuggestionsList />
        </div>

        {notifications.length > 0 && (
          <div className="flex-1 overflow-y-auto">
            <div className="divide-y divide-zinc-100 dark:divide-zinc-800">
              {notifications.map((notif) => {
                const isApproval = notif.type === "skill_approval_needed";
                return (
                  <div
                    key={notif.id}
                    onClick={() => handleNotificationClick(notif)}
                    className={`flex gap-3 px-4 py-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors ${
                      !notif.is_read ? "bg-indigo-50/50 dark:bg-indigo-900/10" : ""
                    } ${isApproval ? "cursor-pointer" : ""}`}
                  >
                    {isApproval && (
                      <div className="flex-shrink-0 w-6 h-6 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center mt-0.5">
                        <AlertTriangle className="w-3 h-3 text-amber-600 dark:text-amber-400" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50">{notif.title}</p>
                      {notif.body && (
                        <p className="text-xs text-zinc-500 mt-0.5 line-clamp-2">{notif.body}</p>
                      )}
                      <p className="text-xs text-zinc-400 mt-1">
                        {format(new Date(notif.created_at), "MMM d, h:mm a")}
                      </p>
                      {isApproval && (
                        <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                          Click to review
                        </p>
                      )}
                    </div>
                    <button
                      onClick={(e) => { e.stopPropagation(); dismiss(notif.id); }}
                      className="flex-shrink-0 p-1 text-zinc-300 hover:text-zinc-500 dark:hover:text-zinc-400 mt-0.5"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {pendingImports.length === 0 && notifications.length === 0 && (
          <div className="flex-1 flex flex-col items-center justify-center py-16 text-center">
            <Bell className="w-8 h-8 text-zinc-300 dark:text-zinc-600 mb-3" />
            <p className="text-sm text-zinc-400">{t("notifications.empty")}</p>
          </div>
        )}
      </div>

      {approvalRun && (
        <SkillApprovalModal
          run={approvalRun}
          onClose={() => setApprovalRun(null)}
        />
      )}
    </div>
  );
}
