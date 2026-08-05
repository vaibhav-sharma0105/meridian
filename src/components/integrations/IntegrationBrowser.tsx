import { useState, useMemo } from "react";
import { Search, Filter } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useProjectStore } from "@/stores/projectStore";
import {
  useIntegrationCache,
  useIntegrationSearch,
} from "@/hooks/useIntegrationBrowser";
import { IntegrationItemRow } from "./IntegrationItemRow";
import { listIntegrations } from "@/lib/tauri";

const itemTypes = ["all", "issue", "pr", "commit", "thread", "message"];

export function IntegrationBrowser() {
  const { activeProjectId, getActiveProject } = useProjectStore();
  const activeProject = getActiveProject();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedType, setSelectedType] = useState<string | undefined>();
  const [selectedItemType, setSelectedItemType] = useState("all");

  const isSearching = searchQuery.length >= 2;

  // Fetch integrations to get sync intervals
  const integrationsQuery = useQuery({
    queryKey: ["integrations"],
    queryFn: listIntegrations,
  });

  // Build a map of integration_id -> sync_interval_minutes
  const syncIntervalMap = useMemo(() => {
    const map: Record<string, number> = {};
    if (integrationsQuery.data) {
      for (const integration of integrationsQuery.data) {
        map[integration.id] = integration.sync_interval_minutes;
      }
    }
    return map;
  }, [integrationsQuery.data]);

  const cacheQuery = useIntegrationCache(
    activeProjectId || "",
    selectedType,
    selectedItemType === "all" ? undefined : selectedItemType
  );

  const searchResults = useIntegrationSearch(
    searchQuery,
    activeProjectId || undefined
  );

  const items = isSearching ? searchResults.data : cacheQuery.data;
  const isLoading = isSearching ? searchResults.isLoading : cacheQuery.isLoading;

  if (!activeProject) {
    return (
      <div className="p-6 text-center text-zinc-500">
        Select a project to browse integrations
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-3 border-b border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center gap-3 mb-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
            <input
              type="text"
              placeholder="Search integration items..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-2 text-sm bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-zinc-400" />
          <select
            value={selectedType || ""}
            onChange={(e) => setSelectedType(e.target.value || undefined)}
            className="px-2 py-1 text-xs bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded"
          >
            <option value="">All Sources</option>
            <option value="github">GitHub</option>
            <option value="jira">Jira</option>
            <option value="slack">Slack</option>
          </select>
          <select
            value={selectedItemType}
            onChange={(e) => setSelectedItemType(e.target.value)}
            className="px-2 py-1 text-xs bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded"
          >
            {itemTypes.map((type) => (
              <option key={type} value={type}>
                {type === "all"
                  ? "All Types"
                  : type.charAt(0).toUpperCase() + type.slice(1)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {isLoading ? (
          <div className="text-center text-zinc-500 py-8">Loading...</div>
        ) : !items || items.length === 0 ? (
          <div className="text-center text-zinc-500 py-8">
            {isSearching
              ? "No results found"
              : "No integration data for this project"}
          </div>
        ) : (
          items.map((item) => (
            <IntegrationItemRow
              key={item.id}
              item={item}
              syncIntervalMinutes={syncIntervalMap[item.integration_id] || 15}
            />
          ))
        )}
      </div>
    </div>
  );
}
