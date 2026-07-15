import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useTasks } from "@/hooks/useTasks";
import TaskCard from "./TaskCard";
import EmptyState from "@/components/shared/EmptyState";
import { LayoutList } from "lucide-react";
import { WorkflowSuggestionsList, SmartDefaultBadge } from "@/components/patterns";
import * as api from "@/lib/tauri";
import type { Task } from "@/lib/tauri";

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);
  useEffect(() => {
    const handler = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(handler);
  }, [value, delay]);
  return debouncedValue;
}

interface Props {
  projectId: string | null;
}

export default function TaskListView({ projectId }: Props) {
  const { t } = useTranslation();
  const { tasks: filtered, createTask, refetch } = useTasks(projectId);
  const [adding, setAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [recentlyCompletedTaskId, setRecentlyCompletedTaskId] = useState<string | null>(null);

  // Track when a task becomes "done"
  useEffect(() => {
    const justCompleted = filtered.find(
      (t) => t.status === "done" && t.completed_at &&
      Date.now() - new Date(t.completed_at).getTime() < 60000 // within last minute
    );
    if (justCompleted && justCompleted.id !== recentlyCompletedTaskId) {
      setRecentlyCompletedTaskId(justCompleted.id);
    }
  }, [filtered]);

  // Fetch suggestions for recently completed task
  const { data: suggestions = [] } = useQuery({
    queryKey: ["workflow-suggestions", recentlyCompletedTaskId, projectId],
    queryFn: () =>
      recentlyCompletedTaskId && projectId
        ? api.getWorkflowSuggestions(recentlyCompletedTaskId, projectId)
        : Promise.resolve([]),
    enabled: !!recentlyCompletedTaskId && !!projectId,
    staleTime: 30000,
  });

  const handleCreateFromSuggestion = async (title: string) => {
    if (!projectId) return;
    await createTask({ project_id: projectId, title });
    setRecentlyCompletedTaskId(null);
  };

  // Debounce title for smart defaults query
  const debouncedTitle = useDebounce(newTitle, 300);

  // Fetch smart defaults based on title
  const { data: smartDefaults } = useQuery({
    queryKey: ["smart-defaults", debouncedTitle, projectId],
    queryFn: () =>
      debouncedTitle.trim().length >= 3 && projectId
        ? api.getSmartDefaults(debouncedTitle, projectId)
        : Promise.resolve(null),
    enabled: adding && debouncedTitle.trim().length >= 3 && !!projectId,
    staleTime: 10000,
  });

  const hasSmartDefaults = smartDefaults && (smartDefaults.suggested_priority || smartDefaults.suggested_assignee);

  // Organize tasks into parent-child hierarchy
  const { parentTasks, subtaskMap } = useMemo(() => {
    const subtaskMap = new Map<string, Task[]>();
    const parentTasks: Task[] = [];

    // First pass: separate parents and subtasks
    for (const task of filtered) {
      if (task.parent_task_id) {
        const existing = subtaskMap.get(task.parent_task_id) || [];
        existing.push(task);
        subtaskMap.set(task.parent_task_id, existing);
      } else {
        parentTasks.push(task);
      }
    }

    return { parentTasks, subtaskMap };
  }, [filtered]);

  const handleAdd = async () => {
    if (!newTitle.trim() || !projectId) return;
    await createTask({
      project_id: projectId,
      title: newTitle.trim(),
      priority: smartDefaults?.suggested_priority || undefined,
      assignee: smartDefaults?.suggested_assignee || undefined,
    });
    setNewTitle("");
    setAdding(false);
  };

  return (
    <div className="p-5 space-y-2.5">
      {filtered.length === 0 && !adding && (
        <EmptyState
          title={t("tasks.noTasks")}
          description={t("tasks.noTasksDesc")}
          icon={<LayoutList className="w-10 h-10 text-zinc-400" />}
          action={{ label: t("tasks.addTask"), onClick: () => setAdding(true) }}
        />
      )}

      {parentTasks.map((task) => (
        <div key={task.id}>
          <TaskCard task={task} />
          {/* Render subtasks indented under parent */}
          {subtaskMap.get(task.id)?.map((subtask) => (
            <div key={subtask.id} className="ml-6 mt-1 pl-3 border-l-2 border-zinc-200 dark:border-zinc-700">
              <TaskCard task={subtask} isSubtask />
            </div>
          ))}
        </div>
      ))}

      {/* Workflow suggestions after task completion */}
      {suggestions.length > 0 && projectId && (
        <div className="mt-3">
          <WorkflowSuggestionsList
            suggestions={suggestions}
            projectId={projectId}
            onCreateTask={handleCreateFromSuggestion}
          />
        </div>
      )}

      {adding ? (
        <div className="space-y-2 animate-fade-in">
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
              className="flex-1 px-3.5 py-2.5 text-[13.5px] rounded-xl border border-[#e2e2e8] dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 outline-none focus-visible:ring-2 focus-visible:ring-indigo-400/40 focus:border-indigo-400 transition-all"
            />
            <button
              onClick={handleAdd}
              disabled={!newTitle.trim()}
              className="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white rounded-xl text-[13.5px] font-medium disabled:opacity-50 transition-all shadow-sm hover:shadow-md"
            >
              {t("common.add")}
            </button>
            <button
              onClick={() => setAdding(false)}
              className="px-4 py-2.5 border border-[#e2e2e8] dark:border-zinc-700 rounded-xl text-[13.5px] text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors"
            >
              {t("common.cancel")}
            </button>
          </div>
          {hasSmartDefaults && (
            <div className="flex items-center gap-3 px-1 text-xs text-zinc-500">
              <SmartDefaultBadge source={smartDefaults.source} />
              {smartDefaults.suggested_priority && (
                <span>Priority: <span className="font-medium text-zinc-700 dark:text-zinc-300">{smartDefaults.suggested_priority}</span></span>
              )}
              {smartDefaults.suggested_assignee && (
                <span>Assignee: <span className="font-medium text-zinc-700 dark:text-zinc-300">{smartDefaults.suggested_assignee}</span></span>
              )}
            </div>
          )}
        </div>
      ) : (
        filtered.length > 0 && (
          <button
            onClick={() => setAdding(true)}
            className="flex items-center gap-2 w-full px-4 py-2.5 text-[13.5px] text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800/60 rounded-xl transition-all duration-150"
          >
            <Plus className="w-4 h-4" />
            {t("tasks.addTask")}
          </button>
        )
      )}
    </div>
  );
}
