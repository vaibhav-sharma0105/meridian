import { useState, useEffect } from "react";
import {
  Shield, Lock, Laptop, AlertTriangle, CheckCircle,
  Loader, Database, HardDrive, Eye, EyeOff, ArrowRight
} from "lucide-react";
import {
  getMigrationStatus, migrateDatabase, checkPasswordStrength,
  type MigrationStatus, type MigrationResult, type PasswordStrength
} from "@/lib/tauri";

interface Props {
  open: boolean;
  onClose: () => void;
  onComplete: () => void;
}

type Step = "check" | "choose" | "password" | "migrating" | "complete" | "error";

export default function MigrationWizard({ open, onClose, onComplete }: Props) {
  const [step, setStep] = useState<Step>("check");
  const [status, setStatus] = useState<MigrationStatus | null>(null);
  const [mode, setMode] = useState<"device" | "password">("device");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [strength, setStrength] = useState<PasswordStrength | null>(null);
  const [result, setResult] = useState<MigrationResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      checkStatus();
    }
  }, [open]);

  useEffect(() => {
    if (password.length >= 4) {
      checkPasswordStrength(password).then(setStrength);
    } else {
      setStrength(null);
    }
  }, [password]);

  const checkStatus = async () => {
    try {
      const s = await getMigrationStatus();
      setStatus(s);
      if (s.needs_migration) {
        setStep("choose");
      } else {
        setStep("complete");
      }
    } catch (e) {
      setError(String(e));
      setStep("error");
    }
  };

  const handleMigrate = async () => {
    setStep("migrating");
    setError(null);

    try {
      const pwd = mode === "password" ? password : undefined;
      const res = await migrateDatabase(pwd);
      setResult(res);

      if (res.success) {
        setStep("complete");
      } else {
        setError(res.error || "Migration failed");
        setStep("error");
      }
    } catch (e) {
      setError(String(e));
      setStep("error");
    }
  };

  const passwordsMatch = password === confirmPassword && password.length > 0;
  const canProceed = mode === "device" || (mode === "password" && passwordsMatch && strength && strength.score >= 3);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-full max-w-md mx-4 border border-zinc-200 dark:border-zinc-700">
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-zinc-100 dark:border-zinc-800">
          <Shield className="w-5 h-5 text-indigo-500" />
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">
            Database Encryption
          </h2>
        </div>

        {/* Content */}
        <div className="px-6 py-5">
          {step === "check" && (
            <div className="flex flex-col items-center py-8">
              <Loader className="w-8 h-8 animate-spin text-indigo-500 mb-4" />
              <p className="text-sm text-zinc-500">Checking database status...</p>
            </div>
          )}

          {step === "choose" && status && (
            <div className="space-y-4">
              <div className="flex items-start gap-3 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
                <AlertTriangle className="w-5 h-5 text-amber-500 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-amber-700 dark:text-amber-300">
                    Unencrypted Database Detected
                  </p>
                  <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                    Your database ({status.database_size_mb.toFixed(1)} MB) is not encrypted.
                    We recommend encrypting it to protect your data.
                  </p>
                </div>
              </div>

              <p className="text-sm text-zinc-600 dark:text-zinc-400">
                Choose how to protect your database:
              </p>

              <div className="grid grid-cols-2 gap-3">
                <button
                  onClick={() => { setMode("device"); setStep("password"); }}
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    mode === "device"
                      ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                      : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300"
                  }`}
                >
                  <Laptop className="w-6 h-6 mb-2 text-indigo-500" />
                  <div className="font-medium text-zinc-900 dark:text-zinc-50 text-sm">Device Key</div>
                  <div className="text-xs text-zinc-500 mt-1">
                    Automatic. Tied to this computer.
                  </div>
                </button>

                <button
                  onClick={() => { setMode("password"); setStep("password"); }}
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    mode === "password"
                      ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20"
                      : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300"
                  }`}
                >
                  <Lock className="w-6 h-6 mb-2 text-indigo-500" />
                  <div className="font-medium text-zinc-900 dark:text-zinc-50 text-sm">Password</div>
                  <div className="text-xs text-zinc-500 mt-1">
                    Portable. You set the password.
                  </div>
                </button>
              </div>
            </div>
          )}

          {step === "password" && (
            <div className="space-y-4">
              {mode === "device" ? (
                <>
                  <div className="flex items-center gap-3 p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
                    <Laptop className="w-8 h-8 text-indigo-500" />
                    <div>
                      <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50">
                        Device-Based Encryption
                      </p>
                      <p className="text-xs text-zinc-500 mt-0.5">
                        Your database will be encrypted using a key derived from this computer's identity.
                      </p>
                    </div>
                  </div>

                  <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
                    <p className="text-xs text-amber-700 dark:text-amber-300">
                      <strong>Note:</strong> If you move to a different computer, you'll need to export your data first.
                    </p>
                  </div>
                </>
              ) : (
                <>
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
                        className="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-400"
                      >
                        {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                    </div>
                    {strength && (
                      <div className="mt-2">
                        <div className="h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                          <div
                            className={`h-full transition-all ${
                              strength.strength === "weak" ? "bg-red-500" :
                              strength.strength === "fair" ? "bg-yellow-500" :
                              strength.strength === "good" ? "bg-blue-500" : "bg-green-500"
                            }`}
                            style={{ width: `${(strength.score / 7) * 100}%` }}
                          />
                        </div>
                        <span className={`text-xs mt-1 ${
                          strength.strength === "weak" ? "text-red-500" :
                          strength.strength === "fair" ? "text-yellow-600" :
                          strength.strength === "good" ? "text-blue-500" : "text-green-500"
                        }`}>
                          {strength.label}
                        </span>
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

                  <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                    <p className="text-xs text-red-700 dark:text-red-300">
                      <strong>Warning:</strong> If you forget your password, your data cannot be recovered.
                      There is no password reset.
                    </p>
                  </div>
                </>
              )}

              <div className="flex items-center gap-2 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
                <HardDrive className="w-4 h-4 text-zinc-400" />
                <span className="text-xs text-zinc-500">
                  A backup will be created before migration
                </span>
              </div>
            </div>
          )}

          {step === "migrating" && (
            <div className="flex flex-col items-center py-8">
              <div className="relative">
                <Database className="w-12 h-12 text-zinc-300 dark:text-zinc-600" />
                <Loader className="w-6 h-6 animate-spin text-indigo-500 absolute -right-1 -bottom-1" />
              </div>
              <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50 mt-4">
                Encrypting Database...
              </p>
              <p className="text-xs text-zinc-500 mt-1">
                This may take a moment. Please don't close the app.
              </p>
            </div>
          )}

          {step === "complete" && (
            <div className="flex flex-col items-center py-6">
              <div className="w-12 h-12 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center mb-4">
                <CheckCircle className="w-6 h-6 text-green-600 dark:text-green-400" />
              </div>
              <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50">
                {result ? "Migration Complete" : "Database Already Encrypted"}
              </p>
              {result && (
                <p className="text-xs text-zinc-500 mt-1">
                  {result.tables_migrated} tables migrated successfully
                </p>
              )}
              <p className="text-xs text-zinc-400 mt-3 text-center">
                Your data is now protected with AES-256 encryption.
              </p>
              {result && result.safe_backup_path && (
                <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg w-full">
                  <p className="text-xs text-blue-700 dark:text-blue-300 font-medium">
                    Safe backup location:
                  </p>
                  <p className="text-xs text-blue-600 dark:text-blue-400 mt-1 font-mono break-all">
                    {result.safe_backup_path}
                  </p>
                  <p className="text-xs text-blue-500 dark:text-blue-400 mt-2">
                    This backup is stored outside the app data folder for extra safety.
                  </p>
                </div>
              )}
            </div>
          )}

          {step === "error" && (
            <div className="space-y-4">
              <div className="flex items-start gap-3 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                <AlertTriangle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-red-700 dark:text-red-300">
                    Migration Failed
                  </p>
                  <p className="text-xs text-red-600 dark:text-red-400 mt-1">
                    {error}
                  </p>
                </div>
              </div>
              <p className="text-xs text-zinc-500">
                Your original database has not been modified.
              </p>
              {result && result.safe_backup_path && (
                <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <p className="text-xs text-green-700 dark:text-green-300 font-medium">
                    Your data is safely backed up at:
                  </p>
                  <p className="text-xs text-green-600 dark:text-green-400 mt-1 font-mono break-all">
                    {result.safe_backup_path}
                  </p>
                  <p className="text-xs text-green-500 dark:text-green-400 mt-2">
                    This backup is stored outside the app data folder and will survive even if ~/.meridian is deleted.
                  </p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-zinc-100 dark:border-zinc-800">
          {(step === "choose" || step === "error") && (
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg"
            >
              Later
            </button>
          )}

          {step === "password" && (
            <>
              <button
                onClick={() => setStep("choose")}
                className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg"
              >
                Back
              </button>
              <button
                onClick={handleMigrate}
                disabled={!canProceed}
                className="flex items-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
              >
                Encrypt Now
                <ArrowRight className="w-4 h-4" />
              </button>
            </>
          )}

          {step === "complete" && (
            <button
              onClick={() => { onComplete(); onClose(); }}
              className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg text-sm font-medium"
            >
              Done
            </button>
          )}

          {step === "error" && (
            <button
              onClick={() => setStep("choose")}
              className="px-4 py-2 bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 rounded-lg text-sm font-medium"
            >
              Try Again
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
