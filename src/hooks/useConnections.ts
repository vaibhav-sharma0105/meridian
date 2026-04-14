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

  const sheetsRelayQuery = useQuery({
    queryKey: ["connection", "sheets_relay"],
    queryFn: () => api.getConnection("sheets_relay"),
  });

  const connectZoomMutation = useMutation({
    mutationFn: api.connectZoom,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "zoom"] }),
  });

  const connectGmailMutation = useMutation({
    mutationFn: api.connectGmail,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["connection", "gmail"] }),
  });

  const saveSheetRelayMutation = useMutation({
    mutationFn: ({ scriptUrl, secretKey }: { scriptUrl: string; secretKey: string }) =>
      api.saveSheetRelayConfig(scriptUrl, secretKey),
    onSuccess: () =>
      qc.invalidateQueries({ queryKey: ["connection", "sheets_relay"] }),
  });

  const testSheetRelayMutation = useMutation({
    mutationFn: api.testSheetsRelay,
  });

  const disconnectMutation = useMutation({
    mutationFn: api.disconnectProvider,
    onSuccess: (_: unknown, provider: string) => {
      qc.invalidateQueries({ queryKey: ["connection", provider] });
      if (provider === "sheets_relay") {
        api.resetSheetsRelaySync().catch(() => {});
      }
    },
  });

  return {
    zoom: zoomQuery.data ?? null,
    gmail: gmailQuery.data ?? null,
    sheetsRelay: sheetsRelayQuery.data ?? null,
    isLoadingZoom: zoomQuery.isLoading,
    isLoadingGmail: gmailQuery.isLoading,
    isLoadingSheetsRelay: sheetsRelayQuery.isLoading,
    connectZoom: connectZoomMutation.mutateAsync,
    isConnectingZoom: connectZoomMutation.isPending,
    connectGmail: connectGmailMutation.mutateAsync,
    isConnectingGmail: connectGmailMutation.isPending,
    saveSheetRelayConfig: saveSheetRelayMutation.mutateAsync,
    isSavingSheetRelay: saveSheetRelayMutation.isPending,
    testSheetsRelay: testSheetRelayMutation.mutateAsync,
    isTestingSheetsRelay: testSheetRelayMutation.isPending,
    sheetsRelayTestResult: testSheetRelayMutation.data ?? null,
    sheetsRelayTestError: testSheetRelayMutation.error as string | null,
    disconnect: disconnectMutation.mutateAsync,
    isDisconnecting: disconnectMutation.isPending,
    zoomError: connectZoomMutation.error as string | null,
    gmailError: connectGmailMutation.error as string | null,
    sheetsRelayError: saveSheetRelayMutation.error as string | null,
  };
}
