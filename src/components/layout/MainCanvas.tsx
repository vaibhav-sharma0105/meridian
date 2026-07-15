import { useState } from "react";
import { useTranslation } from "react-i18next";
import { LayoutList, Columns, Table, Plus, FileText, Upload, BarChart2, MessageSquare, Archive } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import TaskListView from "@/components/tasks/TaskListView";
import TaskKanbanView from "@/components/tasks/TaskKanbanView";
import TaskTableView from "@/components/tasks/TaskTableView";
import TaskFilters from "@/components/tasks/TaskFilters";
import TaskBulkActions from "@/components/tasks/TaskBulkActions";
import MeetingIngest from "@/components/meetings/MeetingIngest";
import { useMeetings } from "@/hooks/useMeetings";
import MeetingCard from "@/components/meetings/MeetingCard";
import DocFolder from "@/components/documents/DocFolder";
import ProjectDashboard from "@/components/analytics/ProjectDashboard";
import AIChatPanel from "@/components/ai/AIChatPanel";
import EmptyState from "@/components/shared/EmptyState";
import { useTaskStore } from "@/stores/taskStore";
import { SkillsPage } from "@/components/skills/SkillsPage";

const VIEW_ICONS = {
  list: LayoutList,
  kanban: Columns,
  table: Table,
};

