import { useState } from "react";
import {
  RefreshCw,
  Plus,
  Settings2,
  Plug,
  Server,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  AlertCircle,
  CheckCircle2,
  Clock,
} from "lucide-react";
import { useIntegrations, useAvailableIntegrations, useSyncIntegration } from "@/hooks/useIntegrations";
import { useConnections } from "@/hooks/useConnections";
import { IntegrationCard } from "./IntegrationCard";
import { SetupWizard } from "./SetupWizard";
import { GitHubSettings } from "./GitHubSettings";
import { JiraSettings } from "./JiraSettings";
import { SlackSettings } from "./SlackSettings";
import { MCPSettings } from "./MCPSettings";
import type { Integration } from "@/lib/tauri";

// ─── Types ────────────────────────────────────────────────────────────────────

interface IntegrationTypeInfo {
  type: string;
  name: string;
  icon: string;
  description: string;
  category: "native" | "mcp";
}

// ─── Available Integration Types ──────────────────────────────────────────────

const AVAILABLE_INTEGRATIONS: IntegrationTypeInfo[] = [
  {
    type: "zoom",
    name: "Zoom",
    icon: "🎥",
    description: "Import meeting recordings and transcripts",
    category: "native",
  },
  {
    type: "github",
    name: "GitHub",
    icon: "🐙",
    description: "Sync issues and PRs, link tasks to GitHub items",
    category: "native",
  },
  {
    type: "jira",
    name: "Jira",
    icon: "🔷",
    description: "Sync Jira issues, link tasks to tickets",
    category: "native",
  },
  {
    type: "slack",
    name: "Slack",
    icon: "💬",
    description: "Send drafts to channels, receive mentions",
    category: "native",
  },
  {
    type: "sheets_relay",
    name: "Google Sheets Relay",
    icon: "📊",
    description: "Import meeting data from Google Sheets",
    category: "native",
  },
];

const MCP_INTEGRATIONS: IntegrationTypeInfo[] = [
  {
    type: "mcp_server",
    name: "MCP Server",
    icon: "🔌",
    description: "Connect AI tools via Model Context Protocol",
    category: "mcp",
  },
];

// ─── Helper Components ────────────────────────────────────────────────────────

function CategoryHeader({
  title,
  description,
  icon,
  count,
  expanded,
  onToggle,
}: {
  title: string;
  description: string;
  icon: React.ReactNode;
  count: number;
  expanded: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className="w-full flex items-center justify-between p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
    >
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-lg bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 flex items-center justify-center">
          {icon}
        </div>
        <div className="text-left">
          <div className="font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
            {title}
            {count > 0 && (
              <span className="px-1.5 py-0.5 text-xs font-medium bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded">
                {count} connected
              </span>
            )}
          </div>
          <div className="text-xs text-zinc-500 mt-0.5">{description}</div>
        </div>
      </div>
      {expanded ? (
        <ChevronDown className="w-5 h-5 text-zinc-400" />
      ) : (
        <ChevronRight className="w-5 h-5 text-zinc-400" />
      )}
    </button>
  );
}

