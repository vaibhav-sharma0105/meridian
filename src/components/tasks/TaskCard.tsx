import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Calendar, User, Tag, Video, Check, Archive, Link2, MoreHorizontal } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTaskStore } from "@/stores/taskStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import { IntegrationLinkBadge } from "./IntegrationLinkBadge";
import { TAG_COLORS } from "@/lib/constants";
import { parseTags } from "@/lib/validators";
import { parseAssignees } from "./AssigneeChipInput";
import type { Task } from "@/lib/tauri";
import { useMeetings } from "@/hooks/useMeetings";
import { format } from "date-fns";

interface Props {
  task: Task;
  compact?: boolean;
  isSubtask?: boolean;
}

const PRIORITY_COLORS: Record<string, string> = {
  critical: "border-l-red-500",
  high: "border-l-orange-400",
  medium: "border-l-yellow-400",
  low: "border-l-zinc-300 dark:border-l-zinc-600",
};

export default function TaskCard({ task, compact = false, isSubtask = false }: Props) {
  const { setSelectedTask, setLinkPickerTaskId } = useUIStore();
  const { selectedTaskIds, toggleTaskSelection } = useTaskStore();
  const { meetings } = useMeetings(task.project_id);
  const [showMenu, setShowMenu] = useState(false);
  const isSelected = selectedTaskIds.includes(task.id);
  const linkedMeeting = task.meeting_id ? meetings.find((m) => m.id === task.meeting_id) : null;
  const tags = parseTags(task.tags);
  const priorityBorder = task.priority ? (PRIORITY_COLORS[task.priority] ?? "border-l-zinc-300") : "border-l-zinc-300";

  const hasMetadata = task.assignee || task.due_date || linkedMeeting || tags.length > 0;

  const handleLinkTo = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowMenu(false);
    setLinkPickerTaskId(task.id);
  };

  const metaItems: React.ReactNode[] = [];
  if (!compact) {
    if (task.assignee) {
      parseAssignees(task.assignee).forEach((name) => {
        metaItems.push(
          <span key={`a-${name}`} className="flex items-center gap-1 text-[12px] text-zinc-400 dark:text-zinc-500">
            <User className="w-3 h-3" />
            {name}
          </span>
        );
      });
    }
    if (task.due_date) {
      metaItems.push(
        <span key="due" className="flex items-center gap-1 text-[12px] text-zinc-400 dark:text-zinc-500">
          <Calendar className="w-3 h-3" />
          {(() => { try { return format(new Date(task.due_date), "MMM d"); } catch { return task.due_date; } })()}
        </span>
      );
    }
    if (linkedMeeting) {
      metaItems.push(
        <span key="mtg" className="flex items-center gap-1 text-[12px] text-indigo-400/80 dark:text-indigo-400/60 max-w-[110px] truncate" title={linkedMeeting.title}>
          <Video className="w-3 h-3 flex-shrink-0" />
          {linkedMeeting.title}
        </span>
      );
    }
    tags.slice(0, 3).forEach((tag) => {
      const tc = TAG_COLORS[tag] ?? "bg-zinc-100 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400";
      metaItems.push(
        <span key={`t-${tag}`} className={`inline-flex items-center gap-1 text-[12px] px-2 py-0.5 rounded-full ${tc}`}>
          <Tag className="w-2.5 h-2.5" />
          {tag}
        </span>
      );
    });
  }

  const isArchived = !!task.archived_at;

  return (
    <div
      className={`group bg-white dark:bg-[#18181b] border border-[#ebebf0] dark:border-zinc-800/60 cursor-pointer transition-all duration-150 ${
        isSubtask
          ? "rounded-lg border-l-2"
          : "rounded-xl border-l-[3px]"
      } ${priorityBorder} ${
        isArchived
          ? "opacity-55"
          : isSelected
          ? "bg-indigo-50/70 dark:bg-indigo-950/30 border-indigo-200/80 dark:border-indigo-900/60 shadow-sm shadow-indigo-500/8"
          : "hover:bg-zinc-50 dark:hover:bg-zinc-800/60 hover:border-zinc-200 dark:hover:border-zinc-700 hover:-translate-y-px hover:shadow-[0_4px_12px_rgba(0,0,0,0.07)] dark:hover:shadow-[0_4px_12px_rgba(0,0,0,0.25)]"
      }`}
      onClick={() => setSelectedTask(task.id)}
    >
      <div className={`flex items-start gap-3 ${isSubtask ? "px-3 py-2" : "px-4 py-3"}`}>
        {/* Custom checkbox */}
        <label
          className="flex-shrink-0 pt-[3px] cursor-pointer"
          onClick={(e) => e.stopPropagation()}
        >
          <input
            type="checkbox"
            checked={isSelected}
            onChange={(e) => { e.stopPropagation(); toggleTaskSelection(task.id); }}
            className="sr-only"
          />
          <div className={`w-[17px] h-[17px] rounded-[5px] border-[1.5px] flex items-center justify-center transition-all duration-150 ${
            isSelected
              ? "bg-indigo-500 border-indigo-500 shadow-sm"
              : "border-zinc-300 dark:border-zinc-600 opacity-0 group-hover:opacity-100 group-hover:border-indigo-300 dark:group-hover:border-indigo-700"
          }`}>
            {isSelected && <Check className="w-2.5 h-2.5 text-white stroke-[3]" />}
          </div>
        </label>

        {/* Card content */}
        <div className="flex-1 min-w-0">
          {/* Title row */}
          <div className="flex items-start justify-between gap-2 min-w-0">
            <div className="flex items-center gap-2 min-w-0 flex-1">
              <p className={`text-zinc-900 dark:text-zinc-100 leading-snug min-w-0 break-words tracking-[-0.012em] ${
                isSubtask ? "text-[13px] font-medium" : compact ? "text-[13px] font-semibold line-clamp-2" : "text-[14.5px] font-semibold"
              }`}>
                {task.title}
              </p>
              <IntegrationLinkBadge taskId={task.id} />
            </div>
            <div className="flex items-center gap-1 flex-shrink-0">
              {isArchived && (
                <span className="flex items-center gap-0.5 text-[11px] text-zinc-400 dark:text-zinc-500">
                  <Archive className="w-3 h-3" />
                  Archived
                </span>
              )}
              {task.confidence_score !== undefined && task.confidence_score !== null && (
                <TaskConfidenceBadge confidence={task.confidence_score} />
              )}
              {/* Context menu button */}
              <div className="relative">
                <button
                  onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
                  className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 opacity-0 group-hover:opacity-100 transition-opacity rounded hover:bg-zinc-100 dark:hover:bg-zinc-800"
                >
                  <MoreHorizontal className="w-4 h-4" />
                </button>
                {showMenu && (
                  <div
                    className="absolute right-0 top-full mt-1 w-40 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl z-20 py-1"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <button
                      onClick={handleLinkTo}
                      className="w-full flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800"
                    >
                      <Link2 className="w-4 h-4" />
                      Link to...
                    </button>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Description */}
          {!compact && task.description && (
            <p className="text-[13px] text-zinc-500 dark:text-zinc-400 mt-1.5 line-clamp-2 leading-relaxed">
              {task.description}
            </p>
          )}

          {/* Metadata — dot-separated */}
          {!compact && metaItems.length > 0 && (
            <div className="flex items-center gap-0 mt-2 flex-wrap">
              {metaItems.map((item, i) => (
                <span key={i} className="flex items-center">
                  {i > 0 && (
                    <span className="mx-2 text-zinc-300 dark:text-zinc-700 text-[11px] leading-none select-none">·</span>
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
