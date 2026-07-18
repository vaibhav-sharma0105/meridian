import { useState } from "react";
import { X, ExternalLink, Loader2 } from "lucide-react";
import { useStartOAuth, useHandleOAuthCallback } from "@/hooks/useIntegrations";
import { useIntegrationStore } from "@/stores/integrationStore";
import { openUrl } from "@/lib/tauri";

interface ConnectWizardProps {
  integrationType: string;
  onClose: () => void;
}

export function ConnectWizard({ integrationType, onClose }: ConnectWizardProps) {
  const [step, setStep] = useState<"start" | "waiting" | "callback" | "done">("start");
  const [error, setError] = useState<string | null>(null);
  const startOAuth = useStartOAuth();
  const handleCallback = useHandleOAuthCallback();
  const { oauthState, startOAuth: setOAuthState, completeOAuth, failOAuth } = useIntegrationStore();

  const integrationName = {
    github: "GitHub",
    jira: "Jira",
    slack: "Slack",
  }[integrationType] || integrationType;

  const handleStart = async () => {
    setError(null);
    try {
      const redirectUri = "http://localhost:8765/oauth/callback";
      const authUrl = await startOAuth.mutateAsync({
        integrationType,
        redirectUri,
      });

      setOAuthState(integrationType, authUrl);
      setStep("waiting");

      await openUrl(authUrl);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to start OAuth flow");
    }
  };

  const handleManualCallback = () => {
    setStep("callback");
  };

  const handleSubmitCallback = async (code: string, state: string) => {
    setError(null);
    try {
      await handleCallback.mutateAsync({ code, state });
      completeOAuth();
      setStep("done");
      setTimeout(onClose, 1500);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to complete OAuth");
      failOAuth(e instanceof Error ? e.message : "Failed");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md bg-white dark:bg-zinc-900 rounded-xl shadow-xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
            Connect {integrationName}
          </h3>
          <button
            onClick={onClose}
            className="p-1 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6">
          {step === "start" && (
            <div className="space-y-4">
              <p className="text-sm text-zinc-600 dark:text-zinc-400">
                Click below to open {integrationName} and authorize Meridian to access your data.
              </p>

              {error && (
                <div className="p-3 text-sm text-red-600 bg-red-50 dark:bg-red-900/20 rounded-lg">
                  {error}
                </div>
              )}

              <button
                onClick={handleStart}
                disabled={startOAuth.isPending}
                className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-lg transition-colors disabled:opacity-50"
              >
                {startOAuth.isPending ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <ExternalLink className="w-4 h-4" />
                )}
                Connect with {integrationName}
              </button>
            </div>
          )}

          {step === "waiting" && (
            <div className="space-y-4 text-center">
              <Loader2 className="w-8 h-8 mx-auto text-indigo-500 animate-spin" />
              <p className="text-sm text-zinc-600 dark:text-zinc-400">
                Complete authorization in your browser, then return here.
              </p>
              <button
                onClick={handleManualCallback}
                className="text-sm text-indigo-600 hover:text-indigo-700"
              >
                Enter callback manually
              </button>
            </div>
          )}

          {step === "callback" && (
            <CallbackForm onSubmit={handleSubmitCallback} error={error} />
          )}

          {step === "done" && (
            <div className="text-center space-y-2">
              <div className="w-12 h-12 mx-auto bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center">
                <svg className="w-6 h-6 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <p className="font-medium text-zinc-900 dark:text-zinc-100">
                Connected successfully!
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function CallbackForm({
  onSubmit,
  error,
}: {
  onSubmit: (code: string, state: string) => void;
  error: string | null;
}) {
  const [code, setCode] = useState("");
  const [state, setState] = useState("");

  return (
    <div className="space-y-4">
      <p className="text-sm text-zinc-600 dark:text-zinc-400">
        Paste the callback URL or enter the code and state from the redirect.
      </p>

      {error && (
        <div className="p-3 text-sm text-red-600 bg-red-50 dark:bg-red-900/20 rounded-lg">
          {error}
        </div>
      )}

      <div className="space-y-3">
        <input
          type="text"
          placeholder="Authorization code"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
        />
        <input
          type="text"
          placeholder="State"
          value={state}
          onChange={(e) => setState(e.target.value)}
          className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
        />
      </div>

      <button
        onClick={() => onSubmit(code, state)}
        disabled={!code || !state}
        className="w-full px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-lg transition-colors disabled:opacity-50"
      >
        Complete Connection
      </button>
    </div>
  );
}