function IntegrationListItem({
  info,
  integration,
  isConnected,
  onConnect,
  onSettings,
}: {
  info: IntegrationTypeInfo;
  integration?: Integration;
  isConnected: boolean;
  onConnect: () => void;
  onSettings: () => void;
}) {
  return (
    <div
      className={`flex items-center justify-between p-4 rounded-lg border transition-colors ${
        isConnected
          ? "bg-green-50 dark:bg-green-900/10 border-green-200 dark:border-green-800"
          : "bg-white dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
      }`}
    >
      <div className="flex items-center gap-3">
        <span className="text-2xl">{info.icon}</span>
        <div>
          <div className="font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
            {info.name}
            {isConnected && (
              <CheckCircle2 className="w-4 h-4 text-green-500" />
            )}
          </div>
          <div className="text-xs text-zinc-500 mt-0.5">{info.description}</div>
          {integration?.last_sync && (
            <div className="flex items-center gap-1 text-xs text-zinc-400 mt-1">
              <Clock className="w-3 h-3" />
              Last synced: {new Date(integration.last_sync).toLocaleString()}
            </div>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {isConnected ? (
          <>
            <button
              onClick={onSettings}
              className="px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors"
            >
              Settings
            </button>
          </>
        ) : (
          <button
            onClick={onConnect}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md transition-colors"
          >
            <Plus className="w-4 h-4" />
            Connect
          </button>
        )}
      </div>
    </div>
  );
}

function LegacyConnectionCard({
  type,
  name,
  icon,
  description,
  isConnected,
  lastSync,
  onConnect,
  onDisconnect,
  isConnecting,
  error,
}: {
  type: string;
  name: string;
  icon: string;
  description: string;
  isConnected: boolean;
  lastSync?: string;
  onConnect: () => void;
  onDisconnect: () => void;
  isConnecting: boolean;
  error?: string;
}) {
  const [showConfirm, setShowConfirm] = useState(false);

  return (
    <div
      className={`flex items-center justify-between p-4 rounded-lg border transition-colors ${
        isConnected
          ? "bg-green-50 dark:bg-green-900/10 border-green-200 dark:border-green-800"
          : "bg-white dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700"
      }`}
    >
      <div className="flex items-center gap-3">
        <span className="text-2xl">{icon}</span>
        <div>
          <div className="font-medium text-zinc-900 dark:text-zinc-100 flex items-center gap-2">
            {name}
            {isConnected && <CheckCircle2 className="w-4 h-4 text-green-500" />}
          </div>
          <div className="text-xs text-zinc-500 mt-0.5">{description}</div>
          {lastSync && (
            <div className="flex items-center gap-1 text-xs text-zinc-400 mt-1">
              <Clock className="w-3 h-3" />
              Last synced: {new Date(lastSync).toLocaleString()}
            </div>
          )}
          {error && (
            <div className="flex items-center gap-1 text-xs text-red-500 mt-1">
              <AlertCircle className="w-3 h-3" />
              {error}
            </div>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {isConnected ? (
          showConfirm ? (
            <div className="flex items-center gap-2">
              <button
                onClick={() => {
                  onDisconnect();
                  setShowConfirm(false);
                }}
                className="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md"
              >
                Confirm
              </button>
              <button
                onClick={() => setShowConfirm(false)}
                className="px-3 py-1.5 text-sm text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md"
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              onClick={() => setShowConfirm(true)}
              className="px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors"
            >
              Disconnect
            </button>
          )
        ) : (
          <button
            onClick={onConnect}
            disabled={isConnecting}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md transition-colors disabled:opacity-50"
          >
            {isConnecting ? (
              <RefreshCw className="w-4 h-4 animate-spin" />
            ) : (
              <Plus className="w-4 h-4" />
            )}
            Connect
          </button>
        )}
      </div>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

export function IntegrationsPage() {
  // State
  const [nativeExpanded, setNativeExpanded] = useState(true);
  const [mcpExpanded, setMcpExpanded] = useState(true);
  const [showSetupWizard, setShowSetupWizard] = useState(false);
  const [setupType, setSetupType] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState<string | null>(null);

  // Data hooks
  const { data: integrations = [], isLoading } = useIntegrations();
  const syncMutation = useSyncIntegration();

  // Legacy connections (Zoom, Sheets Relay)
  const {
    zoom,
    sheetsRelay,
    connectZoom,
    isConnectingZoom,
    disconnect,
    zoomError,
  } = useConnections();

  // Helper functions
  const getIntegration = (type: string) =>
    integrations.find((i) => i.type === type);

  const isIntegrationConnected = (type: string) => {
    if (type === "zoom") return !!zoom;
    if (type === "sheets_relay") return !!sheetsRelay;
    return integrations.some((i) => i.type === type && i.status === "connected");
  };

  const handleConnect = (type: string) => {
    if (type === "zoom") {
      setSetupType("zoom");
      setShowSetupWizard(true);
    } else if (type === "sheets_relay") {
      setSetupType("sheets_relay");
      setShowSetupWizard(true);
    } else {
      setSetupType(type);
      setShowSetupWizard(true);
    }
  };

  const handleSettings = (type: string) => {
    setShowSettings(type);
  };

  const handleSyncAll = () => {
    integrations
      .filter((i) => i.status === "connected")
      .forEach((i) => syncMutation.mutate(i.id));
  };

  // Count connected
  const connectedNativeCount = AVAILABLE_INTEGRATIONS.filter((i) =>
    isIntegrationConnected(i.type)
  ).length;
  const connectedMcpCount = MCP_INTEGRATIONS.filter((i) =>
    isIntegrationConnected(i.type)
  ).length;

  if (isLoading) {
    return (
      <div className="p-6 text-zinc-500 flex items-center gap-2">
        <RefreshCw className="w-4 h-4 animate-spin" />
        Loading integrations...
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
            Integrations & Connections
          </h2>
          <p className="text-sm text-zinc-500 mt-1">
            Connect external services to sync data and enable AI-powered automation
          </p>
        </div>
        {connectedNativeCount + connectedMcpCount > 0 && (
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

      {/* Native Integrations Section */}
      <div className="space-y-3">
        <CategoryHeader
          title="Native Integrations"
          description="Direct connections to external services with OAuth"
          icon={<Plug className="w-5 h-5 text-blue-500" />}
          count={connectedNativeCount}
          expanded={nativeExpanded}
          onToggle={() => setNativeExpanded(!nativeExpanded)}
        />

        {nativeExpanded && (
          <div className="space-y-2 ml-4 pl-4 border-l-2 border-zinc-200 dark:border-zinc-700">
            {/* Zoom - Legacy connection */}
            <LegacyConnectionCard
              type="zoom"
              name="Zoom"
              icon="🎥"
              description="Import meeting recordings and transcripts"
              isConnected={!!zoom}
              lastSync={zoom?.last_sync_at ?? undefined}
              onConnect={connectZoom}
              onDisconnect={() => disconnect("zoom")}
              isConnecting={isConnectingZoom}
              error={zoomError ?? undefined}
            />

            {/* Sheets Relay - Legacy connection */}
            <LegacyConnectionCard
              type="sheets_relay"
              name="Google Sheets Relay"
              icon="📊"
              description="Import meeting data from Google Sheets"
              isConnected={!!sheetsRelay}
              lastSync={sheetsRelay?.last_sync_at ?? undefined}
              onConnect={() => handleConnect("sheets_relay")}
              onDisconnect={() => disconnect("sheets_relay")}
              isConnecting={false}
            />

            {/* GitHub */}
            <IntegrationListItem
              info={AVAILABLE_INTEGRATIONS.find((i) => i.type === "github")!}
              integration={getIntegration("github")}
              isConnected={isIntegrationConnected("github")}
              onConnect={() => handleConnect("github")}
              onSettings={() => handleSettings("github")}
            />

            {/* Jira */}
            <IntegrationListItem
              info={AVAILABLE_INTEGRATIONS.find((i) => i.type === "jira")!}
              integration={getIntegration("jira")}
              isConnected={isIntegrationConnected("jira")}
              onConnect={() => handleConnect("jira")}
              onSettings={() => handleSettings("jira")}
            />

            {/* Slack */}
            <IntegrationListItem
              info={AVAILABLE_INTEGRATIONS.find((i) => i.type === "slack")!}
              integration={getIntegration("slack")}
              isConnected={isIntegrationConnected("slack")}
              onConnect={() => handleConnect("slack")}
              onSettings={() => handleSettings("slack")}
            />
          </div>
        )}
      </div>

      {/* MCP Section */}
      <div className="space-y-3">
        <CategoryHeader
          title="MCP Servers"
          description="AI tool connections via Model Context Protocol"
          icon={<Server className="w-5 h-5 text-purple-500" />}
          count={connectedMcpCount}
          expanded={mcpExpanded}
          onToggle={() => setMcpExpanded(!mcpExpanded)}
        />

        {mcpExpanded && (
          <div className="space-y-2 ml-4 pl-4 border-l-2 border-zinc-200 dark:border-zinc-700">
            {/* MCP Server info card */}
            <div className="p-4 rounded-lg border border-purple-200 dark:border-purple-800 bg-purple-50 dark:bg-purple-900/10">
              <div className="flex items-start gap-3">
                <span className="text-2xl">🔌</span>
                <div className="flex-1">
                  <div className="font-medium text-zinc-900 dark:text-zinc-100">
                    Meridian MCP Server
                  </div>
                  <div className="text-xs text-zinc-500 mt-0.5">
                    Expose Meridian data to AI tools via Model Context Protocol
                  </div>
                  <div className="mt-3 p-3 bg-zinc-100 dark:bg-zinc-900 rounded text-xs font-mono text-zinc-700 dark:text-zinc-300">
                    cd src-tauri/meridian-mcp && cargo run -- --port 3100
                  </div>
                </div>
                <button
                  onClick={() => handleSettings("mcp")}
                  className="px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md transition-colors"
                >
                  Settings
                </button>
              </div>
            </div>

            {/* Help text */}
            <div className="flex items-start gap-2 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <AlertCircle className="w-4 h-4 text-zinc-400 mt-0.5 flex-shrink-0" />
              <div className="text-xs text-zinc-500">
                <strong>MCP vs Native Integrations:</strong> MCP servers allow AI assistants
                (like Claude) to read and write Meridian data. Native integrations sync data
                directly with external services like GitHub or Jira.
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Setup Wizard Modal */}
      {showSetupWizard && setupType && (
        <SetupWizard
          integrationType={setupType}
          isMcp={MCP_INTEGRATIONS.some((i) => i.type === setupType)}
          onClose={() => {
            setShowSetupWizard(false);
            setSetupType(null);
          }}
          onComplete={() => {
            setShowSetupWizard(false);
            setSetupType(null);
          }}
        />
      )}

      {/* Settings Modals */}
      {showSettings === "github" && (
        <GitHubSettings
          integration={getIntegration("github")}
          onClose={() => setShowSettings(null)}
        />
      )}
      {showSettings === "jira" && (
        <JiraSettings
          integration={getIntegration("jira")}
          onClose={() => setShowSettings(null)}
        />
      )}
      {showSettings === "slack" && (
        <SlackSettings
          integration={getIntegration("slack")}
          onClose={() => setShowSettings(null)}
        />
      )}
      {showSettings === "mcp" && (
        <MCPSettings onClose={() => setShowSettings(null)} />
      )}
    </div>
  );
}
