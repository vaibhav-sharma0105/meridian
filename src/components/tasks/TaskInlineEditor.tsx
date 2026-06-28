import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import AssigneeChipInput, { parseAssignees } from "./AssigneeChipInput";
import { KANBAN_COLUMNS } from "@/lib/constants";
import { X, Save, Archive, ArchiveRestore, Trash2 } from "lucide-react";
import { parseTags } from "@/lib/validators";
import type { Task } from "@/lib/tauri";
import { useAutoSave } from "@/hooks/useAutoSave";
import { useTasks } from "@/hooks/useTasks";
import { useMeetings } from "@/hooks/useMeetings";

interface Props {
  task: Task;
}

const fieldCls =
  "w-full px-2.5 py-1.5 text-[13px] rounded-lg border border-[#e2e2e8] dark:border-zinc-700 " +
  "bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 outline-none " +
  "focus:ring-2 focus:ring-indigo-400/40 focus:border-indigo-400 transition-all duration-150";

const labelCls = "block text-[11.5px] font-medium text-zinc-400 dark:text-zinc-500 mb-1 uppercase tracking-[0.05em]";

export default function TaskInlineEditor({ task }: Props) {
  const { t } = useTranslation();
  const { tasks: allProjectTasks, updateTask, archiveTask, unarchiveTask, deleteTask } = useTasks(task.project_id);
  const { meetings } = useMeetings(task.project_id);

  const assigneeSuggestions = Array.from(
    new Set(
      allProjectTasks
        .flatMap((t) => parseAssignees(t.assignee ?? ""))
        .filter(Boolean)
    )
  );
  const { setSelectedTask } = useUIStore();
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description ?? "");
  const [status, setStatus] = useState(task.status);
  const [priority, setPriority] = useState(task.priority ?? "medium");
  const [assignee, setAssignee] = useState(task.assignee ?? "");
  const [dueDate, setDueDate] = useState(task.due_date ?? "");
  const [tagsStr, setTagsStr] = useState(parseTags(task.tags).join(", "));
  const [meetingId, setMeetingId] = useState<string | null>(task.meeting_id);

  useEffect(() => {
    setTitle(task.title);
    setDescription(task.description ?? "");
    setStatus(task.status);
    setPriority(task.priority ?? "medium");
    setAssignee(task.assignee ?? "");
    setDueDate(task.due_date ?? "");
    setTagsStr(parseTags(task.tags).join(", "));
    setMeetingId(task.meeting_id);
  }, [task.id]);

  const save = async () => {
    const tagArray = tagsStr.split(",").map((t) => t.trim()).filter(Boolean);
    await updateTask({
      id: task.id,
      title,
      description,
      status,
      priority,
      assignee: assignee || undefined,
      due_date: dueDate || undefined,
      tags: tagArray,
      meeting_id: meetingId,
    });
  };

  const { saved } = useAutoSave([title, description, status, priority, assignee, dueDate, tagsStr, meetingId], save);

  return (
    <div className="space-y-3.5">
      {/* Title + close */}
      <div className="flex items-start justify-between gap-2">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="flex-1 text-[15px] font-semibold text-zinc-900 dark:text-zinc-50 bg-transparent border-b border-transparent hover:border-zinc-200 dark:hover:border-zinc-700 focus:border-indigo-400 outline-none pb-0.5 transition-colors"
        />
        <button
          onClick={() => setSelectedTask(null)}
          className="p-1.5 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg text-zinc-400 hover:text-zinc-600 transition-colors flex-shrink-0"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {task.confidence_score !== null && task.confidence_score !== undefined && (
        <div className="flex items-center gap-2">
          <span className="text-[12px] text-zinc-500">AI confidence:</span>
          <TaskConfidenceBadge confidence={task.confidence_score} />
        </div>
      )}

      {/* Status + Priority */}
      <div className="grid grid-cols-2 gap-2.5">
        <div>
          <label className={labelCls}>{t("tasks.status")}</label>
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value as Task["status"])}
            className={fieldCls}
          >
            {KANBAN_COLUMNS.map((col) => (
              <option key={col.id} value={col.id}>{col.label}</option>
            ))}
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
        <div>
          <label className={labelCls}>{t("tasks.priority")}</label>
          <select
            value={priority}
            onChange={(e) => setPriority(e.target.value as Task["priority"])}
            className={fieldCls}
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
          </select>
        </div>
      </div>

      {/* Assignee */}
      <div>
        <label className={labelCls}>{t("tasks.assignee")}</label>
        <AssigneeChipInput
          value={assignee}
          onChange={setAssignee}
          suggestions={assigneeSuggestions}
        />
      </div>

      {/* Due date */}
      <div>
        <label className={labelCls}>{t("tasks.dueDate")}</label>
        <input
          type="date"
          value={dueDate}
          onChange={(e) => setDueDate(e.target.value)}
          className={fieldCls}
        />
      </div>

      {/* Meeting */}
      <div>
        <label className={labelCls}>Meeting</label>
        <select
          value={meetingId ?? ""}
          onChange={(e) => setMeetingId(e.target.value || null)}
          className={fieldCls}
        >
          <option value="">None</option>
          {meetings.map((m) => (
            <option key={m.id} value={m.id}>{m.title}</option>
          ))}
        </select>
      </div>

      {/* Tags */}
      <div>
        <label className={labelCls}>{t("tasks.tags")}</label>
        <input
          type="text"
          value={tagsStr}
          onChange={(e) => setTagsStr(e.target.value)}
          placeholder="tag1, tag2"
          className={fieldCls + " placeholder:text-zinc-400"}
        />
      </div>

      {/* Description */}
      <div>
        <label className={labelCls}>{t("tasks.description")}</label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={4}
          className={fieldCls + " resize-none leading-relaxed"}
        />
      </div>

      {/* Saved indicator */}
      {saved && (
        <div className="flex items-center gap-1.5 text-[12px] text-emerald-600 dark:text-emerald-500">
          <Save className="w-3.5 h-3.5" />
          Saved
        </div>
      )}

      {/* Archive / Delete */}
      <div className="flex items-center gap-2 pt-2 border-t border-[#ebebf0] dark:border-zinc-800">
        {task.archived_at ? (
          <button
            onClick={async () => { await unarchiveTask(task.id); setSelectedTask(null); }}
            className="flex items-center gap-1.5 text-[12.5px] text-zinc-500 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors"
          >
            <ArchiveRestore className="w-3.5 h-3.5" />
            Unarchive
          </button>
        ) : (
          <button
            onClick={async () => { await archiveTask(task.id); setSelectedTask(null); }}
            className="flex items-center gap-1.5 text-[12.5px] text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 transition-colors"
          >
            <Archive className="w-3.5 h-3.5" />
            Archive
          </button>
        )}
        <button
          onClick={async () => {
            if (window.confirm(`Delete "${task.title}"? This cannot be undone.`)) {
              await deleteTask(task.id);
              setSelectedTask(null);
            }
          }}
          className="flex items-center gap-1.5 text-[12.5px] text-zinc-400 hover:text-red-500 dark:hover:text-red-400 transition-colors ml-auto"
        >
          <Trash2 className="w-3.5 h-3.5" />
          Delete
        </button>
      </div>
    </div>
  );
}
