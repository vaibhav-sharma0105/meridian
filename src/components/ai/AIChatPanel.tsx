import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Send, Loader, Copy, CheckCheck, Bot, User, AlertCircle, Settings } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { useAI } from "@/hooks/useAI";
import { useUIStore } from "@/stores/uiStore";
import OutputTemplates from "./OutputTemplates";
import { MAX_CHAT_CHARS } from "@/lib/constants";

interface Props {
  projectId: string | null;
  fullPage?: boolean;
}

export default function AIChatPanel({ projectId, fullPage = false }: Props) {
  const { t } = useTranslation();
  const { setSettingsOpen } = useUIStore();
  const { messages, streaming, chatError, sendMessage, clearMessages, settings } = useAI(projectId);
  const [input, setInput] = useState("");
  const [copied, setCopied] = useState<number | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || streaming) return;
    setInput("");
    await sendMessage(text);
    // errors are handled inside useAI and surfaced via chatError
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
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

  return (
    <div className={`flex flex-col bg-white dark:bg-zinc-900 ${fullPage ? "h-full" : "h-full"}`}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-zinc-100 dark:border-zinc-800 flex-shrink-0">
        <div className="flex items-center gap-1.5">
          <Bot className="w-4 h-4 text-indigo-500" />
          <span className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">{t("ai.title")}</span>
        </div>
        {messages.length > 0 && (
          <button
            onClick={clearMessages}
            className="text-xs text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
          >
            {t("ai.clear")}
          </button>
        )}
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-3 py-3 space-y-4">
        {/* No AI configured banner */}
        {!settings && projectId && (
          <div className="flex items-start gap-2 px-3 py-2.5 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
            <AlertCircle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-xs text-amber-700 dark:text-amber-300">AI provider not configured.</p>
              <button
                onClick={() => setSettingsOpen(true)}
                className="text-xs text-amber-700 dark:text-amber-300 underline flex items-center gap-1 mt-0.5"
              >
                <Settings className="w-3 h-3" /> Open AI Settings
              </button>
            </div>
          </div>
        )}

        {messages.length === 0 && !chatError && (
          <div className="flex flex-col items-center justify-center h-full text-center py-8">
            <Bot className="w-8 h-8 text-zinc-300 dark:text-zinc-600 mb-3" />
            <p className="text-sm text-zinc-400">{t("ai.emptyHint")}</p>
          </div>
        )}

        {messages.map((msg, i) => (
          <div key={i} className={`flex gap-2 ${msg.role === "user" ? "flex-row-reverse" : "flex-row"}`}>
            <div className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center ${msg.role === "user" ? "bg-indigo-500" : "bg-zinc-200 dark:bg-zinc-700"}`}>
              {msg.role === "user"
                ? <User className="w-3.5 h-3.5 text-white" />
                : <Bot className="w-3.5 h-3.5 text-zinc-600 dark:text-zinc-300" />
              }
            </div>
            <div className={`group relative max-w-[85%] ${msg.role === "user" ? "items-end" : "items-start"} flex flex-col`}>
              <div className={`px-3 py-2 rounded-xl text-sm leading-relaxed ${msg.role === "user" ? "bg-indigo-500 text-white" : "bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-200"}`}>
                {msg.role === "assistant"
                  ? <ReactMarkdown className="prose prose-sm dark:prose-invert max-w-none prose-p:my-1 prose-pre:my-1">{msg.content}</ReactMarkdown>
                  : msg.content
                }
              </div>
              {msg.role === "assistant" && (
                <button
                  onClick={() => handleCopy(msg.content, i)}
                  className="mt-1 opacity-0 group-hover:opacity-100 transition-opacity text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
                >
                  {copied === i ? <CheckCheck className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
                </button>
              )}
            </div>
          </div>
        ))}

        {streaming && (
          <div className="flex gap-2">
            <div className="w-6 h-6 rounded-full bg-zinc-200 dark:bg-zinc-700 flex items-center justify-center">
              <Bot className="w-3.5 h-3.5 text-zinc-600 dark:text-zinc-300" />
            </div>
            <div className="px-3 py-2 bg-zinc-100 dark:bg-zinc-800 rounded-xl">
              <Loader className="w-4 h-4 text-zinc-400 animate-spin" />
            </div>
          </div>
        )}

        {chatError && (
          <div className="flex gap-2 items-start px-3 py-2.5 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
            <p className="text-xs text-red-700 dark:text-red-300">{chatError}</p>
          </div>
        )}

        <div ref={bottomRef} />
      </div>

      {/* Templates */}
      <OutputTemplates projectId={projectId} onOutput={handleTemplateOutput} />

      {/* Input */}
      <div className="flex-shrink-0 px-3 py-3 border-t border-zinc-100 dark:border-zinc-800">
        <div className="flex gap-2 items-end">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value.slice(0, MAX_CHAT_CHARS))}
            onKeyDown={handleKeyDown}
            placeholder={projectId ? t("ai.inputPlaceholder") : t("ai.noProject")}
            disabled={!projectId}
            rows={2}
            className="flex-1 px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 resize-none placeholder:text-zinc-400 disabled:opacity-50"
          />
          <button
            onClick={handleSend}
            disabled={streaming || !input.trim() || !projectId}
            className="p-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg transition-colors flex-shrink-0"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
        <p className="text-xs text-zinc-400 mt-1 text-right">
          {input.length}/{MAX_CHAT_CHARS}
        </p>
      </div>
    </div>
  );
}
