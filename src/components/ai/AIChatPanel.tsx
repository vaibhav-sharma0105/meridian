import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Send, Copy, CheckCheck, Bot, User, AlertCircle, Settings, Sparkles } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { useAI } from "@/hooks/useAI";
import { useUIStore } from "@/stores/uiStore";
import OutputTemplates from "./OutputTemplates";
import { MAX_CHAT_CHARS } from "@/lib/constants";

interface Props {
  projectId: string | null;
  fullPage?: boolean;
}

const SUGGESTED_PROMPTS = [
  { icon: "📋", label: "Summarize open tasks" },
  { icon: "🚧", label: "What's blocking progress?" },
  { icon: "📊", label: "Draft a status update" },
  { icon: "⚡", label: "Find high-priority items" },
];

function TypingDots() {
  return (
    <div className="flex items-center gap-1 px-3 py-2.5">
      <div className="typing-dot" />
      <div className="typing-dot" />
      <div className="typing-dot" />
    </div>
  );
}

export default function AIChatPanel({ projectId, fullPage = false }: Props) {
  const { t } = useTranslation();
  const { setSettingsOpen } = useUIStore();
  const { messages, streaming, chatError, sendMessage, clearMessages, settings } = useAI(projectId);
  const [input, setInput] = useState("");
  const [copied, setCopied] = useState<number | null>(null);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const el = messagesContainerRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages, streaming]);

  const handleSend = async (text?: string) => {
    const msg = (text ?? input).trim();
    if (!msg || streaming) return;
    setInput("");
    await sendMessage(msg);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void handleSend();
    }
  };

  const handleCopy = async (text: string, idx: number) => {
    await navigator.clipboard.writeText(text);
    setCopied(idx);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleTemplateOutput = (text: string) => {
    setInput(text.slice(0, MAX_CHAT_CHARS));
    inputRef.current?.focus();
  };

  const isEmpty = messages.length === 0 && !chatError;

  return (
    <div className="flex flex-col bg-white dark:bg-[#0f0f12] h-full">

      {/* ── Header ──────────────────────────────────────────────────────── */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-[#ebebf0] dark:border-[#1a1a1e] flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-lg ai-gradient flex items-center justify-center shadow-sm">
            <Sparkles className="w-3.5 h-3.5 text-white" />
          </div>
          <span className="text-[13.5px] font-semibold text-zinc-800 dark:text-zinc-200">AI Chat</span>
        </div>
        {messages.length > 0 && (
          <button
            onClick={clearMessages}
            className="text-[12px] text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors px-1.5 py-0.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            {t("ai.clear")}
          </button>
        )}
      </div>

      {/* ── No AI configured banner ──────────────────────────────────────── */}
      {!settings && projectId && (
        <div className="mx-3 mt-3 flex items-start gap-2 px-3 py-2.5 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl flex-shrink-0">
          <AlertCircle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
          <div className="flex-1 min-w-0">
            <p className="text-[12.5px] text-amber-700 dark:text-amber-300">AI provider not configured.</p>
            <button
              onClick={() => setSettingsOpen(true)}
              className="text-[12.5px] text-amber-700 dark:text-amber-300 underline flex items-center gap-1 mt-0.5"
            >
              <Settings className="w-3 h-3" /> Open AI Settings
            </button>
          </div>
        </div>
      )}

      {/* ── Messages / Empty state ───────────────────────────────────────── */}
      <div ref={messagesContainerRef} className="flex-1 overflow-y-auto px-4 py-4 space-y-4 min-h-0">

        {/* AI-first empty state with suggested prompts */}
        {isEmpty && (
          <div className="flex flex-col items-center justify-center h-full text-center space-y-5 pb-4 animate-fade-in">
            <div className="w-14 h-14 rounded-2xl ai-gradient flex items-center justify-center shadow-lg">
              <Sparkles className="w-7 h-7 text-white" />
            </div>
            <div className="space-y-1">
              <p className="text-[14px] font-semibold text-zinc-700 dark:text-zinc-300">
                {projectId ? "Ask anything about this project" : "Select a project to start"}
              </p>
              {projectId && (
                <p className="text-[12.5px] text-zinc-400 dark:text-zinc-500 max-w-[200px] leading-relaxed">
                  {t("ai.emptyHint")}
                </p>
              )}
            </div>
            {projectId && (
              <div className="grid grid-cols-2 gap-2 w-full max-w-[260px]">
                {SUGGESTED_PROMPTS.map((p) => (
                  <button
                    key={p.label}
                    onClick={() => void handleSend(p.label)}
                    disabled={streaming}
                    className="flex flex-col items-start gap-1.5 px-3 py-2.5 text-left rounded-xl border border-[#e2e2e8] dark:border-zinc-800 bg-zinc-50/80 dark:bg-zinc-800/40 hover:bg-indigo-50 dark:hover:bg-indigo-950/30 hover:border-indigo-200 dark:hover:border-indigo-800/60 transition-all duration-150 group"
                  >
                    <span className="text-[16px] leading-none">{p.icon}</span>
                    <span className="text-[12px] text-zinc-600 dark:text-zinc-400 group-hover:text-indigo-700 dark:group-hover:text-indigo-300 font-medium leading-snug">{p.label}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Messages */}
        {messages.map((msg, i) => (
          <div key={i} className={`flex gap-2.5 ${msg.role === "user" ? "flex-row-reverse" : "flex-row"} animate-fade-in`}>
            <div className={`flex-shrink-0 w-7 h-7 rounded-full flex items-center justify-center shadow-sm ${
              msg.role === "user"
                ? "bg-indigo-500"
                : "ai-gradient"
            }`}>
              {msg.role === "user"
                ? <User className="w-3.5 h-3.5 text-white" />
                : <Sparkles className="w-3.5 h-3.5 text-white" />
              }
            </div>
            <div className={`group relative max-w-[84%] flex flex-col ${msg.role === "user" ? "items-end" : "items-start"}`}>
              <div className={`px-3.5 py-2.5 rounded-2xl text-[13.5px] leading-relaxed ${
                msg.role === "user"
                  ? "bg-indigo-500 text-white rounded-tr-md"
                  : "bg-[#f4f4f8] dark:bg-zinc-800 text-zinc-800 dark:text-zinc-200 rounded-tl-md"
              }`}>
                {msg.role === "assistant"
                  ? <ReactMarkdown className="prose prose-sm dark:prose-invert max-w-none prose-p:my-1 prose-pre:my-1">{msg.content}</ReactMarkdown>
                  : msg.content
                }
              </div>
              {msg.role === "assistant" && (
                <button
                  onClick={() => handleCopy(msg.content, i)}
                  className="mt-1 opacity-0 group-hover:opacity-100 transition-opacity text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 p-0.5"
                  title="Copy"
                >
                  {copied === i ? <CheckCheck className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                </button>
              )}
            </div>
          </div>
        ))}

        {/* Typing indicator */}
        {streaming && (
          <div className="flex gap-2.5 animate-fade-in">
            <div className="flex-shrink-0 w-7 h-7 rounded-full ai-gradient flex items-center justify-center shadow-sm">
              <Sparkles className="w-3.5 h-3.5 text-white" />
            </div>
            <div className="bg-[#f4f4f8] dark:bg-zinc-800 rounded-2xl rounded-tl-md">
              <TypingDots />
            </div>
          </div>
        )}

        {chatError && (
          <div className="flex gap-2 items-start px-3 py-2.5 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl">
            <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
            <p className="text-[12.5px] text-red-700 dark:text-red-300">{chatError}</p>
          </div>
        )}
      </div>

      {/* ── Templates ───────────────────────────────────────────────────── */}
      <OutputTemplates projectId={projectId} onOutput={handleTemplateOutput} />

      {/* ── Input ───────────────────────────────────────────────────────── */}
      <div className="flex-shrink-0 px-3 pb-3 pt-2 border-t border-[#ebebf0] dark:border-[#1a1a1e]">
        <div className="relative flex items-end gap-0 bg-[#f4f4f8] dark:bg-zinc-800 rounded-2xl border border-[#e2e2e8] dark:border-zinc-700 focus-within:border-indigo-400 focus-within:ring-2 focus-within:ring-indigo-400/20 transition-all duration-150">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value.slice(0, MAX_CHAT_CHARS))}
            onKeyDown={handleKeyDown}
            placeholder={projectId ? t("ai.inputPlaceholder") : t("ai.noProject")}
            disabled={!projectId}
            rows={2}
            className="flex-1 px-4 py-3 text-[13.5px] bg-transparent text-zinc-900 dark:text-zinc-50 resize-none placeholder:text-zinc-400 disabled:opacity-50 outline-none leading-relaxed"
          />
          <button
            onClick={() => void handleSend()}
            disabled={streaming || !input.trim() || !projectId}
            className="m-2 p-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-40 disabled:cursor-not-allowed text-white rounded-xl transition-all duration-150 shadow-sm hover:shadow-md flex-shrink-0"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
        <p className="text-[11px] text-zinc-300 dark:text-zinc-600 mt-1.5 text-right tabular-nums">
          {input.length}/{MAX_CHAT_CHARS}
        </p>
      </div>
    </div>
  );
}
