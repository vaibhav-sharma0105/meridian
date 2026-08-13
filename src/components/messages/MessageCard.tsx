import { useState } from "react";
import {
  FileText,
  RefreshCw,
  Zap,
  Calendar,
  Pin,
  Trash2,
  RotateCcw,
  ChevronDown,
  ChevronUp,
  ExternalLink,
} from "lucide-react";
import type { Message } from "@/lib/tauri";

interface MessageCardProps {
  message: Message;
  onDelete?: (id: string) => void;
  onRestore?: (id: string) => void;
}

// Keys mirror the Rust `MessageType` enum. A type with no entry here falls
// back to rendering its raw snake_case identifier, so keep these in sync.
const TYPE_ICONS: Record<string, React.ElementType> = {
  skill_result: Zap,
  digest: Calendar,
  pinned_chat: Pin,
  integration_sync: RefreshCw,
};

const TYPE_LABELS: Record<string, string> = {
  skill_result: "Skill Result",
  digest: "Digest",
  pinned_chat: "Pinned Chat",
  integration_sync: "Integration Sync",
};

export function MessageCard({ message, onDelete, onRestore }: MessageCardProps) {
  const [expanded, setExpanded] = useState(false);

  const Icon = TYPE_ICONS[message.message_type] || FileText;
  const typeLabel = TYPE_LABELS[message.message_type] || message.message_type;
  const isDeleted = !!message.deleted_at;

  const formattedDate = new Date(message.created_at).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });

  return (
    <div
      className={`border rounded-lg p-3 transition-colors ${
        isDeleted
          ? "border-red-200 dark:border-red-900/50 bg-red-50/50 dark:bg-red-900/10"
          : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
      }`}
    >
      <div className="flex items-start gap-3">
        <div
          className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
            isDeleted
              ? "bg-red-100 dark:bg-red-900/30"
              : "bg-zinc-100 dark:bg-zinc-800"
          }`}
        >
          <Icon
            className={`w-4 h-4 ${
              isDeleted ? "text-red-500" : "text-zinc-500"
            }`}
          />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span
              className={`text-xs px-1.5 py-0.5 rounded ${
                isDeleted
                  ? "bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400"
                  : "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400"
              }`}
            >
              {typeLabel}
            </span>
            {message.auto_pinned && (
              <Pin className="w-3 h-3 text-indigo-500" />
            )}
            <span className="text-xs text-zinc-400 ml-auto">{formattedDate}</span>
          </div>

          <h4 className="font-medium text-sm text-zinc-900 dark:text-zinc-100 mt-1 truncate">
            {message.title}
          </h4>

          {message.content && (
            <div className="mt-2">
              <p
                className={`text-xs text-zinc-500 dark:text-zinc-400 ${
                  expanded ? "" : "line-clamp-2"
                }`}
              >
                {message.content}
              </p>
              {message.content.length > 150 && (
                <button
                  onClick={() => setExpanded(!expanded)}
                  className="text-xs text-indigo-500 hover:text-indigo-600 mt-1 flex items-center gap-1"
                >
                  {expanded ? (
                    <>
                      <ChevronUp className="w-3 h-3" />
                      Show less
                    </>
                  ) : (
                    <>
                      <ChevronDown className="w-3 h-3" />
                      Show more
                    </>
                  )}
                </button>
              )}
            </div>
          )}

          {message.file_refs && message.file_refs.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {message.file_refs.map((ref, idx) => (
                <span
                  key={idx}
                  className="inline-flex items-center gap-1 text-xs bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 px-2 py-0.5 rounded"
                >
                  <FileText className="w-3 h-3" />
                  {ref.split("/").pop()}
                </span>
              ))}
            </div>
          )}

          {message.source_type && (
            <div className="mt-2 flex items-center gap-1 text-xs text-zinc-400">
              <ExternalLink className="w-3 h-3" />
              <span>
                From {message.source_type}
                {message.source_id && `: ${message.source_id.slice(0, 8)}...`}
              </span>
            </div>
          )}
        </div>

        <div className="flex flex-col gap-1">
          {isDeleted ? (
            onRestore && (
              <button
                onClick={() => onRestore(message.id)}
                className="p-1.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                title="Restore"
              >
                <RotateCcw className="w-4 h-4 text-zinc-400 hover:text-indigo-500" />
              </button>
            )
          ) : (
            onDelete && (
              <button
                onClick={() => onDelete(message.id)}
                className="p-1.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                title="Delete"
              >
                <Trash2 className="w-4 h-4 text-zinc-400 hover:text-red-500" />
              </button>
            )
          )}
        </div>
      </div>
    </div>
  );
}
