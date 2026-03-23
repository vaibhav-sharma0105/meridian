import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/uiStore";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import { KANBAN_COLUMNS } from "@/lib/constants";
import { X, Save } from "lucide-react";
import { parseTags } from "@/lib/validators";
import type { Task } from "@/lib/tauri";
import { useAutoSave } from "@/hooks/useAutoSave";
import { useTasks } from "@/hooks/useTasks";

interface Props {
  task: Task;
}

export default function TaskInlineEditor({ task }: Props) {
  const { t } = useTranslation();
  const { updateTask } = useTasks(task.project_id);
  const { setSelectedTask } = useUIStore();
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description ?? "");
  const [status, setStatus] = useState(task.status);
  const [priority, setPriority] = useState(task.priority ?? "medium");
  const [assignee, setAssignee] = useState(task.assignee ?? "");
  const [dueDate, setDueDate] = useState(task.due_date ?? "");
  const [tagsStr, setTagsStr] = useState(parseTags(task.tags).join(", "));

  useEffect(() => {
    setTitle(task.title);
    setDescription(task.description ?? "");
    setStatus(task.status);
    setPriority(task.priority ?? "medium");
    setAssignee(task.assignee ?? "");
    setDueDate(task.due_date ?? "");
    setTagsStr(parseTags(task.tags).join(", "));
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
    });
  };

  const { saved } = useAutoSave([title, description, status, priority, assignee, dueDate, tagsStr], save);

  return (
    <div className="space-y-3">
      <div className="flex items-start justify-between gap-2">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          className="flex-1 text-sm font-semibold text-zinc-900 dark:text-zinc-50 bg-transparent border-b border-transparent hover:border-zinc-200 focus:border-indigo-400 outline-none pb-0.5"
        />
        <button
          onClick={() => setSelectedTask(null)}
          className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded text-zinc-400"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {task.confidence_score !== null && task.confidence_score !== undefined && (
        <div className="flex items-center gap-2">
          <span className="text-xs text-zinc-500">AI confidence:</span>
          <TaskConfidenceBadge confidence={task.confidence_score} />
        </div>
      )}

      <div className="grid grid-cols-2 gap-2">
        <div>
          <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.status")}</label>
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value as Task["status"])}
            className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          >
            {KANBAN_COLUMNS.map((col) => (
              <option key={col.id} value={col.id}>{col.label}</option>
            ))}
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
        <div>
          <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.priority")}</label>
          <select
            value={priority}
            onChange={(e) => setPriority(e.target.value as Task["priority"])}
            className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
          </select>
        </div>
      </div>

      <div>
        <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.assignee")}</label>
        <input
          type="text"
          value={assignee}
          onChange={(e) => setAssignee(e.target.value)}
          className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
        />
      </div>

      <div>
        <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.dueDate")}</label>
        <input
          type="date"
          value={dueDate}
          onChange={(e) => setDueDate(e.target.value)}
          className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
        />
      </div>

      <div>
        <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.tags")}</label>
        <input
          type="text"
          value={tagsStr}
          onChange={(e) => setTagsStr(e.target.value)}
          placeholder="tag1, tag2"
          className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
        />
      </div>

      <div>
        <label className="block text-xs text-zinc-500 mb-0.5">{t("tasks.description")}</label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={4}
          className="w-full px-2 py-1 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 resize-none"
        />
      </div>

      {saved && (
        <div className="flex items-center gap-1 text-xs text-green-600">
          <Save className="w-3 h-3" />
          Saved
        </div>
      )}
    </div>
  );
}
