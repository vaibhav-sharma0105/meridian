import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X, CheckCircle, XCircle, Loader, Info } from "lucide-react";
import { getAiSettings, saveAiSettings, verifyAiConnection, checkOllamaStatus } from "@/lib/tauri";
import { useAIStore } from "@/stores/aiStore";
import { AI_PROVIDERS } from "@/lib/constants";
import ModelPicker from "./ModelPicker";

interface Props {
  open: boolean;
  onClose: () => void;
}

const API_KEY_PLACEHOLDERS: Record<string, string> = {
  openai: "sk-...",
  anthropic: "sk-ant-...",
  gemini: "AIza...",
  groq: "gsk_...",
  litellm: "sk-... (your LiteLLM proxy key)",
  custom: "API key",
};

export default function AISettings({ open, onClose }: Props) {
  const { t } = useTranslation();
  const { loadSettings } = useAIStore();

  const [provider, setProvider] = useState("litellm");
  const [baseUrl, setBaseUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [modelId, setModelId] = useState("");
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [ollamaModel, setOllamaModel] = useState("nomic-embed-text");
  const [saving, setSaving] = useState(false);
  const [verifyState, setVerifyState] = useState<"idle" | "loading" | "ok" | "error">("idle");
  const [verifyError, setVerifyError] = useState("");
  const [ollamaStatus, setOllamaStatus] = useState<"idle" | "checking" | "running" | "offline">("idle");

  const selectedProvider = AI_PROVIDERS.find((p) => p.value === provider);
  const showBaseUrl = ["litellm", "ollama", "custom"].includes(provider);
  const showApiKey = !["ollama"].includes(provider);
  const effectiveUrl = baseUrl || selectedProvider?.defaultUrl || "";
  const apiKeyLabel = `${provider}-main`;

  useEffect(() => {
    if (!open) return;
    getAiSettings().then((s) => {
      if (s) {
        setProvider(s.provider);
        setBaseUrl(s.base_url ?? "");
        setModelId(s.model_id ?? "");
        setOllamaUrl(s.ollama_base_url || "http://localhost:11434");
        setOllamaModel(s.ollama_model || "nomic-embed-text");
      }
    }).catch(console.error);
  }, [open]);

  if (!open) return null;

  const handleVerify = async () => {
    setVerifyState("loading");
    setVerifyError("");
    try {
      const result = await verifyAiConnection({
        provider,
        baseUrl: effectiveUrl || undefined,
        apiKey,
        modelId: modelId || undefined,
      });
      setVerifyState(result.success ? "ok" : "error");
      if (!result.success) setVerifyError(result.error ?? "Connection failed");
    } catch (e) {
      setVerifyState("error");
      setVerifyError(String(e));
    }
  };

  const handleCheckOllama = async () => {
    setOllamaStatus("checking");
    try {
      const status = await checkOllamaStatus();
      setOllamaStatus(status.running ? "running" : "offline");
    } catch {
      setOllamaStatus("offline");
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveAiSettings({
        label: apiKeyLabel,
        provider,
        base_url: effectiveUrl || undefined,
        api_key: apiKey || undefined,
        model_id: modelId || undefined,
        ollama_base_url: ollamaUrl || undefined,
        ollama_model: ollamaModel || undefined,
        embedding_provider: "ollama",
      });
      await loadSettings();
      onClose();
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />
      <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-full max-w-md mx-4 border border-zinc-200 dark:border-zinc-700 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex-shrink-0">
          <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50">{t("settings.aiSettings")}</h2>
          <button onClick={onClose} className="p-1 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded">
            <X className="w-4 h-4 text-zinc-500" />
          </button>
        </div>

        {/* Scrollable body */}
        <div className="px-6 py-4 space-y-4 overflow-y-auto flex-1">

          {/* Provider */}
          <div>
            <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.aiSetup.provider")}
            </label>
            <select
              value={provider}
              onChange={(e) => { setProvider(e.target.value); setVerifyState("idle"); }}
              className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
            >
              {AI_PROVIDERS.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
          </div>

          {/* Base URL */}
          {showBaseUrl && (
            <div>
              <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                {t("onboarding.aiSetup.baseUrl")}
              </label>
              <input
                type="text"
                value={baseUrl || selectedProvider?.defaultUrl || ""}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder={selectedProvider?.defaultUrl}
                className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
              />
            </div>
          )}

          {/* LiteLLM + Bedrock hint */}
          {provider === "litellm" && (
            <div className="flex gap-2 px-3 py-2 bg-indigo-50 dark:bg-indigo-900/20 rounded-lg border border-indigo-100 dark:border-indigo-800">
              <Info className="w-4 h-4 text-indigo-500 flex-shrink-0 mt-0.5" />
              <p className="text-xs text-indigo-700 dark:text-indigo-300">
                Using AWS Bedrock via LiteLLM? Set your LiteLLM server URL above and use the model format:{" "}
                <code className="font-mono">bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0</code>
              </p>
            </div>
          )}

          {/* API Key */}
          {showApiKey && (
            <div>
              <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                {t("onboarding.aiSetup.apiKey")}
              </label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => { setApiKey(e.target.value); setVerifyState("idle"); }}
                placeholder={API_KEY_PLACEHOLDERS[provider] ?? "API key"}
                className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 font-mono text-zinc-900 dark:text-zinc-50"
              />
              <p className="text-xs text-zinc-400 mt-1">Stored securely in OS keychain. Re-enter to update.</p>
            </div>
          )}

          {/* Model */}
          <div>
            <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("onboarding.aiSetup.selectModel")}
            </label>
            <ModelPicker
              value={modelId}
              onSelect={setModelId}
              provider={provider}
              baseUrl={effectiveUrl}
              apiKeyLabel={apiKeyLabel}
              apiKey={apiKey}
            />
          </div>

          {/* Verify button */}
          <button
            onClick={handleVerify}
            disabled={verifyState === "loading" || (!apiKey && !["ollama", "litellm"].includes(provider))}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 rounded-lg text-sm font-medium disabled:opacity-50"
          >
            {verifyState === "loading" && <Loader className="w-4 h-4 animate-spin" />}
            {verifyState === "ok" && <CheckCircle className="w-4 h-4 text-green-500" />}
            {verifyState === "error" && <XCircle className="w-4 h-4 text-red-500" />}
            {verifyState === "loading" ? t("onboarding.aiSetup.verifying") : t("onboarding.aiSetup.verify")}
          </button>
          {verifyError && <p className="text-sm text-red-600 dark:text-red-400">{verifyError}</p>}

          {/* Ollama section */}
          <div className="border-t border-zinc-100 dark:border-zinc-800 pt-4">
            <h3 className="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-wide mb-3">
              {t("settings.ai.ollamaSection")}
            </h3>

            <div className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  {t("onboarding.aiSetup.ollamaUrl")}
                </label>
                <div className="flex gap-1">
                  <input
                    type="text"
                    value={ollamaUrl}
                    onChange={(e) => { setOllamaUrl(e.target.value); setOllamaStatus("idle"); }}
                    placeholder="http://localhost:11434"
                    className="flex-1 px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                  />
                  <button
                    onClick={handleCheckOllama}
                    disabled={ollamaStatus === "checking"}
                    className="px-3 py-2 text-xs rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 disabled:opacity-50 whitespace-nowrap flex items-center gap-1"
                  >
                    {ollamaStatus === "checking" && <Loader className="w-3 h-3 animate-spin" />}
                    {ollamaStatus === "running" && <CheckCircle className="w-3 h-3 text-green-500" />}
                    {ollamaStatus === "offline" && <XCircle className="w-3 h-3 text-red-500" />}
                    Check
                  </button>
                </div>
                {ollamaStatus === "running" && (
                  <p className="text-xs text-green-600 dark:text-green-400 mt-1">Ollama is running</p>
                )}
                {ollamaStatus === "offline" && (
                  <p className="text-xs text-zinc-400 mt-1">{t("onboarding.aiSetup.ollamaNotDetected")}</p>
                )}
              </div>

              <div>
                <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  {t("settings.ai.ollamaModel")}
                </label>
                <input
                  type="text"
                  value={ollamaModel}
                  onChange={(e) => setOllamaModel(e.target.value)}
                  placeholder="nomic-embed-text"
                  className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-zinc-100 dark:border-zinc-800 flex-shrink-0">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-zinc-200 dark:border-zinc-700 rounded-lg text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
          >
            {saving ? t("common.loading") : t("settings.ai.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
