import { useState, useEffect, useMemo } from "react";
import {
  X,
  Search,
  Link2,
  Github,
  ExternalLink,
  Loader2,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";
import { useIntegrations, useCachedItems } from "@/hooks/useIntegrations";
import { useCreateLink } from "@/hooks/useIntegrationLinks";
import type { IntegrationCache } from "@/lib/tauri";
import toast from "react-hot-toast";

interface LinkPickerProps {
  taskId: string;
  onClose: () => void;
  onLinked?: () => void;
}

type ExternalItemType = "issue" | "pr" | "jira_issue" | "slack_channel";

interface ExternalItem {
  id: string;
  type: ExternalItemType;
  integrationId: string;
  title: string;
  url?: string;
  meta?: Record<string, unknown>;
}

const TYPE_LABELS: Record<ExternalItemType, { label: string; icon: React.ReactNode }> = {
  issue: { label: "GitHub Issue", icon: <Github className="w-4 h-4" /> },
  pr: { label: "GitHub PR", icon: <Github className="w-4 h-4" /> },
  jira_issue: { label: "Jira Issue", icon: <span className="text-sm">🔷</span> },
  slack_channel: { label: "Slack Channel", icon: <span className="text-sm">💬</span> },
};

export function LinkPicker({ taskId, onClose, onLinked }: LinkPickerProps) {
  const [search, setSearch] = useState("");
  const [selectedType, setSelectedType] = useState<ExternalItemType | "all">("all");
  const [selectedItem, setSelectedItem] = useState<ExternalItem | null>(null);

  const { data: integrations = [] } = useIntegrations();
  const createLink = useCreateLink();

  // Get cached items from all connected integrations
  const githubIntegration = integrations.find((i) => i.type === "github" && i.status === "connected");
  const jiraIntegration = integrations.find((i) => i.type === "jira" && i.status === "connected");

  const { data: githubIssues = [] } = useCachedItems(githubIntegration?.id ?? "", "issue");
  const { data: githubPRs = [] } = useCachedItems(githubIntegration?.id ?? "", "pr");
  const { data: jiraIssues = [] } = useCachedItems(jiraIntegration?.id ?? "", "jira_issue");

  // Combine and transform all external items
  const allItems = useMemo<ExternalItem[]>(() => {
    const items: ExternalItem[] = [];

    if (githubIntegration) {
      githubIssues.forEach((item) => {
        items.push({
          id: item.external_id,
          type: "issue",
          integrationId: item.integration_id,
          title: (item.data as { title?: string })?.title ?? `Issue #${item.external_id}`,
          url: item.external_url,
          meta: item.data as Record<string, unknown>,
        });
      });
      githubPRs.forEach((item) => {
        items.push({
          id: item.external_id,
          type: "pr",
          integrationId: item.integration_id,
          title: (item.data as { title?: string })?.title ?? `PR #${item.external_id}`,
          url: item.external_url,
          meta: item.data as Record<string, unknown>,
        });
      });
    }

    if (jiraIntegration) {
      jiraIssues.forEach((item) => {
        items.push({
          id: item.external_id,
          type: "jira_issue",
          integrationId: item.integration_id,
          title: (item.data as { summary?: string })?.summary ?? item.external_id,
          url: item.external_url,
          meta: item.data as Record<string, unknown>,
        });
      });
    }

    return items;
  }, [githubIntegration, jiraIntegration, githubIssues, githubPRs, jiraIssues]);

  // Filter items based on search and type
  const filteredItems = useMemo(() => {
    return allItems.filter((item) => {
      const matchesType = selectedType === "all" || item.type === selectedType;
      const matchesSearch =
        search === "" ||
        item.title.toLowerCase().includes(search.toLowerCase()) ||
        item.id.toLowerCase().includes(search.toLowerCase());
      return matchesType && matchesSearch;
    });
  }, [allItems, selectedType, search]);

  const handleLink = async () => {
    if (!selectedItem) return;

    try {
      await createLink.mutateAsync({
        integration_id: selectedItem.integrationId,
        local_type: "task",
        local_id: taskId,
        external_type: selectedItem.type,
        external_id: selectedItem.id,
        external_url: selectedItem.url,
        sync_enabled: true,
      });
      toast.success(`Linked to ${TYPE_LABELS[selectedItem.type].label}`);
      onLinked?.();
      onClose();
    } catch (e) {
      toast.error("Failed to create link");
    }
  };

  const hasConnectedIntegrations = githubIntegration || jiraIntegration;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
              <Link2 className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Link to External Item
              </h3>
              <p className="text-xs text-zinc-500">
                Connect this task to GitHub issues, Jira tickets, etc.
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {!hasConnectedIntegrations ? (
          <div className="p-6">
            <div className="flex items-start gap-3 p-4 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
              <AlertCircle className="w-5 h-5 text-amber-500 mt-0.5 flex-shrink-0" />
              <div>
                <div className="text-sm font-medium text-amber-800 dark:text-amber-200">
                  No Integrations Connected
                </div>
                <p className="text-xs text-amber-700 dark:text-amber-300 mt-1">
                  Connect GitHub or Jira from the Integrations page to link tasks
                  to external items.
                </p>
              </div>
            </div>
          </div>
        ) : (
          <div className="p-6 space-y-4">
            {/* Search */}
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
              <input
                type="text"
                placeholder="Search issues, PRs, tickets..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                autoFocus
              />
            </div>

            {/* Type Filter */}
            <div className="flex items-center gap-2 overflow-x-auto pb-1">
              <button
                onClick={() => setSelectedType("all")}
                className={`px-3 py-1.5 text-xs font-medium rounded-full whitespace-nowrap transition-colors ${
                  selectedType === "all"
                    ? "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300"
                    : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
                }`}
              >
                All
              </button>
              {Object.entries(TYPE_LABELS).map(([type, { label, icon }]) => (
                <button
                  key={type}
                  onClick={() => setSelectedType(type as ExternalItemType)}
                  className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-full whitespace-nowrap transition-colors ${
                    selectedType === type
                      ? "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300"
                      : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
                  }`}
                >
                  {icon}
                  {label}
                </button>
              ))}
            </div>

            {/* Results */}
            <div className="max-h-64 overflow-y-auto border border-zinc-200 dark:border-zinc-700 rounded-lg divide-y divide-zinc-200 dark:divide-zinc-700">
              {filteredItems.length === 0 ? (
                <div className="p-4 text-center text-sm text-zinc-500">
                  {search
                    ? "No items match your search"
                    : "No items synced. Try syncing your integrations first."}
                </div>
              ) : (
                filteredItems.map((item) => (
                  <button
                    key={`${item.type}-${item.id}`}
                    onClick={() => setSelectedItem(item)}
                    className={`w-full flex items-center justify-between p-3 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors ${
                      selectedItem?.id === item.id && selectedItem?.type === item.type
                        ? "bg-indigo-50 dark:bg-indigo-900/20"
                        : ""
                    }`}
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <div className="flex-shrink-0 text-zinc-500">
                        {TYPE_LABELS[item.type].icon}
                      </div>
                      <div className="min-w-0">
                        <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100 truncate">
                          {item.title}
                        </div>
                        <div className="text-xs text-zinc-500 flex items-center gap-2">
                          <span>{item.id}</span>
                          {item.url && (
                            <a
                              href={item.url}
                              target="_blank"
                              rel="noopener noreferrer"
                              onClick={(e) => e.stopPropagation()}
                              className="text-indigo-500 hover:text-indigo-600"
                            >
                              <ExternalLink className="w-3 h-3" />
                            </a>
                          )}
                        </div>
                      </div>
                    </div>
                    {selectedItem?.id === item.id && selectedItem?.type === item.type && (
                      <CheckCircle2 className="w-5 h-5 text-indigo-500 flex-shrink-0" />
                    )}
                  </button>
                ))
              )}
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg"
          >
            Cancel
          </button>
          <button
            onClick={handleLink}
            disabled={!selectedItem || createLink.isPending}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {createLink.isPending ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Link2 className="w-4 h-4" />
            )}
            Link Item
          </button>
        </div>
      </div>
    </div>
  );
}
