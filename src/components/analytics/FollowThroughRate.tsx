import { useTranslation } from "react-i18next";
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import type { Task } from "@/lib/tauri";

interface Props {
  tasks: Task[];
}

export default function FollowThroughRate({ tasks }: Props) {
  const { t } = useTranslation();

  const done = tasks.filter((t) => t.status === "done").length;
  const total = tasks.length;
  const pct = total > 0 ? Math.round((done / total) * 100) : 0;

  const data = [
    { name: t("analytics.completed"), value: done },
    { name: t("analytics.remaining"), value: total - done },
  ];

  const COLORS = ["#6366f1", "#e4e4e7"];

  return (
    <div>
      <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-1">{t("analytics.followThrough")}</p>
      <p className="text-3xl font-bold text-zinc-900 dark:text-zinc-50 mb-3">{pct}%</p>
      {total > 0 && (
        <ResponsiveContainer width="100%" height={100}>
          <PieChart>
            <Pie data={data} dataKey="value" cx="50%" cy="50%" innerRadius={30} outerRadius={45}>
              {data.map((_, i) => (
                <Cell key={i} fill={COLORS[i % COLORS.length]} />
              ))}
            </Pie>
            <Tooltip contentStyle={{ fontSize: 11, borderRadius: 8 }} />
          </PieChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}
