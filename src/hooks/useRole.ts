import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getUserProfile,
  getInferenceStatus,
  confirmRole,
  changeRole,
  dismissDriftAlert,
  runRoleInference,
  updateRetentionSettings,
  updateUserIdentity,
  getRoleDriftAlert,
  type UserProfile,
  type InferenceStatus,
  type RoleScores,
  type RoleDriftAlert,
} from "@/lib/tauri";

export function useUserProfile() {
  return useQuery({
    queryKey: ["user-profile"],
    queryFn: getUserProfile,
  });
}

export function useInferenceStatus() {
  return useQuery({
    queryKey: ["inference-status"],
    queryFn: getInferenceStatus,
  });
}

/**
 * Role scores ride along on `user_profile.role_scores` — there is no separate
 * backend command for them, so derive from the profile query rather than
 * duplicating API surface.
 */
export function useRoleScores() {
  const { data: profile, ...rest } = useUserProfile();
  return { ...rest, data: profile?.role_scores ?? null };
}

export function useUpdateRetentionSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: updateRetentionSettings,
    onSuccess: (profile) => {
      queryClient.setQueryData(["user-profile"], profile);
      queryClient.invalidateQueries({ queryKey: ["productivity-insights"] });
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });
}

/**
 * Identity drives role-based ordering, so saving it must refetch My Activity.
 */
export function useUpdateUserIdentity() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: updateUserIdentity,
    onSuccess: (profile) => {
      queryClient.setQueryData(["user-profile"], profile);
      queryClient.invalidateQueries({ queryKey: ["attention-items"] });
    },
  });
}

export function useConfirmRole() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      role,
      customDescription,
    }: {
      role: string;
      customDescription?: string;
    }) => confirmRole(role, customDescription),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });
      queryClient.invalidateQueries({ queryKey: ["inference-status"] });
      // My Activity is ordered by role server-side, so a role change must
      // refetch it — this is the spec's "immediately reorders" behaviour.
      queryClient.invalidateQueries({ queryKey: ["attention-items"] });
    },
  });
}

export function useChangeRole() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      role,
      customDescription,
    }: {
      role: string;
      customDescription?: string;
    }) => changeRole(role, customDescription),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });
      queryClient.invalidateQueries({ queryKey: ["inference-status"] });
      // My Activity is ordered by role server-side, so a role change must
      // refetch it — this is the spec's "immediately reorders" behaviour.
      queryClient.invalidateQueries({ queryKey: ["attention-items"] });
    },
  });
}

export function useDismissDriftAlert() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: dismissDriftAlert,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["inference-status"] });
      queryClient.setQueryData(["role-drift-alert"], null);
    },
  });
}

export function useRunRoleInference() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: runRoleInference,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });
      queryClient.invalidateQueries({ queryKey: ["inference-status"] });
      // My Activity is ordered by role server-side, so a role change must
      // refetch it — this is the spec's "immediately reorders" behaviour.
      queryClient.invalidateQueries({ queryKey: ["attention-items"] });
    },
  });
}

/**
 * Drift is recomputed by the daily `infer_role` daemon job, so a low-frequency
 * poll is sufficient — there is no event to subscribe to.
 */
export function useRoleDriftAlert() {
  return useQuery({
    queryKey: ["role-drift-alert"],
    queryFn: getRoleDriftAlert,
    staleTime: 5 * 60 * 1000,
    refetchInterval: 15 * 60 * 1000,
  });
}

export const ROLE_LABELS: Record<string, string> = {
  tech_lead: "Tech Lead",
  ic: "Individual Contributor",
  pm: "Product Manager",
  manager: "People Manager",
};

export const ROLE_DESCRIPTIONS: Record<string, string> = {
  tech_lead: "Reviews code, unblocks team, balances hands-on work with coordination",
  ic: "Focuses on individual task execution and deep technical work",
  pm: "Prioritizes features, coordinates with stakeholders, tracks milestones",
  manager: "Manages team workload, runs 1:1s, focuses on people and process",
};

export type { UserProfile, InferenceStatus, RoleScores, RoleDriftAlert };
