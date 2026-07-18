import { useState } from "react";
import { RefreshCw, Plus, Settings2 } from "lucide-react";
import { useIntegrations, useAvailableIntegrations, useSyncIntegration } from "@/hooks/useIntegrations";
import { IntegrationCard } from "./IntegrationCard";
import { ConnectWizard } from "./ConnectWizard";

export function IntegrationHub() {
  const { data: integrations = [], isLoading } = useIntegrations();
  const { data: available = [] } = useAvailableIntegrations();
  const [showConnect, setShowConnect] = useState(false);
  const [connectType, setConnectType] = useState<string | null>(null);
  const syncMutation = useSyncIntegration();

  const connected = integrations.filter((i) => i.status === "connected");
  const disconnected = available.filter(
    (a) => !integrations.find((i) => i.type === a.type)
  );

  const handleConnect = (type: string) => {
    setConnectType(type);
    setShowConnect(true);
  };

  const handleSyncAll = () => {
    connected.forEach((i) => syncMutation.mutate(i.id));
  };

  if (isLoading) {
    return (
      <div className="p-6 text-zinc-500">Loading integrations...</div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
            Integrations
          </h2>
          <p className="text-sm text-zinc-500 mt-1">
            Connect external services to sync data and enable automation
          </p>
        </div>
        {connected.length > 0 && (
          <button
            onClick={handleSyncAll}
            disabled={syncMutation.isPending}
            className="flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors"
          >
            <RefreshCw
              className={`w-4 h-4 ${syncMutation.isPending ? "animate-spin" : ""}`}
            />
            Sync All
          </button>
        )}
      </div>

      {connected.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
            Connected
          </h3>
          <div className="grid gap-3">
            {connected.map((integration) => (
              <IntegrationCard key={integration.id} integration={integration} />
            ))}
          </div>
        </div>
      )}

      {disconnected.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
            Available
          </h3>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {disconnected.map((avail) => (
              <button
                key={avail.type}
                onClick={() => handleConnect(avail.type)}
                className="flex items-start gap-3 p-4 text-left border border-zinc-200 dark:border-zinc-700 rounded-lg hover:border-indigo-300 dark:hover:border-indigo-700 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
              >
                <div className="w-10 h-10 flex items-center justify-center bg-zinc-100 dark:bg-zinc-800 rounded-lg">
                  <Settings2 className="w-5 h-5 text-zinc-600 dark:text-zinc-400" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-zinc-900 dark:text-zinc-100">
                    {avail.name}
                  </div>
                  <div className="text-xs text-zinc-500 mt-0.5 line-clamp-2">
                    {avail.description}
                  </div>
                </div>
                <Plus className="w-4 h-4 text-zinc-400 flex-shrink-0" />
              </button>
            ))}
          </div>
        </div>
      )}

      {showConnect && connectType && (
        <ConnectWizard
          integrationType={connectType}
          onClose={() => {
            setShowConnect(false);
            setConnectType(null);
          }}
        />
      )}
    </div>
  );
}
