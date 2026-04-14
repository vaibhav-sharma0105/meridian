import { useTranslation } from "react-i18next";
import { Calendar, User, Tag, Video, Check } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTaskStore } from "@/stores/taskStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import { TAG_COLORS } from "@/lib/constants";
import { parseTags } from "@/lib/validators";
import { parseAssignees } from "./AssigneeChipInput";
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

  const hasMetadata = task.assignee || task.due_date || linkedMeeting || tags.length > 0;

  // Metadata items as an array so we can render separators cleanly
  const metaItems: React.ReactNode[] = [];
  if (!compact) {
    if (task.assignee) {
      parseAssignees(task.assignee).forEach((name) => {
        metaItems.push(
          <span key={`a-${name}`} className="flex items-center gap-1 text-[11px] text-zinc-400 dark:text-zinc-500">
            <User className="w-2.5 h-2.5" />
            {name}
          </span>
        );
      });
    }
    if (task.due_date) {
      metaItems.push(
        <span key="due" className="flex items-center gap-1 text-[11px] text-zinc-400 dark:text-zinc-500">
          <Calendar className="w-2.5 h-2.5" />
          {(() => { try { return format(new Date(task.due_date), "MMM d"); } catch { return task.due_date; } })()}
        </span>
      );
    }
    if (linkedMeeting) {
      metaItems.push(
        <span key="mtg" className="flex items-center gap-1 text-[11px] text-indigo-400/80 dark:text-indigo-400/60 max-w-[100px] truncate" title={linkedMeeting.title}>
          <Video className="w-2.5 h-2.5 flex-shrink-0" />
          {linkedMeeting.title}
        </span>
      );
    }
    tags.slice(0, 3).forEach((tag) => {
      const tc = TAG_COLORS[tag] ?? "bg-zinc-100 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400";
      metaItems.push(
        <span key={`t-${tag}`} className={`inline-flex items-center gap-0.5 text-[11px] px-1.5 py-0.5 rounded ${tc}`}>
          <Tag className="w-2 h-2" />
          {tag}
        </span>
      );
    });
  }

  return (
    <div
      className={`group bg-white dark:bg-[#18181b] border border-zinc-100 dark:border-zinc-800/50 rounded-lg cursor-pointer transition-all duration-120 border-l-[3px] ${priorityBorder} ${
        isSelected
          ? "bg-indigo-50/60 dark:bg-indigo-950/30 border-zinc-200 dark:border-indigo-900/60 shadow-sm shadow-indigo-500/5"
          : "hover:bg-zinc-50 dark:hover:bg-zinc-800/60 hover:border-zinc-200 dark:hover:border-zinc-700 hover:shadow-sm"
      }`}
      onClick={() => setSelectedTask(task.id)}
    >
      <div className="flex items-start gap-2.5 px-3 py-2.5">
        {/* Custom checkbox — always in flow, never overlaps */}
        <label
          className="flex-shrink-0 pt-[2px] cursor-pointer"
          onClick={(e) => e.stopPropagation()}
        >
          <input
            type="checkbox"
            checked={isSelected}
            onChange={(e) => { e.stopPropagation(); toggleTaskSelection(task.id); }}
            className="sr-only"
          />
          <div className={`w-[15px] h-[15px] rounded-[4px] border flex items-center justify-center transition-all duration-120 ${
            isSelected
              ? "bg-indigo-500 border-indigo-500"
              : "border-zinc-300 dark:border-zinc-600 opacity-0 group-hover:opacity-100 group-hover:border-indigo-300 dark:group-hover:border-indigo-700"
          }`}>
            {isSelected && <Check className="w-2.5 h-2.5 text-white stroke-[2.5]" />}
          </div>
        </label>

        {/* Card content */}
        <div className="flex-1 min-w-0">
          {/* Title row */}
          <div className="flex items-start justify-between gap-2 min-w-0">
            <p className={`text-zinc-900 dark:text-zinc-100 font-semibold leading-snug min-w-0 break-words tracking-[-0.01em] ${compact ? "text-xs line-clamp-2" : "text-[13.5px]"}`}>
              {task.title}
            </p>
            {task.confidence_score !== undefined && task.confidence_score !== null && (
              <TaskConfidenceBadge confidence={task.confidence_score} />
            )}
          </div>

          {/* Description — 2 lines so the actual task context is visible */}
          {!compact && task.description && (
            <p className="text-[12px] text-zinc-500 dark:text-zinc-400 mt-1 line-clamp-2 leading-relaxed">
              {task.description}
            </p>
          )}

          {/* Metadata — dot-separated items */}
          {!compact && metaItems.length > 0 && (
            <div className="flex items-center gap-0 mt-1.5 flex-wrap">
              {metaItems.map((item, i) => (
                <span key={i} className="flex items-center">
                  {i > 0 && (
                    <span className="mx-1.5 text-zinc-300 dark:text-zinc-700 text-[10px] leading-none select-none">·</span>
                  )}
                  {item}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
