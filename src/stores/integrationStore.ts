import { create } from "zustand";

interface OAuthState {
  isConnecting: boolean;
  integrationType: string | null;
  authUrl: string | null;
  error: string | null;
}

interface SyncProgress {
  integrationId: string;
  status: "idle" | "syncing" | "success" | "error";
  itemsSynced: number;
  error: string | null;
}

interface IntegrationStore {
  oauthState: OAuthState;
  syncProgress: Record<string, SyncProgress>;

  startOAuth: (integrationType: string, authUrl: string) => void;
  completeOAuth: () => void;
  failOAuth: (error: string) => void;
  resetOAuth: () => void;

  setSyncProgress: (integrationId: string, progress: Partial<SyncProgress>) => void;
  clearSyncProgress: (integrationId: string) => void;
}

export const useIntegrationStore = create<IntegrationStore>((set) => ({
  oauthState: {
    isConnecting: false,
    integrationType: null,
    authUrl: null,
    error: null,
  },
  syncProgress: {},

  startOAuth: (integrationType, authUrl) =>
    set({
      oauthState: {
        isConnecting: true,
        integrationType,
        authUrl,
        error: null,
      },
    }),

  completeOAuth: () =>
    set({
      oauthState: {
        isConnecting: false,
        integrationType: null,
        authUrl: null,
        error: null,
      },
    }),

  failOAuth: (error) =>
    set((state) => ({
      oauthState: {
        ...state.oauthState,
        isConnecting: false,
        error,
      },
    })),

  resetOAuth: () =>
    set({
      oauthState: {
        isConnecting: false,
        integrationType: null,
        authUrl: null,
        error: null,
      },
    }),

  setSyncProgress: (integrationId, progress) =>
    set((state) => {
      const existing = state.syncProgress[integrationId] ?? {
        integrationId,
        status: "idle" as const,
        itemsSynced: 0,
        error: null,
      };
      return {
        syncProgress: {
          ...state.syncProgress,
          [integrationId]: { ...existing, ...progress },
        },
      };
    }),

  clearSyncProgress: (integrationId) =>
    set((state) => {
      const { [integrationId]: _, ...rest } = state.syncProgress;
      return { syncProgress: rest };
    }),
}));
