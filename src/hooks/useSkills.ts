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
