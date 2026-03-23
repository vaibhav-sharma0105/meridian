import { create } from "zustand";
import type { AiSettings, ChatMessage } from "@/lib/tauri";
import * as api from "@/lib/tauri";

interface AIStore {
  settings: AiSettings | null;
  messages: ChatMessage[];
  streaming: boolean;
  streamingContent: string;
  chatScope: { projectId: string | null; meetingId: string | null };
  // Actions
  setSettings: (settings: AiSettings | null) => void;
  loadSettings: () => Promise<void>;
  addMessage: (message: ChatMessage) => void;
  clearMessages: () => void;
  setStreaming: (streaming: boolean) => void;
  appendStreamChunk: (chunk: string) => void;
  resetStream: () => void;
  setChatScope: (projectId: string | null, meetingId?: string | null) => void;
}

export const useAIStore = create<AIStore>((set, get) => ({
  settings: null,
  messages: [],
  streaming: false,
  streamingContent: "",
  chatScope: { projectId: null, meetingId: null },

  setSettings: (settings) => set({ settings }),

  loadSettings: async () => {
    const s = await api.getAiSettings();
    set({ settings: s });
  },

  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),

  clearMessages: () => set({ messages: [], streamingContent: "" }),

  setStreaming: (streaming) => set({ streaming }),

  appendStreamChunk: (chunk) =>
    set((state) => ({
      streamingContent: state.streamingContent + chunk,
    })),

  resetStream: () => set({ streamingContent: "", streaming: false }),

  setChatScope: (projectId, meetingId = null) =>
    set({ chatScope: { projectId, meetingId } }),
}));
