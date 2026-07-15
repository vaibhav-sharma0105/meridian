import { useState, useEffect, useCallback } from "react";
import { Copy, Check, Trash2, Loader2 } from "lucide-react";
import { SensitiveWarning } from "./SensitiveWarning";
import type { DraftMessage, SensitiveWarning as SensitiveWarningType, UpdateDraftInput } from "@/lib/tauri";
import * as api from "@/lib/tauri";

interface DraftEditorProps {
  draft: DraftMessage;
  onUpdate: (draft: DraftMessage) => void;
  onDelete: () => void;
}

export function DraftEditor({ draft, onUpdate, onDelete }: DraftEditorProps) {
  const [body, setBody] = useState(draft.body);
  const [aiSignature, setAiSignature] = useState(draft.ai_signature);
  const [warnings, setWarnings] = useState<SensitiveWarningType[]>([]);
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const [warningsDismissed, setWarningsDismissed] = useState(false);

  const scanContent = useCallback(async () => {
    const result = await api.scanDraft(body, draft.id);
    setWarnings(result);
    setWarningsDismissed(false);
  }, [body, draft.id]);

  useEffect(() => {
    const timeout = setTimeout(scanContent, 2000);
    return () => clearTimeout(timeout);
  }, [body, scanContent]);

  const handleSave = async () => {
    setSaving(true);
    try {
      const input: UpdateDraftInput = {
        body,
        ai_signature: aiSignature,
      };
      const updated = await api.updateDraft(draft.id, input);
      onUpdate(updated);
    } finally {
      setSaving(false);
    }
  };

  const handleCopy = async () => {
    let textToCopy = body;
    if (aiSignature && !body.includes("Drafted by Meridian")) {
      textToCopy += "\n\nDrafted by Meridian";
    }
    await navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDelete = async () => {
    await api.deleteDraft(draft.id);
    onDelete();
  };

  return (
    <div className="space-y-3">
      {!warningsDismissed && (
        <SensitiveWarning
          warnings={warnings}
          onDismiss={() => setWarningsDismissed(true)}
        />
      )}

      <div className="flex items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400">
        <span className="px-2 py-0.5 bg-zinc-100 dark:bg-zinc-800 rounded">
          {draft.channel}
        </span>
        {draft.subject && (
          <span className="truncate">Re: {draft.subject}</span>
        )}
      </div>

      <textarea
        value={body}
        onChange={(e) => setBody(e.target.value)}
        onBlur={handleSave}
        className="w-full h-48 p-3 text-sm border border-zinc-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        placeholder="Draft message..."
      />

      <div className="flex items-center justify-between">
        <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
          <input
            type="checkbox"
            checked={aiSignature}
            onChange={(e) => {
              setAiSignature(e.target.checked);
            }}
            className="rounded border-zinc-300 dark:border-zinc-600 text-indigo-500 focus:ring-indigo-500"
          />
          Include "Drafted by Meridian" signature
        </label>

        <div className="flex items-center gap-2">
          <button
            onClick={handleDelete}
            className="p-2 text-zinc-400 hover:text-red-500"
            title="Delete draft"
          >
            <Trash2 className="w-4 h-4" />
          </button>

          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-white bg-indigo-500 hover:bg-indigo-600 rounded-lg"
          >
            {copied ? (
              <>
                <Check className="w-4 h-4" />
                Copied
              </>
            ) : (
              <>
                <Copy className="w-4 h-4" />
                Copy to clipboard
              </>
            )}
          </button>
        </div>
      </div>

      {saving && (
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          <Loader2 className="w-3 h-3 animate-spin" />
          Saving...
        </div>
      )}
    </div>
  );
}
