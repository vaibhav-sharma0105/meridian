import { Search, X, Filter } from "lucide-react";
import type { MessageFilters as MessageFiltersType } from "@/lib/tauri";

interface MessageFiltersProps {
  filters: MessageFiltersType;
  onChange: (filters: MessageFiltersType) => void;
}

// Must stay in sync with the Rust `MessageType` enum. Offering a filter for a
// type nothing can produce is worse than omitting it — it always returns empty
// and reads as a bug.
const MESSAGE_TYPES = [
  { value: "", label: "All Types" },
  { value: "skill_result", label: "Skill Results" },
  { value: "integration_sync", label: "Integration Syncs" },
  { value: "pinned_chat", label: "Pinned Chats" },
  { value: "digest", label: "Digests" },
];

export function MessageFiltersBar({ filters, onChange }: MessageFiltersProps) {
  const hasActiveFilters = filters.message_type || filters.search || filters.include_deleted;

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-zinc-200 dark:border-zinc-700">
      <div className="relative flex-1">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
        <input
          type="text"
          placeholder="Search messages..."
          value={filters.search || ""}
          onChange={(e) =>
            onChange({ ...filters, search: e.target.value || undefined })
          }
          className="w-full pl-8 pr-3 py-1.5 text-sm bg-zinc-100 dark:bg-zinc-800 border-none rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none"
        />
        {filters.search && (
          <button
            onClick={() => onChange({ ...filters, search: undefined })}
            className="absolute right-2.5 top-1/2 -translate-y-1/2"
          >
            <X className="w-4 h-4 text-zinc-400 hover:text-zinc-600" />
          </button>
        )}
      </div>

      <select
        aria-label="Filter by message type"
        value={filters.message_type || ""}
        onChange={(e) =>
          onChange({ ...filters, message_type: e.target.value || undefined })
        }
        className="px-3 py-1.5 text-sm bg-zinc-100 dark:bg-zinc-800 border-none rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none"
      >
        {MESSAGE_TYPES.map((type) => (
          <option key={type.value} value={type.value}>
            {type.label}
          </option>
        ))}
      </select>

      <label className="flex items-center gap-2 text-sm text-zinc-500">
        <input
          type="checkbox"
          checked={filters.include_deleted || false}
          onChange={(e) =>
            onChange({ ...filters, include_deleted: e.target.checked || undefined })
          }
          className="rounded border-zinc-300 text-indigo-500 focus:ring-indigo-500"
        />
        Deleted
      </label>

      {hasActiveFilters && (
        <button
          onClick={() => onChange({})}
          className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
        >
          <X className="w-3 h-3" />
          Clear
        </button>
      )}
    </div>
  );
}
