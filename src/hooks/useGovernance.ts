import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";

export function usePendingApprovals(status?: string) {
  return useQuery({
    queryKey: ["pendingApprovals", status],
    queryFn: () => api.getPendingApprovals(status),
    refetchInterval: 5000,
  });
}

export function usePendingApprovalCount() {
  return useQuery({
    queryKey: ["pendingApprovalCount"],
    queryFn: () => api.getPendingApprovalCount(),
    refetchInterval: 10000,
  });
}

export function useUndoableActions(limit?: number) {
  return useQuery({
    queryKey: ["undoableActions", limit],
    queryFn: () => api.getUndoableActions(limit),
  });
}

export function useActionHistory(
  entityType?: string,
  entityId?: string,
  limit?: number
) {
  return useQuery({
    queryKey: ["actionHistory", entityType, entityId, limit],
    queryFn: () => api.getActionHistory(entityType, entityId, limit),
  });
}

export function useGovernanceMetrics(
  startDate: string,
  endDate: string,
  metricType?: string
) {
  return useQuery({
    queryKey: ["governanceMetrics", startDate, endDate, metricType],
    queryFn: () => api.getGovernanceMetrics(startDate, endDate, metricType),
    staleTime: 60000,
  });
}

export function useAutonomySetting(key: string) {
  return useQuery({
    queryKey: ["autonomySetting", key],
    queryFn: () => api.getAutonomySetting(key),
  });
}

export function useApproveAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.approvePendingAction(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pendingApprovals"] });
      qc.invalidateQueries({ queryKey: ["pendingApprovalCount"] });
    },
  });
}

export function useRejectAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason?: string }) =>
      api.rejectPendingAction(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pendingApprovals"] });
      qc.invalidateQueries({ queryKey: ["pendingApprovalCount"] });
    },
  });
}

export function useBulkApprove() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (ids: string[]) => api.bulkApproveActions(ids),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pendingApprovals"] });
      qc.invalidateQueries({ queryKey: ["pendingApprovalCount"] });
    },
  });
}

export function useBulkReject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ ids, reason }: { ids: string[]; reason?: string }) =>
      api.bulkRejectActions(ids, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["pendingApprovals"] });
      qc.invalidateQueries({ queryKey: ["pendingApprovalCount"] });
    },
  });
}

export function useUndoAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (actionId: string) => api.undoAction(actionId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["undoableActions"] });
      qc.invalidateQueries({ queryKey: ["actionHistory"] });
    },
  });
}

export function useSetAutonomySetting() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value?: string }) =>
      api.setAutonomySetting(key, value),
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: ["autonomySetting", variables.key] });
    },
  });
}

export function useCreateRiskAdjustment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      adjustmentType: string;
      targetType: string;
      targetId: string;
      riskDelta: number;
      reason?: string;
    }) =>
      api.createRiskAdjustment(
        params.adjustmentType,
        params.targetType,
        params.targetId,
        params.riskDelta,
        params.reason
      ),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["riskAdjustment"] });
    },
  });
}

export function useDeleteRiskAdjustment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ targetType, targetId }: { targetType: string; targetId: string }) =>
      api.deleteRiskAdjustment(targetType, targetId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["riskAdjustment"] });
    },
  });
}
