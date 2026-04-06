import { useTranslation } from "react-i18next";
import { Calendar, User, Tag, Video } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTaskStore } from "@/stores/taskStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import { TAG_COLORS } from "@/lib/constants";
import { parseTags } from "@/lib/validators";
import type { Task } from "@/lib/tauri";
import { useMeetings } from "@/hooks/useMeetings";
import { format } from "date-fns";

interface Props {
  task: Task;
  compact?: boolean;
}

const PRIORITY_COLORS: Record<string, string> = {
  critical: "border-l-red-500",
  high: "border-l-orange-400",
  medium: "border-l-yellow-400",
  low: "border-l-zinc-300",
};

export default function TaskCard({ task, compact = false }: Props) {
  const { setSelectedTask } = useUIStore();
  const { selectedTaskIds, toggleTaskSelection } = useTaskStore();
  const { meetings } = useMeetings(task.project_id);
  const isSelected = selectedTaskIds.includes(task.id);
  const linkedMeeting = task.meeting_id ? meetings.find((m) => m.id === task.meeting_id) : null;
  const tags = parseTags(task.tags);
  const priorityBorder = task.priority ? (PRIORITY_COLORS[task.priority] ?? "border-l-zinc-300") : "border-l-zinc-300";

  return (
    <div
      className={`group relative bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg p-3 cursor-pointer hover:shadow-sm transition-shadow border-l-4 ${priorityBorder} ${isSelected ? "ring-2 ring-indigo-500" : ""}`}
      onClick={() => setSelectedTask(task.id)}
    >
      {/* Checkbox for bulk select */}
      <input
        type="checkbox"
        checked={isSelected}
        onChange={(e) => {
          e.stopPropagation();
          toggleTaskSelection(task.id);
        }}
        className="absolute top-3 left-3 opacity-0 group-hover:opacity-100 transition-opacity"
        onClick={(e) => e.stopPropagation()}
      />

      <div className="flex items-start justify-between gap-2 min-w-0">
        <p className={`text-zinc-900 dark:text-zinc-50 font-medium leading-snug min-w-0 break-words ${compact ? "text-xs line-clamp-2" : "text-sm"}`}>
          {task.title}
        </p>
        {task.confidence_score !== undefined && task.confidence_score !== null && (
          <TaskConfidenceBadge confidence={task.confidence_score} />
        )}
      </div>

      {!compact && task.description && (
        <p className="text-xs text-zinc-500 mt-1 line-clamp-2">{task.description}</p>
      )}

      <div className="flex items-center gap-3 mt-2 flex-wrap">
        {task.assignee && (
          <span className="flex items-center gap-1 text-xs text-zinc-500">
            <User className="w-3 h-3" />
            {task.assignee}
          </span>
        )}
        {task.due_date && (
          <span className="flex items-center gap-1 text-xs text-zinc-500">
            <Calendar className="w-3 h-3" />
            {(() => { try { return format(new Date(task.due_date), "MMM d"); } catch { return task.due_date; } })()}
          </span>
        )}
        {linkedMeeting && (
          <span className="flex items-center gap-1 text-xs text-indigo-600 dark:text-indigo-400 max-w-[120px] truncate" title={linkedMeeting.title}>
            <Video className="w-3 h-3 flex-shrink-0" />
            {linkedMeeting.title}
          </span>
        )}
        {tags.slice(0, 3).map((tag) => {
          const tc = TAG_COLORS[tag] ?? "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400";
          return (
            <span key={tag} className={`inline-flex items-center gap-0.5 text-xs px-1.5 py-0.5 rounded ${tc}`}>
              <Tag className="w-2.5 h-2.5" />
              {tag}
            </span>
          );
        })}
      </div>
    </div>
  );
}
