import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";

export function useDocuments(projectId: string | null) {
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["documents", projectId],
    queryFn: () => api.getDocumentsForProject(projectId!),
    enabled: !!projectId,
  });

  const uploadMutation = useMutation({
    mutationFn: ({ filePath }: { filePath: string }) =>
      api.uploadDocument(projectId!, filePath),
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["documents", projectId] }),
  });

  const uploadUrlMutation = useMutation({
    mutationFn: ({ url }: { url: string }) =>
      api.uploadUrl(projectId!, url),
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["documents", projectId] }),
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteDocument,
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["documents", projectId] }),
  });

  return {
    documents: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    uploadDocument: uploadMutation.mutateAsync,
    uploadUrl: uploadUrlMutation.mutateAsync,
    deleteDocument: deleteMutation.mutateAsync,
    isUploading: uploadMutation.isPending || uploadUrlMutation.isPending,
    refetch: query.refetch,
  };
}
