import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { Skill, CreateSkillInput, UpdateSkillInput } from "@/lib/tauri";

export interface SkillFilters {
  shared?: boolean;
  category?: string;
  enabled?: boolean;
}

export function useSkills(filters?: SkillFilters) {
  return useQuery({
    queryKey: ["skills", filters],
    queryFn: () => api.listSkills(filters),
  });
}

export function useSkill(id: string | null) {
  return useQuery({
    queryKey: ["skill", id],
    queryFn: () => (id ? api.getSkill(id) : null),
    enabled: !!id,
  });
}

export function useSkillStats(skillId: string | null) {
  return useQuery({
    queryKey: ["skill-stats", skillId],
    queryFn: () => (skillId ? api.getSkillStats(skillId) : null),
    enabled: !!skillId,
  });
}

export function useSkillRuns(skillId: string | null, status?: string) {
  return useQuery({
    queryKey: ["skill-runs", skillId, status],
    queryFn: () =>
      skillId ? api.getSkillRuns({ skillId, status, limit: 50 }) : [],
    enabled: !!skillId,
  });
}

export function useCreateSkill() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateSkillInput) => api.createSkill(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useUpdateSkill() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: (input: UpdateSkillInput) => api.updateSkill(input),
    onSuccess: (skill) => {
      qc.setQueryData<Skill>(["skill", skill.id], skill);
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useDeleteSkill() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.deleteSkill(id),
    onSuccess: (_, id) => {
      qc.removeQueries({ queryKey: ["skill", id] });
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useToggleSkillEnabled() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      api.toggleSkillEnabled(id, enabled),
    onSuccess: (skill) => {
      qc.setQueryData<Skill>(["skill", skill.id], skill);
      // Update all cached skill lists (with any filter combination)
      qc.setQueriesData<Skill[]>({ queryKey: ["skills"] }, (old) =>
        old?.map((s) => (s.id === skill.id ? skill : s))
      );
    },
  });
}

export function useRunSkillManually() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: (skillId: string) => api.runSkillManually(skillId),
    onSuccess: (run) => {
      qc.invalidateQueries({ queryKey: ["skill-runs", run.skill_id] });
      qc.invalidateQueries({ queryKey: ["skill-stats", run.skill_id] });
    },
  });
}

export function useTestRunSkill() {
  return useMutation({
    mutationFn: (skillId: string) => api.testRunSkill(skillId),
  });
}

export function useCloneSkill() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({ skillId, newName }: { skillId: string; newName?: string }) =>
      api.cloneSkill(skillId, newName),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useApproveSkillRun() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({
      runId,
      projectId,
    }: {
      runId: string;
      projectId?: string;
    }) => api.approveSkillRun(runId, projectId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skill-runs"] });
      qc.invalidateQueries({ queryKey: ["tasks"] });
    },
  });
}

export function useRejectSkillRun() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({ runId, reason }: { runId: string; reason?: string }) =>
      api.rejectSkillRun(runId, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skill-runs"] });
    },
  });
}

export function useResetBuiltinSkills() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: () => api.resetBuiltinSkills(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

// ─── Phase 9: GitHub Sync & Trust ─────────────────────────────────────────────

export function useListImportableSkills() {
  return useMutation({
    mutationFn: ({ integrationId, owner, repo }: { integrationId: string; owner: string; repo: string }) =>
      api.listImportableSkills(integrationId, owner, repo),
  });
}

export function useImportSkillFromRepo() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({
      integrationId,
      skillPath,
      localName,
    }: {
      integrationId: string;
      skillPath: string;
      localName?: string;
    }) => api.importSkillFromRepo(integrationId, skillPath, localName),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useCheckSkillUpdates() {
  return useMutation({
    mutationFn: (skillId: string) => api.checkSkillUpdates(skillId),
  });
}

export function useSyncSkill() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({
      skillId,
      strategy,
    }: {
      skillId: string;
      strategy: "keep_local" | "use_remote" | "manual";
    }) => api.syncSkill(skillId, strategy),
    onSuccess: (result) => {
      qc.invalidateQueries({ queryKey: ["skill", result.skill_id] });
      qc.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useGrantSkillTrust() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({
      skillId,
      networkMode,
      allowlist,
    }: {
      skillId: string;
      networkMode: "none" | "allowlist" | "full";
      allowlist?: string[];
    }) => api.grantSkillTrust(skillId, networkMode, allowlist),
    onSuccess: (_, { skillId }) => {
      qc.invalidateQueries({ queryKey: ["skill", skillId] });
      qc.invalidateQueries({ queryKey: ["skill-trust", skillId] });
    },
  });
}

export function useRevokeSkillTrust() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: (skillId: string) => api.revokeSkillTrust(skillId),
    onSuccess: (_, skillId) => {
      qc.invalidateQueries({ queryKey: ["skill", skillId] });
      qc.invalidateQueries({ queryKey: ["skill-trust", skillId] });
    },
  });
}

export function useSkillTrustState(skillId: string | null) {
  return useQuery({
    queryKey: ["skill-trust", skillId],
    queryFn: () => (skillId ? api.getSkillTrustState(skillId) : null),
    enabled: !!skillId,
  });
}

export function useExecuteSkillSandboxed() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({
      skillId,
      scriptPath,
      inputs,
    }: {
      skillId: string;
      scriptPath: string;
      inputs?: Record<string, unknown>;
    }) => api.executeSkillSandboxed(skillId, scriptPath, inputs),
    onSuccess: (result) => {
      qc.invalidateQueries({ queryKey: ["skill-runs"] });
    },
  });
}
