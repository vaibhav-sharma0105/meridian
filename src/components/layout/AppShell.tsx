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
import { getNotifications } from "@/lib/tauri";
import { useNotificationStore } from "@/stores/notificationStore";

export default function AppShell() {
  const { sidebarOpen, rightPanelOpen, notificationCenterOpen, settingsOpen, setNotificationCenterOpen, setSettingsOpen } = useUIStore();
  const { fetchProjects } = useProjectStore();
  const { setNotifications } = useNotificationStore();

  useKeyboardShortcuts();

  useEffect(() => {
    fetchProjects();
    getNotifications().then(setNotifications).catch(console.error);
  }, []);

  return (
    <div className="flex flex-col h-full bg-zinc-50 dark:bg-zinc-950 overflow-hidden">
      <UpdateBanner />
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar */}
        {sidebarOpen && (
          <div className="w-60 flex-shrink-0 border-r border-zinc-200 dark:border-zinc-800 flex flex-col overflow-hidden">
            <Sidebar />
          </div>
        )}

        {/* Main Canvas */}
        <div className="flex-1 flex flex-col overflow-hidden min-w-0">
          <MainCanvas />
        </div>

        {/* Right Context Panel */}
        {rightPanelOpen && (
          <div className="w-90 flex-shrink-0 border-l border-zinc-200 dark:border-zinc-800 flex flex-col overflow-hidden"
               style={{ width: "360px" }}>
            <ContextPanel />
          </div>
        )}
      </div>

      <CommandPalette />
      <NotificationCenter open={notificationCenterOpen} onClose={() => setNotificationCenterOpen(false)} />
      <AISettings open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}
