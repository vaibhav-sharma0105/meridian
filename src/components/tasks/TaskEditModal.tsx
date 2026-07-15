import { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { X, Save, Archive, ArchiveRestore, Trash2, Calendar, User, Tag, FolderInput } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import AssigneeChipInput, { parseAssignees } from "./AssigneeChipInput";
import { KANBAN_COLUMNS } from "@/lib/constants";
import { parseTags } from "@/lib/validators";
import type { Task } from "@/lib/tauri";
import { moveTaskToProject } from "@/lib/tauri";
import { useAutoSave } from "@/hooks/useAutoSave";
import { useTasks } from "@/hooks/useTasks";
import { useMeetings } from "@/hooks/useMeetings";
import { useProjects } from "@/hooks/useProjects";
import { TAG_COLORS } from "@/lib/constants";
import { PlanSection } from "@/components/plans/PlanSection";
import { DraftsTab } from "@/components/drafts/DraftsTab";

interface Props {
  task: Task;
}

const PRIORITY_ACCENT: Record<string, string> = {
  critical: "bg-red-500",
  high:     "bg-orange-400",
  medium:   "bg-yellow-400",
  low:      "bg-zinc-300 dark:bg-zinc-600",
};

const PRIORITY_LABEL: Record<string, string> = {
  critical: "text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950/40 border-red-200 dark:border-red-900",
  high:     "text-orange-600 dark:text-orange-400 bg-orange-50 dark:bg-orange-950/40 border-orange-200 dark:border-orange-900",
  medium:   "text-yellow-700 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-950/40 border-yellow-200 dark:border-yellow-900",
  low:      "text-zinc-500 dark:text-zinc-400 bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-800",
};

const fieldCls =
  "w-full px-3 py-2 text-[13px] rounded-xl border border-[#e2e2e8] dark:border-zinc-700 " +
  "bg-white dark:bg-zinc-800/80 text-zinc-900 dark:text-zinc-50 outline-none " +
  "focus-visible:ring-2 focus-visible:ring-indigo-400/30 focus:border-indigo-400 transition-all duration-150";

const labelCls =
  "block text-[11px] font-semibold text-zinc-400 dark:text-zinc-500 mb-1.5 uppercase tracking-[0.06em]";

export default function TaskEditModal({ task }: Props) {
  const { t } = useTranslation();
  const { tasks: allProjectTasks, updateTask, archiveTask, unarchiveTask, deleteTask } = useTasks(task.project_id, {});
  const { meetings } = useMeetings(task.project_id);
  const { setSelectedTask } = useUIStore();
  const backdropRef = useRef<HTMLDivElement>(null);

  const { projects } = useProjects();
  const otherProjects = projects.filter((p) => p.id !== task.project_id && !p.archived_at);

  const assigneeSuggestions = Array.from(
    new Set(allProjectTasks.flatMap((t) => parseAssignees(t.assignee ?? "")).filter(Boolean))
  );

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

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setSelectedTask(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

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

  const tags = parseTags(task.tags);

  const modal = (
    <div
      ref={backdropRef}
      className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-backdrop-in"
      style={{ backgroundColor: "rgba(0,0,0,0.45)", backdropFilter: "blur(4px)" }}
      onMouseDown={(e) => {
        if (e.target === backdropRef.current) setSelectedTask(null);
      }}
    >
      <div className="animate-modal-in w-full max-w-[560px] max-h-[90vh] flex flex-col rounded-2xl bg-white dark:bg-[#18181b] shadow-2xl overflow-hidden">

        {/* Priority accent strip */}
        <div className={`h-1 w-full flex-shrink-0 ${PRIORITY_ACCENT[priority] ?? "bg-zinc-300"} transition-colors duration-150`} />

        {/* Header */}
        <div className="flex items-start gap-3 px-5 pt-4 pb-3 flex-shrink-0">
          <div className="flex-1 min-w-0">
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full text-[17px] font-bold text-zinc-900 dark:text-zinc-50 bg-transparent border-b border-transparent hover:border-zinc-200 dark:hover:border-zinc-700 focus:border-indigo-400 outline-none pb-0.5 tracking-[-0.02em] transition-colors"
              placeholder="Task title"
            />
            <div className="flex items-center gap-2 mt-2 flex-wrap">
              {/* Status pill */}
              {(() => {
                const col = KANBAN_COLUMNS.find((c) => c.id === status);
                return (
                  <span className="inline-flex items-center gap-1 text-[11.5px] font-medium px-2 py-0.5 rounded-full bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300">
                    <span className={`w-1.5 h-1.5 rounded-full ${
                      status === "in_progress" ? "bg-indigo-500" :
                      status === "done" ? "bg-emerald-500" : "bg-zinc-400"
                    }`} />
                    {col?.label ?? status}
                  </span>
                );
              })()}
              {/* Priority pill */}
              <span className={`inline-flex items-center text-[11.5px] font-medium px-2 py-0.5 rounded-full border capitalize ${PRIORITY_LABEL[priority] ?? ""}`}>
                {priority}
              </span>
              {task.confidence_score !== null && task.confidence_score !== undefined && (
                <TaskConfidenceBadge confidence={task.confidence_score} />
              )}
              {saved && (
                <span className="flex items-center gap-1 text-[11.5px] text-emerald-600 dark:text-emerald-500">
                  <Save className="w-3 h-3" /> Saved
                </span>
              )}
            </div>
          </div>
          <button
            onClick={() => setSelectedTask(null)}
            className="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors flex-shrink-0 mt-0.5"
          >
            <X className="w-4.5 h-4.5" />
          </button>
        </div>

        {/* Body — scrollable */}
        <div className="flex-1 overflow-y-auto px-5 pb-5 space-y-4">

          {/* Status + Priority row */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className={labelCls}>{t("tasks.status")}</label>
              <select value={status} onChange={(e) => setStatus(e.target.value as Task["status"])} className={fieldCls}>
                {KANBAN_COLUMNS.map((col) => (
                  <option key={col.id} value={col.id}>{col.label}</option>
                ))}
                <option value="cancelled">Cancelled</option>
              </select>
            </div>
            <div>
              <label className={labelCls}>{t("tasks.priority")}</label>
              <select value={priority} onChange={(e) => setPriority(e.target.value as Task["priority"])} className={fieldCls}>
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </div>
          </div>

          {/* Assignee */}
          <div>
            <label className={labelCls}>
              <User className="inline w-3 h-3 mr-1 -mt-px" />{t("tasks.assignee")}
            </label>
            <AssigneeChipInput
              value={assignee}
              onChange={setAssignee}
              suggestions={assigneeSuggestions}
            />
          </div>

          {/* Due date + Meeting row */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className={labelCls}>
                <Calendar className="inline w-3 h-3 mr-1 -mt-px" />{t("tasks.dueDate")}
              </label>
              <input
                type="date"
                value={dueDate}
                onChange={(e) => setDueDate(e.target.value)}
                className={fieldCls}
              />
            </div>
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
          </div>

          {/* Tags */}
          <div>
            <label className={labelCls}>
              <Tag className="inline w-3 h-3 mr-1 -mt-px" />{t("tasks.tags")}
            </label>
            <input
              type="text"
              value={tagsStr}
              onChange={(e) => setTagsStr(e.target.value)}
              placeholder="tag1, tag2"
              className={fieldCls + " placeholder:text-zinc-400"}
            />
            {tags.length > 0 && (
              <div className="flex flex-wrap gap-1.5 mt-2">
                {tags.map((tag) => {
                  const tc = TAG_COLORS[tag] ?? "bg-zinc-100 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400";
                  return (
                    <span key={tag} className={`inline-flex items-center gap-1 text-[11.5px] px-2 py-0.5 rounded-full ${tc}`}>
                      {tag}
                    </span>
                  );
                })}
              </div>
            )}
          </div>

          {/* Description */}
          <div>
            <label className={labelCls}>{t("tasks.description")}</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              placeholder="Add details, context, or notes…"
              className={fieldCls + " resize-none leading-relaxed placeholder:text-zinc-400"}
            />
          </div>

          {/* AI Plan Section */}
          <PlanSection taskId={task.id} taskTitle={task.title} />

          {/* Drafts Section */}
          <DraftsTab taskId={task.id} taskTitle={task.title} />
        </div>

        {/* Footer actions */}
        <div className="flex items-center gap-3 px-5 py-3 border-t border-[#ebebf0] dark:border-zinc-800 bg-zinc-50/60 dark:bg-zinc-900/40 flex-shrink-0 flex-wrap">
          {/* Move to project */}
          {otherProjects.length > 0 && (
            <div className="flex items-center gap-1.5">
              <FolderInput className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
              <select
                defaultValue=""
                onChange={async (e) => {
                  const targetId = e.target.value;
                  if (!targetId) return;
                  const target = projects.find((p) => p.id === targetId);
                  if (!target) return;
                  if (!window.confirm(`Move "${task.title}" to "${target.name}"?`)) {
                    e.target.value = "";
                    return;
                  }
                  await moveTaskToProject(task.id, targetId);
                  setSelectedTask(null);
                }}
                className="text-[12px] text-zinc-500 dark:text-zinc-400 bg-transparent border border-zinc-200 dark:border-zinc-700 rounded-lg px-2 py-1 outline-none hover:border-indigo-300 dark:hover:border-indigo-700 focus:border-indigo-400 focus-visible:ring-1 focus-visible:ring-indigo-400/30 transition-all cursor-pointer"
              >
                <option value="">Move to project…</option>
                {otherProjects.map((p) => (
                  <option key={p.id} value={p.id}>{p.name}</option>
                ))}
              </select>
            </div>
          )}

          {/* Archive / Unarchive */}
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

          {/* Delete */}
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
    </div>
  );

  return createPortal(modal, document.body);
}
