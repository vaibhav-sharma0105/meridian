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
import IndexingBanner from "@/components/shared/IndexingBanner";
import NotificationCenter from "@/components/notifications/NotificationCenter";
import AISettings from "@/components/ai/AISettings";
import ConnectionsSettings from "@/components/connections/ConnectionsSettings";
import { IntegrationsPage, LinkPicker } from "@/components/integrations";
import MigrationWizard from "@/components/settings/MigrationWizard";
import { getNotifications, getMigrationStatus, queueEmbeddingMigration } from "@/lib/tauri";
import { useNotificationStore } from "@/stores/notificationStore";
import { useSync } from "@/hooks/useSync";

export default function AppShell() {
  const { sidebarOpen, rightPanelOpen, activeView, notificationCenterOpen, settingsOpen, settingsTab, setNotificationCenterOpen, setSettingsOpen, selectedTaskId, linkPickerTaskId, setLinkPickerTaskId } = useUIStore();
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

    // Queue embedding jobs for documents that need them (runs in background)
    queueEmbeddingMigration().catch(console.error);
  }, []);

  return (
    <div className="flex flex-col h-full bg-[#f0f0f4] dark:bg-[#0a0a0d] overflow-hidden">
      <UpdateBanner />
      <IndexingBanner />
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
      <AISettings open={settingsOpen && settingsTab === "ai"} onClose={() => setSettingsOpen(false)} />
      <ConnectionsSettings open={settingsOpen && settingsTab === "connections"} onClose={() => setSettingsOpen(false)} runSync={runSync} isSyncing={isSyncing} />
      {settingsOpen && settingsTab === "integrations" && (
        <div className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 overflow-y-auto py-8" onClick={() => setSettingsOpen(false)}>
          <div className="relative w-full max-w-4xl bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-zinc-200 dark:border-zinc-800 mx-4" onClick={(e) => e.stopPropagation()}>
            <button
              onClick={() => setSettingsOpen(false)}
              className="absolute top-4 right-4 p-1.5 text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800 z-10"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
            <IntegrationsPage />
          </div>
        </div>
      )}
      <MigrationWizard open={showMigration} onClose={() => setShowMigration(false)} onComplete={() => setShowMigration(false)} />
      {selectedTask && <TaskEditModal task={selectedTask} />}
      {linkPickerTaskId && (
        <LinkPicker
          taskId={linkPickerTaskId}
          onClose={() => setLinkPickerTaskId(null)}
        />
      )}
    </div>
  );
}
