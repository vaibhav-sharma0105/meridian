import { useState } from "react";
import {
  X,
  ExternalLink,
  AlertCircle,
  AlertTriangle,
  Info,
  CheckSquare,
  Github,
  MessageSquare,
  Calendar,
  Shield,
} from "lucide-react";
import type { AttentionItem as AttentionItemType } from "@/lib/tauri";

interface AttentionItemProps {
  item: AttentionItemType;
  onDismiss: (id: string) => void;
}

const SOURCE_ICONS: Record<string, React.ElementType> = {
  task: CheckSquare,
  approval: Shield,
  github: Github,
  jira: Calendar,
  slack: MessageSquare,
  integration_cache: ExternalLink,
};

const SEVERITY_CONFIG = {
  critical: {
    icon: AlertCircle,
    bg: "bg-red-50 dark:bg-red-950/30",
    border: "border-red-200 dark:border-red-800",
    iconColor: "text-red-500",
  },
  warning: {
    icon: AlertTriangle,
    bg: "bg-amber-50 dark:bg-amber-950/30",
    border: "border-amber-200 dark:border-amber-800",
    iconColor: "text-amber-500",
  },
  info: {
    icon: Info,
    bg: "bg-blue-50 dark:bg-blue-950/30",
    border: "border-blue-200 dark:border-blue-800",
    iconColor: "text-blue-500",
  },
};

export function AttentionItem({ item, onDismiss }: AttentionItemProps) {
  const [dismissing, setDismissing] = useState(false);

  const handleDismiss = async () => {
    setDismissing(true);
    try {
      await onDismiss(item.id);
    } finally {
      setDismissing(false);
    }
  };

  const severityConfig =
    SEVERITY_CONFIG[item.severity as keyof typeof SEVERITY_CONFIG] ||
    SEVERITY_CONFIG.info;
  const SeverityIcon = severityConfig.icon;
  const SourceIcon = SOURCE_ICONS[item.source_type] || ExternalLink;

  const formatCategory = (category: string) => {
    return category
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  };

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
    <div
      className={`rounded-lg border p-3 ${severityConfig.bg} ${severityConfig.border}`}
    >
      <div className="flex items-start gap-3">
        <SeverityIcon
          className={`w-4 h-4 mt-0.5 flex-shrink-0 ${severityConfig.iconColor}`}
        />

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <SourceIcon className="w-3.5 h-3.5 text-zinc-400" />
            <span className="text-xs font-medium text-zinc-500 uppercase">
              {item.source_type}
            </span>
            <span className="text-xs text-zinc-400">·</span>
            <span className="text-xs text-zinc-400">
              {formatCategory(item.category)}
            </span>
          </div>

          {item.reason_text && (
            <p className="text-sm text-zinc-700 dark:text-zinc-300 line-clamp-2">
              {item.reason_text}
            </p>
          )}

          <p className="text-xs text-zinc-400 mt-1">
            {timeAgo(item.computed_at)}
          </p>
        </div>

        <button
          onClick={handleDismiss}
          disabled={dismissing}
          className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors disabled:opacity-50"
          title="Dismiss"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
