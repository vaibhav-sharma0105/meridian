import { useEffect, useRef, useState } from "react";
import { Inbox, ChevronLeft, ChevronRight, Settings2 } from "lucide-react";
import {
  useMessages,
  useDeleteMessage,
  useRestoreMessage,
} from "@/hooks/useMessages";
import type { MessageFilters } from "@/lib/tauri";
import { MessageCard } from "./MessageCard";
import { MessageFiltersBar } from "./MessageFilters";
import { StorageUsageBar } from "./StorageUsageBar";
import { MessageCenterSettings } from "./MessageCenterSettings";
import { useUIStore } from "@/stores/uiStore";

interface MessageCenterViewProps {
  projectId?: string;
}

export function MessageCenterView({ projectId }: MessageCenterViewProps) {
  const [filters, setFilters] = useState<MessageFilters>({
    project_id: projectId,
  });
  const [page, setPage] = useState(1);
  const [showSettings, setShowSettings] = useState(false);
  const perPage = 20;

  // Set when the user arrives via a notification's "View full result" link.
  const focusedMessageId = useUIStore((st) => st.focusedMessageId);
  const setFocusedMessageId = useUIStore((st) => st.setFocusedMessageId);
  const focusedRef = useRef<HTMLDivElement | null>(null);

  const { data, isLoading, error } = useMessages(filters, page, perPage);
  const deleteMessage = useDeleteMessage();
  const restoreMessage = useRestoreMessage();

  const handleFiltersChange = (newFilters: MessageFilters) => {
    setFilters({ ...newFilters, project_id: projectId });
    setPage(1);
  };

  const totalPages = data ? Math.ceil(data.total / perPage) : 0;

  // Scroll the deep-linked message into view once it renders, then clear the
  // marker so the highlight does not persist across later visits.
  useEffect(() => {
    if (!focusedMessageId || !focusedRef.current) return;
    focusedRef.current.scrollIntoView({ block: "center", behavior: "smooth" });
    const timer = setTimeout(() => setFocusedMessageId(null), 3000);
    return () => clearTimeout(timer);
  }, [focusedMessageId, data, setFocusedMessageId]);

  return (
    <div className="flex flex-col h-full bg-white dark:bg-zinc-900">
      <div className="px-4 py-3 border-b border-zinc-200 dark:border-zinc-700 flex items-start justify-between">
        <div>
          <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
            Message Center
          </h2>
          <p className="text-sm text-zinc-500 mt-0.5">
            Skill results, digests, and saved conversations
          </p>
        </div>
        <button
          onClick={() => setShowSettings((s) => !s)}
          aria-label="Message Center settings"
          aria-pressed={showSettings}
          className={`p-1.5 rounded-lg transition-colors ${
            showSettings
              ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300"
              : "text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800"
          }`}
        >
          <Settings2 className="w-4 h-4" />
        </button>
      </div>

      {showSettings && <MessageCenterSettings />}

      <MessageFiltersBar filters={filters} onChange={handleFiltersChange} />

      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="w-8 h-8 border-2 border-zinc-200 border-t-indigo-500 rounded-full animate-spin" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
            <p className="text-sm text-red-500">Failed to load messages</p>
          </div>
        ) : data?.messages.length === 0 ? (
          <EmptyState hasFilters={!!filters.search || !!filters.message_type} />
        ) : (
          <div className="p-4 space-y-3">
            {data?.messages.map((message) => {
              const isFocused = message.id === focusedMessageId;
              return (
                <div
                  key={message.id}
                  ref={isFocused ? focusedRef : undefined}
                  className={
                    isFocused
                      ? "rounded-lg ring-2 ring-indigo-500 ring-offset-2 dark:ring-offset-zinc-900"
                      : undefined
                  }
                >
                  <MessageCard
                    message={message}
                    onDelete={(id) => deleteMessage.mutate(id)}
                    onRestore={(id) => restoreMessage.mutate(id)}
                  />
                </div>
              );
            })}
          </div>
        )}
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-between px-4 py-2 border-t border-zinc-200 dark:border-zinc-700">
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
            Previous
          </button>

          <span className="text-sm text-zinc-500">
            Page {page} of {totalPages}
          </span>

          <button
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            Next
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}

      <StorageUsageBar />
    </div>
  );
}

function EmptyState({ hasFilters }: { hasFilters: boolean }) {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <div className="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mb-4">
        <Inbox className="w-6 h-6 text-zinc-400" />
      </div>
      <h3 className="text-lg font-semibold text-zinc-700 dark:text-zinc-300 mb-2">
        {hasFilters ? "No messages match filters" : "No messages yet"}
      </h3>
      <p className="text-sm text-zinc-500 max-w-sm">
        {hasFilters
          ? "Try adjusting your search or filter criteria."
          : "Skill results, digests, and pinned AI conversations will appear here."}
      </p>
    </div>
  );
}
