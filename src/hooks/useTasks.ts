import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { TaskFilters, UpdateTaskInput } from "@/lib/tauri";
import { useTaskStore } from "@/stores/taskStore";

export function useTasks(projectId: string | null, filters?: TaskFilters) {
  const qc = useQueryClient();
  const storeFilters = useTaskStore((s) => s.filters);
  const baselineDate = useTaskStore((s) => s.baselineDate);
  // Use explicitly passed filters, or fall back to the global filter store
  const effectiveFilters = filters ?? storeFilters;

  // Apply baseline date as implicit date_from floor (only when no explicit date_from is set)
  const filtersWithBaseline: TaskFilters = {
    ...effectiveFilters,
    date_from: effectiveFilters.date_from ?? (baselineDate ?? undefined),
  };

  // Strip client-side-only fields before sending to backend
  // show_archived is intentionally kept — it's handled by the Rust query
  const backendFilters = {
    ...filtersWithBaseline,
    project_id: undefined,
    meeting_ids: undefined,
  };

  const query = useQuery({
    queryKey: ["tasks", projectId, filtersWithBaseline],
    queryFn: async () => {
      let tasks = projectId
        ? await api.getTasksForProject(projectId, backendFilters)
        : await api.getAllTasks(backendFilters);

      // project_id filter is client-side
      if (!projectId && filtersWithBaseline.project_id) {
        tasks = tasks.filter((t) => t.project_id === filtersWithBaseline.project_id);
      }

      // meeting_ids multi-select is client-side
      if (filtersWithBaseline.meeting_ids?.length) {
        tasks = tasks.filter(
          (t) => t.meeting_id && filtersWithBaseline.meeting_ids!.includes(t.meeting_id)
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
      const prev = qc.getQueryData(["tasks", projectId, filtersWithBaseline]);
      qc.setQueryData(["tasks", projectId, filtersWithBaseline], (old: any[]) =>
        old?.map((t) => (t.id === input.id ? { ...t, ...input } : t))
      );
      return { prev };
    },
    onError: (_err, _input, ctx) => {
      qc.setQueryData(["tasks", projectId, filtersWithBaseline], ctx?.prev);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ["tasks"] }),
  });

  const createMutation = useMutation({
    mutationFn: api.createTask,
    onSuccess: (newTask) => {
      // Optimistically append to cache so it appears instantly
      qc.setQueryData<any[]>(["tasks", projectId, filtersWithBaseline], (old) =>
        old ? [...old, newTask] : [newTask]
      );
      qc.invalidateQueries({ queryKey: ["tasks", projectId] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteTask,
    onMutate: async (taskId: string) => {
      await qc.cancelQueries({ queryKey: ["tasks", projectId] });
      const prev = qc.getQueryData(["tasks", projectId, filtersWithBaseline]);
      qc.setQueryData(["tasks", projectId, filtersWithBaseline], (old: any[]) =>
        old?.filter((t) => t.id !== taskId)
      );
      return { prev };
    },
    onError: (_err, _taskId, ctx) => {
      qc.setQueryData(["tasks", projectId, filtersWithBaseline], ctx?.prev);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ["tasks"] }),
  });

  const archiveMutation = useMutation({
    mutationFn: api.archiveTask,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tasks", projectId] }),
  });

  const unarchiveMutation = useMutation({
    mutationFn: api.unarchiveTask,
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
    archiveTask: archiveMutation.mutateAsync,
    unarchiveTask: unarchiveMutation.mutateAsync,
    bulkUpdateTasks: bulkUpdateMutation.mutateAsync,
    reorderTask: reorderMutation.mutateAsync,
    refetch: query.refetch,
  };
}
