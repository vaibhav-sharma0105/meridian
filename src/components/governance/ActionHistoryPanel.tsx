import { useState } from "react";
import { History, Undo2, ChevronDown, Filter } from "lucide-react";
import { useActionHistory, useUndoAction } from "@/hooks/useGovernance";
import type { ActionHistory } from "@/lib/tauri";

interface ActionHistoryPanelProps {
  entityType?: string;
  entityId?: string;
  limit?: number;
  className?: string;
}

const ENTITY_TYPES = [
  { value: undefined, label: "All types" },
  { value: "task", label: "Tasks" },
  { value: "meeting", label: "Meetings" },
  { value: "project", label: "Projects" },
  { value: "skill", label: "Skills" },
];

const ACTION_TYPES = [
  { value: undefined, label: "All actions" },
  { value: "create", label: "Create" },
  { value: "update", label: "Update" },
  { value: "delete", label: "Delete" },
];

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

interface ActionItemProps {
  action: ActionHistory;
  onUndo: () => void;
  isUndoing: boolean;
}

function ActionItem({ action, onUndo, isUndoing }: ActionItemProps) {
  const [showDiff, setShowDiff] = useState(false);

  const canUndo = action.undoable && !action.undo_action_id;

  let beforeState: Record<string, unknown> | null = null;
  let afterState: Record<string, unknown> | null = null;

  try {
    if (action.before_state) beforeState = JSON.parse(action.before_state);
    if (action.after_state) afterState = JSON.parse(action.after_state);
  } catch {
    // ignore
  }

  return (
    <div className="border-b border-zinc-100 dark:border-zinc-800 last:border-0 py-2.5">
      <div className="flex items-center gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
              {action.action_type.replace(/_/g, " ")}
            </span>
            <span className="text-xs text-zinc-400">
              {action.entity_type}
            </span>
            {action.undo_action_id && (
              <span className="px-1.5 py-0.5 text-[10px] bg-zinc-100 dark:bg-zinc-800 text-zinc-500 rounded">
                Undone
              </span>
            )}
          </div>
          <div className="text-xs text-zinc-500 mt-0.5">
            {formatRelativeTime(action.created_at)} · {action.entity_id.slice(0, 8)}
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          {(beforeState || afterState) && (
            <button
              onClick={() => setShowDiff(!showDiff)}
              className="p-1.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 transition-colors"
              title="View changes"
            >
              <ChevronDown
                className={`w-4 h-4 transition-transform ${showDiff ? "rotate-180" : ""}`}
              />
            </button>
          )}
          {canUndo && (
            <button
              onClick={onUndo}
              disabled={isUndoing}
              className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded transition-colors disabled:opacity-50"
            >
              <Undo2 className="w-3 h-3" />
              Undo
            </button>
          )}
        </div>
      </div>

      {showDiff && (beforeState || afterState) && (
        <div className="mt-2 grid grid-cols-2 gap-2 text-xs">
          <div>
            <div className="text-zinc-500 mb-1">Before</div>
            <pre className="p-2 bg-red-50 dark:bg-red-900/20 rounded overflow-x-auto text-red-700 dark:text-red-300">
              {beforeState ? JSON.stringify(beforeState, null, 2) : "—"}
            </pre>
          </div>
          <div>
            <div className="text-zinc-500 mb-1">After</div>
            <pre className="p-2 bg-green-50 dark:bg-green-900/20 rounded overflow-x-auto text-green-700 dark:text-green-300">
              {afterState ? JSON.stringify(afterState, null, 2) : "—"}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}

export function ActionHistoryPanel({
  entityType: initialEntityType,
  entityId: initialEntityId,
  limit = 50,
  className,
}: ActionHistoryPanelProps) {
  const [entityType, setEntityType] = useState<string | undefined>(initialEntityType);
  const [showFilters, setShowFilters] = useState(false);

  const { data: actions = [], isLoading } = useActionHistory(
    entityType,
    initialEntityId,
    limit
  );
  const undoAction = useUndoAction();

  const handleUndo = (actionId: string) => {
    undoAction.mutate(actionId);
  };

  return (
    <div className={className}>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <History className="w-4 h-4 text-zinc-500" />
          <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
            Action History
          </h3>
        </div>
        <button
          onClick={() => setShowFilters(!showFilters)}
          className={`p-1.5 rounded hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors ${
            showFilters || entityType ? "text-indigo-500" : "text-zinc-400"
          }`}
        >
          <Filter className="w-4 h-4" />
        </button>
      </div>

      {showFilters && (
        <div className="flex gap-2 mb-3">
          <select
            value={entityType || ""}
            onChange={(e) => setEntityType(e.target.value || undefined)}
            className="flex-1 px-2 py-1.5 text-sm border border-zinc-200 dark:border-zinc-700 rounded bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
          >
            {ENTITY_TYPES.map((type) => (
              <option key={type.value || "all"} value={type.value || ""}>
                {type.label}
              </option>
            ))}
          </select>
        </div>
      )}

      {isLoading ? (
        <div className="text-sm text-zinc-500 py-4 text-center">
          Loading history...
        </div>
      ) : actions.length === 0 ? (
        <div className="text-sm text-zinc-500 py-8 text-center">
          No actions recorded yet
        </div>
      ) : (
        <div className="divide-y divide-zinc-100 dark:divide-zinc-800">
          {actions.map((action) => (
            <ActionItem
              key={action.id}
              action={action}
              onUndo={() => handleUndo(action.id)}
              isUndoing={undoAction.isPending}
            />
          ))}
        </div>
      )}
    </div>
  );
}
