import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { useEffect } from "react";

export function useProjects() {
  const { setActiveProject } = useProjectStore();
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["projects"],
    queryFn: api.getProjects,
  });

  const createMutation = useMutation({
    mutationFn: api.createProject,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["projects"] }),
  });

  const updateMutation = useMutation({
    mutationFn: api.updateProject,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["projects"] }),
  });

  const archiveMutation = useMutation({
    mutationFn: api.archiveProject,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["projects"] }),
  });

  return {
    projects: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    createProject: createMutation.mutateAsync,
    updateProject: updateMutation.mutateAsync,
    archiveProject: archiveMutation.mutateAsync,
    refetch: query.refetch,
  };
}
