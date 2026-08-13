import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getMessages,
  getMessage,
  createMessage,
  softDeleteMessage,
  restoreMessage,
  pinFromSource,
  getStorageStats,
  runMessageCleanup,
  type Message,
  type MessageFilters,
  type PaginatedMessages,
  type CreateMessageInput,
  type StorageStats,
  type CleanupStats,
} from "@/lib/tauri";

export function useMessages(
  filters?: MessageFilters,
  page?: number,
  perPage?: number
) {
  return useQuery({
    queryKey: ["messages", filters, page, perPage],
    queryFn: () => getMessages(filters, page, perPage),
  });
}

export function useMessage(id: string | undefined) {
  return useQuery({
    queryKey: ["message", id],
    queryFn: () => getMessage(id!),
    enabled: !!id,
  });
}

export function useStorageStats() {
  return useQuery({
    queryKey: ["message-storage-stats"],
    queryFn: getStorageStats,
    staleTime: 60000,
  });
}

export function useCreateMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateMessageInput) => createMessage(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
      queryClient.invalidateQueries({ queryKey: ["message-storage-stats"] });
    },
  });
}

export function useDeleteMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: softDeleteMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });
}

export function useRestoreMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: restoreMessage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });
}

export function usePinFromSource() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      sourceType,
      sourceId,
      title,
      content,
      projectId,
    }: {
      sourceType: string;
      sourceId: string;
      title: string;
      content?: string;
      projectId?: string;
    }) => pinFromSource(sourceType, sourceId, title, content, projectId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });
}

export function useRunMessageCleanup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: runMessageCleanup,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
      queryClient.invalidateQueries({ queryKey: ["message-storage-stats"] });
    },
  });
}

export type {
  Message,
  MessageFilters,
  PaginatedMessages,
  CreateMessageInput,
  StorageStats,
  CleanupStats,
};
