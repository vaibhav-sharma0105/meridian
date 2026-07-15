import { useState } from "react";
import { Lightbulb, X, Plus } from "lucide-react";
import * as api from "../../lib/tauri";

interface WorkflowSuggestionProps {
  suggestion: api.WorkflowSuggestion;
  onAccept: (suggestedAction: string) => void;
  onDismiss: () => void;
}

export function WorkflowSuggestion({ suggestion, onAccept, onDismiss }: WorkflowSuggestionProps) {
  const [dismissing, setDismissing] = useState(false);

  const handleDismiss = async () => {
    setDismissing(true);
    onDismiss();
  };

  const confidenceLabel = suggestion.confidence >= 0.8 ? "Often" : suggestion.confidence >= 0.6 ? "Sometimes" : "Occasionally";

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-indigo-50 dark:bg-indigo-950/30 border border-indigo-200 dark:border-indigo-800 rounded-lg text-sm animate-fade-in">
      <Lightbulb className="w-4 h-4 text-indigo-500 flex-shrink-0" />
      <span className="text-zinc-700 dark:text-zinc-300 flex-1">
        <span className="text-zinc-500 dark:text-zinc-400">{confidenceLabel} after this, you</span>{" "}
        <span className="font-medium">{suggestion.suggested_action}</span>
      </span>
      <button
        onClick={() => onAccept(suggestion.suggested_action)}
        className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 rounded transition-colors"
      >
        <Plus className="w-3 h-3" />
        Create
      </button>
      <button
        onClick={handleDismiss}
        disabled={dismissing}
        className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
        title="Dismiss suggestion"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}

interface WorkflowSuggestionsListProps {
  suggestions: api.WorkflowSuggestion[];
  projectId: string;
  onCreateTask: (title: string) => void;
}

export function WorkflowSuggestionsList({ suggestions, projectId, onCreateTask }: WorkflowSuggestionsListProps) {
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());

  if (suggestions.length === 0) return null;

  const visibleSuggestions = suggestions.filter((s) => !dismissed.has(s.sequence_id));
  if (visibleSuggestions.length === 0) return null;

  const handleDismiss = async (suggestion: api.WorkflowSuggestion) => {
    setDismissed((prev) => new Set(prev).add(suggestion.sequence_id));
    try {
      await api.dismissWorkflowSuggestion(suggestion.sequence_id, projectId);
    } catch (e) {
      console.error("Failed to dismiss suggestion:", e);
    }
  };

  return (
    <div className="space-y-2">
      {visibleSuggestions.map((suggestion) => (
        <WorkflowSuggestion
          key={suggestion.sequence_id}
          suggestion={suggestion}
          onAccept={(title) => onCreateTask(title)}
          onDismiss={() => handleDismiss(suggestion)}
        />
      ))}
    </div>
  );
}
