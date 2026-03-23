import { useTranslation } from "react-i18next";
import { useTasks } from "@/hooks/useTasks";
import { useMeetings } from "@/hooks/useMeetings";
import HealthScoreCard from "./HealthScoreCard";
import VelocityChart from "./VelocityChart";
import WorkloadHeatmap from "./WorkloadHeatmap";
import FollowThroughRate from "./FollowThroughRate";
import EmptyState from "@/components/shared/EmptyState";
import { BarChart2 } from "lucide-react";

interface Props {
  projectId: string | null;
}

export default function ProjectDashboard({ projectId }: Props) {
  const { t } = useTranslation();
  const { tasks } = useTasks(projectId);
  const { meetings } = useMeetings(projectId);

  if (!projectId) {
    return (
      <EmptyState
        title={t("analytics.noProject")}
        icon={<BarChart2 className="w-10 h-10 text-zinc-400" />}
      />
    );
  }

  const avgHealth =
    meetings.length > 0
      ? Math.round(
          meetings.reduce((sum, m) => sum + (m.health_score ?? 0), 0) / meetings.length
        )
      : 0;

  const todoCount = tasks.filter((t) => t.status === "open").length;
  const inProgressCount = tasks.filter((t) => t.status === "in_progress").length;
  const doneCount = tasks.filter((t) => t.status === "done").length;

  return (
    <div className="p-4 space-y-6">
      {/* Summary row */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {[
          { label: t("analytics.totalTasks"), value: tasks.length },
          { label: t("analytics.inProgress"), value: inProgressCount },
          { label: t("analytics.done"), value: doneCount },
          { label: t("analytics.meetings"), value: meetings.length },
        ].map(({ label, value }) => (
          <div key={label} className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4">
            <p className="text-xs text-zinc-500 mb-1">{label}</p>
            <p className="text-2xl font-bold text-zinc-900 dark:text-zinc-50">{value}</p>
          </div>
        ))}
      </div>

      {/* Charts row */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4">
          <HealthScoreCard score={avgHealth} label={t("analytics.avgMeetingHealth")} />
        </div>
        <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4">
          <FollowThroughRate tasks={tasks} />
        </div>
        <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4">
          <WorkloadHeatmap tasks={tasks} />
        </div>
      </div>

      {/* Velocity chart */}
      <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4">
        <VelocityChart tasks={tasks} />
      </div>
    </div>
  );
}
