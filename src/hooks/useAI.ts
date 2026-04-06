import { useCallback, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import { useAIStore } from "@/stores/aiStore";
import type { ChatMessage } from "@/lib/tauri";

export function useAI(projectId: string | null) {
  const {
    messages,
    streaming,
    streamingContent,
    addMessage,
    setStreaming,
    resetStream,
    clearMessages,
  } = useAIStore();

  const [chatError, setChatError] = useState<string | null>(null);

  const settingsQuery = useQuery({
    queryKey: ["ai-settings"],
    queryFn: api.getAiSettings,
  });

  const sendMessage = useCallback(
    async (text: string) => {
      if (!projectId) return;

      setChatError(null);

      const userMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: text,
      };
      addMessage(userMsg);
      setStreaming(true);

      try {
        // Read latest messages from store to avoid stale closure
        const currentMessages = useAIStore.getState().messages;
        const history = currentMessages.slice(-10).map((m) => ({
          role: m.role,
          content: m.content,
        }));

        const result = await api.chatWithProject({
          projectId,
          message: text,
          conversationHistory: history,
        });

        addMessage({
          id: result.id ?? crypto.randomUUID(),
          role: "assistant",
          content: result.content,
        });
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setChatError(
          msg.includes("No AI provider")
            ? "AI provider not configured. Open Settings → AI Settings to connect."
            : msg
        );
      } finally {
        setStreaming(false);
      }
    },
    [projectId, addMessage, setStreaming]
  );

  const stopStreaming = useCallback(() => {
    resetStream();
  }, [resetStream]);

  return {
    settings: settingsQuery.data,
    isLoadingSettings: settingsQuery.isLoading,
    messages,
    streaming,
    streamingContent,
    chatError,
    sendMessage,
    stopStreaming,
    clearMessages,
  };
}
