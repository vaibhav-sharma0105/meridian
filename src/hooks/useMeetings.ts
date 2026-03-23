import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";

export function useMeetings(projectId: string | null) {
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["meetings", projectId],
    queryFn: () => api.getMeetingsForProject(projectId!),
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

  return {
    meetings: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    ingestMeeting: ingestMutation.mutateAsync,
    isIngesting: ingestMutation.isPending,
    ingestError: ingestMutation.error,
    deleteMeeting: deleteMutation.mutateAsync,
    refetch: query.refetch,
  };
}
