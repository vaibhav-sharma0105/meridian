import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { TaskFilters, UpdateTaskInput } from "@/lib/tauri";
import { useTaskStore } from "@/stores/taskStore";

export function useTasks(projectId: string | null, filters?: TaskFilters) {
  const qc = useQueryClient();
  const storeFilters = useTaskStore((s) => s.filters);
  // Use explicitly passed filters, or fall back to the global filter store
  const effectiveFilters = filters ?? storeFilters;

  // Strip client-side-only fields before sending to backend
  const backendFilters = {
    ...effectiveFilters,
    project_id: undefined,
    meeting_ids: undefined,
  };

  const query = useQuery({
    queryKey: ["tasks", projectId, effectiveFilters],
    queryFn: async () => {
      let tasks = projectId
        ? await api.getTasksForProject(projectId, backendFilters)
        : await api.getAllTasks(backendFilters);

      // project_id filter is client-side
      if (!projectId && effectiveFilters.project_id) {
        tasks = tasks.filter((t) => t.project_id === effectiveFilters.project_id);
      }

      // meeting_ids multi-select is client-side
      if (effectiveFilters.meeting_ids?.length) {
        tasks = tasks.filter(
          (t) => t.meeting_id && effectiveFilters.meeting_ids!.includes(t.meeting_id)
        );
      }

      return tasks;
    },
    enabled: true,
  });

  const updateMutation = useMutation({
    mutationFn: api.updateTask,
    onMutate: async (input: UpdateTaskInput) => {
      // Optimistic update
      await qc.cancelQueries({ queryKey: ["tasks", projectId] });
      const prev = qc.getQueryData(["tasks", projectId, effectiveFilters]);
      qc.setQueryData(["tasks", projectId, effectiveFilters], (old: any[]) =>
        old?.map((t) => (t.id === input.id ? { ...t, ...input } : t))
      );
      return { prev };
    },
    onError: (_err, _input, ctx) => {
      qc.setQueryData(["tasks", projectId, effectiveFilters], ctx?.prev);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  const createMutation = useMutation({
    mutationFn: api.createTask,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteTask,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  const bulkUpdateMutation = useMutation({
    mutationFn: ({ taskIds, updates }: { taskIds: string[]; updates: any }) =>
      api.bulkUpdateTasks(taskIds, updates),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  const reorderMutation = useMutation({
    mutationFn: ({
      taskId,
      newColumn,
      newOrder,
    }: {
      taskId: string;
      newColumn: string;
      newOrder: number;
    }) => api.reorderTask(taskId, newColumn, newOrder),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  return {
    tasks: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    updateTask: updateMutation.mutateAsync,
    createTask: createMutation.mutateAsync,
    deleteTask: deleteMutation.mutateAsync,
    bulkUpdateTasks: bulkUpdateMutation.mutateAsync,
    reorderTask: reorderMutation.mutateAsync,
    refetch: query.refetch,
  };
}
