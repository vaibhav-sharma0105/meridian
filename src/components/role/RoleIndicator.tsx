import { useState } from "react";
import { User, ChevronDown, HelpCircle } from "lucide-react";
import { useInferenceStatus } from "@/hooks/useRole";
import { ROLE_LABELS, ROLE_DESCRIPTIONS } from "@/hooks/useRole";

interface RoleIndicatorProps {
  onClick?: () => void;
}

export function RoleIndicator({ onClick }: RoleIndicatorProps) {
  const { data: status, isLoading } = useInferenceStatus();
  const [showTooltip, setShowTooltip] = useState(false);

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-100 dark:bg-zinc-800 rounded-lg animate-pulse">
        <div className="w-4 h-4 bg-zinc-200 dark:bg-zinc-700 rounded" />
        <div className="w-16 h-4 bg-zinc-200 dark:bg-zinc-700 rounded" />
      </div>
    );
  }

  if (!status) return null;

  const isLearning = status.type === "Learning";
  const isPending = status.type === "PendingConfirmation";
  const isConfirmed = status.type === "Confirmed";

  const getRole = () => {
    if (isLearning) return null;
    if (isPending) return status.inferred;
    if (isConfirmed) return status.role;
    return null;
  };

  const role = getRole();
  const roleLabel = role ? ROLE_LABELS[role] || role : null;
  const roleDescription = role ? ROLE_DESCRIPTIONS[role] : null;

  return (
    <div className="relative">
      <button
        onClick={onClick}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
        className={`flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors ${
          isLearning
            ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-500"
            : isPending
            ? "bg-amber-50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400 border border-amber-200 dark:border-amber-800"
            : "bg-indigo-50 dark:bg-indigo-900/20 text-indigo-600 dark:text-indigo-400"
        } hover:opacity-80`}
      >
        <User className="w-4 h-4" />
        {isLearning ? (
          <span className="text-sm">Learning...</span>
        ) : (
          <>
            <span className="text-sm font-medium">{roleLabel}</span>
            {isPending && (
              <span className="text-xs bg-amber-200 dark:bg-amber-800 px-1.5 py-0.5 rounded">
                Confirm?
              </span>
            )}
          </>
        )}
        <ChevronDown className="w-3 h-3" />
      </button>

      {showTooltip && roleDescription && (
        <div className="absolute top-full left-0 mt-2 z-50 w-64 p-3 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg">
          <div className="flex items-start gap-2">
            <HelpCircle className="w-4 h-4 text-zinc-400 mt-0.5 flex-shrink-0" />
            <div>
              <p className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                {roleLabel}
              </p>
              <p className="text-xs text-zinc-500">{roleDescription}</p>
              {isLearning && status.type === "Learning" && (
                <div className="mt-2">
                  <div className="h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-indigo-500 rounded-full transition-all"
                      style={{ width: `${status.progress}%` }}
                    />
                  </div>
                  <p className="text-xs text-zinc-400 mt-1">{status.message}</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
