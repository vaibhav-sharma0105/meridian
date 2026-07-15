import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Lightbulb } from "lucide-react";
import { SuggestionCard } from "./SuggestionCard";
import * as api from "@/lib/tauri";

interface SuggestionsListProps {
  projectId?: string;
}

export function SuggestionsList({ projectId }: SuggestionsListProps) {
  const queryClient = useQueryClient();

  const { data: suggestions = [], isLoading } = useQuery({
    queryKey: ["suggestions", projectId],
    queryFn: () => api.getPendingSuggestions(projectId),
    refetchInterval: 60000,
  });

  const handleAction = () => {
    queryClient.invalidateQueries({ queryKey: ["suggestions"] });
  };

  if (isLoading) {
    return (
      <div className="p-4 text-center text-sm text-zinc-500">
        Loading suggestions...
      </div>
    );
  }

  if (suggestions.length === 0) {
    return (
      <div className="px-4 py-3">
        <div className="flex items-start gap-2.5">
          <Lightbulb className="w-4 h-4 text-amber-400 dark:text-amber-500 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-[13px] font-medium text-zinc-700 dark:text-zinc-300">
              Smart suggestions
            </p>
            <p className="text-[11.5px] text-zinc-400 dark:text-zinc-500 mt-0.5 leading-relaxed">
              Meridian learns from your workflow and surfaces reminders when:
            </p>
            <ul className="text-[11.5px] text-zinc-400 dark:text-zinc-500 mt-1 space-y-0.5 list-none">
              <li>• Tasks are overdue or stale (no updates in 7+ days)</li>
              <li>• Meetings have no follow-up tasks created</li>
              <li>• A common next step is detected from your patterns</li>
            </ul>
            <p className="text-[11px] text-zinc-400 dark:text-zinc-500 mt-1.5 italic">
              Suggestions appear automatically as you use Meridian.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3 p-4">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
          <Lightbulb className="w-4 h-4 text-amber-500" />
          Suggestions
        </h3>
        <span className="text-xs text-zinc-500">{suggestions.length}</span>
      </div>
      {suggestions.map((suggestion) => (
        <SuggestionCard
          key={suggestion.id}
          suggestion={suggestion}
          onAction={handleAction}
        />
      ))}
    </div>
  );
}
