import { useState, useEffect } from "react";
import {
  MessageSquare,
  Clock,
  X,
  Send,
  Trash2,
  Hash,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
} from "lucide-react";
import { format, formatDistanceToNow } from "date-fns";
import toast from "react-hot-toast";

interface SlackDraft {
  id: string;
  channel_id: string;
  channel_name: string;
  text: string;
  created_at: string;
  send_at: string | null;
  status: "pending" | "scheduled" | "sent" | "cancelled";
}

interface SlackDraftsPanelProps {
  integrationId: string;
  onClose: () => void;
}

export function SlackDraftsPanel({ integrationId, onClose }: SlackDraftsPanelProps) {
  const [drafts, setDrafts] = useState<SlackDraft[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadDrafts();
  }, [integrationId]);

  const loadDrafts = async () => {
    setIsLoading(true);
    try {
      // In a full implementation, this would call:
      // const result = await api.getSlackDrafts(integrationId);
      // For now, show empty state
      setDrafts([]);
    } catch (e) {
      console.error("Failed to load drafts:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSendNow = async (draftId: string) => {
    try {
      // In a full implementation: await api.sendSlackDraftNow(draftId);
      toast.success("Draft sent!");
      setDrafts((prev) =>
        prev.map((d) => (d.id === draftId ? { ...d, status: "sent" as const } : d))
      );
    } catch (e) {
      toast.error("Failed to send draft");
    }
  };

  const handleCancel = async (draftId: string) => {
    try {
      // In a full implementation: await api.cancelSlackDraft(draftId);
      setDrafts((prev) =>
        prev.map((d) => (d.id === draftId ? { ...d, status: "cancelled" as const } : d))
      );
      toast.success("Draft cancelled");
    } catch (e) {
      toast.error("Failed to cancel draft");
    }
  };

  const pendingDrafts = drafts.filter((d) => d.status === "pending" || d.status === "scheduled");
  const recentDrafts = drafts.filter((d) => d.status === "sent" || d.status === "cancelled");

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center">
              <MessageSquare className="w-5 h-5 text-purple-600 dark:text-purple-400" />
            </div>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Slack Draft Queue
              </h3>
              <p className="text-xs text-zinc-500">
                Review and manage pending messages
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

        <div className="p-6 space-y-6 max-h-[60vh] overflow-y-auto">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <RefreshCw className="w-5 h-5 animate-spin text-zinc-400" />
            </div>
          ) : pendingDrafts.length === 0 && recentDrafts.length === 0 ? (
            <div className="text-center py-8">
              <MessageSquare className="w-10 h-10 mx-auto text-zinc-300 dark:text-zinc-600 mb-3" />
              <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                No drafts
              </h4>
              <p className="text-xs text-zinc-500">
                When Meridian creates Slack message drafts, they'll appear here for review.
              </p>
            </div>
          ) : (
            <>
              {/* Pending Drafts */}
              {pendingDrafts.length > 0 && (
                <div>
                  <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3">
                    Pending ({pendingDrafts.length})
                  </h4>
                  <div className="space-y-3">
                    {pendingDrafts.map((draft) => (
                      <DraftCard
                        key={draft.id}
                        draft={draft}
                        onSendNow={() => handleSendNow(draft.id)}
                        onCancel={() => handleCancel(draft.id)}
                      />
                    ))}
                  </div>
                </div>
              )}

              {/* Recent Drafts */}
              {recentDrafts.length > 0 && (
                <div>
                  <h4 className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3">
                    Recent ({recentDrafts.length})
                  </h4>
                  <div className="space-y-2">
                    {recentDrafts.slice(0, 5).map((draft) => (
                      <div
                        key={draft.id}
                        className="flex items-center gap-3 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg opacity-60"
                      >
                        <div className="flex-shrink-0">
                          {draft.status === "sent" ? (
                            <CheckCircle2 className="w-4 h-4 text-green-500" />
                          ) : (
                            <X className="w-4 h-4 text-zinc-400" />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-1.5 text-xs text-zinc-500 mb-1">
                            <Hash className="w-3 h-3" />
                            {draft.channel_name}
                          </div>
                          <p className="text-sm text-zinc-700 dark:text-zinc-300 truncate">
                            {draft.text}
                          </p>
                        </div>
                        <span className="text-xs text-zinc-400">
                          {formatDistanceToNow(new Date(draft.created_at), { addSuffix: true })}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          <div className="text-xs text-zinc-500">
            Drafts are held for review before sending
          </div>
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

function DraftCard({
  draft,
  onSendNow,
  onCancel,
}: {
  draft: SlackDraft;
  onSendNow: () => void;
  onCancel: () => void;
}) {
  const isScheduled = draft.status === "scheduled" && draft.send_at;
  const sendTime = isScheduled ? new Date(draft.send_at!) : null;

  return (
    <div className="p-4 border border-zinc-200 dark:border-zinc-700 rounded-lg">
      {/* Channel */}
      <div className="flex items-center gap-1.5 text-xs text-zinc-500 mb-2">
        <Hash className="w-3 h-3" />
        {draft.channel_name}
        {isScheduled && sendTime && (
          <span className="flex items-center gap-1 ml-2 text-amber-600 dark:text-amber-400">
            <Clock className="w-3 h-3" />
            Sends {format(sendTime, "h:mm a")}
          </span>
        )}
      </div>

      {/* Message Preview */}
      <p className="text-sm text-zinc-700 dark:text-zinc-300 mb-3 line-clamp-3">
        {draft.text}
      </p>

      {/* Actions */}
      <div className="flex items-center gap-2">
        <button
          onClick={onSendNow}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-green-600 hover:bg-green-700 text-white rounded transition-colors"
        >
          <Send className="w-3 h-3" />
          Send Now
        </button>
        <button
          onClick={onCancel}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
        >
          <Trash2 className="w-3 h-3" />
          Cancel
        </button>
        <span className="ml-auto text-xs text-zinc-400">
          Created {formatDistanceToNow(new Date(draft.created_at), { addSuffix: true })}
        </span>
      </div>
    </div>
  );
}
