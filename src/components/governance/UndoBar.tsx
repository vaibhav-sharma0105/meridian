import { useState, useEffect, useCallback } from "react";
import { Undo2, X } from "lucide-react";
import { useUndoAction } from "@/hooks/useGovernance";
import type { ActionHistory } from "@/lib/tauri";

interface UndoItem {
  action: ActionHistory;
  expiresAt: number;
}

const UNDO_DURATION_MS = 10000;

export function useUndoBar() {
  const [items, setItems] = useState<UndoItem[]>([]);

  const addUndo = useCallback((action: ActionHistory) => {
    if (!action.undoable) return;

    setItems((prev) => [
      ...prev.filter((i) => i.action.id !== action.id),
      { action, expiresAt: Date.now() + UNDO_DURATION_MS },
    ]);
  }, []);

  const removeUndo = useCallback((actionId: string) => {
    setItems((prev) => prev.filter((i) => i.action.id !== actionId));
  }, []);

  const clearAll = useCallback(() => {
    setItems([]);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      setItems((prev) => prev.filter((i) => i.expiresAt > now));
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return { items, addUndo, removeUndo, clearAll };
}

interface UndoBarItemProps {
  item: UndoItem;
  onUndo: () => void;
  onDismiss: () => void;
}

function UndoBarItem({ item, onUndo, onDismiss }: UndoBarItemProps) {
  const [progress, setProgress] = useState(100);

  useEffect(() => {
    const startTime = Date.now();
    const duration = item.expiresAt - startTime;

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, 100 - (elapsed / duration) * 100);
      setProgress(remaining);

      if (remaining > 0) {
        requestAnimationFrame(animate);
      }
    };

    const frame = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(frame);
  }, [item.expiresAt]);

  const formatAction = (action: ActionHistory) => {
    const type = action.action_type.replace(/_/g, " ");
    const entity = action.entity_type;
    return `${type} ${entity}`;
  };

  return (
    <div className="relative bg-zinc-900 text-white rounded-lg shadow-lg overflow-hidden">
      <div
        className="absolute bottom-0 left-0 h-1 bg-indigo-500 transition-none"
        style={{ width: `${progress}%` }}
      />
      <div className="flex items-center gap-3 px-4 py-3">
        <Undo2 className="w-4 h-4 text-zinc-400" />
        <span className="flex-1 text-sm">{formatAction(item.action)}</span>
        <button
          onClick={onUndo}
          className="px-3 py-1 text-sm font-medium bg-indigo-500 hover:bg-indigo-600 rounded transition-colors"
        >
          Undo
        </button>
        <button
          onClick={onDismiss}
          className="p-1 hover:bg-zinc-800 rounded transition-colors"
        >
          <X className="w-4 h-4 text-zinc-400" />
        </button>
      </div>
    </div>
  );
}

interface UndoBarProps {
  items: UndoItem[];
  onUndo: (actionId: string) => void;
  onDismiss: (actionId: string) => void;
}

export function UndoBar({ items, onUndo, onDismiss }: UndoBarProps) {
  const undoAction = useUndoAction();

  const handleUndo = async (actionId: string) => {
    await undoAction.mutateAsync(actionId);
    onUndo(actionId);
  };

  if (items.length === 0) return null;

  return (
    <div className="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 max-w-md w-full px-4">
      {items.map((item) => (
        <UndoBarItem
          key={item.action.id}
          item={item}
          onUndo={() => handleUndo(item.action.id)}
          onDismiss={() => onDismiss(item.action.id)}
        />
      ))}
    </div>
  );
}
