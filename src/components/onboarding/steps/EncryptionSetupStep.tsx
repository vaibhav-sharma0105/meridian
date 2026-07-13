import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Shield, Lock, Laptop, Eye, EyeOff, CheckCircle } from "lucide-react";
import { getEncryptionStatus, checkPasswordStrength } from "@/lib/tauri";
import type { EncryptionStatus, PasswordStrength } from "@/lib/tauri";

interface Props {
  onNext: () => void;
  onSkip: () => void;
}

export default function EncryptionSetupStep({ onNext, onSkip }: Props) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<EncryptionStatus | null>(null);
  const [mode, setMode] = useState<"device" | "password">("device");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [strength, setStrength] = useState<PasswordStrength | null>(null);

  useEffect(() => {
    getEncryptionStatus().then(setStatus);
  }, []);

  useEffect(() => {
    if (password.length >= 4) {
      checkPasswordStrength(password).then(setStrength);
    } else {
      setStrength(null);
    }
  }, [password]);

  const passwordsMatch = password === confirmPassword && password.length > 0;
  const canProceed = mode === "device" || (mode === "password" && passwordsMatch && strength && strength.score >= 3);

  const getStrengthColor = () => {
    if (!strength) return "bg-zinc-200 dark:bg-zinc-700";
    switch (strength.strength) {
      case "weak": return "bg-red-500";
      case "fair": return "bg-yellow-500";
      case "good": return "bg-blue-500";
      case "strong": return "bg-green-500";
    }
  };

  const getStrengthWidth = () => {
    if (!strength) return "0%";
    return `${(strength.score / 7) * 100}%`;
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 max-w-lg mx-auto w-full">
      <Shield className="w-12 h-12 text-indigo-500 mb-4" />
      <h2 className="text-2xl font-bold text-zinc-900 dark:text-zinc-50 mb-1">
        Data Security
      </h2>
      <p className="text-zinc-500 text-sm mb-6 text-center">
        Your data is encrypted at rest. Choose how to protect it.
      </p>

      {status?.initialized && (
        <div className="w-full mb-6 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg flex items-center gap-2">
          <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />
          <span className="text-sm text-green-700 dark:text-green-300">
            Database already encrypted ({status.mode} mode)
          </span>
        </div>
      )}

      <div className="w-full space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <button
            onClick={() => setMode("device")}
            className={`p-4 rounded-lg border-2 text-left transition-colors ${
              mode === "device"
                ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
            }`}
          >
            <Laptop className={`w-6 h-6 mb-2 ${mode === "device" ? "text-indigo-500" : "text-zinc-400"}`} />
            <div className="font-medium text-zinc-900 dark:text-zinc-50 text-sm">Device Key</div>
            <div className="text-xs text-zinc-500 mt-1">
              Automatic encryption tied to this computer. No password needed.
            </div>
          </button>

          <button
            onClick={() => setMode("password")}
            className={`p-4 rounded-lg border-2 text-left transition-colors ${
              mode === "password"
                ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
            }`}
          >
            <Lock className={`w-6 h-6 mb-2 ${mode === "password" ? "text-indigo-500" : "text-zinc-400"}`} />
            <div className="font-medium text-zinc-900 dark:text-zinc-50 text-sm">Password</div>
            <div className="text-xs text-zinc-500 mt-1">
              Portable across machines. Requires password to unlock.
            </div>
          </button>
        </div>

        {mode === "password" && (
          <div className="space-y-3 pt-2">
            <div>
              <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                Encryption Password
              </label>
              <div className="relative">
                <input
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter a strong password"
                  className="w-full px-3 py-2 pr-10 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-sm"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600"
                >
                  {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
              {strength && (
                <div className="mt-2">
                  <div className="h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                    <div
                      className={`h-full transition-all duration-300 ${getStrengthColor()}`}
                      style={{ width: getStrengthWidth() }}
                    />
                  </div>
                  <div className="flex justify-between mt-1">
                    <span className={`text-xs ${
                      strength.strength === "weak" ? "text-red-500" :
                      strength.strength === "fair" ? "text-yellow-600" :
                      strength.strength === "good" ? "text-blue-500" : "text-green-500"
                    }`}>
                      {strength.label}
                    </span>
                  </div>
                  {strength.suggestions.length > 0 && strength.score < 5 && (
                    <ul className="mt-2 text-xs text-zinc-500 space-y-0.5">
                      {strength.suggestions.slice(0, 2).map((s, i) => (
                        <li key={i}>• {s}</li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                Confirm Password
              </label>
              <input
                type={showPassword ? "text" : "password"}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm your password"
                className={`w-full px-3 py-2 rounded-lg border bg-white dark:bg-zinc-800 text-sm ${
                  confirmPassword && !passwordsMatch
                    ? "border-red-300 dark:border-red-700"
                    : "border-zinc-200 dark:border-zinc-700"
                }`}
              />
              {confirmPassword && !passwordsMatch && (
                <p className="mt-1 text-xs text-red-500">Passwords do not match</p>
              )}
            </div>
          </div>
        )}

        <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
          <p className="text-xs text-amber-700 dark:text-amber-300">
            {mode === "device" ? (
              <>
                <strong>Note:</strong> Device-based encryption is tied to this computer.
                If you move to a new machine, you'll need to export and re-import your data.
              </>
            ) : (
              <>
                <strong>Important:</strong> If you forget your password, your data cannot be recovered.
                There is no password reset. Store your password securely.
              </>
            )}
          </p>
        </div>

        <div className="flex gap-3 pt-2">
          <button
            onClick={onSkip}
            className="flex-1 px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {status?.initialized ? "Keep Current" : "Skip"}
          </button>
          <button
            onClick={onNext}
            disabled={!canProceed}
            className="flex-1 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {status?.initialized ? "Continue" : "Enable Encryption"}
          </button>
        </div>
      </div>
    </div>
  );
}
