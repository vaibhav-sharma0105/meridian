import { useTranslation } from "react-i18next";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from "recharts";
import type { Task } from "@/lib/tauri";
import { format, startOfWeek, addWeeks, isWithinInterval } from "date-fns";

interface Props {
  tasks: Task[];
}

export default function VelocityChart({ tasks }: Props) {
  const { t } = useTranslation();

  // Build last 6 weeks of completed tasks
  const now = new Date();
  const data = Array.from({ length: 6 }, (_, i) => {
    const weekStart = startOfWeek(addWeeks(now, i - 5));
    const weekEnd = addWeeks(weekStart, 1);
    const completed = tasks.filter(
      (task) =>
        task.status === "done" &&
        task.updated_at &&
        isWithinInterval(new Date(task.updated_at), { start: weekStart, end: weekEnd })
    ).length;
    return { week: format(weekStart, "MMM d"), completed };
  });

  return (
    <div>
      <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-3">{t("analytics.velocity")}</p>
      <ResponsiveContainer width="100%" height={150}>
        <BarChart data={data} margin={{ top: 0, right: 0, bottom: 0, left: -20 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(0,0,0,0.06)" />
          <XAxis dataKey="week" tick={{ fontSize: 10 }} />
          <YAxis tick={{ fontSize: 10 }} allowDecimals={false} />
          <Tooltip
            contentStyle={{ fontSize: 12, borderRadius: 8 }}
            formatter={(v) => [v, t("analytics.tasksCompleted")]}
          />
          <Bar dataKey="completed" fill="#6366f1" radius={[4, 4, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
