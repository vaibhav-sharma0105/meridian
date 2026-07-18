import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Bell, Settings, Settings2, Plus, LayoutList, Zap,
  Sun, Moon, Monitor, Link2, Sparkles, ChevronDown, ChevronRight, ArchiveRestore,
} from "lucide-react";
import { useProjectStore } from "@/stores/projectStore";
import { useUIStore } from "@/stores/uiStore";
import { useNotificationStore } from "@/stores/notificationStore";
import { useProjects } from "@/hooks/useProjects";
import { usePendingImports } from "@/hooks/usePendingImports";
import ProjectCreate from "@/components/projects/ProjectCreate";
import ProjectSettings from "@/components/projects/ProjectSettings";
import { setAppSetting, getArchivedProjects, unarchiveProject } from "@/lib/tauri";
import type { Project } from "@/lib/tauri";

function MeridianLogo() {
  return (
    <svg width="26" height="26" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="28" height="28" rx="7" fill="#6366f1" />
      <path d="M7 20 L14 8 L21 20" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" fill="none" />
      <path d="M10.5 16 L17.5 16" stroke="white" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export default function Sidebar() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { projects } = useProjects();
  const { activeProjectId, setActiveProject, loadProjects } = useProjectStore();
  const { theme, setTheme, setNotificationCenterOpen, setSettingsOpen, activeView, setActiveView, rightPanelOpen, toggleRightPanel } = useUIStore();
  const { unreadCount } = useNotificationStore();
  const { pendingCount } = usePendingImports();
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [settingsProject, setSettingsProject] = useState<Project | null>(null);
  const [showArchived, setShowArchived] = useState(false);

  const { data: archivedProjects = [] } = useQuery({
    queryKey: ["archivedProjects"],
    queryFn: getArchivedProjects,
  });

  const handleUnarchive = async (id: string) => {
    await unarchiveProject(id);
    // Refetch both queries to ensure UI updates immediately
    await Promise.all([
      queryClient.refetchQueries({ queryKey: ["projects"] }),
      queryClient.refetchQueries({ queryKey: ["archivedProjects"] }),
    ]);
    await loadProjects();
  };

  const cycleTheme = async () => {
    const next = theme === "light" ? "dark" : theme === "dark" ? "system" : "light";
    setTheme(next);
    await setAppSetting("theme", next);
  };

  const ThemeIcon = theme === "light" ? Sun : theme === "dark" ? Moon : Monitor;
  const themeLabel = theme === "light" ? "Switch to dark mode" : theme === "dark" ? "Switch to system theme" : "Switch to light mode";
  const totalBadge = unreadCount + pendingCount;

  return (
    <div className="flex flex-col h-full bg-white dark:bg-[#0f0f12] select-none">

      {/* ── Brand ──────────────────────────────────────────────────────────── */}
      <div className="flex items-center gap-3 px-4 h-14 flex-shrink-0 border-b border-[#ebebf0] dark:border-[#1a1a1e]">
        <MeridianLogo />
        <span className="font-bold text-[14px] tracking-[-0.025em] text-zinc-900 dark:text-zinc-50">Meridian</span>
      </div>

      {/* ── Global nav ─────────────────────────────────────────────────────── */}
      <div className="px-3 pt-3 pb-1 space-y-0.5">
        <NavItem
          icon={<LayoutList className="w-[17px] h-[17px]" />}
          label={t("nav.allTasks")}
          active={activeProjectId === null && activeView === "tasks"}
          onClick={() => { setActiveProject(null); setActiveView("tasks"); }}
        />
        <NavItem
          icon={<Zap className="w-[17px] h-[17px]" />}
          label="Skills"
          active={activeView === "skills"}
          onClick={() => { setActiveProject(null); setActiveView("skills"); }}
        />
      </div>

      {/* ── Projects section ───────────────────────────────────────────────── */}
      <div className="flex items-center justify-between px-4 pt-4 pb-1.5">
        <span className="text-[11px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-[0.07em]">
          Projects
        </span>
        <button
          onClick={() => setShowCreateProject(true)}
          title={t("nav.newProject")}
          className="p-1 rounded-md text-zinc-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 transition-all duration-150"
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* ── Project list ───────────────────────────────────────────────────── */}
      <div className="flex-1 overflow-y-auto px-3 pb-3">
        {projects.map((project) => {
          const isActive = activeProjectId === project.id;
          return (
            <div
              key={project.id}
              className={`group flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13.5px] transition-all duration-150 cursor-pointer ${
                isActive
                  ? "bg-indigo-50 dark:bg-indigo-950/50 text-indigo-700 dark:text-indigo-300 shadow-[inset_2px_0_0_0_#6366f1] font-medium"
                  : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 hover:text-zinc-900 dark:hover:text-zinc-200"
              }`}
              onClick={() => { setActiveProject(project.id); setActiveView("tasks"); }}
              onContextMenu={(e) => { e.preventDefault(); setSettingsProject(project); }}
            >
              <span
                className="w-2 h-2 rounded-full flex-shrink-0 shadow-[0_0_0_1.5px_rgba(0,0,0,0.08)] dark:shadow-[0_0_0_1.5px_rgba(255,255,255,0.06)]"
                style={{ backgroundColor: project.color }}
              />
              <span className="flex-1 truncate">{project.name}</span>
              {(project.open_task_count ?? 0) > 0 && (
                <span className={`text-[11px] tabular-nums px-1.5 py-0.5 rounded-full leading-none font-medium ${
                  isActive
                    ? "bg-indigo-100 dark:bg-indigo-900/60 text-indigo-600 dark:text-indigo-400"
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
                <Settings2 className="w-3.5 h-3.5" />
              </button>
            </div>
          );
        })}

        {projects.length === 0 && (
          <p className="text-[12.5px] text-zinc-400 dark:text-zinc-600 px-3 py-2.5 italic leading-relaxed">No projects yet.<br />Create one to get started.</p>
        )}

        {/* Archived Projects */}
        {archivedProjects.length > 0 && (
          <div className="mt-4 pt-3 border-t border-zinc-100 dark:border-zinc-800">
            <button
              onClick={() => setShowArchived(!showArchived)}
              className="w-full flex items-center gap-2 px-3 py-1.5 text-[11px] font-medium text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-400"
            >
              {showArchived ? (
                <ChevronDown className="w-3 h-3" />
              ) : (
                <ChevronRight className="w-3 h-3" />
              )}
              Archived ({archivedProjects.length})
            </button>
            {showArchived && (
              <div className="mt-1 space-y-0.5">
                {archivedProjects.map((project) => (
                  <div
                    key={project.id}
                    className="group flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13px] text-zinc-400 dark:text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800/30"
                  >
                    <span
                      className="w-2 h-2 rounded-full flex-shrink-0 opacity-50"
                      style={{ backgroundColor: project.color }}
                    />
                    <span className="flex-1 truncate line-through">{project.name}</span>
                    <button
                      onClick={() => handleUnarchive(project.id)}
                      title="Restore project"
                      className="p-1 rounded opacity-0 group-hover:opacity-100 text-zinc-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 transition-all"
                    >
                      <ArchiveRestore className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* ── Utility strip — icon buttons with tooltips ───────────────────── */}
      <div className="border-t border-[#ebebf0] dark:border-[#1a1a1e] px-3 py-2.5 flex items-center gap-0.5">
        {/* Notifications */}
        <button
          onClick={() => setNotificationCenterOpen(true)}
          title={`Notifications${totalBadge > 0 ? ` (${totalBadge})` : ""}`}
          className="relative flex-1 flex items-center justify-center p-2 rounded-lg text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all duration-150"
        >
          <Bell className="w-[17px] h-[17px]" />
          {totalBadge > 0 && (
            <span className="absolute top-1.5 right-1.5 w-[7px] h-[7px] bg-red-500 rounded-full ring-[2px] ring-white dark:ring-[#0f0f12] shadow-sm" />
          )}
        </button>

        {/* Integrations */}
        <button
          onClick={() => setSettingsOpen(true, "integrations")}
          title="Integrations"
          className="flex-1 flex items-center justify-center p-2 rounded-lg text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all duration-150"
        >
          <Link2 className="w-[17px] h-[17px]" />
        </button>

        {/* AI Panel toggle — always indigo-tinted to stand out; gradient fill when active */}
        <button
          onClick={toggleRightPanel}
          title={rightPanelOpen && activeView !== "chat" ? "Hide AI panel" : "Show AI panel"}
          className={`relative flex-1 flex items-center justify-center p-2 rounded-lg transition-all duration-200 ${
            rightPanelOpen && activeView !== "chat"
              ? "shadow-sm"
              : "text-indigo-400 dark:text-indigo-500 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40"
          }`}
          style={rightPanelOpen && activeView !== "chat" ? {
            background: "linear-gradient(135deg, #6366f1, #8b5cf6)",
            color: "white",
          } : undefined}
        >
          <Sparkles className="w-[17px] h-[17px]" />
        </button>

        {/* Settings */}
        <button
          onClick={() => setSettingsOpen(true, "ai")}
          title={t("nav.settings")}
          className="flex-1 flex items-center justify-center p-2 rounded-lg text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all duration-150"
        >
          <Settings className="w-[17px] h-[17px]" />
        </button>

        {/* Theme */}
        <button
          onClick={cycleTheme}
          title={themeLabel}
          className="flex-1 flex items-center justify-center p-2 rounded-lg text-zinc-400 dark:text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all duration-150"
        >
          <ThemeIcon className="w-[17px] h-[17px]" />
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
      className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13.5px] transition-all duration-150 ${
        active
          ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 font-medium shadow-[inset_2px_0_0_0_#6366f1]"
          : "text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 hover:text-zinc-800 dark:hover:text-zinc-200"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}
