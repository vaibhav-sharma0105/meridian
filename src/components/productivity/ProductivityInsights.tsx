import { useState } from "react";
import {
  Clock,
  TrendingUp,
  Trash2,
  Download,
  ToggleLeft,
  ToggleRight,
  AlertTriangle,
} from "lucide-react";
import {
  useProductivityInsights,
  useUpdateProductivitySettings,
  useClearProductivityData,
  useExportProductivityData,
  formatHour,
  formatHourRange,
  TASK_CATEGORIES,
} from "@/hooks/useProductivity";

export function ProductivityInsights() {
  const { data: insights, isLoading, error } = useProductivityInsights();
  const updateSettings = useUpdateProductivitySettings();
  const clearData = useClearProductivityData();
  const exportData = useExportProductivityData();

  const [showClearConfirm, setShowClearConfirm] = useState(false);

  if (isLoading) {
    return (
      <div className="p-6 animate-pulse">
        <div className="h-6 w-40 bg-zinc-200 dark:bg-zinc-700 rounded mb-4" />
        <div className="space-y-3">
          <div className="h-4 w-full bg-zinc-200 dark:bg-zinc-700 rounded" />
          <div className="h-4 w-3/4 bg-zinc-200 dark:bg-zinc-700 rounded" />
        </div>
      </div>
    );
  }

  if (error || !insights) {
    return (
      <div className="p-6 text-center">
        <p className="text-sm text-red-500">Failed to load productivity data</p>
      </div>
    );
  }

  const { patterns, status, storage_warning } = insights;
  const isDisabled = status.type === "Disabled";
  const isLearning = status.type === "Learning";
  const isReady = status.type === "Ready";

  const handleToggleTracking = () => {
    updateSettings.mutate({
      tracking_enabled: !patterns.tracking_enabled,
      show_suggestions: true,
      data_retention_days: 365,
    });
  };

  const handleClearData = () => {
    clearData.mutate(undefined, {
      onSuccess: () => setShowClearConfirm(false),
    });
  };

  const handleExport = () => {
    exportData.mutate(undefined, {
      onSuccess: (data) => {
        const blob = new Blob([JSON.stringify(data, null, 2)], {
          type: "application/json",
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "productivity-export.json";
        a.click();
        URL.revokeObjectURL(url);
      },
    });
  };

  return (
    <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700">
      <div className="px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
              <TrendingUp className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Productivity Patterns
              </h3>
              <p className="text-sm text-zinc-500">
                {isDisabled
                  ? "Tracking is disabled"
                  : isLearning
                  ? `Learning (${(status as { completions_needed: number }).completions_needed} more tasks needed)`
                  : `${patterns.total_completions} tasks analyzed`}
              </p>
            </div>
          </div>

          <button
            onClick={handleToggleTracking}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
          >
            {patterns.tracking_enabled ? (
              <ToggleRight className="w-5 h-5 text-indigo-500" />
            ) : (
              <ToggleLeft className="w-5 h-5 text-zinc-400" />
            )}
            <span className="text-sm text-zinc-600 dark:text-zinc-400">
              {patterns.tracking_enabled ? "On" : "Off"}
            </span>
          </button>
        </div>
      </div>

      {!isDisabled && (
        <div className="p-6">
          {storage_warning && (
            <div className="mb-4 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg flex items-start gap-2">
              <AlertTriangle className="w-4 h-4 text-amber-500 mt-0.5" />
              <p className="text-sm text-amber-700 dark:text-amber-300">
                {storage_warning}
              </p>
            </div>
          )}

          {isReady ? (
            <div className="space-y-6">
              <div>
                <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-3">
                  Peak Hours by Category
                </h4>
                <div className="space-y-3">
                  {Object.entries(TASK_CATEGORIES).map(([key, label]) => {
                    const hours = patterns.peak_hours[key] || [];
                    return (
                      <div key={key} className="flex items-center gap-3">
                        <span className="text-sm text-zinc-600 dark:text-zinc-400 w-24">
                          {label}
                        </span>
                        <div className="flex-1 flex gap-1">
                          {hours.map((hour) => (
                            <span
                              key={hour}
                              className="px-2 py-0.5 text-xs bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 rounded"
                            >
                              {formatHour(hour)}
                            </span>
                          ))}
                          {hours.length === 0 && (
                            <span className="text-xs text-zinc-400">
                              No data
                            </span>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {patterns.low_productivity_hours.length > 0 && (
                <div>
                  <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                    Low Productivity Hours
                  </h4>
                  <p className="text-sm text-zinc-500">
                    {formatHourRange(patterns.low_productivity_hours)}
                  </p>
                  <p className="text-xs text-zinc-400 mt-1">
                    Consider avoiding deep work during these times
                  </p>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-6">
              <Clock className="w-10 h-10 text-zinc-300 dark:text-zinc-600 mx-auto mb-3" />
              <p className="text-sm text-zinc-600 dark:text-zinc-400">
                Complete more tasks to unlock productivity insights
              </p>
              {isLearning && (
                <div className="mt-4 max-w-xs mx-auto">
                  <div className="h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-indigo-500 rounded-full transition-all"
                      style={{
                        width: `${Math.max(
                          5,
                          ((50 -
                            (status as { completions_needed: number })
                              .completions_needed) /
                            50) *
                            100
                        )}%`,
                      }}
                    />
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      <div className="px-6 py-3 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
        <button
          onClick={() => setShowClearConfirm(true)}
          disabled={isDisabled || patterns.total_completions === 0}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Trash2 className="w-4 h-4" />
          Clear Data
        </button>

        <button
          onClick={handleExport}
          disabled={exportData.isPending || isDisabled}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors disabled:opacity-50"
        >
          <Download className="w-4 h-4" />
          Export
        </button>
      </div>

      {showClearConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white dark:bg-zinc-900 rounded-xl shadow-xl max-w-sm w-full p-6">
            <h3 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100 mb-2">
              Clear Productivity Data?
            </h3>
            <p className="text-sm text-zinc-500 mb-4">
              This will delete all learned productivity patterns. This action
              cannot be undone.
            </p>
            <div className="flex items-center justify-end gap-2">
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleClearData}
                disabled={clearData.isPending}
                className="px-4 py-2 text-sm bg-red-500 hover:bg-red-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
              >
                {clearData.isPending ? "Clearing..." : "Clear Data"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
