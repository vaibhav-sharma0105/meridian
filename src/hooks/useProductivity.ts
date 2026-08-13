import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getProductivityInsights,
  getTimeSuggestion,
  getTimeSuggestionForTask,
  getMeetingBatchingSuggestion,
  updateProductivitySettings,
  clearProductivityData,
  exportProductivityData,
  type ProductivityInsights,
  type TimeSuggestion,
  type ProductivityPatterns,
  type ProductivityStatus,
  type ProductivityExport,
  type ProductivitySettings,
  type BatchingSuggestion,
} from "@/lib/tauri";

export function useProductivityInsights() {
  return useQuery({
    queryKey: ["productivity-insights"],
    queryFn: getProductivityInsights,
    staleTime: 60000,
  });
}

export function useTimeSuggestion(category: string) {
  return useQuery({
    queryKey: ["time-suggestion", category],
    queryFn: () => getTimeSuggestion(category),
    enabled: !!category,
    staleTime: 300000,
  });
}

/**
 * Meeting-batching suggestion for a given day (defaults to today). Returns
 * null when the day's schedule is not fragmented enough to be worth batching.
 */
export function useMeetingBatchingSuggestion(date?: string) {
  return useQuery<BatchingSuggestion | null>({
    queryKey: ["meeting-batching-suggestion", date ?? "today"],
    queryFn: () => getMeetingBatchingSuggestion(date),
    staleTime: 300000,
  });
}

export function useTimeSuggestionForTask(taskId: string | undefined) {
  return useQuery({
    queryKey: ["time-suggestion-task", taskId],
    queryFn: () => getTimeSuggestionForTask(taskId!),
    enabled: !!taskId,
    staleTime: 300000,
  });
}

export function useUpdateProductivitySettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: ProductivitySettings) =>
      updateProductivitySettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });
      queryClient.invalidateQueries({ queryKey: ["productivity-insights"] });
    },
  });
}

export function useClearProductivityData() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: clearProductivityData,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productivity-insights"] });
    },
  });
}

export function useExportProductivityData() {
  return useMutation({
    mutationFn: exportProductivityData,
  });
}

export const TASK_CATEGORIES = {
  focus_work: "Focus Work",
  meetings: "Meetings",
  quick_tasks: "Quick Tasks",
} as const;

export function formatHour(hour: number): string {
  if (hour === 0) return "12 AM";
  if (hour === 12) return "12 PM";
  if (hour < 12) return `${hour} AM`;
  return `${hour - 12} PM`;
}

export function formatHourRange(hours: number[]): string {
  if (hours.length === 0) return "No data";
  const sorted = [...hours].sort((a, b) => a - b);
  return sorted.map(formatHour).join(", ");
}

export type {
  ProductivityInsights,
  TimeSuggestion,
  ProductivityPatterns,
  ProductivityStatus,
  ProductivityExport,
  ProductivitySettings,
};
