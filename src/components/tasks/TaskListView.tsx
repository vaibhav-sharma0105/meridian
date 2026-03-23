import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { useTasks } from "@/hooks/useTasks";
import TaskCard from "./TaskCard";
import EmptyState from "@/components/shared/EmptyState";
import { LayoutList } from "lucide-react";

interface Props {
  projectId: string | null;
}

export default function TaskListView({ projectId }: Props) {
  const { t } = useTranslation();
  const { tasks: filtered, createTask, refetch } = useTasks(projectId);
  const [adding, setAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");

  const handleAdd = async () => {
    if (!newTitle.trim() || !projectId) return;
    await createTask({ project_id: projectId, title: newTitle.trim() });
    setNewTitle("");
    setAdding(false);
  };

  if (!projectId) {
    return (
      <EmptyState
        title={t("tasks.noProject")}
        description={t("tasks.noProjectDesc")}
        icon={<LayoutList className="w-10 h-10 text-zinc-400" />}
      />
    );
  }

  return (
    <div className="p-4 space-y-2">
      {filtered.length === 0 && !adding && (
        <EmptyState
          title={t("tasks.noTasks")}
          description={t("tasks.noTasksDesc")}
          icon={<LayoutList className="w-10 h-10 text-zinc-400" />}
          action={{ label: t("tasks.addTask"), onClick: () => setAdding(true) }}
        />
      )}

      {filtered.map((task) => (
        <TaskCard key={task.id} task={task} />
      ))}

      {adding ? (
        <div className="flex gap-2">
          <input
            type="text"
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
            placeholder={t("tasks.titlePlaceholder")}
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAdd();
              if (e.key === "Escape") setAdding(false);
            }}
            className="flex-1 px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          />
          <button
            onClick={handleAdd}
            disabled={!newTitle.trim()}
            className="px-3 py-2 bg-indigo-500 text-white rounded-lg text-sm disabled:opacity-50"
          >
            {t("common.add")}
          </button>
          <button
            onClick={() => setAdding(false)}
            className="px-3 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400"
          >
            {t("common.cancel")}
          </button>
        </div>
      ) : (
        filtered.length > 0 && (
          <button
            onClick={() => setAdding(true)}
            className="flex items-center gap-1.5 w-full px-3 py-2 text-sm text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
            {t("tasks.addTask")}
          </button>
        )
      )}
    </div>
  );
}
