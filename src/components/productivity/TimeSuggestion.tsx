import { useState } from "react";
import { Clock, Lightbulb, X } from "lucide-react";
import { useTimeSuggestion, formatHour } from "@/hooks/useProductivity";

interface TimeSuggestionProps {
  category?: string;
  onDismiss?: () => void;
}

export function TimeSuggestion({
  category = "focus_work",
  onDismiss,
}: TimeSuggestionProps) {
  const { data: suggestion, isLoading, error } = useTimeSuggestion(category);
  const [dismissed, setDismissed] = useState(false);

  if (dismissed || isLoading || error || !suggestion) {
    return null;
  }

  const handleDismiss = () => {
    setDismissed(true);
    onDismiss?.();
  };

  // "Default" means the backend fell back to research-based hours because there
  // is not enough completion history yet — label it honestly rather than
  // presenting it as a learned pattern.
  const isLearned = suggestion.confidence === "High";

  return (
    <div className="flex items-start gap-2 p-2 bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-100 dark:border-indigo-800 rounded-lg">
      <Lightbulb className="w-4 h-4 text-indigo-500 mt-0.5 flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <p className="text-xs text-indigo-700 dark:text-indigo-300">
          {suggestion.reason}
        </p>
        <div className="flex items-center gap-2 mt-1">
          <Clock className="w-3 h-3 text-indigo-400" />
          <span className="text-xs text-indigo-600 dark:text-indigo-400">
            Best: {formatHour(suggestion.suggested_hour)}
          </span>
          {!isLearned && (
            <span className="text-xs text-indigo-400">· typical default</span>
          )}
        </div>
      </div>
      {onDismiss && (
        <button
          onClick={handleDismiss}
          className="p-1 hover:bg-indigo-100 dark:hover:bg-indigo-800 rounded transition-colors"
        >
          <X className="w-3 h-3 text-indigo-400" />
        </button>
      )}
    </div>
  );
}

export function TimeSuggestionInline({
  category = "focus_work",
}: {
  category?: string;
}) {
  const { data: suggestion } = useTimeSuggestion(category);

  if (!suggestion) {
    return null;
  }

  return (
    <span className="inline-flex items-center gap-1 text-xs text-indigo-500">
      <Clock className="w-3 h-3" />
      Best at {formatHour(suggestion.suggested_hour)}
    </span>
  );
}
