import { useQuery } from "@tanstack/react-query";
import {
  getIntegrationCacheForProject,
  searchCachedIntegrationItems,
} from "@/lib/tauri";

export function useIntegrationCache(
  projectId: string,
  integrationType?: string,
  itemType?: string
) {
  return useQuery({
    queryKey: ["integration-cache", projectId, integrationType, itemType],
    queryFn: () =>
      getIntegrationCacheForProject(projectId, integrationType, itemType),
    enabled: !!projectId,
  });
}

export function useIntegrationSearch(query: string, projectId?: string) {
  return useQuery({
    queryKey: ["integration-search", query, projectId],
    queryFn: () => searchCachedIntegrationItems(query, projectId),
    enabled: query.length >= 2,
  });
}
