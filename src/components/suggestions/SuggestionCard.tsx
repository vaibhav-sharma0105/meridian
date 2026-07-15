import { useState } from "react";
import { Check, X, Ban, ChevronDown, ChevronUp, AlertCircle, AlertTriangle, Info } from "lucide-react";
import type { Suggestion } from "@/lib/tauri";
import * as api from "@/lib/tauri";

interface SuggestionCardProps {
  suggestion: Suggestion;
  onAction: () => void;
}

export function SuggestionCard({ suggestion, onAction }: SuggestionCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleAccept = async () => {
    setLoading(true);
    try {
      await api.acceptSuggestion(suggestion.id);
      onAction();
    } finally {
      setLoading(false);
    }
  };

  const handleDismiss = async () => {
    setLoading(true);
    try {
      await api.dismissSuggestion(suggestion.id);
      onAction();
    } finally {
      setLoading(false);
    }
  };

  const handleStopSuggesting = async () => {
    setLoading(true);
    try {
      await api.stopSuggesting(suggestion.id, suggestion.type);
      onAction();
    } finally {
      setLoading(false);
    }
  };

  const SeverityIcon = {
    critical: AlertCircle,
    warning: AlertTriangle,
    info: Info,
  }[suggestion.severity] || Info;

  const severityColors = {
    critical: "text-red-500 bg-red-50 dark:bg-red-950/30",
    warning: "text-amber-500 bg-amber-50 dark:bg-amber-950/30",
    info: "text-blue-500 bg-blue-50 dark:bg-blue-950/30",
  };

  return (
    <div className={`rounded-lg border border-zinc-200 dark:border-zinc-700 p-3 ${severityColors[suggestion.severity]}`}>
      <div className="flex items-start gap-3">
        <SeverityIcon className="w-4 h-4 mt-0.5 flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <h4 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 line-clamp-2">
              {suggestion.title}
            </h4>
            <button
              onClick={() => setExpanded(!expanded)}
              className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
            >
              {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </button>
          </div>

          {suggestion.description && (
            <p className="text-xs text-zinc-500 dark:text-zinc-400 mt-1 line-clamp-2">
              {suggestion.description}
            </p>
          )}

          {expanded && suggestion.reasoning && (
            <div className="mt-2 p-2 bg-white/50 dark:bg-zinc-800/50 rounded text-xs text-zinc-600 dark:text-zinc-300">
              <span className="font-medium">Why: </span>
              {suggestion.reasoning}
            </div>
          )}

          <div className="flex items-center gap-2 mt-3">
            <button
              onClick={handleAccept}
              disabled={loading}
              className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-white bg-indigo-500 hover:bg-indigo-600 rounded disabled:opacity-50"
            >
              <Check className="w-3 h-3" />
              Do it
            </button>
            <button
              onClick={handleDismiss}
              disabled={loading}
              className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded disabled:opacity-50"
            >
              <X className="w-3 h-3" />
              Dismiss
            </button>
            <button
              onClick={handleStopSuggesting}
              disabled={loading}
              className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 disabled:opacity-50"
              title="Stop suggesting this type"
            >
              <Ban className="w-3 h-3" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
