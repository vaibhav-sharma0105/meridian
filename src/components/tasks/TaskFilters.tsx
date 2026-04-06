import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";
import { useTaskStore } from "@/stores/taskStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTasks } from "@/hooks/useTasks";
import { KANBAN_COLUMNS } from "@/lib/constants";

export default function TaskFilters() {
  const { t } = useTranslation();
  const { filters, setFilters } = useTaskStore();
  const { activeProjectId } = useProjectStore();
  // Fetch all tasks (no filters) to build assignee suggestions
  const { tasks: allTasks } = useTasks(activeProjectId, {});

  const assignees = Array.from(
    new Set(
      allTasks
        .map((task) => task.assignee)
        .filter((a): a is string => !!a)
    )
  );

  const hasFilters = filters.search_query || filters.status || filters.assignee;

  return (
    <div className="flex items-center gap-2 flex-wrap">
      {/* Search */}
      <div className="relative">
        <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-400" />
        <input
          type="text"
          value={filters.search_query ?? ""}
          onChange={(e) => setFilters({ search_query: e.target.value })}
          placeholder={t("tasks.searchPlaceholder")}
          className="pl-7 pr-3 py-1 text-xs rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 w-48"
        />
      </div>

      {/* Status */}
      <select
        value={filters.status ?? ""}
        onChange={(e) => setFilters({ status: e.target.value || undefined })}
        className="px-2 py-1 text-xs rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
      >
        <option value="">{t("tasks.allStatuses")}</option>
        {KANBAN_COLUMNS.map((col) => (
          <option key={col.id} value={col.id}>{col.label}</option>
        ))}
        <option value="cancelled">Cancelled</option>
      </select>

      {/* Assignee */}
      <input
        type="text"
        list="assignee-suggestions"
        value={filters.assignee ?? ""}
        onChange={(e) => setFilters({ assignee: e.target.value || undefined })}
        placeholder={t("tasks.filterAssignee")}
        className="px-2 py-1 text-xs rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 w-32"
      />
      <datalist id="assignee-suggestions">
        {assignees.map((a) => (
          <option key={a} value={a} />
        ))}
      </datalist>

      {/* Clear */}
      {hasFilters && (
        <button
          onClick={() => setFilters({ search_query: undefined, status: undefined, assignee: undefined })}
          className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
        >
          <X className="w-3 h-3" />
          {t("tasks.clearFilters")}
        </button>
      )}
    </div>
  );
}
