import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import type {
  Integration,
  CreateIntegrationInput,
  UpdateIntegrationInput,
  SyncState,
  AvailableIntegration,
  IntegrationCache,
} from "@/lib/tauri";

export function useIntegrations() {
  return useQuery({
    queryKey: ["integrations"],
    queryFn: () => api.listIntegrations(),
  });
}

export function useIntegration(id: string | undefined) {
  return useQuery({
    queryKey: ["integration", id],
    queryFn: () => (id ? api.getIntegration(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useAvailableIntegrations() {
  return useQuery({
    queryKey: ["available-integrations"],
    queryFn: () => api.getAvailableIntegrations(),
    staleTime: Infinity,
  });
}

export function useCreateIntegration() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateIntegrationInput) => api.createIntegration(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["integrations"] });
    },
  });
}

export function useUpdateIntegration() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateIntegrationInput) => api.updateIntegration(input),
    onSuccess: (data) => {
      qc.invalidateQueries({ queryKey: ["integrations"] });
      qc.setQueryData(["integration", data.id], data);
    },
  });
}

export function useDeleteIntegration() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteIntegration(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["integrations"] });
    },
  });
}

export function useSyncIntegration() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.syncIntegration(id),
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: ["integration", id] });
      qc.invalidateQueries({ queryKey: ["integrations"] });
      qc.invalidateQueries({ queryKey: ["integration-cache", id] });
    },
  });
}

export function useSyncStatus(id: string | undefined) {
  return useQuery({
    queryKey: ["sync-status", id],
    queryFn: () => (id ? api.getSyncStatus(id) : Promise.resolve(null)),
    enabled: !!id,
    refetchInterval: (query) => {
      const data = query.state.data;
      if (data?.status === "syncing") {
        return 2000;
      }
      return false;
    },
  });
}

export function useCachedItems(integrationId: string | undefined, externalType?: string) {
  return useQuery({
    queryKey: ["integration-cache", integrationId, externalType],
    queryFn: () =>
      integrationId
        ? api.getCachedItems(integrationId, externalType)
        : Promise.resolve([]),
    enabled: !!integrationId,
  });
}

export function useClearCache() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.clearIntegrationCache(id),
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: ["integration-cache", id] });
    },
  });
}

export function useStartOAuth() {
  return useMutation({
    mutationFn: ({
      integrationType,
      redirectUri,
    }: {
      integrationType: string;
      redirectUri: string;
    }) => api.startOAuthFlow(integrationType, redirectUri),
  });
}

export function useHandleOAuthCallback() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ state, code }: { state: string; code: string }) =>
      api.handleOAuthCallback(state, code),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["integrations"] });
    },
  });
}

export function useRefreshToken() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.refreshIntegrationToken(id),
    onSuccess: (data) => {
      qc.setQueryData(["integration", data.id], data);
      qc.invalidateQueries({ queryKey: ["integrations"] });
    },
  });
}
