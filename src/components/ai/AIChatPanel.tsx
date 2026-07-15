import { useState, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Send, Copy, CheckCheck, User, AlertCircle, Settings, Sparkles, Wand2, Zap, CheckCircle2 } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { useAI, UnifiedSkill } from "@/hooks/useAI";
import { useUIStore } from "@/stores/uiStore";
import { ChatToSkillPreview } from "@/components/skills/ChatToSkillPreview";
import { SkillPicker, SkillBadge } from "./SkillPicker";
import { MAX_CHAT_CHARS } from "@/lib/constants";
import type { ExtractedSkillDefinition } from "@/lib/tauri";

// Parse skill invocation markers from AI response
function parseSkillInvocation(content: string): { skillName: string | null; cleanContent: string } {
  const match = content.match(/\*\*\[SKILL_INVOKE:\s*([^\]]+)\]\*\*/);
  if (match) {
    const skillName = match[1].trim();
    const cleanContent = content.replace(match[0], "").trim();
    return { skillName, cleanContent };
  }
  return { skillName: null, cleanContent: content };
}

type SkillExecStatus = "running" | "completed";

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
  const {
    messages,
    streaming,
    chatError,
    sendMessage,
    clearMessages,
    settings,
    allEnabledSkills,
    markSkillInvoked,
    isSkillInvoked,
    executeSkill,
  } = useAI(projectId);

  const [input, setInput] = useState("");
  const [copied, setCopied] = useState<number | null>(null);
  const [skillExtractMsg, setSkillExtractMsg] = useState<number | null>(null);
  const [showSkillPicker, setShowSkillPicker] = useState(false);
  const [selectedSkill, setSelectedSkill] = useState<UnifiedSkill | null>(null);
  const [skillExecStatus, setSkillExecStatus] = useState<Record<number, { status: SkillExecStatus; skillName: string }>>({});
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const inputContainerRef = useRef<HTMLDivElement>(null);
  // Track processed messages to prevent duplicate execution
  const processedMsgIndices = useRef<Set<number>>(new Set());

  useEffect(() => {
    const el = messagesContainerRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages, streaming]);

  // Execute skill for a message
  const executeSkillForMessage = useCallback(async (
    idx: number,
    skillName: string,
    matchedSkill: UnifiedSkill
  ) => {
    // Prevent duplicate execution
    if (processedMsgIndices.current.has(idx)) return;
    processedMsgIndices.current.add(idx);

    // Check if already invoked in this conversation
    if (isSkillInvoked(skillName)) {
      setSkillExecStatus(prev => ({
        ...prev,
        [idx]: { status: "completed", skillName }
      }));
      return;
    }

    setSkillExecStatus(prev => ({ ...prev, [idx]: { status: "running", skillName } }));
    markSkillInvoked(skillName);

    // Execute the skill
    const result = await executeSkill(matchedSkill);

    // Always mark as completed (subtle UI - don't show failures prominently)
    setSkillExecStatus(prev => ({ ...prev, [idx]: { status: "completed", skillName } }));

    if (!result.success) {
      console.warn(`Skill "${skillName}" execution completed with issues`);
    }
  }, [isSkillInvoked, markSkillInvoked, executeSkill]);

  // Auto-execute skills when LLM invokes them
  useEffect(() => {
    const lastIdx = messages.length - 1;
    if (lastIdx < 0) return;

    const msg = messages[lastIdx];
    if (msg.role !== "assistant") return;
    if (processedMsgIndices.current.has(lastIdx)) return;
    if (skillExecStatus[lastIdx]) return;

    const { skillName } = parseSkillInvocation(msg.content);
    if (!skillName) return;

    // Find matching skill
    const matchedSkill = allEnabledSkills.find(
      s => s.name.toLowerCase() === skillName.toLowerCase()
    );

    if (matchedSkill) {
      executeSkillForMessage(lastIdx, skillName, matchedSkill);
    }
  }, [messages.length, allEnabledSkills, skillExecStatus, executeSkillForMessage]);

  // Reset on clear
  useEffect(() => {
    if (messages.length === 0) {
      processedMsgIndices.current.clear();
      setSkillExecStatus({});
    }
  }, [messages.length]);

  useEffect(() => {
    if (input.endsWith("/skill") || input === "/skill") {
      setShowSkillPicker(true);
    }
  }, [input]);

  const handleSkillSelect = (skill: UnifiedSkill) => {
    setSelectedSkill(skill);
    setShowSkillPicker(false);
    setInput(input.replace(/\/skill\s*$/, "").trim());
    inputRef.current?.focus();
  };

  const handleSend = async (text?: string) => {
    const msg = (text ?? input).trim();
    if (!msg && !selectedSkill) return;
    if (streaming) return;

    const skillToUse = selectedSkill;
    setInput("");
    setSelectedSkill(null);
    await sendMessage(msg, skillToUse);
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

  const isEmpty = messages.length === 0 && !chatError;

  return (
    <div className="flex flex-col bg-white dark:bg-[#0f0f12] h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-[#ebebf0] dark:border-[#1a1a1e] flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-lg ai-gradient flex items-center justify-center shadow-sm">
            <Sparkles className="w-3.5 h-3.5 text-white" />
          </div>
          <span className="text-[13.5px] font-semibold text-zinc-800 dark:text-zinc-200">AI Chat</span>
          {allEnabledSkills.length > 0 && (
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400">
              {allEnabledSkills.length} skills
            </span>
          )}
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

      {/* No AI configured banner */}
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

      {/* Messages */}
      <div ref={messagesContainerRef} className="flex-1 overflow-y-auto px-4 py-4 space-y-4 min-h-0">
        {/* Empty state */}
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

        {/* Message list */}
        {messages.map((msg, i) => {
          const { skillName, cleanContent } = msg.role === "assistant"
            ? parseSkillInvocation(msg.content)
            : { skillName: null, cleanContent: msg.content };

          const execStatus = skillExecStatus[i];

          return (
            <div key={i} className={`flex gap-2.5 ${msg.role === "user" ? "flex-row-reverse" : "flex-row"} animate-fade-in`}>
              <div className={`flex-shrink-0 w-7 h-7 rounded-full flex items-center justify-center shadow-sm ${
                msg.role === "user" ? "bg-indigo-500" : "ai-gradient"
              }`}>
                {msg.role === "user"
                  ? <User className="w-3.5 h-3.5 text-white" />
                  : <Sparkles className="w-3.5 h-3.5 text-white" />
                }
              </div>
              <div className={`group relative max-w-[84%] flex flex-col ${msg.role === "user" ? "items-end" : "items-start"}`}>
                {/* Subtle skill indicator - only show when completed */}
                {skillName && execStatus?.status === "completed" && (
                  <div className="mb-1 flex items-center gap-1 text-[10px] text-emerald-600 dark:text-emerald-400">
                    <CheckCircle2 className="w-3 h-3" />
                    <span>{skillName}</span>
                  </div>
                )}
                <div className={`px-3.5 py-2.5 rounded-2xl text-[13.5px] leading-relaxed ${
                  msg.role === "user"
                    ? "bg-indigo-500 text-white rounded-tr-md"
                    : "bg-[#f4f4f8] dark:bg-zinc-800 text-zinc-800 dark:text-zinc-200 rounded-tl-md"
                }`}>
                  {msg.role === "assistant"
                    ? <ReactMarkdown className="prose prose-sm dark:prose-invert max-w-none prose-p:my-1 prose-pre:my-1">{cleanContent}</ReactMarkdown>
                    : cleanContent
                  }
                </div>
                {msg.role === "assistant" && (
                  <div className="mt-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleCopy(msg.content, i)}
                      className="text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 p-0.5"
                      title="Copy"
                    >
                      {copied === i ? <CheckCheck className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                    </button>
                    <button
                      onClick={() => setSkillExtractMsg(skillExtractMsg === i ? null : i)}
                      className="text-zinc-400 hover:text-indigo-500 dark:hover:text-indigo-400 p-0.5"
                      title="Create skill from this"
                    >
                      <Wand2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                )}
                {skillExtractMsg === i && msg.role === "assistant" && (
                  <ChatToSkillPreview
                    description={msg.content}
                    onCreateSkill={(def: ExtractedSkillDefinition) => {
                      setSkillExtractMsg(null);
                      useUIStore.getState().setActiveView("skills");
                      useUIStore.getState().setSkillEditorData(def as unknown as Record<string, unknown>);
                    }}
                    onCancel={() => setSkillExtractMsg(null)}
                  />
                )}
              </div>
            </div>
          );
        })}

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

      {/* Input area */}
      <div className="px-4 py-3 border-t border-[#ebebf0] dark:border-[#1a1a1e] flex-shrink-0">
        <div ref={inputContainerRef} className="relative">
          {/* Selected skill badge */}
          {selectedSkill && (
            <div className="mb-2">
              <SkillBadge skill={selectedSkill} onRemove={() => setSelectedSkill(null)} />
            </div>
          )}

          {/* Skill picker popup */}
          {showSkillPicker && inputContainerRef.current && (
            <SkillPicker
              skills={allEnabledSkills}
              onSelect={handleSkillSelect}
              onClose={() => setShowSkillPicker(false)}
              position={{
                bottom: inputContainerRef.current.offsetHeight + 8,
                left: 0,
              }}
            />
          )}

          <div className="flex gap-2 items-end">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={projectId ? "Ask about this project... (type /skill to use a skill)" : "Select a project first"}
              disabled={!projectId || streaming}
              rows={1}
              maxLength={MAX_CHAT_CHARS}
              className="flex-1 resize-none px-3.5 py-2.5 text-[13.5px] bg-[#f4f4f8] dark:bg-zinc-800 border-0 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 placeholder:text-zinc-400"
              style={{ minHeight: "42px", maxHeight: "120px" }}
            />
            <button
              onClick={() => void handleSend()}
              disabled={(!input.trim() && !selectedSkill) || !projectId || streaming}
              className="p-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-40 disabled:hover:bg-indigo-500 text-white rounded-xl transition-colors flex-shrink-0"
            >
              <Send className="w-4 h-4" />
            </button>
          </div>

          {/* Hint */}
          <div className="mt-1.5 flex items-center justify-between text-[10px] text-zinc-400">
            <span>Type /skill to use a skill</span>
            <span>{input.length}/{MAX_CHAT_CHARS}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
