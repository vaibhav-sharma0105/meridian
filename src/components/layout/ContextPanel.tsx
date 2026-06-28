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
    <div className="flex flex-col h-full bg-white dark:bg-[#0f0f12] overflow-hidden">
      {/* Task Detail — exactly half the panel height when open */}
      {selectedTask && (
        <div className="h-1/2 flex-shrink-0 overflow-y-auto p-5 border-b border-[#ebebf0] dark:border-[#1a1a1e] animate-fade-in">
          <TaskInlineEditor task={selectedTask} />
        </div>
      )}

      {/* No empty state — AI panel fills full height when nothing selected */}

      {/* AI Chat Panel — other half when task open, full height otherwise */}
      <div className={`${selectedTask ? "h-1/2 flex-shrink-0" : "flex-1 min-h-0"} overflow-hidden border-t border-[#ebebf0] dark:border-[#1a1a1e]`}>
        <AIChatPanel projectId={pid} />
      </div>
    </div>
  );
}
