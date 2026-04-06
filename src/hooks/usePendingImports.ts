import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type { ImportApproval } from "@/lib/tauri";

export function usePendingImports() {
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["pending-imports"],
    queryFn: api.getPendingImports,
  });

  const countQuery = useQuery({
    queryKey: ["pending-imports-count"],
    queryFn: api.countPendingImports,
  });

  const approveMutation = useMutation({
    mutationFn: (input: ImportApproval) => api.approveImport(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pending-imports"] });
      qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
      qc.invalidateQueries({ queryKey: ["meetings"] });
      qc.invalidateQueries({ queryKey: ["tasks"] });
    },
  });

  const dismissMutation = useMutation({
    mutationFn: api.dismissImport,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pending-imports"] });
      qc.invalidateQueries({ queryKey: ["pending-imports-count"] });
    },
  });

  return {
    pendingImports: query.data ?? [],
    pendingCount: countQuery.data ?? 0,
    isLoading: query.isLoading,
    approveImport: approveMutation.mutateAsync,
    isApproving: approveMutation.isPending,
    approvingId: approveMutation.variables?.pending_import_id ?? null,
    dismissImport: dismissMutation.mutateAsync,
    isDismissing: dismissMutation.isPending,
  };
}
