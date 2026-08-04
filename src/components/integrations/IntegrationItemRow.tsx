import { useState } from "react";
import {
  ChevronRight,
  ChevronDown,
  ExternalLink,
  GitPullRequest,
  AlertCircle,
  GitCommit,
  MessageSquare,
  FileText,
} from "lucide-react";
import type { IntegrationCache } from "@/lib/tauri";
import { IntegrationItemDetail } from "./IntegrationItemDetail";

const typeIcons: Record<string, React.ElementType> = {
  pr: GitPullRequest,
  pull_request: GitPullRequest,
  issue: AlertCircle,
  commit: GitCommit,
  thread: MessageSquare,
  message: MessageSquare,
  default: FileText,
};

interface Props {
  item: IntegrationCache;
}

export function IntegrationItemRow({ item }: Props) {
  const [expanded, setExpanded] = useState(false);
  const Icon = typeIcons[item.external_type] || typeIcons.default;
  const data =
    typeof item.data === "string" ? JSON.parse(item.data) : item.data;

  const title = data.title || data.message || data.subject || item.external_id;

  const timeAgo = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHrs = Math.floor(diffMins / 60);
    if (diffHrs < 24) return `${diffHrs}h ago`;
    const diffDays = Math.floor(diffHrs / 24);
    return `${diffDays}d ago`;
  };

  return (
    <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 p-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors text-left"
      >
        {expanded ? (
          <ChevronDown className="w-4 h-4 text-zinc-400 flex-shrink-0" />
        ) : (
          <ChevronRight className="w-4 h-4 text-zinc-400 flex-shrink-0" />
        )}
        <Icon className="w-4 h-4 text-zinc-500 flex-shrink-0" />
        <span className="flex-1 text-sm font-medium text-zinc-900 dark:text-zinc-100 truncate">
          {title}
        </span>
        <span className="text-xs text-zinc-400 flex-shrink-0">
          {timeAgo(item.synced_at)}
        </span>
        {item.external_url && (
          <a
            href={item.external_url}
            target="_blank"
            rel="noopener noreferrer"
            onClick={(e) => e.stopPropagation()}
            className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 flex-shrink-0"
          >
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
        )}
      </button>
      {expanded && <IntegrationItemDetail item={item} />}
    </div>
  );
}