export default function MainCanvas() {
  const { t } = useTranslation();
  const { viewMode, setViewMode, activeView, setActiveView, setIngestModalOpen } = useUIStore();
  const { activeProjectId, getActiveProject } = useProjectStore();
  const { selectedTaskIds } = useTaskStore();
  const activeProject = getActiveProject();
  const [showArchivedMeetings, setShowArchivedMeetings] = useState(false);
  const { meetings, deleteMeeting, forceDeleteMeeting, unarchiveMeeting } = useMeetings(activeProjectId, showArchivedMeetings);

  const tabs = [
    { id: "tasks", label: t("tasks.title"), icon: LayoutList },
    { id: "meetings", label: t("meetings.title"), icon: FileText },
    { id: "documents", label: t("documents.title"), icon: Upload },
    { id: "analytics", label: t("analytics.title"), icon: BarChart2 },
    { id: "chat", label: t("ai.title"), icon: MessageSquare },
  ] as const;

  // Skills view is global (no project needed)
  if (activeView === "skills") {
    return <SkillsPage />;
  }

  if (!activeProjectId && activeView !== "tasks") {
    return (
      <div className="flex-1 flex items-center justify-center">
        <EmptyState
          title="Select a project"
          description="Choose a project from the sidebar or create a new one."
          icon={<LayoutList className="w-10 h-10 text-zinc-400" />}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* Project identity header */}
      {activeProjectId && activeProject && (
        <div className="flex items-center gap-2.5 px-5 pt-4 pb-0 flex-shrink-0">
          <span
            className="w-2.5 h-2.5 rounded-full flex-shrink-0 shadow-sm"
            style={{ backgroundColor: activeProject.color ?? "#6366f1" }}
          />
          <h1 className="text-[17px] font-bold tracking-[-0.025em] text-zinc-900 dark:text-zinc-50">{activeProject.name}</h1>
          {(activeProject.open_task_count ?? 0) > 0 && (
            <span className="text-[12px] tabular-nums px-2 py-0.5 rounded-full bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 font-medium leading-none">
              {activeProject.open_task_count} open
            </span>
          )}
        </div>
      )}

      {/* Tab bar — underline style */}
      {activeProjectId && (
        <div className="flex items-center px-4 border-b border-[#e2e2e8] dark:border-[#1e1e24] bg-white dark:bg-[#111114] flex-shrink-0">
          <div className="flex items-center flex-1 -mb-px gap-0.5">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveView(tab.id)}
                className={`flex items-center gap-2 px-3.5 py-3.5 text-[13.5px] font-medium border-b-[2.5px] transition-all duration-150 rounded-t-sm ${
                  activeView === tab.id
                    ? "border-indigo-500 text-zinc-900 dark:text-zinc-50"
                    : "border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:border-zinc-200 dark:hover:border-zinc-600"
                }`}
              >
                <tab.icon className="w-4 h-4" />
                {tab.label}
              </button>
            ))}
          </div>

          {/* View switcher for tasks */}
          {activeView === "tasks" && (
            <div className="flex items-center bg-zinc-100 dark:bg-zinc-800 rounded-lg p-[3px] gap-[2px]">
              {(["list", "kanban", "table"] as const).map((mode) => {
                const Icon = VIEW_ICONS[mode];
                return (
                  <button
                    key={mode}
                    onClick={() => setViewMode(mode)}
                    title={t(`tasks.views.${mode}`)}
                    className={`p-1.5 rounded-md transition-all duration-150 ${
                      viewMode === mode
                        ? "bg-white dark:bg-zinc-700 text-zinc-800 dark:text-zinc-100 shadow-sm"
                        : "text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-300"
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                  </button>
                );
              })}
            </div>
          )}

          {/* Meetings tab controls */}
          {activeView === "meetings" && (
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowArchivedMeetings((s) => !s)}
                title={showArchivedMeetings ? "Hide archived meetings" : "Show archived meetings"}
                className={`flex items-center gap-1.5 px-2.5 py-1.5 text-[13px] rounded-lg border transition-all duration-150 ${
                  showArchivedMeetings
                    ? "border-zinc-400 dark:border-zinc-500 bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300"
                    : "border-[#e2e2e8] dark:border-zinc-700/60 bg-white dark:bg-zinc-800/80 text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 hover:border-zinc-300"
                }`}
              >
                <Archive className="w-3.5 h-3.5" />
                {showArchivedMeetings ? "Archived on" : "Archived"}
              </button>
              <button
                onClick={() => setIngestModalOpen(true)}
                className="flex items-center gap-1.5 px-3.5 py-1.5 bg-indigo-500 hover:bg-indigo-600 active:bg-indigo-700 text-white rounded-lg text-[13.5px] font-medium transition-all duration-150 shadow-sm hover:shadow-md"
              >
                <Plus className="w-4 h-4" />
                {t("meetings.new")}
              </button>
            </div>
          )}
        </div>
      )}

      {/* Bulk actions bar */}
      {activeView === "tasks" && selectedTaskIds.length > 0 && (
        <TaskBulkActions />
      )}

      {/* All Tasks header — shown when no project is active */}
      {activeView === "tasks" && !activeProjectId && (
        <div className="flex items-center justify-between px-4 py-3 border-b border-[#e2e2e8] dark:border-[#1e1e24] bg-white dark:bg-[#111114] flex-shrink-0">
          <span className="text-[14px] font-semibold text-zinc-900 dark:text-zinc-50 tracking-tight">All Tasks</span>
          <div className="flex items-center bg-zinc-100 dark:bg-zinc-800 rounded-lg p-[3px] gap-[2px]">
            {(["list", "table"] as const).map((mode) => {
              const Icon = VIEW_ICONS[mode];
              return (
                <button
                  key={mode}
                  onClick={() => setViewMode(mode)}
                  title={mode}
                  className={`p-1.5 rounded-md transition-all duration-150 ${
                    viewMode === mode
                      ? "bg-white dark:bg-zinc-700 text-zinc-800 dark:text-zinc-100 shadow-sm"
                      : "text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-300"
                  }`}
                >
                  <Icon className="w-4 h-4" />
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Filters for tasks */}
      {activeView === "tasks" && (
        <div className="px-4 py-2.5 border-b border-[#ebebf0] dark:border-[#1a1a1e] bg-white dark:bg-[#111114] flex-shrink-0">
          <TaskFilters showProjectFilter={!activeProjectId} />
        </div>
      )}

      {/* Main content */}
      <div className={`flex-1 min-h-0 ${viewMode === "kanban" && activeView === "tasks" ? "overflow-hidden" : "overflow-auto"}`}>
        {activeView === "tasks" && (
          <>
            {/* Kanban requires a project (for task creation); fall back to list when in All Tasks */}
            {viewMode === "list" && <TaskListView projectId={activeProjectId} />}
            {viewMode === "kanban" && activeProjectId && <TaskKanbanView projectId={activeProjectId} />}
            {viewMode === "kanban" && !activeProjectId && <TaskListView projectId={null} />}
            {viewMode === "table" && <TaskTableView projectId={activeProjectId} />}
          </>
        )}

        {activeView === "meetings" && (
          <div className="p-5 space-y-3">
            {meetings.length === 0 ? (
              <EmptyState
                title={t("meetings.noMeetings")}
                description={t("meetings.noMeetingsDesc")}
                icon={<FileText className="w-10 h-10 text-zinc-400" />}
                action={{
                  label: t("meetings.ingest"),
                  onClick: () => setIngestModalOpen(true),
                }}
              />
            ) : (
              meetings.map((meeting) => (
                <MeetingCard
                  key={meeting.id}
                  meeting={meeting}
                  onArchive={deleteMeeting}
                  onUnarchive={unarchiveMeeting}
                  onForceDelete={forceDeleteMeeting}
                />
              ))
            )}
          </div>
        )}

        {activeView === "documents" && <DocFolder projectId={activeProjectId} />}
        {activeView === "analytics" && <ProjectDashboard projectId={activeProjectId} />}
        {activeView === "chat" && (
          <div className="h-full">
            <AIChatPanel projectId={activeProjectId} fullPage />
          </div>
        )}
      </div>

      {/* Ingest modal */}
      <MeetingIngest />
    </div>
  );
}
