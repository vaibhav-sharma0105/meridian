import { useTranslation } from "react-i18next";
import { LayoutList, Columns, Table, Plus, FileText, Upload, BarChart2, MessageSquare } from "lucide-react";
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
  const { meetings, deleteMeeting } = useMeetings(activeProjectId);

  const activeProject = getActiveProject();

  const tabs = [
    { id: "tasks", label: t("tasks.title"), icon: LayoutList },
    { id: "meetings", label: t("meetings.title"), icon: FileText },
    { id: "documents", label: t("documents.title"), icon: Upload },
    { id: "analytics", label: t("analytics.title"), icon: BarChart2 },
    { id: "chat", label: t("ai.title"), icon: MessageSquare },
  ] as const;

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
      {/* Tab bar — underline style */}
      {activeProjectId && (
        <div className="flex items-center px-4 border-b border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 flex-shrink-0">
          <div className="flex items-center flex-1 -mb-px">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveView(tab.id)}
                className={`flex items-center gap-1.5 px-3 py-2.5 text-[13px] font-medium border-b-2 transition-colors ${
                  activeView === tab.id
                    ? "border-indigo-500 text-zinc-900 dark:text-zinc-50"
                    : "border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:border-zinc-300 dark:hover:border-zinc-600"
                }`}
              >
                <tab.icon className="w-3.5 h-3.5" />
                {tab.label}
              </button>
            ))}
          </div>

          {/* View switcher for tasks */}
          {activeView === "tasks" && (
            <div className="flex items-center bg-zinc-100 dark:bg-zinc-800 rounded-md p-0.5 gap-0.5">
              {(["list", "kanban", "table"] as const).map((mode) => {
                const Icon = VIEW_ICONS[mode];
                return (
                  <button
                    key={mode}
                    onClick={() => setViewMode(mode)}
                    title={t(`tasks.views.${mode}`)}
                    className={`p-1 rounded transition-colors ${
                      viewMode === mode
                        ? "bg-white dark:bg-zinc-700 text-zinc-800 dark:text-zinc-100 shadow-sm"
                        : "text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-300"
                    }`}
                  >
                    <Icon className="w-3.5 h-3.5" />
                  </button>
                );
              })}
            </div>
          )}

          {/* New meeting button */}
          {activeView === "meetings" && (
            <button
              onClick={() => setIngestModalOpen(true)}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-500 hover:bg-indigo-600 active:bg-indigo-700 text-white rounded-md text-[13px] font-medium transition-colors"
            >
              <Plus className="w-3.5 h-3.5" />
              {t("meetings.new")}
            </button>
          )}
        </div>
      )}

      {/* Bulk actions bar */}
      {activeView === "tasks" && selectedTaskIds.length > 0 && (
        <TaskBulkActions />
      )}

      {/* All Tasks header — shown when no project is active */}
      {activeView === "tasks" && !activeProjectId && (
        <div className="flex items-center justify-between px-4 py-2.5 border-b border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 flex-shrink-0">
          <span className="text-[13px] font-semibold text-zinc-900 dark:text-zinc-50 tracking-tight">All Tasks</span>
          <div className="flex items-center bg-zinc-100 dark:bg-zinc-800 rounded-md p-0.5 gap-0.5">
            {(["list", "table"] as const).map((mode) => {
              const Icon = VIEW_ICONS[mode];
              return (
                <button
                  key={mode}
                  onClick={() => setViewMode(mode)}
                  title={mode}
                  className={`p-1 rounded transition-colors ${
                    viewMode === mode
                      ? "bg-white dark:bg-zinc-700 text-zinc-800 dark:text-zinc-100 shadow-sm"
                      : "text-zinc-400 dark:text-zinc-500 hover:text-zinc-600 dark:hover:text-zinc-300"
                  }`}
                >
                  <Icon className="w-3.5 h-3.5" />
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Filters for tasks */}
      {activeView === "tasks" && (
        <div className="px-4 py-2 border-b border-zinc-100 dark:border-[#1a1a1e] bg-white dark:bg-zinc-900 flex-shrink-0">
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
          <div className="p-4 space-y-3">
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
                <MeetingCard key={meeting.id} meeting={meeting} onDelete={deleteMeeting} />
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
