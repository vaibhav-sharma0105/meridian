import { useTranslation } from "react-i18next";
import { Trash2, CheckSquare, X } from "lucide-react";
import { useTaskStore } from "@/stores/taskStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTasks } from "@/hooks/useTasks";

export default function TaskBulkActions() {
  const { t } = useTranslation();
  const { selectedTaskIds, clearTaskSelection } = useTaskStore();
  const { activeProjectId } = useProjectStore();
  const { tasks, updateTask, deleteTask, refetch } = useTasks(activeProjectId);
  const count = selectedTaskIds.length;

  const handleBulkStatus = async (status: string) => {
    await Promise.all(
      selectedTaskIds.map((id) => {
        const task = tasks.find((t) => t.id === id);
        if (!task) return Promise.resolve();
        return updateTask({ id, status });
      })
    );
    await refetch();
    clearTaskSelection();
  };

  const handleBulkDelete = async () => {
    await Promise.all(selectedTaskIds.map((id) => deleteTask(id)));
    await refetch();
    clearTaskSelection();
  };

  return (
    <div className="flex items-center gap-2 px-4 py-1.5 bg-indigo-500/[0.06] dark:bg-indigo-500/10 border-b border-indigo-200/50 dark:border-indigo-800/50 animate-slide-down flex-shrink-0">
      <span className="text-[12px] font-semibold text-indigo-600 dark:text-indigo-400 tabular-nums">
        {count} selected
      </span>

      <div className="h-3.5 w-px bg-indigo-200 dark:bg-indigo-800 mx-1" />

      <button
        onClick={() => handleBulkStatus("done")}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium text-emerald-700 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 transition-colors"
      >
        <CheckSquare className="w-3 h-3" />
        {t("tasks.markDone")}
      </button>

      <button
        onClick={handleBulkDelete}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[12px] font-medium text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors"
      >
        <Trash2 className="w-3 h-3" />
        {t("common.delete")}
      </button>

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
