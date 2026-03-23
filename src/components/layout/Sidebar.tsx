import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Bell, Settings, Plus, LayoutList, CalendarDays,
  Sun, Moon, Monitor, ChevronRight,
} from "lucide-react";
import { useProjectStore } from "@/stores/projectStore";
import { useUIStore } from "@/stores/uiStore";
import { useNotificationStore } from "@/stores/notificationStore";
import { useProjects } from "@/hooks/useProjects";
import ProjectCreate from "@/components/projects/ProjectCreate";
import { setAppSetting } from "@/lib/tauri";

// Meridian Logo SVG
function MeridianLogo() {
  return (
    <svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="28" height="28" rx="6" fill="#6366f1" />
      <path d="M7 20 L14 8 L21 20" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" fill="none" />
      <path d="M10.5 16 L17.5 16" stroke="white" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export default function Sidebar() {
  const { t } = useTranslation();
  const { projects } = useProjects();
  const { activeProjectId, setActiveProject } = useProjectStore();
  const { theme, setTheme, setNotificationCenterOpen, setSettingsOpen, activeView, setActiveView } = useUIStore();
  const { unreadCount } = useNotificationStore();
  const [showCreateProject, setShowCreateProject] = useState(false);

  const cycleTheme = async () => {
    const next = theme === "light" ? "dark" : theme === "dark" ? "system" : "light";
    setTheme(next);
    await setAppSetting("theme", next);
  };

  const ThemeIcon = theme === "light" ? Sun : theme === "dark" ? Moon : Monitor;

  return (
    <div className="flex flex-col h-full bg-white dark:bg-zinc-900 select-none">
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-4 border-b border-zinc-100 dark:border-zinc-800">
        <MeridianLogo />
        <span className="font-semibold text-zinc-900 dark:text-zinc-50 text-sm">Meridian</span>
      </div>

      {/* New Project */}
      <div className="px-2 pt-2">
        <button
          onClick={() => setShowCreateProject(true)}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Plus className="w-4 h-4" />
          {t("nav.newProject")}
        </button>
      </div>

      {/* Project List */}
      <div className="flex-1 overflow-y-auto px-2 py-1">
        {projects.map((project) => (
          <button
            key={project.id}
            onClick={() => { setActiveProject(project.id); setActiveView("tasks"); }}
            className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors group ${
              activeProjectId === project.id
                ? "bg-indigo-50 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300"
                : "text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800"
            }`}
          >
            <div
              className="w-2.5 h-2.5 rounded-full flex-shrink-0"
              style={{ backgroundColor: project.color }}
            />
            <span className="flex-1 text-left truncate">{project.name}</span>
            {(project.open_task_count ?? 0) > 0 && (
              <span className={`text-xs px-1.5 py-0.5 rounded-full ${
                activeProjectId === project.id
                  ? "bg-indigo-100 text-indigo-600 dark:bg-indigo-800 dark:text-indigo-300"
                  : "bg-zinc-100 text-zinc-500 dark:bg-zinc-800"
              }`}>
                {project.open_task_count}
              </span>
            )}
          </button>
        ))}

        {projects.length === 0 && (
          <p className="text-xs text-zinc-400 px-2 py-2">No projects yet</p>
        )}
      </div>

      {/* Bottom navigation */}
      <div className="border-t border-zinc-100 dark:border-zinc-800 px-2 py-2 space-y-1">
        <button
          onClick={() => { setActiveProject(null); setActiveView("tasks"); }}
          className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
            activeProjectId === null && activeView === "tasks"
              ? "bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-zinc-50"
              : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800"
          }`}
        >
          <LayoutList className="w-4 h-4" />
          {t("nav.allTasks")}
        </button>
        <button
          onClick={() => setActiveView("meetings")}
          className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors ${
            activeView === "meetings"
              ? "bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-zinc-50"
              : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800"
          }`}
        >
          <CalendarDays className="w-4 h-4" />
          {t("nav.meetings")}
        </button>

        <div className="h-px bg-zinc-100 dark:bg-zinc-800 my-1" />

        {/* Notification bell */}
        <button
          onClick={() => setNotificationCenterOpen(true)}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <div className="relative">
            <Bell className="w-4 h-4" />
            {unreadCount > 0 && (
              <span className="absolute -top-1 -right-1 w-3.5 h-3.5 bg-red-500 text-white text-[9px] rounded-full flex items-center justify-center">
                {unreadCount > 9 ? "9+" : unreadCount}
              </span>
            )}
          </div>
          {t("nav.notifications")}
        </button>

        {/* Settings */}
        <button
          onClick={() => setSettingsOpen(true)}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Settings className="w-4 h-4" />
          {t("nav.settings")}
        </button>

        {/* Theme toggle */}
        <button
          onClick={cycleTheme}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <ThemeIcon className="w-4 h-4" />
          <span className="capitalize">{theme}</span>
        </button>

        {/* Version */}
        <div className="px-2 pt-1">
          <span className="text-[10px] text-zinc-400">v0.1.0</span>
        </div>
      </div>

      {/* Modals */}
      {showCreateProject && (
        <ProjectCreate onClose={() => setShowCreateProject(false)} />
      )}
    </div>
  );
}
