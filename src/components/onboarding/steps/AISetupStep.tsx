import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle, XCircle, Loader, ChevronDown } from "lucide-react";
import { verifyAiConnection, fetchAvailableModels, saveAiSettings, checkOllamaStatus } from "@/lib/tauri";
import { AI_PROVIDERS } from "@/lib/constants";
import type { ModelInfo } from "@/lib/tauri";

interface Props {
  onNext: () => void;
  onSkip: () => void;
}

export default function AISetupStep({ onNext, onSkip }: Props) {
  const { t } = useTranslation();
  const [provider, setProvider] = useState("openai");
  const [baseUrl, setBaseUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [modelId, setModelId] = useState("");
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [verifyState, setVerifyState] = useState<"idle" | "loading" | "ok" | "error">("idle");
  const [verifyError, setVerifyError] = useState("");
  const [useOllama, setUseOllama] = useState(false);
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [ollamaStatus, setOllamaStatus] = useState<"idle" | "running" | "not_running">("idle");
  const [saving, setSaving] = useState(false);

  const selectedProvider = AI_PROVIDERS.find((p) => p.value === provider);
  const showBaseUrl = ["litellm", "ollama", "custom"].includes(provider);
  const showApiKey = !["ollama"].includes(provider);
  const effectiveUrl = baseUrl || selectedProvider?.defaultUrl || "";

  const handleVerify = async () => {
    setVerifyState("loading");
    setVerifyError("");
    try {
      const result = await verifyAiConnection({
        provider,
        baseUrl: effectiveUrl,
        apiKey,
        modelId: modelId || undefined,
      });
      if (result.success) {
        setVerifyState("ok");
        // Fetch models
        const mods = await fetchAvailableModels({
          provider,
          baseUrl: effectiveUrl,
          apiKeyLabel: `${provider}-main`,
          apiKey,
        });
        setModels(mods);
      } else {
        setVerifyState("error");
        setVerifyError(result.error || "Connection failed");
      }
    } catch (e) {
      setVerifyState("error");
      setVerifyError(String(e));
    }
  };

  const handleCheckOllama = async () => {
    const status = await checkOllamaStatus();
    setOllamaStatus(status.running ? "running" : "not_running");
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveAiSettings({
        label: `${provider}-main`,
        provider,
        base_url: effectiveUrl || undefined,
        api_key: apiKey,
        model_id: modelId || undefined,
        ollama_base_url: ollamaUrl,
        ollama_model: "nomic-embed-text",
        embedding_provider: useOllama ? "ollama" : "none",
      });
      onNext();
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 max-w-lg mx-auto w-full">
      <h2 className="text-2xl font-bold text-zinc-900 dark:text-zinc-50 mb-1">
        {t("onboarding.aiSetup.title")}
      </h2>
      <p className="text-zinc-500 text-sm mb-8">{t("onboarding.aiSetup.step")}</p>

      <div className="w-full space-y-4">
        {/* Provider */}
        <div>
          <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
            {t("onboarding.aiSetup.provider")}
          </label>
          <select
            value={provider}
            onChange={(e) => { setProvider(e.target.value); setVerifyState("idle"); setModels([]); }}
            className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 text-sm"
          >
            {AI_PROVIDERS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>

        {/* Base URL */}
        {showBaseUrl && (
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.aiSetup.baseUrl")}
            </label>
            <input
              type="text"
              value={baseUrl || selectedProvider?.defaultUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder={selectedProvider?.defaultUrl}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-sm"
            />
          </div>
        )}

        {/* API Key */}
        {showApiKey && (
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.aiSetup.apiKey")}
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => { setApiKey(e.target.value); setVerifyState("idle"); }}
              placeholder="sk-..."
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-sm font-mono"
            />
          </div>
        )}

        {/* Verify button */}
        <button
          onClick={handleVerify}
          disabled={verifyState === "loading" || (!apiKey && showApiKey)}
          className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 rounded-lg text-sm font-medium disabled:opacity-50 transition-colors hover:bg-zinc-700"
        >
          {verifyState === "loading" && <Loader className="w-4 h-4 animate-spin" />}
          {verifyState === "ok" && <CheckCircle className="w-4 h-4 text-green-500" />}
          {verifyState === "error" && <XCircle className="w-4 h-4 text-red-500" />}
          {verifyState === "loading" ? t("onboarding.aiSetup.verifying") : t("onboarding.aiSetup.verify")}
        </button>

        {verifyError && (
          <p className="text-sm text-red-600">{verifyError}</p>
        )}

        {/* Model picker */}
        {models.length > 0 && (
          <div>
            <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.aiSetup.selectModel")}
            </label>
            <select
              value={modelId}
              onChange={(e) => setModelId(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-sm"
            >
              <option value="">-- Select a model --</option>
              {models.map((m) => (
                <option key={m.id} value={m.id}>{m.name}</option>
              ))}
            </select>
          </div>
        )}

        {/* Ollama toggle */}
        <div className="border border-zinc-100 dark:border-zinc-800 rounded-lg p-3">
          <div className="flex items-center justify-between">
            <span className="text-sm text-zinc-700 dark:text-zinc-300">{t("onboarding.aiSetup.ollamaToggle")}</span>
            <button
              onClick={() => setUseOllama(!useOllama)}
              className={`w-10 h-5 rounded-full transition-colors ${useOllama ? "bg-indigo-500" : "bg-zinc-300 dark:bg-zinc-600"}`}
            >
              <div className={`w-4 h-4 bg-white rounded-full shadow transition-transform ${useOllama ? "translate-x-5" : "translate-x-1"}`} />
            </button>
          </div>
          {useOllama && (
            <div className="mt-2 space-y-2">
              <input
                type="text"
                value={ollamaUrl}
                onChange={(e) => setOllamaUrl(e.target.value)}
                className="w-full px-3 py-1.5 rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-sm"
              />
              <button onClick={handleCheckOllama} className="text-sm text-indigo-500 hover:underline">
                {t("onboarding.aiSetup.checkOllama")}
              </button>
              {ollamaStatus === "not_running" && (
                <p className="text-xs text-amber-600">{t("onboarding.aiSetup.ollamaNotDetected")}</p>
              )}
              {ollamaStatus === "running" && (
                <p className="text-xs text-green-600">Ollama is running</p>
              )}
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-3 pt-2">
          <button
            onClick={onSkip}
            className="flex-1 px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.skip")}
          </button>
          <button
            onClick={handleSave}
            disabled={saving || verifyState !== "ok"}
            className="flex-1 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {saving ? t("common.loading") : t("common.next")}
          </button>
        </div>
      </div>
    </div>
  );
}
