import { useTranslation } from "react-i18next";
import { Trash2, CheckSquare, X, Archive, CheckCheck } from "lucide-react";
import { useTaskStore } from "@/stores/taskStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTasks } from "@/hooks/useTasks";
import { archiveTask } from "@/lib/tauri";

export default function TaskBulkActions() {
  const { t } = useTranslation();
  const { selectedTaskIds, clearTaskSelection, selectAllTasks } = useTaskStore();
  const { activeProjectId } = useProjectStore();
  const { tasks, updateTask, deleteTask, refetch } = useTasks(activeProjectId);
  const count = selectedTaskIds.length;

  const visibleTaskIds = tasks.map((t) => t.id);
  const allSelected = visibleTaskIds.length > 0 && visibleTaskIds.every((id) => selectedTaskIds.includes(id));

  const handleSelectAll = () => {
    if (allSelected) {
      clearTaskSelection();
    } else {
      selectAllTasks(visibleTaskIds);
    }
  };

  const handleBulkDone = async () => {
    await Promise.all(selectedTaskIds.map((id) => updateTask({ id, status: "done" })));
    await refetch();
    clearTaskSelection();
  };

  const handleBulkArchive = async () => {
    await Promise.all(selectedTaskIds.map((id) => archiveTask(id)));
    await refetch();
    clearTaskSelection();
  };

  const handleBulkDelete = async () => {
    if (!window.confirm(`Delete ${count} task${count > 1 ? "s" : ""}? This cannot be undone.`)) return;
    await Promise.all(selectedTaskIds.map((id) => deleteTask(id)));
    await refetch();
    clearTaskSelection();
  };

  return (
    <div className="flex items-center gap-1.5 px-4 py-1.5 bg-indigo-500/[0.06] dark:bg-indigo-500/10 border-b border-indigo-200/50 dark:border-indigo-800/50 animate-slide-down flex-shrink-0">
      <span className="text-[12px] font-semibold text-indigo-600 dark:text-indigo-400 tabular-nums min-w-[60px]">
        {count} selected
      </span>

      <div className="h-3.5 w-px bg-indigo-200 dark:bg-indigo-800 mx-1" />

      {/* Select all / deselect all */}
      <button
        onClick={handleSelectAll}
        className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium transition-colors ${
          allSelected
            ? "text-indigo-700 dark:text-indigo-300 bg-indigo-100 dark:bg-indigo-900/40 hover:bg-indigo-200 dark:hover:bg-indigo-900/60"
            : "text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-900/20 hover:bg-indigo-100 dark:hover:bg-indigo-900/40"
        }`}
      >
        <CheckCheck className="w-3 h-3" />
        {allSelected ? "Deselect all" : `Select all (${visibleTaskIds.length})`}
      </button>

      <div className="h-3.5 w-px bg-indigo-200 dark:bg-indigo-800 mx-1" />

      {/* Mark done */}
      <button
        onClick={handleBulkDone}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium text-emerald-700 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 transition-colors"
      >
        <CheckSquare className="w-3 h-3" />
        {t("tasks.markDone")}
      </button>

      {/* Archive */}
      <button
        onClick={handleBulkArchive}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium text-amber-700 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 hover:bg-amber-100 dark:hover:bg-amber-900/40 transition-colors"
      >
        <Archive className="w-3 h-3" />
        Archive
      </button>

      {/* Delete */}
      <button
        onClick={handleBulkDelete}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors"
      >
        <Trash2 className="w-3 h-3" />
        {t("common.delete")}
      </button>

      {/* Clear */}
      <button
        onClick={clearTaskSelection}
        title="Clear selection"
        className="ml-auto p-1 rounded text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}
