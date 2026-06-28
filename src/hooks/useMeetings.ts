import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import { useTaskStore } from "@/stores/taskStore";

export function useMeetings(projectId: string | null, showArchived = false) {
  const qc = useQueryClient();
  const baselineDate = useTaskStore((s) => s.baselineDate);

  const query = useQuery({
    queryKey: ["meetings", projectId, showArchived],
    queryFn: () => api.getMeetingsForProject(projectId!, showArchived),
    enabled: !!projectId,
  });

  const ingestMutation = useMutation({
    mutationFn: api.ingestMeeting,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["meetings", projectId] });
      qc.invalidateQueries({ queryKey: ["tasks", projectId] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteMeeting,
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["meetings", projectId] }),
  });

  const forceDeleteMutation = useMutation({
    mutationFn: api.forceDeleteMeeting,
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["meetings", projectId] }),
  });

  const unarchiveMutation = useMutation({
    mutationFn: api.unarchiveMeeting,
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["meetings", projectId] }),
  });

  // Apply baseline date floor client-side (filter out meetings before the baseline)
  const allMeetings = query.data ?? [];
  const meetings = baselineDate
    ? allMeetings.filter((m) => !m.created_at || m.created_at >= baselineDate)
    : allMeetings;

  return {
    meetings,
    isLoading: query.isLoading,
    error: query.error,
    ingestMeeting: ingestMutation.mutateAsync,
    isIngesting: ingestMutation.isPending,
    ingestError: ingestMutation.error,
    deleteMeeting: deleteMutation.mutateAsync,
    forceDeleteMeeting: forceDeleteMutation.mutateAsync,
    unarchiveMeeting: unarchiveMutation.mutateAsync,
    refetch: query.refetch,
  };
}
