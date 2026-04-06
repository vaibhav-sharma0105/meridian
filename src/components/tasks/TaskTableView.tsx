import { useTranslation } from "react-i18next";
import { useTasks } from "@/hooks/useTasks";
import { useMeetings } from "@/hooks/useMeetings";
import { useTaskStore } from "@/stores/taskStore";
import { useUIStore } from "@/stores/uiStore";
import { KANBAN_COLUMNS } from "@/lib/constants";
import TaskConfidenceBadge from "./TaskConfidenceBadge";
import { format } from "date-fns";
import EmptyState from "@/components/shared/EmptyState";
import { Table } from "lucide-react";

interface Props {
  projectId: string | null;
}

const PRIORITY_BADGE: Record<string, string> = {
  critical: "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
  high: "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400",
  medium: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
  low: "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400",
};

export default function TaskTableView({ projectId }: Props) {
  const { t } = useTranslation();
  const { tasks, updateTask } = useTasks(projectId);
  const { meetings } = useMeetings(projectId);
  const { filters } = useTaskStore();
  const meetingMap = new Map(meetings.map((m) => [m.id, m.title]));
  const { setSelectedTask } = useUIStore();

  const filtered = tasks.filter((task) => {
    if (filters.search_query && !task.title.toLowerCase().includes(filters.search_query.toLowerCase())) return false;
    if (filters.status && task.status !== filters.status) return false;
    if (filters.assignee && task.assignee?.toLowerCase() !== filters.assignee.toLowerCase()) return false;
    return true;
  });

  if (!projectId || filtered.length === 0) {
    return (
      <EmptyState
        title={t("tasks.noTasks")}
        description={t("tasks.noTasksDesc")}
        icon={<Table className="w-10 h-10 text-zinc-400" />}
      />
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm border-collapse">
        <thead>
          <tr className="border-b border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900">
            <th className="text-left px-4 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">{t("tasks.title")}</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">{t("tasks.status")}</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">{t("tasks.priority")}</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">{t("tasks.assignee")}</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">{t("tasks.dueDate")}</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">Meeting</th>
            <th className="text-left px-3 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wide">AI</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((task) => (
            <tr
              key={task.id}
              onClick={() => setSelectedTask(task.id)}
              className="border-b border-zinc-100 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 cursor-pointer transition-colors"
            >
              <td className="px-4 py-2.5 font-medium text-zinc-900 dark:text-zinc-50 max-w-[300px] truncate">{task.title}</td>
              <td className="px-3 py-2.5">
                <select
                  value={task.status}
                  onChange={async (e) => {
                    e.stopPropagation();
                    await updateTask({ id: task.id, status: e.target.value });
                  }}
                  onClick={(e) => e.stopPropagation()}
                  className="text-xs px-1.5 py-0.5 rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300"
                >
                  {KANBAN_COLUMNS.map((col) => (
                    <option key={col.id} value={col.id}>{col.label}</option>
                  ))}
                  <option value="cancelled">Cancelled</option>
                </select>
              </td>
              <td className="px-3 py-2.5">
                {task.priority ? (
                  <span className={`inline-block text-xs px-1.5 py-0.5 rounded capitalize ${PRIORITY_BADGE[task.priority] ?? ""}`}>
                    {task.priority}
                  </span>
                ) : "—"}
              </td>
              <td className="px-3 py-2.5 text-zinc-600 dark:text-zinc-400 text-xs">{task.assignee ?? "—"}</td>
              <td className="px-3 py-2.5 text-zinc-600 dark:text-zinc-400 text-xs">
                {task.due_date ? format(new Date(task.due_date), "MMM d, yyyy") : "—"}
              </td>
              <td className="px-3 py-2.5 text-zinc-600 dark:text-zinc-400 text-xs max-w-[150px] truncate">
                {task.meeting_id ? (meetingMap.get(task.meeting_id) ?? "—") : "—"}
              </td>
              <td className="px-3 py-2.5">
                {task.confidence_score !== null && task.confidence_score !== undefined ? (
                  <TaskConfidenceBadge confidence={task.confidence_score} />
                ) : "—"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
