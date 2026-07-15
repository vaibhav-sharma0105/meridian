import { AlertTriangle, Shield, CreditCard, X } from "lucide-react";
import type { SensitiveWarning as SensitiveWarningType } from "@/lib/tauri";

interface SensitiveWarningProps {
  warnings: SensitiveWarningType[];
  onDismiss?: () => void;
}

export function SensitiveWarning({ warnings, onDismiss }: SensitiveWarningProps) {
  if (warnings.length === 0) return null;

  const hasCritical = warnings.some((w) => w.severity === "critical");

  const WarningIcon = ({ type }: { type: string }) => {
    switch (type) {
      case "credentials":
        return <Shield className="w-4 h-4" />;
      case "financial":
        return <CreditCard className="w-4 h-4" />;
      default:
        return <AlertTriangle className="w-4 h-4" />;
    }
  };

  return (
    <div
      className={`rounded-lg border p-3 mb-3 ${
        hasCritical
          ? "bg-red-50 border-red-200 dark:bg-red-950/30 dark:border-red-800"
          : "bg-amber-50 border-amber-200 dark:bg-amber-950/30 dark:border-amber-800"
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-start gap-2">
          <AlertTriangle
            className={`w-4 h-4 mt-0.5 flex-shrink-0 ${
              hasCritical ? "text-red-500" : "text-amber-500"
            }`}
          />
          <div>
            <p
              className={`text-sm font-medium ${
                hasCritical
                  ? "text-red-700 dark:text-red-300"
                  : "text-amber-700 dark:text-amber-300"
              }`}
            >
              {hasCritical
                ? "Sensitive content detected"
                : "Potential sensitive content"}
            </p>
            <ul className="mt-1 space-y-1">
              {warnings.map((warning, idx) => (
                <li
                  key={idx}
                  className="flex items-center gap-1.5 text-xs text-zinc-600 dark:text-zinc-400"
                >
                  <WarningIcon type={warning.warning_type} />
                  {warning.message}
                </li>
              ))}
            </ul>
          </div>
        </div>
        {onDismiss && (
          <button
            onClick={onDismiss}
            className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>
    </div>
  );
}
