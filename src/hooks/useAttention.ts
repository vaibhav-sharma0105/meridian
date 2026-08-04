import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getAttentionItems,
  getAttentionCount,
  dismissAttentionItem,
  type AttentionItem,
  type AttentionFilters,
} from "@/lib/tauri";

export function useAttentionItems(filters?: AttentionFilters) {
  return useQuery({
    queryKey: ["attention-items", filters],
    queryFn: () => getAttentionItems(filters),
    refetchInterval: 30000,
  });
}

export function useAttentionCount() {
  return useQuery({
    queryKey: ["attention-count"],
    queryFn: getAttentionCount,
    refetchInterval: 30000,
  });
}

export function useDismissAttention() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: dismissAttentionItem,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["attention-items"] });
      queryClient.invalidateQueries({ queryKey: ["attention-count"] });
    },
  });
}

export type { AttentionItem, AttentionFilters };
