import { useState } from "react";
import { AlertTriangle, X, RefreshCw } from "lucide-react";
import { useDismissDriftAlert, useChangeRole } from "@/hooks/useRole";
import { ROLE_LABELS } from "@/hooks/useRole";

interface RoleDriftAlertProps {
  currentRole: string;
  inferredRole: string;
  confidence: number;
  onClose: () => void;
}

export function RoleDriftAlert({
  currentRole,
  inferredRole,
  confidence,
  onClose,
}: RoleDriftAlertProps) {
  const dismissDrift = useDismissDriftAlert();
  const changeRole = useChangeRole();

  const handleDismiss = () => {
    dismissDrift.mutate(undefined, { onSuccess: onClose });
  };

  const handleChangeRole = () => {
    changeRole.mutate({ role: inferredRole }, { onSuccess: onClose });
  };

  const isLoading = dismissDrift.isPending || changeRole.isPending;

  return (
    <div className="fixed bottom-4 right-4 z-50 w-96 bg-white dark:bg-zinc-900 border border-amber-200 dark:border-amber-800 rounded-xl shadow-lg animate-slide-up">
      <div className="p-4">
        <div className="flex items-start gap-3">
          <div className="w-10 h-10 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center flex-shrink-0">
            <AlertTriangle className="w-5 h-5 text-amber-600 dark:text-amber-400" />
          </div>

          <div className="flex-1">
            <div className="flex items-center justify-between">
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Role Change Detected
              </h3>
              <button
                onClick={handleDismiss}
                disabled={isLoading}
                className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
              >
                <X className="w-4 h-4 text-zinc-400" />
              </button>
            </div>

            <p className="text-sm text-zinc-600 dark:text-zinc-400 mt-1">
              Your recent activity suggests you may now be a{" "}
              <span className="font-medium text-zinc-900 dark:text-zinc-100">
                {ROLE_LABELS[inferredRole]}
              </span>{" "}
              ({Math.round(confidence * 100)}% confidence).
            </p>

            <div className="flex items-center gap-2 mt-3">
              <div className="flex-1 text-xs text-zinc-500">
                Current: {ROLE_LABELS[currentRole]}
              </div>
              <RefreshCw className="w-3 h-3 text-zinc-400" />
              <div className="flex-1 text-xs text-zinc-500 text-right">
                Detected: {ROLE_LABELS[inferredRole]}
              </div>
            </div>

            <div className="flex items-center gap-2 mt-4">
              <button
                onClick={handleDismiss}
                disabled={isLoading}
                className="flex-1 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors disabled:opacity-50"
              >
                Keep Current
              </button>
              <button
                onClick={handleChangeRole}
                disabled={isLoading}
                className="flex-1 px-3 py-1.5 text-sm bg-amber-500 hover:bg-amber-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
              >
                Update Role
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
