import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, FileText, Loader2 } from "lucide-react";
import { DraftEditor } from "./DraftEditor";
import type { DraftMessage } from "@/lib/tauri";
import * as api from "@/lib/tauri";

interface DraftsTabProps {
  taskId: string;
  taskTitle: string;
}

export function DraftsTab({ taskId, taskTitle }: DraftsTabProps) {
  const queryClient = useQueryClient();
  const [generatingChannel, setGeneratingChannel] = useState<string | null>(null);

  const { data: drafts = [], isLoading } = useQuery({
    queryKey: ["drafts", taskId],
    queryFn: () => api.getDraftsForTask(taskId),
  });

  const handleGenerate = async (channel: string) => {
    setGeneratingChannel(channel);
    try {
      await api.generateDraft(taskId, channel);
      queryClient.invalidateQueries({ queryKey: ["drafts", taskId] });
    } finally {
      setGeneratingChannel(null);
    }
  };

  const handleUpdate = (updated: DraftMessage) => {
    queryClient.setQueryData<DraftMessage[]>(["drafts", taskId], (old) =>
      old?.map((d) => (d.id === updated.id ? updated : d))
    );
  };

  const handleDelete = () => {
    queryClient.invalidateQueries({ queryKey: ["drafts", taskId] });
  };

  if (isLoading) {
    return (
      <div className="p-4 text-center">
        <Loader2 className="w-6 h-6 mx-auto text-zinc-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
          <FileText className="w-4 h-4 text-zinc-500" />
          Drafts
        </h3>

        <div className="flex items-center gap-2">
          <button
            onClick={() => handleGenerate("email")}
            disabled={generatingChannel !== null}
            className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded disabled:opacity-50"
          >
            {generatingChannel === "email" ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Plus className="w-3 h-3" />
            )}
            Email
          </button>
          <button
            onClick={() => handleGenerate("slack")}
            disabled={generatingChannel !== null}
            className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded disabled:opacity-50"
          >
            {generatingChannel === "slack" ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Plus className="w-3 h-3" />
            )}
            Slack
          </button>
        </div>
      </div>

      {drafts.length === 0 ? (
        <div className="p-6 text-center border border-dashed border-zinc-200 dark:border-zinc-700 rounded-lg">
          <FileText className="w-8 h-8 mx-auto text-zinc-300 dark:text-zinc-600 mb-2" />
          <p className="text-sm text-zinc-500 dark:text-zinc-400">
            No drafts yet
          </p>
          <p className="text-xs text-zinc-400 dark:text-zinc-500 mt-1">
            Generate a draft message for "{taskTitle}"
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {drafts.map((draft) => (
            <div
              key={draft.id}
              className="border border-zinc-200 dark:border-zinc-700 rounded-lg p-4"
            >
              <DraftEditor
                draft={draft}
                onUpdate={handleUpdate}
                onDelete={handleDelete}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
