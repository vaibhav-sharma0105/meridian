import { useState } from "react";
import { fetchAvailableModels } from "@/lib/tauri";
import type { ModelInfo } from "@/lib/tauri";
import { RefreshCw } from "lucide-react";

interface Props {
  onSelect: (modelId: string) => void;
  value: string;
  provider: string;
  baseUrl: string;
  apiKeyLabel: string;
  apiKey?: string;
}

const MODEL_PLACEHOLDERS: Record<string, string> = {
  openai: "e.g. gpt-4o",
  anthropic: "e.g. claude-sonnet-4-6",
  gemini: "e.g. gemini-1.5-pro",
  groq: "e.g. llama-3.1-70b-versatile",
  litellm: "e.g. bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0",
  ollama: "e.g. llama3",
  custom: "Model ID",
};

export default function ModelPicker({ onSelect, value, provider, baseUrl, apiKeyLabel, apiKey }: Props) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [fetched, setFetched] = useState(false);
  const [fetchError, setFetchError] = useState("");

  const handleFetch = async () => {
    setLoading(true);
    setFetchError("");
    try {
      const list = await fetchAvailableModels({ provider, baseUrl: baseUrl || undefined, apiKeyLabel, apiKey: apiKey || undefined });
      setModels(list);
      setFetched(true);
    } catch (e) {
      setFetchError(String(e));
      setFetched(false);
    } finally {
      setLoading(false);
    }
  };

  const placeholder = MODEL_PLACEHOLDERS[provider] ?? "Model ID";

  if (fetched && models.length > 0) {
    return (
      <div className="space-y-1">
        <div className="flex gap-1">
          <select
            value={value}
            onChange={(e) => onSelect(e.target.value)}
            className="flex-1 px-2 py-1.5 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          >
            <option value="">-- Select model --</option>
            {models.map((m) => (
              <option key={m.id} value={m.id}>{m.name}</option>
            ))}
          </select>
          <button
            onClick={() => { setFetched(false); setModels([]); }}
            className="px-2 py-1.5 text-xs rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
            title="Enter model ID manually"
          >
            Type
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      <div className="flex gap-1">
        <input
          type="text"
          value={value}
          onChange={(e) => onSelect(e.target.value)}
          placeholder={placeholder}
          className="flex-1 px-3 py-1.5 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
        />
        <button
          onClick={handleFetch}
          disabled={loading}
          className="px-3 py-1.5 text-xs rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 disabled:opacity-50 whitespace-nowrap flex items-center gap-1"
          title="Fetch available models from provider"
        >
          {loading ? <RefreshCw className="w-3 h-3 animate-spin" /> : <RefreshCw className="w-3 h-3" />}
          Fetch
        </button>
      </div>
      {fetchError && <p className="text-xs text-red-500">{fetchError}</p>}
    </div>
  );
}
