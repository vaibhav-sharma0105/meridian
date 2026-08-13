import { Info } from "lucide-react";
import { useUserProfile, useUpdateRetentionSettings } from "@/hooks/useRole";

const AI_CONTEXT_OPTIONS = [
  { value: 7, label: "7 days" },
  { value: 30, label: "30 days" },
  { value: 90, label: "90 days" },
];

const RETENTION_OPTIONS = [
  { value: "90d", label: "90 days" },
  { value: "1y", label: "1 year" },
  { value: "forever", label: "Forever" },
];

export function MessageCenterSettings() {
  const { data: profile, isLoading } = useUserProfile();
  const updateRetention = useUpdateRetentionSettings();

  if (isLoading || !profile) return null;

  return (
    <div className="px-4 py-3 border-t border-zinc-200 dark:border-zinc-700 space-y-3">
      <div className="flex items-start gap-2 text-xs text-zinc-500">
        <Info className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
        <p>
          The AI only sees messages inside the context window. Message Center
          keeps them for the retention period regardless, so older content stays
          browsable here after the AI stops referencing it.
        </p>
      </div>

      <label className="flex items-center justify-between gap-3">
        <span className="text-sm text-zinc-700 dark:text-zinc-300">
          AI context window
        </span>
        <select
          value={profile.ai_context_days}
          disabled={updateRetention.isPending}
          onChange={(e) =>
            updateRetention.mutate({ aiContextDays: Number(e.target.value) })
          }
          className="px-2 py-1 text-sm bg-zinc-100 dark:bg-zinc-800 border-none rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none disabled:opacity-50"
        >
          {AI_CONTEXT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </label>

      <label className="flex items-center justify-between gap-3">
        <span className="text-sm text-zinc-700 dark:text-zinc-300">
          Keep messages for
        </span>
        <select
          value={profile.message_retention}
          disabled={updateRetention.isPending}
          onChange={(e) =>
            updateRetention.mutate({ messageRetention: e.target.value })
          }
          className="px-2 py-1 text-sm bg-zinc-100 dark:bg-zinc-800 border-none rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none disabled:opacity-50"
        >
          {RETENTION_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </label>

      {profile.message_retention !== "forever" && (
        <p className="text-xs text-amber-600 dark:text-amber-400">
          Messages older than this are moved to deleted and permanently removed
          30 days later.
        </p>
      )}

      <label className="flex items-center justify-between gap-3">
        <span className="text-sm text-zinc-700 dark:text-zinc-300">
          Archive old files
        </span>
        <input
          type="checkbox"
          checked={profile.archive_old_files}
          disabled={updateRetention.isPending}
          onChange={(e) =>
            updateRetention.mutate({ archiveOldFiles: e.target.checked })
          }
          className="w-4 h-4 accent-indigo-500 disabled:opacity-50"
        />
      </label>

      {profile.archive_old_files && (
        <label className="flex items-center justify-between gap-3">
          <span className="text-sm text-zinc-700 dark:text-zinc-300">
            Archive files older than
          </span>
          <select
            value={profile.archive_after_days}
            disabled={updateRetention.isPending}
            onChange={(e) =>
              updateRetention.mutate({ archiveAfterDays: Number(e.target.value) })
            }
            className="px-2 py-1 text-sm bg-zinc-100 dark:bg-zinc-800 border-none rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none disabled:opacity-50"
          >
            <option value={30}>30 days</option>
            <option value={90}>90 days</option>
            <option value={365}>1 year</option>
          </select>
        </label>
      )}

      <p className="text-xs text-zinc-500">
        {profile.archive_old_files
          ? "Generated files are compressed into per-day zips under created_files/archive. Nothing is deleted — files stay readable in the archive."
          : "Generated files stay where they are. Turn this on to compress older ones and reclaim space."}
      </p>
    </div>
  );
}
