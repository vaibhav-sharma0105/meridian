import { useEffect } from "react";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import Sidebar from "./Sidebar";
import MainCanvas from "./MainCanvas";
import ContextPanel from "./ContextPanel";
import CommandPalette from "@/components/command-palette/CommandPalette";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import UpdateBanner from "@/components/shared/UpdateBanner";
import NotificationCenter from "@/components/notifications/NotificationCenter";
import AISettings from "@/components/ai/AISettings";
import ConnectionsSettings from "@/components/connections/ConnectionsSettings";
import { getNotifications } from "@/lib/tauri";
import { useNotificationStore } from "@/stores/notificationStore";
import { useSync } from "@/hooks/useSync";

export default function AppShell() {
  const { sidebarOpen, rightPanelOpen, activeView, notificationCenterOpen, settingsOpen, settingsTab, setNotificationCenterOpen, setSettingsOpen } = useUIStore();
  const { fetchProjects } = useProjectStore();
  const { setNotifications } = useNotificationStore();

  useKeyboardShortcuts();
  const { runSync, isSyncing } = useSync();

  useEffect(() => {
    fetchProjects();
    getNotifications().then(setNotifications).catch(console.error);
  }, []);

  return (
    <div className="flex flex-col h-full bg-[#f0f0f4] dark:bg-[#0a0a0d] overflow-hidden">
      <UpdateBanner />
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar */}
        {sidebarOpen && (
          <div className="w-60 flex-shrink-0 border-r border-[#e2e2e8] dark:border-[#1e1e24] flex flex-col overflow-hidden shadow-[1px_0_0_0_rgba(0,0,0,0.03)]">
            <Sidebar />
          </div>
        )}

        {/* Main Canvas */}
        <div className="flex-1 flex flex-col overflow-hidden min-w-0 bg-white dark:bg-[#111114]">
          <MainCanvas />
        </div>

        {/* Right Context Panel — hidden when chat tab is active (redundant duplicate) */}
        {rightPanelOpen && activeView !== "chat" && (
          <div className="flex-shrink-0 border-l border-[#e2e2e8] dark:border-[#1e1e24] flex flex-col overflow-hidden shadow-[-1px_0_0_0_rgba(0,0,0,0.03)]"
               style={{ width: "320px" }}>
            <ContextPanel />
          </div>
        )}
      </div>

      <CommandPalette />
      <NotificationCenter open={notificationCenterOpen} onClose={() => setNotificationCenterOpen(false)} />
      <AISettings open={settingsOpen && settingsTab !== "connections"} onClose={() => setSettingsOpen(false)} />
      <ConnectionsSettings open={settingsOpen && settingsTab === "connections"} onClose={() => setSettingsOpen(false)} runSync={runSync} isSyncing={isSyncing} />
    </div>
  );
}
