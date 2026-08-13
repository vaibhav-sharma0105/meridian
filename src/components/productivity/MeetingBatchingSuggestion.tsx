import { useState } from "react";
import { CalendarClock, X } from "lucide-react";
import {
  useMeetingBatchingSuggestion,
  formatHourRange,
} from "@/hooks/useProductivity";

/**
 * Surfaces the backend's batching advice above the meetings list. Renders
 * nothing when the day's schedule is not fragmented, so it is safe to mount
 * unconditionally.
 */
export function MeetingBatchingSuggestion() {
  const { data: suggestion, isLoading, error } = useMeetingBatchingSuggestion();
  const [dismissed, setDismissed] = useState(false);

  if (dismissed || isLoading || error || !suggestion) {
    return null;
  }

  const hasBlock =
    Array.isArray(suggestion.suggested_block) &&
    suggestion.suggested_block.length > 0;

  return (
    <div className="flex items-start gap-2 p-3 bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-100 dark:border-indigo-800 rounded-lg">
      <CalendarClock className="w-4 h-4 text-indigo-500 mt-0.5 flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <p className="text-[12.5px] text-indigo-700 dark:text-indigo-300 leading-relaxed">
          {suggestion.message}
        </p>
        <div className="flex items-center gap-3 mt-1 text-[11px] text-indigo-500 dark:text-indigo-400">
          {hasBlock && (
            <span>
              Suggested block:{" "}
              <span className="font-medium">
                {formatHourRange(suggestion.suggested_block as number[])}
              </span>
            </span>
          )}
          {suggestion.freed_hours > 0 && (
            <span>
              Frees{" "}
              <span className="font-medium">
                {suggestion.freed_hours}h
              </span>{" "}
              of focus time
            </span>
          )}
        </div>
      </div>
      <button
        onClick={() => setDismissed(true)}
        aria-label="Dismiss batching suggestion"
        className="flex-shrink-0 p-0.5 text-indigo-300 hover:text-indigo-500 transition-colors"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}
