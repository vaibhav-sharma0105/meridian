import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type {
  TeamMember,
  CreateTeamMemberInput,
  UpdateTeamMemberInput,
  TeamSyncResult,
  AssigneeSuggestion,
} from "@/lib/tauri";

// ─── Query Keys ───────────────────────────────────────────────────────────────

export const teamKeys = {
  all: ["team"] as const,
  members: () => [...teamKeys.all, "members"] as const,
  member: (id: string) => [...teamKeys.all, "member", id] as const,
  suggestions: (title: string, projectId?: string) =>
    [...teamKeys.all, "suggestions", title, projectId] as const,
  workloads: () => [...teamKeys.all, "workloads"] as const,
};

// ─── Team Members ─────────────────────────────────────────────────────────────

export function useTeamMembers() {
  return useQuery({
    queryKey: teamKeys.members(),
    queryFn: () => api.getTeamMembers(),
  });
}

export function useTeamMember(id: string) {
  return useQuery({
    queryKey: teamKeys.member(id),
    queryFn: () => api.getTeamMember(id),
    enabled: !!id,
  });
}

export function useCreateTeamMember() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateTeamMemberInput) => api.createTeamMember(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamKeys.members() });
    },
  });
}

export function useUpdateTeamMember() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: UpdateTeamMemberInput) => api.updateTeamMember(input),
    onSuccess: (updated) => {
      queryClient.setQueryData<TeamMember[]>(teamKeys.members(), (old) =>
        old?.map((m) => (m.id === updated.id ? updated : m))
      );
      queryClient.setQueryData(teamKeys.member(updated.id), updated);
    },
  });
}

export function useDeleteTeamMember() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.deleteTeamMember(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData<TeamMember[]>(teamKeys.members(), (old) =>
        old?.filter((m) => m.id !== id)
      );
      queryClient.removeQueries({ queryKey: teamKeys.member(id) });
    },
  });
}

// ─── Sync ─────────────────────────────────────────────────────────────────────

export function useSyncTeamFromSlack() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => api.syncTeamFromSlack(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamKeys.members() });
    },
  });
}

// ─── Workloads ────────────────────────────────────────────────────────────────

export function useComputeTeamWorkloads() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => api.computeTeamWorkloads(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamKeys.members() });
    },
  });
}

// ─── Assignee Suggestions ─────────────────────────────────────────────────────

export function useAssigneeSuggestions(
  taskTitle: string,
  taskDescription?: string,
  projectId?: string
) {
  return useQuery({
    queryKey: teamKeys.suggestions(taskTitle, projectId),
    queryFn: () =>
      api.getAssigneeSuggestions(taskTitle, taskDescription, projectId),
    enabled: taskTitle.length > 3,
    staleTime: 30000, // Cache for 30 seconds
  });
}
