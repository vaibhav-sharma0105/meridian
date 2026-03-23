import { useTranslation } from "react-i18next";
import type { Task } from "@/lib/tauri";

interface Props {
  tasks: Task[];
}

export default function WorkloadHeatmap({ tasks }: Props) {
  const { t } = useTranslation();

  // Count tasks per assignee
  const byAssignee: Record<string, { todo: number; in_progress: number; done: number }> = {};
  tasks.forEach((task) => {
    const name = task.assignee || "Unassigned";
    if (!byAssignee[name]) byAssignee[name] = { todo: 0, in_progress: 0, done: 0 };
    const key = task.status as "todo" | "in_progress" | "done";
    if (key in byAssignee[name]) byAssignee[name][key]++;
  });

  const rows = Object.entries(byAssignee).slice(0, 8);

  if (rows.length === 0) return null;

  const maxTotal = Math.max(...rows.map(([, v]) => v.todo + v.in_progress + v.done), 1);

  return (
    <div>
      <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-3">{t("analytics.workload")}</p>
      <div className="space-y-2">
        {rows.map(([name, counts]) => {
          const total = counts.todo + counts.in_progress + counts.done;
          const pct = Math.round((total / maxTotal) * 100);
          return (
            <div key={name} className="flex items-center gap-3">
              <span className="text-xs text-zinc-600 dark:text-zinc-400 w-24 truncate">{name}</span>
              <div className="flex-1 h-4 bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-indigo-400 rounded-full transition-all"
                  style={{ width: `${pct}%` }}
                />
              </div>
              <span className="text-xs text-zinc-500 w-8 text-right">{total}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
