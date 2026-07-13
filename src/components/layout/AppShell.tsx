import { useEffect, useState } from "react";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTasks } from "@/hooks/useTasks";
import Sidebar from "./Sidebar";
import MainCanvas from "./MainCanvas";
import ContextPanel from "./ContextPanel";
import TaskEditModal from "@/components/tasks/TaskEditModal";
import CommandPalette from "@/components/command-palette/CommandPalette";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import UpdateBanner from "@/components/shared/UpdateBanner";
import NotificationCenter from "@/components/notifications/NotificationCenter";
import AISettings from "@/components/ai/AISettings";
import ConnectionsSettings from "@/components/connections/ConnectionsSettings";
import MigrationWizard from "@/components/settings/MigrationWizard";
import { getNotifications, getMigrationStatus } from "@/lib/tauri";
import { useNotificationStore } from "@/stores/notificationStore";
import { useSync } from "@/hooks/useSync";

export default function AppShell() {
  const { sidebarOpen, rightPanelOpen, activeView, notificationCenterOpen, settingsOpen, settingsTab, setNotificationCenterOpen, setSettingsOpen, selectedTaskId } = useUIStore();
  const { fetchProjects, activeProjectId } = useProjectStore();
  const { setNotifications } = useNotificationStore();
  const { tasks } = useTasks(activeProjectId);
  const selectedTask = selectedTaskId ? tasks.find((t) => t.id === selectedTaskId) ?? null : null;
  const [showMigration, setShowMigration] = useState(false);

  useKeyboardShortcuts();
  const { runSync, isSyncing } = useSync();

  useEffect(() => {
    fetchProjects();
    getNotifications().then(setNotifications).catch(console.error);

    // Check if database needs migration
    getMigrationStatus().then((status) => {
      if (status.needs_migration) {
        setShowMigration(true);
      }
    }).catch(console.error);
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

        {/* Right AI Panel — slides in/out, hidden when chat tab is active */}
        <div
          className="flex-shrink-0 flex flex-col overflow-hidden transition-[width] duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
          style={{ width: rightPanelOpen && activeView !== "chat" ? "320px" : "0px" }}
        >
          <div className="w-[320px] h-full border-l border-[#e2e2e8] dark:border-[#1e1e24] flex flex-col shadow-[-1px_0_0_0_rgba(0,0,0,0.03)]">
            <ContextPanel />
          </div>
        </div>
      </div>

      <CommandPalette />
      <NotificationCenter open={notificationCenterOpen} onClose={() => setNotificationCenterOpen(false)} />
      <AISettings open={settingsOpen && settingsTab !== "connections"} onClose={() => setSettingsOpen(false)} />
      <ConnectionsSettings open={settingsOpen && settingsTab === "connections"} onClose={() => setSettingsOpen(false)} runSync={runSync} isSyncing={isSyncing} />
      <MigrationWizard open={showMigration} onClose={() => setShowMigration(false)} onComplete={() => setShowMigration(false)} />
      {selectedTask && <TaskEditModal task={selectedTask} />}
    </div>
  );
}
