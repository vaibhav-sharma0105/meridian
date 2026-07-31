import { ChevronDown } from "lucide-react";
import type { AttentionFilters as AttentionFiltersType } from "@/lib/tauri";

interface AttentionFiltersProps {
  filters: AttentionFiltersType;
  onChange: (filters: AttentionFiltersType) => void;
}

export function AttentionFilters({ filters, onChange }: AttentionFiltersProps) {
  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-zinc-100 dark:border-zinc-800">
      <div className="relative">
        <select
          value={filters.source_type || "all"}
          onChange={(e) =>
            onChange({
              ...filters,
              source_type: e.target.value === "all" ? undefined : e.target.value,
            })
          }
          className="appearance-none bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-md pl-3 pr-8 py-1.5 text-sm text-zinc-700 dark:text-zinc-300 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          <option value="all">All Sources</option>
          <option value="task">Tasks</option>
          <option value="approval">Approvals</option>
          <option value="github">GitHub</option>
          <option value="jira">Jira</option>
          <option value="slack">Slack</option>
        </select>
        <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400 pointer-events-none" />
      </div>

      <div className="relative">
        <select
          value={filters.severity || "all"}
          onChange={(e) =>
            onChange({
              ...filters,
              severity: e.target.value === "all" ? undefined : e.target.value,
            })
          }
          className="appearance-none bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-md pl-3 pr-8 py-1.5 text-sm text-zinc-700 dark:text-zinc-300 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          <option value="all">All Severities</option>
          <option value="critical">Critical</option>
          <option value="warning">Warning</option>
          <option value="info">Info</option>
        </select>
        <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400 pointer-events-none" />
      </div>

      <label className="flex items-center gap-2 ml-auto">
        <input
          type="checkbox"
          checked={filters.include_dismissed || false}
          onChange={(e) =>
            onChange({ ...filters, include_dismissed: e.target.checked })
          }
          className="rounded border-zinc-300 dark:border-zinc-600 text-indigo-500 focus:ring-indigo-500"
        />
        <span className="text-xs text-zinc-500">Show dismissed</span>
      </label>
    </div>
  );
}
