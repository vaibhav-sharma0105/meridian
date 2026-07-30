import { useState, useMemo } from "react";
import {
  BarChart3,
  Shield,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  TrendingUp,
  Download,
  Calendar,
  Link2,
  Zap,
} from "lucide-react";
import { useGovernanceMetrics, usePendingApprovals } from "@/hooks/useGovernance";
import type { GovernanceMetrics } from "@/lib/tauri";

type TimeRange = "today" | "week" | "month";

function getDateRange(range: TimeRange): { start: string; end: string } {
  const now = new Date();
  const end = now.toISOString().split("T")[0];
  let start: string;

  switch (range) {
    case "today":
      start = end;
      break;
    case "week":
      const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
      start = weekAgo.toISOString().split("T")[0];
      break;
    case "month":
      const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
      start = monthAgo.toISOString().split("T")[0];
      break;
  }

  return { start, end };
}

interface MetricCardProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  subValue?: string;
  trend?: "up" | "down" | "neutral";
  color?: string;
}

function MetricCard({ icon, label, value, subValue, trend, color = "indigo" }: MetricCardProps) {
  return (
    <div className="bg-white dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 p-4">
      <div className="flex items-center gap-2 text-zinc-500 mb-2">
        {icon}
        <span className="text-xs font-medium">{label}</span>
      </div>
      <div className="flex items-end gap-2">
        <span className={`text-2xl font-semibold text-${color}-600 dark:text-${color}-400`}>
          {value}
        </span>
        {subValue && (
          <span className="text-xs text-zinc-400 mb-1">{subValue}</span>
        )}
        {trend && (
          <TrendingUp
            className={`w-4 h-4 mb-1 ${
              trend === "up"
                ? "text-green-500"
                : trend === "down"
                ? "text-red-500 rotate-180"
                : "text-zinc-400"
            }`}
          />
        )}
      </div>
    </div>
  );
}

interface RiskBarProps {
  data: { level: string; count: number }[];
}

function RiskBar({ data }: RiskBarProps) {
  const total = data.reduce((sum, d) => sum + d.count, 0);
  if (total === 0) {
    return (
      <div className="h-4 bg-zinc-100 dark:bg-zinc-700 rounded-full" />
    );
  }

  const colors: Record<string, string> = {
    low: "bg-green-500",
    medium: "bg-yellow-500",
    high: "bg-orange-500",
    critical: "bg-red-500",
  };

  return (
    <div className="flex h-4 rounded-full overflow-hidden">
      {data.map((d) => (
        <div
          key={d.level}
          className={`${colors[d.level] || "bg-zinc-400"}`}
          style={{ width: `${(d.count / total) * 100}%` }}
          title={`${d.level}: ${d.count}`}
        />
      ))}
    </div>
  );
}

interface GovernanceDashboardProps {
  className?: string;
}

