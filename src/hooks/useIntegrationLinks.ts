import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { CreateLinkInput, IntegrationLink } from "@/lib/tauri";

export function useLinksForTask(taskId: string | undefined) {
  return useQuery({
    queryKey: ["integration-links", "task", taskId],
    queryFn: () => (taskId ? api.getLinksForTask(taskId) : Promise.resolve([])),
    enabled: !!taskId,
  });
}

export function useLinksForMeeting(meetingId: string | undefined) {
  return useQuery({
    queryKey: ["integration-links", "meeting", meetingId],
    queryFn: () =>
      meetingId ? api.getLinksForMeeting(meetingId) : Promise.resolve([]),
    enabled: !!meetingId,
  });
}

export function useCreateLink() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateLinkInput) => api.createIntegrationLink(input),
    onSuccess: (data) => {
      qc.invalidateQueries({
        queryKey: ["integration-links", data.local_type, data.local_id],
      });
    },
  });
}

export function useUnlink() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      linkId,
      localType,
      localId,
    }: {
      linkId: string;
      localType: string;
      localId: string;
    }) => api.unlinkIntegrationItem(linkId),
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({
        queryKey: ["integration-links", variables.localType, variables.localId],
      });
    },
  });
}
