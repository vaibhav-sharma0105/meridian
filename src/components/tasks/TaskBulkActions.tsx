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
    <div className="flex items-center gap-3 px-4 py-2 bg-indigo-50 dark:bg-indigo-900/20 border-b border-indigo-200 dark:border-indigo-800 text-sm">
      <span className="font-medium text-indigo-700 dark:text-indigo-300">{count} selected</span>

      <button
        onClick={() => handleBulkStatus("done")}
        className="flex items-center gap-1.5 px-3 py-1 rounded-md bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 transition-colors text-xs"
      >
        <CheckSquare className="w-3.5 h-3.5" />
        {t("tasks.markDone")}
      </button>

      <button
        onClick={handleBulkDelete}
        className="flex items-center gap-1.5 px-3 py-1 rounded-md bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 transition-colors text-xs"
      >
        <Trash2 className="w-3.5 h-3.5" />
        {t("common.delete")}
      </button>

      <button
        onClick={clearTaskSelection}
        className="ml-auto p-1 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
