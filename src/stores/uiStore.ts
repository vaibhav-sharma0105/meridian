import { create } from "zustand";

type Theme = "light" | "dark" | "system";
type ViewMode = "list" | "kanban" | "table";
type ActiveView = "tasks" | "meetings" | "documents" | "analytics" | "chat";

interface UIStore {
  theme: Theme;
  language: string;
  viewMode: ViewMode;
  rightPanelOpen: boolean;
  sidebarOpen: boolean;
  activeView: ActiveView;
  selectedTaskId: string | null;
  selectedMeetingId: string | null;
  commandPaletteOpen: boolean;
  notificationCenterOpen: boolean;
  settingsOpen: boolean;
  settingsTab: string;
  ingestModalOpen: boolean;
  // Actions
  setTheme: (theme: Theme) => void;
  setLanguage: (lang: string) => void;
  setViewMode: (mode: ViewMode) => void;
  toggleRightPanel: () => void;
  toggleSidebar: () => void;
  setActiveView: (view: ActiveView) => void;
  setSelectedTask: (id: string | null) => void;
  setSelectedMeeting: (id: string | null) => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setNotificationCenterOpen: (open: boolean) => void;
  setSettingsOpen: (open: boolean, tab?: string) => void;
  setIngestModalOpen: (open: boolean) => void;
}

export const useUIStore = create<UIStore>((set) => ({
  theme: "system",
  language: "en",
  viewMode: "kanban",
  rightPanelOpen: true,
  sidebarOpen: true,
  activeView: "tasks",
  selectedTaskId: null,
  selectedMeetingId: null,
  commandPaletteOpen: false,
  notificationCenterOpen: false,
  settingsOpen: false,
  settingsTab: "ai",
  ingestModalOpen: false,

  setTheme: (theme) => set({ theme }),
  setLanguage: (language) => set({ language }),
  setViewMode: (viewMode) => set({ viewMode }),
  toggleRightPanel: () =>
    set((state) => ({ rightPanelOpen: !state.rightPanelOpen })),
  toggleSidebar: () =>
    set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  setActiveView: (activeView) =>
    set({ activeView, selectedTaskId: null, selectedMeetingId: null }),
  setSelectedTask: (selectedTaskId) =>
    set({ selectedTaskId, selectedMeetingId: null }),
  setSelectedMeeting: (selectedMeetingId) =>
    set({ selectedMeetingId, selectedTaskId: null }),
  setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
  setNotificationCenterOpen: (notificationCenterOpen) =>
    set({ notificationCenterOpen }),
  setSettingsOpen: (settingsOpen, tab) =>
    set({ settingsOpen, ...(tab ? { settingsTab: tab } : {}) }),
  setIngestModalOpen: (ingestModalOpen) => set({ ingestModalOpen }),
}));
