import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "@/lib/tauri";

export function useConnections() {
  const qc = useQueryClient();

  const zoomQuery = useQuery({
    queryKey: ["connection", "zoom"],
    queryFn: () => api.getConnection("zoom"),
  });

  const gmailQuery = useQuery({
    queryKey: ["connection", "gmail"],
    queryFn: () => api.getConnection("gmail"),
  });

  const connectZoomMutation = useMutation({
    mutationFn: api.connectZoom,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "zoom"] }),
  });

  const connectGmailMutation = useMutation({
    mutationFn: api.connectGmail,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "gmail"] }),
  });

  const disconnectMutation = useMutation({
    mutationFn: api.disconnectProvider,
    onSuccess: (_: unknown, provider: string) =>
      qc.invalidateQueries({ queryKey: ["connection", provider] }),
  });

  return {
    zoom: zoomQuery.data ?? null,
    gmail: gmailQuery.data ?? null,
    isLoadingZoom: zoomQuery.isLoading,
    isLoadingGmail: gmailQuery.isLoading,
    connectZoom: connectZoomMutation.mutateAsync,
    isConnectingZoom: connectZoomMutation.isPending,
    connectGmail: connectGmailMutation.mutateAsync,
    isConnectingGmail: connectGmailMutation.isPending,
    disconnect: disconnectMutation.mutateAsync,
    isDisconnecting: disconnectMutation.isPending,
    zoomError: connectZoomMutation.error as string | null,
    gmailError: connectGmailMutation.error as string | null,
  };
}
