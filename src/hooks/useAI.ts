import { useCallback, useState, useMemo, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import * as api from "@/lib/tauri";
import { useAIStore } from "@/stores/aiStore";
import type { ChatMessage, Skill, SkillFolder } from "@/lib/tauri";

// Unified skill representation for AI context and picker
export interface UnifiedSkill {
  id: string;
  name: string;
  description: string | null;
  type: "db" | "folder";
  trigger_type: string;
  category: string | null;
  action_type: string | null;
  enabled: boolean;
  // For folder skills, store the folder name for execution
  folderName?: string;
  // For DB skills, keep original for execution
  originalSkill?: Skill;
  // For folder skills, keep original for file access
  originalFolder?: SkillFolder;
}

function dbSkillToUnified(skill: Skill): UnifiedSkill {
  let actionType: string | null = null;
  if (skill.action_config) {
    try {
      const config = JSON.parse(skill.action_config);
      actionType = config.action_type || null;
    } catch {}
  }

  return {
    id: skill.id,
    name: skill.name,
    description: skill.description,
    type: "db",
    trigger_type: skill.trigger_type,
    category: skill.category,
    action_type: actionType,
    enabled: skill.enabled,
    originalSkill: skill,
  };
}

function folderSkillToUnified(folder: SkillFolder): UnifiedSkill {
  return {
    id: `folder:${folder.name}`,
    name: folder.name,
    description: folder.description,
    type: "folder",
    trigger_type: "manual",
    category: "custom",
    action_type: "custom",
    enabled: folder.enabled,
    folderName: folder.name,
    originalFolder: folder,
  };
}

// Compact format for LLM - just essentials for decision making
function formatSkillsAsCompactContext(skills: UnifiedSkill[]): string {
  if (skills.length === 0) return "";

  const skillLines = skills.map(skill => {
    const type = skill.type === "folder" ? "📦" : "⚡";
    const desc = skill.description ? ` - ${skill.description}` : "";
    return `${type} **${skill.name}**${desc}`;
  });

  return skillLines.join("\n");
}

export function useAI(projectId: string | null) {
  const {
    messages,
    streaming,
    streamingContent,
    addMessage,
    setStreaming,
    resetStream,
    clearMessages: storeClearMessages,
  } = useAIStore();

  const [chatError, setChatError] = useState<string | null>(null);
  // Track skills invoked in this conversation to prevent duplicates
  const [invokedSkills, setInvokedSkills] = useState<Set<string>>(new Set());
  // Cache loaded skill content per conversation (progressive loading)
  const loadedSkillContent = useRef<Map<string, string>>(new Map());

  const settingsQuery = useQuery({
    queryKey: ["ai-settings"],
    queryFn: api.getAiSettings,
  });

  // Fetch enabled DB skills
  const dbSkillsQuery = useQuery({
    queryKey: ["skills", { enabled: true }],
    queryFn: () => api.listSkills({ enabled: true }),
  });

  // Fetch folder packages (includes enabled state)
  const folderSkillsQuery = useQuery({
    queryKey: ["skill-folders"],
    queryFn: api.listSkillFolders,
  });

  // Raw data for external use (e.g., SkillPicker)
  const dbSkills = dbSkillsQuery.data ?? [];
  const folderSkills = folderSkillsQuery.data ?? [];

  // Merge DB skills and folder packages into unified list
  const allEnabledSkills = useMemo((): UnifiedSkill[] => {
    const unified: UnifiedSkill[] = [];

    for (const skill of dbSkills) {
      if (skill.enabled) {
        unified.push(dbSkillToUnified(skill));
      }
    }

    for (const folder of folderSkills) {
      if (folder.enabled) {
        unified.push(folderSkillToUnified(folder));
      }
    }

    return unified;
  }, [dbSkills, folderSkills]);

  // Build compact skill context string for LLM (Phase 1: lightweight)
  const skillContext = useMemo(() => {
    return allEnabledSkills.length > 0 ? formatSkillsAsCompactContext(allEnabledSkills) : undefined;
  }, [allEnabledSkills]);

  const clearMessages = useCallback(() => {
    storeClearMessages();
    setInvokedSkills(new Set());
    loadedSkillContent.current.clear();
  }, [storeClearMessages]);

  const markSkillInvoked = useCallback((skillName: string) => {
    setInvokedSkills(prev => new Set(prev).add(skillName.toLowerCase()));
  }, []);

  const isSkillInvoked = useCallback((skillName: string): boolean => {
    return invokedSkills.has(skillName.toLowerCase());
  }, [invokedSkills]);

  // Progressive loading: fetch full skill content when needed
  const loadSkillContent = useCallback(async (skill: UnifiedSkill): Promise<string | null> => {
    const cacheKey = skill.id;
    if (loadedSkillContent.current.has(cacheKey)) {
      return loadedSkillContent.current.get(cacheKey) ?? null;
    }

    try {
      if (skill.type === "folder" && skill.folderName) {
        const content = await api.readSkillFile(skill.folderName, "skill.md");
        loadedSkillContent.current.set(cacheKey, content);
        return content;
      } else if (skill.type === "db" && skill.originalSkill) {
        // For DB skills, construct content from context_config
        const ctxConfig = skill.originalSkill.context_config;
        if (ctxConfig) {
          try {
            const config = JSON.parse(ctxConfig);
            const content = config.system_prompt || skill.description || "";
            loadedSkillContent.current.set(cacheKey, content);
            return content;
          } catch {}
        }
        return skill.description;
      }
    } catch (err) {
      console.error(`Failed to load skill content for ${skill.name}:`, err);
    }
    return null;
  }, []);

  // Execute a skill (handles both DB and folder types)
  const executeSkill = useCallback(async (skill: UnifiedSkill): Promise<{ success: boolean; output?: string }> => {
    try {
      if (skill.type === "db" && skill.originalSkill) {
        await api.runSkillManually(skill.originalSkill.id);
        return { success: true };
      } else if (skill.type === "folder" && skill.folderName && skill.originalFolder) {
        // For folder packages, find and execute the first executable script
        const findExecutables = (entries: api.SkillFileEntry[]): string[] => {
          const executables: string[] = [];
          for (const entry of entries) {
            if (entry.is_executable) {
              executables.push(entry.path);
            }
            if (entry.children) {
              executables.push(...findExecutables(entry.children));
            }
          }
          return executables;
        };

        const executables = findExecutables(skill.originalFolder.files);
        if (executables.length > 0) {
          // Execute the first script (typically main.py or run.sh)
          const mainScript = executables.find(e =>
            e.includes("main.") || e.includes("run.") || e.includes("index.")
          ) || executables[0];

          const output = await api.executeSkillScript(skill.folderName, mainScript);
          return { success: true, output };
        } else {
          // No executable scripts - just load content and return success
          const content = await loadSkillContent(skill);
          return { success: true, output: content ?? undefined };
        }
      }
      return { success: false };
    } catch (err) {
      console.error(`Skill execution failed for ${skill.name}:`, err);
      return { success: false };
    }
  }, [loadSkillContent]);

  const sendMessage = useCallback(
    async (text: string, selectedSkill?: UnifiedSkill | Skill | null) => {
      if (!projectId) return;

      setChatError(null);

      // If a skill is selected, include its context in the message
      let messageText = text;
      let unifiedSkill: UnifiedSkill | null = null;

      if (selectedSkill) {
        // Convert to UnifiedSkill if needed
        if ("type" in selectedSkill && (selectedSkill.type === "db" || selectedSkill.type === "folder")) {
          unifiedSkill = selectedSkill as UnifiedSkill;
        } else {
          // It's a raw Skill from the picker
          unifiedSkill = dbSkillToUnified(selectedSkill as Skill);
        }

        // Load full content for the selected skill (progressive disclosure)
        const fullContent = await loadSkillContent(unifiedSkill);
        const skillInfo = fullContent
          ? `[Using skill: ${unifiedSkill.name}]\n\n${fullContent}`
          : `[Using skill: ${unifiedSkill.name}]\n${unifiedSkill.description || "Execute this skill."}`;

        messageText = skillInfo + "\n\n" + (text || "Run this skill and provide the output.");
        markSkillInvoked(unifiedSkill.name);
      }

      const userMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: selectedSkill ? `[Skill: ${selectedSkill.name}] ${text || "Execute skill"}` : text,
      };
      addMessage(userMsg);

      try {
        setStreaming(true);
        const currentMessages = useAIStore.getState().messages;
        const history = currentMessages.slice(-10).map((m) => ({
          role: m.role,
          content: m.content,
        }));

        const result = await api.chatWithProject({
          projectId,
          message: messageText,
          conversationHistory: history,
          skillContext,
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
    [projectId, addMessage, setStreaming, skillContext, markSkillInvoked, loadSkillContent]
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
    // Expose for skill picker and execution
    allEnabledSkills,
    dbSkills,
    folderSkills,
    markSkillInvoked,
    isSkillInvoked,
    loadSkillContent,
    executeSkill,
  };
}
