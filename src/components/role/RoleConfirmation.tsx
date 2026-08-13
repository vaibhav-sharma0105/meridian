import { useState } from "react";
import { X, User, Check, ChevronRight } from "lucide-react";
import { useConfirmRole, useChangeRole, useRoleScores } from "@/hooks/useRole";
import { ROLE_LABELS, ROLE_DESCRIPTIONS } from "@/hooks/useRole";

interface RoleConfirmationProps {
  inferredRole: string;
  confidence: number;
  onClose: () => void;
}

const ROLES = ["tech_lead", "ic", "pm", "manager"] as const;

export function RoleConfirmation({
  inferredRole,
  confidence,
  onClose,
}: RoleConfirmationProps) {
  const [selectedRole, setSelectedRole] = useState(inferredRole);
  const [customDescription, setCustomDescription] = useState("");
  const [step, setStep] = useState<"confirm" | "customize">("confirm");

  const { data: scores } = useRoleScores();
  const confirmRole = useConfirmRole();
  const changeRole = useChangeRole();

  const handleConfirm = () => {
    if (selectedRole === inferredRole) {
      confirmRole.mutate(
        { role: selectedRole, customDescription: customDescription || undefined },
        { onSuccess: onClose }
      );
    } else {
      changeRole.mutate(
        { role: selectedRole, customDescription: customDescription || undefined },
        { onSuccess: onClose }
      );
    }
  };

  const isLoading = confirmRole.isPending || changeRole.isPending;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
              <User className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                Confirm Your Role
              </h2>
              <p className="text-sm text-zinc-500">
                This helps personalize your experience
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-zinc-400" />
          </button>
        </div>

        <div className="p-6">
          {step === "confirm" ? (
            <>
              <div className="mb-6">
                <p className="text-sm text-zinc-600 dark:text-zinc-400 mb-4">
                  Based on your activity, we think you're a{" "}
                  <span className="font-semibold text-zinc-900 dark:text-zinc-100">
                    {ROLE_LABELS[inferredRole]}
                  </span>{" "}
                  ({Math.round(confidence * 100)}% confidence).
                </p>

                <div className="space-y-2">
                  {ROLES.map((role) => {
                    const score = scores?.[role] ?? 0;
                    const isSelected = selectedRole === role;
                    const isInferred = role === inferredRole;

                    return (
                      <button
                        key={role}
                        onClick={() => setSelectedRole(role)}
                        className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                          isSelected
                            ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                            : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
                        }`}
                      >
                        <div
                          className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                            isSelected
                              ? "border-indigo-500 bg-indigo-500"
                              : "border-zinc-300 dark:border-zinc-600"
                          }`}
                        >
                          {isSelected && (
                            <Check className="w-3 h-3 text-white" />
                          )}
                        </div>

                        <div className="flex-1 text-left">
                          <div className="flex items-center gap-2">
                            <span className="font-medium text-zinc-900 dark:text-zinc-100">
                              {ROLE_LABELS[role]}
                            </span>
                            {isInferred && (
                              <span className="text-xs bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 px-1.5 py-0.5 rounded">
                                Suggested
                              </span>
                            )}
                          </div>
                          <p className="text-xs text-zinc-500 mt-0.5">
                            {ROLE_DESCRIPTIONS[role]}
                          </p>
                        </div>

                        {scores && (
                          <div className="text-right">
                            <span className="text-sm font-medium text-zinc-600 dark:text-zinc-400">
                              {Math.round(score * 100)}%
                            </span>
                          </div>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>

              <div className="flex items-center justify-between">
                <button
                  onClick={() => setStep("customize")}
                  className="text-sm text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 flex items-center gap-1"
                >
                  Add custom description
                  <ChevronRight className="w-4 h-4" />
                </button>

                <button
                  onClick={handleConfirm}
                  disabled={isLoading}
                  className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
                >
                  {isLoading ? "Confirming..." : "Confirm Role"}
                </button>
              </div>
            </>
          ) : (
            <>
              <div className="mb-6">
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  Custom Role Description (Optional)
                </label>
                <textarea
                  value={customDescription}
                  onChange={(e) => setCustomDescription(e.target.value)}
                  placeholder="e.g., Hybrid role: 60% hands-on coding, 40% architecture reviews"
                  className="w-full px-3 py-2 text-sm border border-zinc-200 dark:border-zinc-700 rounded-lg bg-white dark:bg-zinc-800 focus:ring-2 focus:ring-indigo-500 focus:outline-none resize-none"
                  rows={3}
                />
                <p className="text-xs text-zinc-500 mt-1">
                  Help the AI better understand your specific responsibilities
                </p>
              </div>

              <div className="flex items-center justify-between">
                <button
                  onClick={() => setStep("confirm")}
                  className="text-sm text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                >
                  ← Back
                </button>

                <button
                  onClick={handleConfirm}
                  disabled={isLoading}
                  className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
                >
                  {isLoading ? "Confirming..." : "Confirm Role"}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
