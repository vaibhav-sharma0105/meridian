import { useState } from "react";
import { Link2, ChevronDown, ChevronUp, Zap } from "lucide-react";
import { useAttentionItems, useDismissAttention } from "@/hooks/useAttention";
import type { AttentionFilters as AttentionFiltersType } from "@/lib/tauri";
import { AttentionItem } from "./AttentionItem";
import { AttentionFilters } from "./AttentionFilters";

interface SectionProps {
  title: string;
  items: ReturnType<typeof useAttentionItems>["data"];
  severity: string;
  defaultExpanded?: boolean;
  onDismiss: (id: string) => void;
}

function Section({
  title,
  items,
  severity,
  defaultExpanded = true,
  onDismiss,
}: SectionProps) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const filteredItems = items?.filter((item) => item.severity === severity) || [];

  if (filteredItems.length === 0) return null;

  return (
    <div className="mb-4">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 w-full px-4 py-2 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
      >
        {expanded ? (
          <ChevronDown className="w-4 h-4 text-zinc-400" />
        ) : (
          <ChevronUp className="w-4 h-4 text-zinc-400" />
        )}
        <span className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">
          {title}
        </span>
        <span className="text-xs text-zinc-400 ml-auto">
          {filteredItems.length}
        </span>
      </button>

      {expanded && (
        <div className="px-4 space-y-2">
          {filteredItems.map((item) => (
            <AttentionItem key={item.id} item={item} onDismiss={onDismiss} />
          ))}
        </div>
      )}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <div className="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mb-4">
        <Link2 className="w-6 h-6 text-zinc-400" />
      </div>
      <h3 className="text-lg font-semibold text-zinc-700 dark:text-zinc-300 mb-2">
        No integrations connected
      </h3>
      <p className="text-sm text-zinc-500 max-w-sm mb-4">
        Connect GitHub, Jira, or Slack to see items that need your attention
        here.
      </p>
      <a
        href="#"
        className="text-sm text-indigo-500 hover:text-indigo-600 font-medium"
      >
        Connect Integrations →
      </a>
    </div>
  );
}

function AllClearState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <div className="w-12 h-12 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center mb-4">
        <Zap className="w-6 h-6 text-green-500" />
      </div>
      <h3 className="text-lg font-semibold text-zinc-700 dark:text-zinc-300 mb-2">
        All clear!
      </h3>
      <p className="text-sm text-zinc-500">
        Nothing needs your attention right now.
      </p>
    </div>
  );
}

export function MyActivityDashboard() {
  const [filters, setFilters] = useState<AttentionFiltersType>({});
  const { data: items, isLoading, error } = useAttentionItems(filters);
  const dismissMutation = useDismissAttention();

  const handleDismiss = async (id: string) => {
    await dismissMutation.mutateAsync(id);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-red-500">
        Failed to load attention items: {String(error)}
      </div>
    );
  }

  const hasItems = items && items.length > 0;
  const hasActiveItems = items?.some((item) => !item.dismissed_at);

  return (
    <div className="h-full flex flex-col">
      <div className="px-4 py-3 border-b border-zinc-100 dark:border-zinc-800">
        <h2 className="text-lg font-semibold text-zinc-800 dark:text-zinc-200">
          My Activity
        </h2>
        <p className="text-xs text-zinc-500">
          Items that need your attention across all projects
        </p>
      </div>

      <AttentionFilters filters={filters} onChange={setFilters} />

      <div className="flex-1 overflow-y-auto">
        {!hasItems && !filters.include_dismissed && <EmptyState />}
        {hasItems && !hasActiveItems && !filters.include_dismissed && (
          <AllClearState />
        )}

        {hasItems && (
          <>
            <Section
              title="Critical"
              items={items}
              severity="critical"
              onDismiss={handleDismiss}
            />
            <Section
              title="Needs Attention"
              items={items}
              severity="warning"
              onDismiss={handleDismiss}
            />
            <Section
              title="Info"
              items={items}
              severity="info"
              defaultExpanded={false}
              onDismiss={handleDismiss}
            />
          </>
        )}
      </div>
    </div>
  );
}
