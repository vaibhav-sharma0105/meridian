import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Bell, Settings, Settings2, Plus, LayoutList,
  Sun, Moon, Monitor, Link2,
} from "lucide-react";
import { useProjectStore } from "@/stores/projectStore";
import { useUIStore } from "@/stores/uiStore";
import { useNotificationStore } from "@/stores/notificationStore";
import { useProjects } from "@/hooks/useProjects";
import { usePendingImports } from "@/hooks/usePendingImports";
import ProjectCreate from "@/components/projects/ProjectCreate";
import ProjectSettings from "@/components/projects/ProjectSettings";
import { setAppSetting } from "@/lib/tauri";
import type { Project } from "@/lib/tauri";

function MeridianLogo() {
  return (
    <svg width="24" height="24" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
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
  const { pendingCount } = usePendingImports();
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [settingsProject, setSettingsProject] = useState<Project | null>(null);

  const cycleTheme = async () => {
    const next = theme === "light" ? "dark" : theme === "dark" ? "system" : "light";
    setTheme(next);
    await setAppSetting("theme", next);
  };

  const ThemeIcon = theme === "light" ? Sun : theme === "dark" ? Moon : Monitor;
  const themeLabel = theme === "light" ? "Switch to dark mode" : theme === "dark" ? "Switch to system theme" : "Switch to light mode";
  const totalBadge = unreadCount + pendingCount;

  return (
    <div className="flex flex-col h-full bg-white dark:bg-[#111113] select-none">

      {/* ── Brand ──────────────────────────────────────────────────────────── */}
      <div className="flex items-center gap-2.5 px-4 h-12 flex-shrink-0 border-b border-zinc-100/80 dark:border-[#1c1c20]">
        <MeridianLogo />
        <span className="font-semibold text-[13px] tracking-[-0.02em] text-zinc-900 dark:text-zinc-100">Meridian</span>
      </div>

      {/* ── Global nav ─────────────────────────────────────────────────────── */}
      <div className="px-2 pb-1">
        <NavItem
          icon={<LayoutList className="w-4 h-4" />}
          label={t("nav.allTasks")}
          active={activeProjectId === null && activeView === "tasks"}
          onClick={() => { setActiveProject(null); setActiveView("tasks"); }}
        />
      </div>

      {/* ── Projects section ───────────────────────────────────────────────── */}
      <div className="flex items-center justify-between px-4 pt-3 pb-1">
        <span className="text-[11px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider">
          Projects
        </span>
        <button
          onClick={() => setShowCreateProject(true)}
          title={t("nav.newProject")}
          className="p-0.5 rounded text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* ── Project list ───────────────────────────────────────────────────── */}
      <div className="flex-1 overflow-y-auto px-2 pb-2">
        {projects.map((project) => {
          const isActive = activeProjectId === project.id;
          return (
            <div
              key={project.id}
              className={`group flex items-center gap-2 px-2 py-1.5 rounded-md text-[13px] transition-colors cursor-pointer ${
                isActive
                  ? "bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300"
                  : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/60 hover:text-zinc-900 dark:hover:text-zinc-200"
              }`}
              onClick={() => { setActiveProject(project.id); setActiveView("tasks"); }}
              onContextMenu={(e) => { e.preventDefault(); setSettingsProject(project); }}
            >
              <span
                className="w-[7px] h-[7px] rounded-full flex-shrink-0 shadow-[0_0_0_1.5px_rgba(0,0,0,0.08)] dark:shadow-[0_0_0_1.5px_rgba(255,255,255,0.06)]"
                style={{ backgroundColor: project.color }}
              />
              <span className="flex-1 truncate">{project.name}</span>
              {(project.open_task_count ?? 0) > 0 && (
                <span className={`text-[11px] tabular-nums px-1.5 py-0.5 rounded-full leading-none ${
                  isActive
                    ? "bg-indigo-100 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-400"
                    : "bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400"
                }`}>
                  {project.open_task_count}
                </span>
              )}
              <button
                onClick={(e) => { e.stopPropagation(); setSettingsProject(project); }}
                title={t("projects.settings")}
                className="p-0.5 rounded opacity-0 group-hover:opacity-100 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-all flex-shrink-0"
              >
                <Settings2 className="w-3 h-3" />
              </button>
            </div>
          );
        })}

        {projects.length === 0 && (
          <p className="text-[12px] text-zinc-400 dark:text-zinc-600 px-2 py-2 italic">No projects yet</p>
        )}
      </div>

      {/* ── Utility strip — icon-only with tooltips ──────────────────────── */}
      <div className="border-t border-zinc-100 dark:border-[#1f1f23] px-2 py-2 flex items-center gap-0.5">
        {/* Notifications */}
        <button
          onClick={() => setNotificationCenterOpen(true)}
          title={`Notifications${totalBadge > 0 ? ` (${totalBadge})` : ""}`}
          className="relative flex-1 flex items-center justify-center p-1.5 rounded-md text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Bell className="w-[15px] h-[15px]" />
          {totalBadge > 0 && (
            <span className="absolute top-1 right-1.5 w-[6px] h-[6px] bg-red-500 rounded-full ring-[1.5px] ring-white dark:ring-[#111113]" />
          )}
        </button>

        {/* Connections */}
        <button
          onClick={() => setSettingsOpen(true, "connections")}
          title="Connections"
          className="flex-1 flex items-center justify-center p-1.5 rounded-md text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Link2 className="w-[15px] h-[15px]" />
        </button>

        {/* Settings */}
        <button
          onClick={() => setSettingsOpen(true, "ai")}
          title={t("nav.settings")}
          className="flex-1 flex items-center justify-center p-1.5 rounded-md text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Settings className="w-[15px] h-[15px]" />
        </button>

        {/* Theme */}
        <button
          onClick={cycleTheme}
          title={themeLabel}
          className="flex-1 flex items-center justify-center p-1.5 rounded-md text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <ThemeIcon className="w-[15px] h-[15px]" />
        </button>
      </div>

      {/* Modals */}
      {showCreateProject && <ProjectCreate onClose={() => setShowCreateProject(false)} />}
      {settingsProject && (
        <ProjectSettings project={settingsProject} open={true} onClose={() => setSettingsProject(null)} />
      )}
    </div>
  );
}

/* ── Shared nav item ──────────────────────────────────────────────────────── */
function NavItem({
  icon, label, active, onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-[13px] transition-colors ${
        active
          ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 font-medium"
          : "text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/60 hover:text-zinc-800 dark:hover:text-zinc-200"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}
