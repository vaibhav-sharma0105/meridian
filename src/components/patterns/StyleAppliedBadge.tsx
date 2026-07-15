import { useState } from "react";
import { Sparkles, Eye } from "lucide-react";

interface StyleAppliedBadgeProps {
  originalText: string;
  onViewOriginal: () => void;
}

export function StyleAppliedBadge({ originalText, onViewOriginal }: StyleAppliedBadgeProps) {
  const [showOriginal, setShowOriginal] = useState(false);

  return (
    <div className="mt-2 space-y-2">
      <div className="flex items-center gap-2">
        <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/50 rounded-full">
          <Sparkles className="w-3 h-3" />
          Adjusted to match your style
        </span>
        <button
          onClick={() => {
            setShowOriginal(!showOriginal);
            if (!showOriginal) onViewOriginal();
          }}
          className="inline-flex items-center gap-1 text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 transition-colors"
        >
          <Eye className="w-3 h-3" />
          {showOriginal ? "Hide original" : "View original"}
        </button>
      </div>
      {showOriginal && (
        <div className="p-2 bg-zinc-100 dark:bg-zinc-800 rounded text-sm text-zinc-600 dark:text-zinc-400 border-l-2 border-zinc-300 dark:border-zinc-600">
          <div className="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Original</div>
          {originalText}
        </div>
      )}
    </div>
  );
}
