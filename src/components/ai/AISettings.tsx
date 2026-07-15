import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X, CheckCircle, XCircle, Loader, Info, ChevronDown, ChevronRight, AlertTriangle } from "lucide-react";
import { getAiSettings, saveAiSettings, verifyAiConnection, checkOllamaStatus, queueEmbeddingMigration, getEmbeddingMigrationStatus } from "@/lib/tauri";
import { useAIStore } from "@/stores/aiStore";
import { AI_PROVIDERS } from "@/lib/constants";
import ModelPicker from "./ModelPicker";
import DaemonStatus from "@/components/settings/DaemonStatus";
import AuditLogViewer from "@/components/settings/AuditLogViewer";
import { LearningSettings } from "@/components/patterns";
import ConfirmDialog from "@/components/shared/ConfirmDialog";

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
  const [embeddingProvider, setEmbeddingProvider] = useState("bundled");
  const [originalEmbeddingProvider, setOriginalEmbeddingProvider] = useState("bundled");
  const [saving, setSaving] = useState(false);
  const [verifyState, setVerifyState] = useState<"idle" | "loading" | "ok" | "error">("idle");
  const [verifyError, setVerifyError] = useState("");
  const [ollamaStatus, setOllamaStatus] = useState<"idle" | "checking" | "running" | "offline">("idle");
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [learningOpen, setLearningOpen] = useState(false);
  const [showReembedConfirm, setShowReembedConfirm] = useState(false);
  const [pendingEmbeddingProvider, setPendingEmbeddingProvider] = useState<string | null>(null);
  const [documentsNeedingReembed, setDocumentsNeedingReembed] = useState(0);

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
        setEmbeddingProvider(s.embedding_provider || "bundled");
        setOriginalEmbeddingProvider(s.embedding_provider || "bundled");
      }
    }).catch(console.error);

    getEmbeddingMigrationStatus().then((status) => {
      setDocumentsNeedingReembed(status.documents_needing_embedding);
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

  const handleEmbeddingProviderChange = (newProvider: string) => {
    if (newProvider !== originalEmbeddingProvider && documentsNeedingReembed > 0) {
      setPendingEmbeddingProvider(newProvider);
      setShowReembedConfirm(true);
    } else {
      setEmbeddingProvider(newProvider);
    }
  };

  const confirmEmbeddingProviderChange = () => {
    if (pendingEmbeddingProvider) {
      setEmbeddingProvider(pendingEmbeddingProvider);
    }
    setShowReembedConfirm(false);
    setPendingEmbeddingProvider(null);
  };

  const cancelEmbeddingProviderChange = () => {
    setShowReembedConfirm(false);
    setPendingEmbeddingProvider(null);
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
        embedding_provider: embeddingProvider,
      });

      // If embedding provider changed, trigger re-embedding
      if (embeddingProvider !== originalEmbeddingProvider) {
        await queueEmbeddingMigration();
      }

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

          {/* Embeddings section */}
          <div className="border-t border-zinc-100 dark:border-zinc-800 pt-4">
            <h3 className="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-wide mb-3">
              Embeddings for Semantic Search
            </h3>

            <div className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                  Embedding Provider
                </label>
                <select
                  value={embeddingProvider}
                  onChange={(e) => handleEmbeddingProviderChange(e.target.value)}
                  className="w-full px-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
                >
                  <option value="bundled">Bundled Model (offline, recommended)</option>
                  <option value="ollama">Ollama</option>
                  <option value="openai">OpenAI</option>
                </select>
              </div>

              {embeddingProvider === "bundled" && (
                <div className="flex items-start gap-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <CheckCircle className="w-4 h-4 text-green-500 flex-shrink-0 mt-0.5" />
                  <div>
                    <p className="text-xs text-green-700 dark:text-green-300 font-medium">
                      Bundled MiniLM model - works offline
                    </p>
                    <p className="text-xs text-green-600 dark:text-green-400 mt-1">
                      384-dimensional embeddings, no external dependencies required.
                    </p>
                  </div>
                </div>
              )}

              {embeddingProvider === "ollama" && (
                <>
                  {ollamaStatus !== "running" ? (
                    <div className="p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg border border-amber-200 dark:border-amber-800">
                      <div className="flex items-start gap-2">
                        <Info className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
                        <div>
                          <p className="text-xs text-amber-700 dark:text-amber-300">
                            Ollama not detected. Install from{" "}
                            <a href="https://ollama.ai" target="_blank" rel="noopener noreferrer" className="text-indigo-500 hover:underline">ollama.ai</a>
                            {" "}and run: <code className="px-1 py-0.5 bg-amber-200 dark:bg-amber-800 rounded text-[10px]">ollama pull nomic-embed-text</code>
                          </p>
                          <button
                            onClick={handleCheckOllama}
                            disabled={ollamaStatus === "checking"}
                            className="mt-2 px-3 py-1.5 text-xs rounded-lg border border-amber-300 dark:border-amber-700 bg-white dark:bg-zinc-800 text-amber-700 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200 disabled:opacity-50 flex items-center gap-1.5"
                          >
                            {ollamaStatus === "checking" && <Loader className="w-3 h-3 animate-spin" />}
                            Check for Ollama
                          </button>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      <div className="flex items-center gap-2 p-2 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                        <CheckCircle className="w-4 h-4 text-green-500" />
                        <span className="text-xs text-green-700 dark:text-green-300">Ollama connected</span>
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                          Ollama URL
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
                            className="px-3 py-2 text-xs rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                          >
                            Recheck
                          </button>
                        </div>
                      </div>
                      <div>
                        <label className="block text-xs font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                          Embedding Model
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
                  )}
                </>
              )}

              {embeddingProvider === "openai" && (
                <div className="flex items-start gap-2 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                  <Info className="w-4 h-4 text-blue-500 flex-shrink-0 mt-0.5" />
                  <p className="text-xs text-blue-700 dark:text-blue-300">
                    Uses your OpenAI API key (configured above) with text-embedding-3-small. Requires internet connection.
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Learning section */}
          <div className="border-t border-zinc-100 dark:border-zinc-800 pt-4">
            <button
              onClick={() => setLearningOpen(!learningOpen)}
              className="flex items-center gap-2 w-full text-left"
            >
              {learningOpen ? (
                <ChevronDown className="w-4 h-4 text-zinc-400" />
              ) : (
                <ChevronRight className="w-4 h-4 text-zinc-400" />
              )}
              <h3 className="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-wide">
                Learning
              </h3>
            </button>

            {learningOpen && (
              <div className="mt-4">
                <LearningSettings />
              </div>
            )}
          </div>

          {/* Advanced section */}
          <div className="border-t border-zinc-100 dark:border-zinc-800 pt-4">
            <button
              onClick={() => setAdvancedOpen(!advancedOpen)}
              className="flex items-center gap-2 w-full text-left"
            >
              {advancedOpen ? (
                <ChevronDown className="w-4 h-4 text-zinc-400" />
              ) : (
                <ChevronRight className="w-4 h-4 text-zinc-400" />
              )}
              <h3 className="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-wide">
                Advanced
              </h3>
            </button>

            {advancedOpen && (
              <div className="mt-4 space-y-6">
                <DaemonStatus />
                <div className="border-t border-zinc-100 dark:border-zinc-800 pt-4">
                  <AuditLogViewer />
                </div>
              </div>
            )}
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

      <ConfirmDialog
        open={showReembedConfirm}
        title="Change Embedding Provider?"
        message={`Changing the embedding provider will re-index ${documentsNeedingReembed} document${documentsNeedingReembed !== 1 ? "s" : ""} with the new model. This may take some time depending on your documents.`}
        confirmLabel="Change Provider"
        onConfirm={confirmEmbeddingProviderChange}
        onCancel={cancelEmbeddingProviderChange}
      />
    </div>
  );
}
