import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import TaskInlineEditor from "@/components/tasks/TaskInlineEditor";
import AIChatPanel from "@/components/ai/AIChatPanel";
import { useTasks } from "@/hooks/useTasks";
import MeetingHealthBadge from "@/components/meetings/MeetingHealthBadge";
import { parseTags } from "@/lib/validators";
import { TAG_COLORS } from "@/lib/constants";
import { useTranslation } from "react-i18next";

export default function ContextPanel() {
  const { selectedTaskId, selectedMeetingId } = useUIStore();
  const { activeProjectId } = useProjectStore();
  const pid = activeProjectId;
  const { tasks } = useTasks(pid);
  const { t } = useTranslation();

  const selectedTask = selectedTaskId
    ? tasks.find((t) => t.id === selectedTaskId)
    : null;

  return (
    <div className="flex flex-col h-full bg-white dark:bg-zinc-900 overflow-hidden">
      {/* Task Detail */}
      {selectedTask && (
        <div className="flex-1 overflow-y-auto p-4 border-b border-zinc-100 dark:border-zinc-800">
          <TaskInlineEditor task={selectedTask} />
        </div>
      )}

      {!selectedTask && !selectedMeetingId && (
        <div className="flex-1 overflow-hidden flex flex-col">
          <div className="flex-1 flex items-center justify-center text-zinc-400 text-sm p-4 text-center">
            <div>
              <p className="font-medium text-zinc-500 dark:text-zinc-400">Project Context</p>
              <p className="text-xs mt-1">Select a task to view details, or use the chat below</p>
            </div>
          </div>
        </div>
      )}

      {/* AI Chat Panel — always at bottom */}
      <div className="flex-shrink-0" style={{ height: selectedTask ? "50%" : "100%" }}>
        <AIChatPanel projectId={pid} />
      </div>
    </div>
  );
}
