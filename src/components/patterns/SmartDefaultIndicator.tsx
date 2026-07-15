import { Sparkles, X } from "lucide-react";

interface SmartDefaultIndicatorProps {
  label: string;
  onClear: () => void;
}

export function SmartDefaultIndicator({ label, onClear }: SmartDefaultIndicatorProps) {
  return (
    <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/50 rounded">
      <Sparkles className="w-3 h-3" />
      {label}
      <button
        type="button"
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onClear();
        }}
        className="ml-0.5 p-0.5 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 rounded transition-colors"
        title="Clear suggestion"
      >
        <X className="w-2.5 h-2.5" />
      </button>
    </span>
  );
}

interface SmartDefaultBadgeProps {
  source: string;
}

export function SmartDefaultBadge({ source }: SmartDefaultBadgeProps) {
  const label = source === "keyword" ? "Based on keywords" : source === "project" ? "Project default" : "Suggested";

  return (
    <span className="inline-flex items-center gap-1 text-[10px] text-zinc-400 dark:text-zinc-500">
      <Sparkles className="w-3 h-3 text-indigo-400" />
      {label}
    </span>
  );
}
