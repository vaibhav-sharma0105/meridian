import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Search, Loader, Brain, FileText, Sparkles } from "lucide-react";
import { hybridSearchDocuments } from "@/lib/tauri";
import type { SearchResult } from "@/lib/tauri";

interface Props {
  projectId: string;
}

function MatchTypeBadge({ type }: { type: string }) {
  switch (type) {
    case "semantic":
      return (
        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400 rounded">
          <Brain className="w-2.5 h-2.5" />
          Semantic
        </span>
      );
    case "keyword":
      return (
        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400 rounded">
          <FileText className="w-2.5 h-2.5" />
          Keyword
        </span>
      );
    case "both":
      return (
        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400 rounded">
          <Sparkles className="w-2.5 h-2.5" />
          Both
        </span>
      );
    default:
      return null;
  }
}

function highlightKeywords(text: string, query: string): React.ReactNode {
  if (!query.trim()) return text;
  const keywords = query.toLowerCase().split(/\s+/).filter(k => k.length > 2);
  if (keywords.length === 0) return text;

  const pattern = new RegExp(`(${keywords.map(k => k.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`, "gi");
  const parts = text.split(pattern);

  return parts.map((part, i) =>
    keywords.some(k => part.toLowerCase() === k) ? (
      <mark key={i} className="bg-yellow-200 dark:bg-yellow-800/50 text-inherit rounded px-0.5">
        {part}
      </mark>
    ) : (
      part
    )
  );
}

export default function DocSearch({ projectId }: Props) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setSearching(true);
    try {
      const r = await hybridSearchDocuments({ projectId, query: query.trim(), limit: 10 });
      setResults(r);
    } finally {
      setSearching(false);
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder={t("documents.searchPlaceholder")}
            className="w-full pl-9 pr-3 py-2 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={searching || !query.trim()}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg text-sm"
        >
          {searching ? <Loader className="w-4 h-4 animate-spin" /> : <Search className="w-4 h-4" />}
          {t("documents.search")}
        </button>
      </div>

      {results.length > 0 && (
        <div className="space-y-2">
          {results.map((result, i) => (
            <div key={i} className="p-3 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg">
              <div className="flex items-center justify-between mb-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-semibold text-zinc-500">{result.document_title}</span>
                  <MatchTypeBadge type={result.search_type} />
                </div>
                <span className="text-xs text-zinc-400">
                  {Math.round(result.score * 100)}% match
                </span>
              </div>
              <p className="text-sm text-zinc-700 dark:text-zinc-300 line-clamp-3">
                {result.search_type === "keyword" || result.search_type === "both"
                  ? highlightKeywords(result.content, query)
                  : result.content}
              </p>
            </div>
          ))}
        </div>
      )}

      {results.length === 0 && query && !searching && (
        <p className="text-sm text-zinc-400 text-center py-4">{t("documents.noResults")}</p>
      )}
    </div>
  );
}
