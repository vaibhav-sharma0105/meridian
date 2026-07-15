import { useState } from "react";
import { Wand2, Edit3, Check, X, Loader2 } from "lucide-react";
import * as api from "@/lib/tauri";
import type { ExtractedSkillDefinition } from "@/lib/tauri";

interface ChatToSkillPreviewProps {
  description: string;
  onCreateSkill: (definition: ExtractedSkillDefinition) => void;
  onCancel: () => void;
}

export function ChatToSkillPreview({ description, onCreateSkill, onCancel }: ChatToSkillPreviewProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [definition, setDefinition] = useState<ExtractedSkillDefinition | null>(null);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState("");
  const [editDescription, setEditDescription] = useState("");

  useState(() => {
    api.extractSkillFromChat(description)
      .then((result) => {
        setDefinition(result);
        setEditName(result.name);
        setEditDescription(result.description);
        setLoading(false);
      })
      .catch((e) => {
        setError(typeof e === "string" ? e : "Failed to extract skill");
        setLoading(false);
      });
  });

  if (loading) {
    return (
      <div className="border border-indigo-200 dark:border-indigo-800 rounded-lg p-3 mt-2 bg-indigo-50/50 dark:bg-indigo-950/30">
        <div className="flex items-center gap-2 text-sm text-indigo-600 dark:text-indigo-400">
          <Loader2 className="w-4 h-4 animate-spin" />
          Extracting skill from description...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="border border-red-200 dark:border-red-800 rounded-lg p-3 mt-2">
        <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        <button onClick={onCancel} className="text-xs text-zinc-500 mt-1 hover:text-zinc-700">
          Dismiss
        </button>
      </div>
    );
  }

  if (!definition) return null;

  const triggerLabel = definition.trigger_type === "schedule"
    ? `Schedule: ${definition.trigger_config?.cron || "custom"}`
    : definition.trigger_type === "event"
    ? `Event: ${definition.trigger_config?.event_type || "custom"}`
    : "Manual";

  return (
    <div className="border border-indigo-200 dark:border-indigo-800 rounded-lg p-3 mt-2 bg-indigo-50/50 dark:bg-indigo-950/30">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-1.5 text-xs font-medium text-indigo-600 dark:text-indigo-400">
          <Wand2 className="w-3.5 h-3.5" />
          Skill Extracted
        </div>
        <button onClick={() => setEditing(!editing)} className="text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300">
          <Edit3 className="w-3 h-3" />
        </button>
      </div>

      {editing ? (
        <div className="space-y-2">
          <input
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            className="w-full text-sm px-2 py-1 border border-zinc-200 dark:border-zinc-700 rounded bg-white dark:bg-zinc-900"
            placeholder="Skill name"
          />
          <input
            value={editDescription}
            onChange={(e) => setEditDescription(e.target.value)}
            className="w-full text-sm px-2 py-1 border border-zinc-200 dark:border-zinc-700 rounded bg-white dark:bg-zinc-900"
            placeholder="Description"
          />
        </div>
      ) : (
        <div className="mb-2">
          <p className="text-sm font-medium text-zinc-900 dark:text-zinc-100">{definition.name}</p>
          <p className="text-xs text-zinc-500">{definition.description}</p>
        </div>
      )}

      <div className="flex gap-2 text-xs text-zinc-500 mt-1.5 mb-2.5">
        <span className="px-1.5 py-0.5 bg-zinc-100 dark:bg-zinc-800 rounded">{triggerLabel}</span>
        <span className="px-1.5 py-0.5 bg-zinc-100 dark:bg-zinc-800 rounded">{definition.action_type}</span>
      </div>

      <div className="flex gap-2">
        <button
          onClick={() => {
            const final_def = editing
              ? { ...definition, name: editName, description: editDescription }
              : definition;
            onCreateSkill(final_def);
          }}
          className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium text-white bg-indigo-500 rounded hover:bg-indigo-600"
        >
          <Check className="w-3 h-3" />
          Create Skill
        </button>
        <button
          onClick={onCancel}
          className="flex items-center gap-1 px-2.5 py-1 text-xs text-zinc-600 dark:text-zinc-400 border border-zinc-200 dark:border-zinc-700 rounded hover:bg-zinc-50 dark:hover:bg-zinc-800"
        >
          <X className="w-3 h-3" />
          Cancel
        </button>
      </div>
    </div>
  );
}