export function GovernanceDashboard({ className }: GovernanceDashboardProps) {
  const [timeRange, setTimeRange] = useState<TimeRange>("week");
  const { start, end } = getDateRange(timeRange);

  const { data: metrics = [], isLoading } = useGovernanceMetrics(start, end);
  const { data: pendingApprovals = [] } = usePendingApprovals("pending");

  const aggregated = useMemo(() => {
    const result = {
      totalActions: 0,
      riskDistribution: [] as { level: string; count: number }[],
      autonomyBreakdown: [] as { mode: string; count: number }[],
      approvalRate: { approved: 0, rejected: 0, archived: 0 },
      integrationActivity: [] as { name: string; count: number }[],
      skillActivity: [] as { name: string; count: number }[],
    };

    const riskMap = new Map<string, number>();
    const autonomyMap = new Map<string, number>();
    const integrationMap = new Map<string, number>();
    const skillMap = new Map<string, number>();

    for (const m of metrics) {
      if (m.metric_type === "action_count") {
        result.totalActions += m.value;
      } else if (m.metric_type === "risk_distribution" && m.breakdown_key) {
        riskMap.set(m.breakdown_key, (riskMap.get(m.breakdown_key) || 0) + m.value);
      } else if (m.metric_type === "autonomy_breakdown" && m.breakdown_key) {
        autonomyMap.set(m.breakdown_key, (autonomyMap.get(m.breakdown_key) || 0) + m.value);
      } else if (m.metric_type === "approval_rate" && m.breakdown_key) {
        const key = m.breakdown_key as keyof typeof result.approvalRate;
        if (key in result.approvalRate) {
          result.approvalRate[key] += m.value;
        }
      } else if (m.metric_type === "integration_activity" && m.breakdown_key) {
        integrationMap.set(m.breakdown_key, (integrationMap.get(m.breakdown_key) || 0) + m.value);
      } else if (m.metric_type === "skill_activity" && m.breakdown_key) {
        skillMap.set(m.breakdown_key, (skillMap.get(m.breakdown_key) || 0) + m.value);
      }
    }

    for (const level of ["low", "medium", "high", "critical"]) {
      const count = riskMap.get(level) || 0;
      if (count > 0) {
        result.riskDistribution.push({ level, count });
      }
    }

    for (const [mode, count] of autonomyMap) {
      result.autonomyBreakdown.push({ mode, count });
    }

    for (const [name, count] of integrationMap) {
      result.integrationActivity.push({ name, count });
    }
    result.integrationActivity.sort((a, b) => b.count - a.count);

    for (const [name, count] of skillMap) {
      result.skillActivity.push({ name, count });
    }
    result.skillActivity.sort((a, b) => b.count - a.count);

    return result;
  }, [metrics]);

  const approvalTotal =
    aggregated.approvalRate.approved +
    aggregated.approvalRate.rejected +
    aggregated.approvalRate.archived;
  const approvalPercent =
    approvalTotal > 0
      ? Math.round((aggregated.approvalRate.approved / approvalTotal) * 100)
      : 0;

  const handleExport = () => {
    const csv = [
      ["Date", "Metric Type", "Breakdown Key", "Value"],
      ...metrics.map((m) => [
        m.date,
        m.metric_type,
        m.breakdown_key || "",
        m.value.toString(),
      ]),
    ]
      .map((row) => row.join(","))
      .join("\n");

    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `governance-metrics-${start}-${end}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className={className}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <BarChart3 className="w-5 h-5 text-zinc-500" />
          <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
            Governance Dashboard
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex bg-zinc-100 dark:bg-zinc-800 rounded-lg p-0.5">
            {(["today", "week", "month"] as const).map((r) => (
              <button
                key={r}
                onClick={() => setTimeRange(r)}
                className={`px-3 py-1 text-xs font-medium rounded-md transition-colors ${
                  timeRange === r
                    ? "bg-white dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100 shadow-sm"
                    : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                }`}
              >
                {r.charAt(0).toUpperCase() + r.slice(1)}
              </button>
            ))}
          </div>
          <button
            onClick={handleExport}
            className="p-1.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 transition-colors"
            title="Export CSV"
          >
            <Download className="w-4 h-4" />
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="text-sm text-zinc-500 py-8 text-center">
          Loading metrics...
        </div>
      ) : (
        <>
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 mb-6">
            <MetricCard
              icon={<Shield className="w-4 h-4" />}
              label="Total Actions"
              value={aggregated.totalActions}
              subValue={`${timeRange === "today" ? "today" : `past ${timeRange === "week" ? "7" : "30"} days`}`}
            />
            <MetricCard
              icon={<Clock className="w-4 h-4" />}
              label="Pending Approvals"
              value={pendingApprovals.length}
              color="yellow"
            />
            <MetricCard
              icon={<CheckCircle className="w-4 h-4" />}
              label="Approval Rate"
              value={`${approvalPercent}%`}
              subValue={`${aggregated.approvalRate.approved}/${approvalTotal}`}
              color="green"
            />
            <MetricCard
              icon={<XCircle className="w-4 h-4" />}
              label="Rejected"
              value={aggregated.approvalRate.rejected}
              color="red"
            />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <div className="bg-white dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 p-4">
              <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-3">
                Risk Distribution
              </h3>
              <RiskBar data={aggregated.riskDistribution} />
              <div className="flex justify-between mt-2 text-xs text-zinc-500">
                {aggregated.riskDistribution.map((d) => (
                  <span key={d.level}>
                    {d.level}: {d.count}
                  </span>
                ))}
              </div>
            </div>

            <div className="bg-white dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 p-4">
              <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-3">
                Autonomy Breakdown
              </h3>
              {aggregated.autonomyBreakdown.length === 0 ? (
                <div className="text-sm text-zinc-500 py-4 text-center">
                  No data for this period
                </div>
              ) : (
                <div className="space-y-2">
                  {aggregated.autonomyBreakdown.map((a) => (
                    <div key={a.mode} className="flex items-center gap-2">
                      <div className="flex-1">
                        <div className="flex justify-between text-xs mb-1">
                          <span className="text-zinc-600 dark:text-zinc-400">
                            {a.mode}
                          </span>
                          <span className="text-zinc-500">{a.count}</span>
                        </div>
                        <div className="h-2 bg-zinc-100 dark:bg-zinc-700 rounded-full overflow-hidden">
                          <div
                            className={`h-full ${
                              a.mode === "manual"
                                ? "bg-yellow-500"
                                : a.mode === "supervised"
                                ? "bg-blue-500"
                                : "bg-green-500"
                            }`}
                            style={{
                              width: `${
                                (a.count / aggregated.totalActions) * 100
                              }%`,
                            }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {(aggregated.integrationActivity.length > 0 || aggregated.skillActivity.length > 0) && (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
              <div className="bg-white dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 p-4">
                <div className="flex items-center gap-2 mb-3">
                  <Link2 className="w-4 h-4 text-zinc-500" />
                  <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                    Integration Activity
                  </h3>
                </div>
                {aggregated.integrationActivity.length === 0 ? (
                  <div className="text-sm text-zinc-500 py-4 text-center">
                    No integration activity
                  </div>
                ) : (
                  <div className="space-y-2">
                    {aggregated.integrationActivity.slice(0, 5).map((i) => (
                      <div key={i.name} className="flex items-center justify-between">
                        <span className="text-xs text-zinc-600 dark:text-zinc-400 truncate max-w-[120px]">
                          {i.name}
                        </span>
                        <span className="text-xs font-medium text-zinc-500 bg-zinc-100 dark:bg-zinc-700 px-2 py-0.5 rounded-full">
                          {i.count}
                        </span>
                      </div>
                    ))}
                    {aggregated.integrationActivity.length > 5 && (
                      <div className="text-xs text-zinc-400 text-center pt-1">
                        +{aggregated.integrationActivity.length - 5} more
                      </div>
                    )}
                  </div>
                )}
              </div>

              <div className="bg-white dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700 p-4">
                <div className="flex items-center gap-2 mb-3">
                  <Zap className="w-4 h-4 text-zinc-500" />
                  <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                    Skill Activity
                  </h3>
                </div>
                {aggregated.skillActivity.length === 0 ? (
                  <div className="text-sm text-zinc-500 py-4 text-center">
                    No skill activity
                  </div>
                ) : (
                  <div className="space-y-2">
                    {aggregated.skillActivity.slice(0, 5).map((s) => (
                      <div key={s.name} className="flex items-center justify-between">
                        <span className="text-xs text-zinc-600 dark:text-zinc-400 truncate max-w-[120px]">
                          {s.name}
                        </span>
                        <span className="text-xs font-medium text-zinc-500 bg-zinc-100 dark:bg-zinc-700 px-2 py-0.5 rounded-full">
                          {s.count}
                        </span>
                      </div>
                    ))}
                    {aggregated.skillActivity.length > 5 && (
                      <div className="text-xs text-zinc-400 text-center pt-1">
                        +{aggregated.skillActivity.length - 5} more
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}

          {pendingApprovals.some((a) => a.risk_level === "critical") && (
            <div className="mt-4 flex items-start gap-2 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-100 dark:border-red-900/50">
              <AlertTriangle className="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
              <div>
                <div className="text-sm font-medium text-red-600 dark:text-red-400">
                  Critical approvals pending
                </div>
                <p className="text-xs text-red-500 dark:text-red-400/80 mt-0.5">
                  {pendingApprovals.filter((a) => a.risk_level === "critical").length}{" "}
                  critical-risk actions are waiting for your review.
                </p>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
