import { useState, useEffect } from "react";
import {
  X,
  ExternalLink,
  Loader2,
  Check,
  Copy,
  ChevronRight,
  AlertCircle,
  RefreshCw,
  Key,
  Globe,
  Shield,
  Server,
} from "lucide-react";
import { useStartOAuth, useHandleOAuthCallback, useCreateIntegration } from "@/hooks/useIntegrations";
import { useIntegrationStore } from "@/stores/integrationStore";
import { openUrl } from "@/lib/tauri";
import toast from "react-hot-toast";

// ─── Types ────────────────────────────────────────────────────────────────────

interface SetupWizardProps {
  integrationType: string;
  isMcp?: boolean;
  onClose: () => void;
  onComplete?: () => void;
}

interface SetupStep {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  action?: "manual" | "oauth" | "api_key" | "none";
  completed?: boolean;
}

type IntegrationCategory = "native" | "mcp";

// ─── Setup Instructions per Integration ───────────────────────────────────────

const INTEGRATION_SETUP: Record<
  string,
  {
    name: string;
    category: IntegrationCategory;
    icon: string;
    description: string;
    prerequisites: string[];
    steps: SetupStep[];
    oauthConfig?: {
      scopes: string[];
      redirectUri: string;
    };
    apiKeyConfig?: {
      label: string;
      placeholder: string;
      helpUrl?: string;
    };
  }
> = {
  github: {
    name: "GitHub",
    category: "native",
    icon: "🐙",
    description: "Sync issues and PRs, link tasks to GitHub items",
    prerequisites: [
      "GitHub account with access to target repositories",
      "GitHub OAuth App (created in Settings → Developer settings → OAuth Apps)",
    ],
    steps: [
      {
        id: "create_oauth_app",
        title: "Create GitHub OAuth App",
        description:
          "Go to GitHub Settings → Developer settings → OAuth Apps → New OAuth App. Set callback URL to http://localhost:8765/oauth/callback",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_credentials",
        title: "Configure Client Credentials",
        description: "Enter your OAuth App's Client ID and Client Secret",
        icon: <Shield className="w-4 h-4" />,
        action: "api_key",
      },
      {
        id: "authorize",
        title: "Authorize Meridian",
        description:
          "Click to open GitHub and grant Meridian access to your repositories",
        icon: <Globe className="w-4 h-4" />,
        action: "oauth",
      },
    ],
    oauthConfig: {
      scopes: ["repo", "read:user"],
      redirectUri: "http://localhost:8765/oauth/callback",
    },
  },
  jira: {
    name: "Jira",
    category: "native",
    icon: "🔷",
    description: "Sync Jira issues, link tasks to tickets",
    prerequisites: [
      "Atlassian account with Jira Cloud access",
      "OAuth 2.0 (3LO) app created in Atlassian Developer Console",
    ],
    steps: [
      {
        id: "create_oauth_app",
        title: "Create Atlassian OAuth App",
        description:
          "Go to developer.atlassian.com → Create new app → Configure OAuth 2.0 (3LO). Add callback URL: http://localhost:8765/oauth/callback",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_permissions",
        title: "Configure Permissions",
        description:
          "In your app settings, add Jira API scopes: read:jira-work, write:jira-work, read:jira-user",
        icon: <Shield className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_credentials",
        title: "Enter Client Credentials",
        description: "Enter your OAuth app's Client ID and Secret",
        icon: <Key className="w-4 h-4" />,
        action: "api_key",
      },
      {
        id: "authorize",
        title: "Authorize Meridian",
        description: "Click to open Atlassian and grant access to your Jira projects",
        icon: <Globe className="w-4 h-4" />,
        action: "oauth",
      },
    ],
    oauthConfig: {
      scopes: ["read:jira-work", "write:jira-work", "read:jira-user", "offline_access"],
      redirectUri: "http://localhost:8765/oauth/callback",
    },
  },
  slack: {
    name: "Slack",
    category: "native",
    icon: "💬",
    description: "Send drafts to channels, receive mentions",
    prerequisites: [
      "Slack workspace admin access",
      "Slack App created at api.slack.com/apps",
    ],
    steps: [
      {
        id: "create_slack_app",
        title: "Create Slack App",
        description:
          "Go to api.slack.com/apps → Create New App → From scratch. Choose your workspace.",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_oauth",
        title: "Configure OAuth Scopes",
        description:
          "In OAuth & Permissions, add Bot Token Scopes: channels:read, chat:write, app_mentions:read. Add Redirect URL: http://localhost:8765/oauth/callback",
        icon: <Shield className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_credentials",
        title: "Enter Client Credentials",
        description: "Enter your Slack app's Client ID and Secret from Basic Information",
        icon: <Key className="w-4 h-4" />,
        action: "api_key",
      },
      {
        id: "authorize",
        title: "Install to Workspace",
        description: "Click to install the Slack app to your workspace",
        icon: <Globe className="w-4 h-4" />,
        action: "oauth",
      },
    ],
    oauthConfig: {
      scopes: ["channels:read", "chat:write", "app_mentions:read"],
      redirectUri: "http://localhost:8765/oauth/callback",
    },
  },
  zoom: {
    name: "Zoom",
    category: "native",
    icon: "🎥",
    description: "Import meeting recordings and transcripts",
    prerequisites: [
      "Zoom account with recording access",
      "Zoom OAuth App created in Zoom Marketplace",
    ],
    steps: [
      {
        id: "create_zoom_app",
        title: "Create Zoom OAuth App",
        description:
          "Go to marketplace.zoom.us → Develop → Build App → OAuth. Choose User-managed app.",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_scopes",
        title: "Add Required Scopes",
        description:
          "In Scopes section, add: meeting:read, recording:read, user:read. Set Redirect URL to: http://localhost:8766/callback",
        icon: <Shield className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_credentials",
        title: "Configure Credentials",
        description:
          "Set ZOOM_CLIENT_ID and ZOOM_CLIENT_SECRET environment variables, then restart Meridian",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "authorize",
        title: "Connect Zoom Account",
        description: "Click to open Zoom and authorize Meridian to access your recordings",
        icon: <Globe className="w-4 h-4" />,
        action: "oauth",
      },
    ],
    oauthConfig: {
      scopes: ["meeting:read", "recording:read", "user:read"],
      redirectUri: "http://localhost:8766/callback",
    },
  },
  sheets_relay: {
    name: "Google Sheets Relay",
    category: "native",
    icon: "📊",
    description: "Import meeting data from Google Sheets via Apps Script",
    prerequisites: [
      "Google account with Sheets access",
      "Google Apps Script deployment",
    ],
    steps: [
      {
        id: "create_sheet",
        title: "Create Import Sheet",
        description:
          "Create a new Google Sheet with columns: import_id, created_at, title, meeting_date, summary, action_items, source_subject",
        icon: <Key className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "deploy_script",
        title: "Deploy Apps Script",
        description:
          "Go to Extensions → Apps Script. Paste the relay script, run setSecretKey() once, then deploy as Web App (execute as: me, access: anyone)",
        icon: <Server className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure",
        title: "Configure Relay",
        description: "Enter the Web App URL and your secret key",
        icon: <Key className="w-4 h-4" />,
        action: "api_key",
      },
    ],
    apiKeyConfig: {
      label: "Web App URL & Secret Key",
      placeholder: "https://script.google.com/macros/s/.../exec",
      helpUrl: "https://developers.google.com/apps-script/guides/web",
    },
  },
  mcp_server: {
    name: "MCP Server",
    category: "mcp",
    icon: "🔌",
    description: "Connect AI tools via Model Context Protocol",
    prerequisites: [
      "MCP server running locally or remotely",
      "Server URL and optional API key",
    ],
    steps: [
      {
        id: "install_server",
        title: "Start MCP Server",
        description:
          "Run the Meridian MCP server: cd src-tauri/meridian-mcp && cargo run -- --port 3100",
        icon: <Server className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "configure_permissions",
        title: "Configure Permissions",
        description:
          "Set which operations MCP clients can perform: read tasks, create tasks, update tasks, run skills",
        icon: <Shield className="w-4 h-4" />,
        action: "manual",
      },
      {
        id: "test_connection",
        title: "Test Connection",
        description: "Verify the MCP server is accessible and responding",
        icon: <Globe className="w-4 h-4" />,
        action: "none",
      },
    ],
  },
};

// ─── Helper Components ────────────────────────────────────────────────────────

function CopyButton({ text, label }: { text: string; label?: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    toast.success("Copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
    >
      {copied ? <Check className="w-3 h-3 text-green-500" /> : <Copy className="w-3 h-3" />}
      {label ?? (copied ? "Copied!" : "Copy")}
    </button>
  );
}

function StepIndicator({
  step,
  index,
  isActive,
  isCompleted,
}: {
  step: SetupStep;
  index: number;
  isActive: boolean;
  isCompleted: boolean;
}) {
  return (
    <div
      className={`flex items-start gap-3 p-3 rounded-lg transition-colors ${
        isActive
          ? "bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800"
          : isCompleted
          ? "bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800"
          : "bg-zinc-50 dark:bg-zinc-800/50 border border-transparent"
      }`}
    >
      <div
        className={`flex-shrink-0 w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold ${
          isCompleted
            ? "bg-green-500 text-white"
            : isActive
            ? "bg-indigo-500 text-white"
            : "bg-zinc-200 dark:bg-zinc-700 text-zinc-500 dark:text-zinc-400"
        }`}
      >
        {isCompleted ? <Check className="w-4 h-4" /> : index + 1}
      </div>
      <div className="flex-1 min-w-0">
        <div
          className={`font-medium text-sm ${
            isActive
              ? "text-indigo-900 dark:text-indigo-100"
              : isCompleted
              ? "text-green-800 dark:text-green-200"
              : "text-zinc-600 dark:text-zinc-400"
          }`}
        >
          {step.title}
        </div>
        <div
          className={`text-xs mt-0.5 ${
            isActive
              ? "text-indigo-700 dark:text-indigo-300"
              : "text-zinc-500 dark:text-zinc-500"
          }`}
        >
          {step.description}
        </div>
      </div>
      {step.icon}
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

export function SetupWizard({ integrationType, isMcp, onClose, onComplete }: SetupWizardProps) {
  const config = INTEGRATION_SETUP[integrationType];
  const [currentStep, setCurrentStep] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<Set<number>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // OAuth state
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");

  // Hooks
  const startOAuth = useStartOAuth();
  const handleCallback = useHandleOAuthCallback();
  const createIntegration = useCreateIntegration();
  const { startOAuth: setOAuthState, completeOAuth, failOAuth } = useIntegrationStore();

  if (!config) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="bg-white dark:bg-zinc-900 rounded-xl p-6 max-w-md">
          <p className="text-red-500">Unknown integration type: {integrationType}</p>
          <button onClick={onClose} className="mt-4 px-4 py-2 bg-zinc-200 rounded">
            Close
          </button>
        </div>
      </div>
    );
  }

  const steps = config.steps;
  const currentStepConfig = steps[currentStep];
  const isLastStep = currentStep === steps.length - 1;

  const handleMarkComplete = () => {
    setCompletedSteps((prev) => new Set([...prev, currentStep]));
    if (!isLastStep) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handleStartOAuth = async () => {
    if (!clientId) {
      setError("Please enter Client ID first");
      return;
    }

    setError(null);
    setIsLoading(true);

    try {
      const redirectUri = config.oauthConfig?.redirectUri || "http://localhost:8765/oauth/callback";
      const authUrl = await startOAuth.mutateAsync({
        integrationType,
        redirectUri,
      });

      setOAuthState(integrationType, authUrl);
      await openUrl(authUrl);

      // Wait for callback (in a real implementation, this would listen for the callback)
      toast.success("Complete authorization in your browser, then return here");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to start OAuth flow");
      failOAuth(e instanceof Error ? e.message : "Failed");
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateIntegration = async () => {
    setError(null);
    setIsLoading(true);

    try {
      await createIntegration.mutateAsync({
        integration_type: integrationType,
        name: config.name,
        config: {
          client_id: clientId || undefined,
          client_secret: clientSecret || undefined,
          api_token: apiKey || undefined,
          base_url: baseUrl || undefined,
        },
      });

      completeOAuth();
      toast.success(`${config.name} connected successfully!`);
      onComplete?.();
      setTimeout(onClose, 1000);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create integration");
    } finally {
      setIsLoading(false);
    }
  };

  const handleCompleteSetup = () => {
    if (currentStepConfig.action === "oauth") {
      handleStartOAuth();
    } else if (currentStepConfig.action === "api_key") {
      handleMarkComplete();
    } else {
      handleMarkComplete();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-2xl bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <span className="text-2xl">{config.icon}</span>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                Connect {config.name}
              </h3>
              <p className="text-xs text-zinc-500 mt-0.5">{config.description}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span
              className={`px-2 py-0.5 text-xs font-medium rounded-full ${
                config.category === "mcp"
                  ? "bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300"
                  : "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
              }`}
            >
              {config.category === "mcp" ? "MCP Server" : "Integration"}
            </span>
            <button
              onClick={onClose}
              className="p-1.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Prerequisites */}
        {config.prerequisites.length > 0 && currentStep === 0 && (
          <div className="px-6 py-4 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800">
            <div className="flex items-start gap-2">
              <AlertCircle className="w-4 h-4 text-amber-600 dark:text-amber-400 mt-0.5 flex-shrink-0" />
              <div>
                <div className="text-sm font-medium text-amber-800 dark:text-amber-200">
                  Prerequisites
                </div>
                <ul className="mt-1 text-xs text-amber-700 dark:text-amber-300 space-y-1">
                  {config.prerequisites.map((prereq, i) => (
                    <li key={i} className="flex items-start gap-1.5">
                      <span className="text-amber-500">•</span>
                      {prereq}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        )}

        {/* Steps Overview */}
        <div className="px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="text-xs font-medium text-zinc-500 uppercase tracking-wide mb-3">
            Setup Steps ({completedSteps.size}/{steps.length} complete)
          </div>
          <div className="space-y-2">
            {steps.map((step, index) => (
              <button
                key={step.id}
                onClick={() => setCurrentStep(index)}
                className="w-full text-left"
              >
                <StepIndicator
                  step={step}
                  index={index}
                  isActive={currentStep === index}
                  isCompleted={completedSteps.has(index)}
                />
              </button>
            ))}
          </div>
        </div>

        {/* Current Step Content */}
        <div className="px-6 py-5">
          <div className="flex items-center gap-2 mb-4">
            <div className="w-8 h-8 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center text-indigo-600 dark:text-indigo-400">
              {currentStepConfig.icon}
            </div>
            <div>
              <h4 className="font-medium text-zinc-900 dark:text-zinc-100">
                Step {currentStep + 1}: {currentStepConfig.title}
              </h4>
              <p className="text-xs text-zinc-500">{currentStepConfig.description}</p>
            </div>
          </div>

          {/* Action-specific content */}
          {currentStepConfig.action === "api_key" && (
            <div className="space-y-4 mt-4">
              {integrationType !== "sheets_relay" ? (
                <>
                  <div>
                    <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                      Client ID
                    </label>
                    <input
                      type="text"
                      value={clientId}
                      onChange={(e) => setClientId(e.target.value)}
                      placeholder="Enter Client ID"
                      className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                      Client Secret
                    </label>
                    <input
                      type="password"
                      value={clientSecret}
                      onChange={(e) => setClientSecret(e.target.value)}
                      placeholder="Enter Client Secret"
                      className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                    />
                  </div>
                </>
              ) : (
                <>
                  <div>
                    <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                      Web App URL
                    </label>
                    <input
                      type="url"
                      value={baseUrl}
                      onChange={(e) => setBaseUrl(e.target.value)}
                      placeholder={config.apiKeyConfig?.placeholder}
                      className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
                      Secret Key
                    </label>
                    <input
                      type="password"
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="Enter your secret key"
                      className="w-full px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                    />
                  </div>
                </>
              )}
            </div>
          )}

          {currentStepConfig.action === "manual" && integrationType === "github" && currentStep === 0 && (
            <div className="mt-4 p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <p className="text-sm text-zinc-600 dark:text-zinc-400 mb-3">
                Redirect URL to use in your GitHub OAuth App:
              </p>
              <div className="flex items-center gap-2">
                <code className="flex-1 px-3 py-2 text-xs bg-zinc-100 dark:bg-zinc-900 rounded font-mono text-zinc-800 dark:text-zinc-200">
                  http://localhost:8765/oauth/callback
                </code>
                <CopyButton text="http://localhost:8765/oauth/callback" />
              </div>
              <button
                onClick={() => openUrl("https://github.com/settings/developers")}
                className="mt-3 inline-flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Open GitHub Developer Settings
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            </div>
          )}

          {currentStepConfig.action === "manual" && integrationType === "jira" && currentStep === 0 && (
            <div className="mt-4 p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <p className="text-sm text-zinc-600 dark:text-zinc-400 mb-3">
                Callback URL to use in your Atlassian OAuth App:
              </p>
              <div className="flex items-center gap-2">
                <code className="flex-1 px-3 py-2 text-xs bg-zinc-100 dark:bg-zinc-900 rounded font-mono text-zinc-800 dark:text-zinc-200">
                  http://localhost:8765/oauth/callback
                </code>
                <CopyButton text="http://localhost:8765/oauth/callback" />
              </div>
              <button
                onClick={() => openUrl("https://developer.atlassian.com/console/myapps/")}
                className="mt-3 inline-flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Open Atlassian Developer Console
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            </div>
          )}

          {currentStepConfig.action === "manual" && integrationType === "slack" && currentStep === 0 && (
            <div className="mt-4 p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <button
                onClick={() => openUrl("https://api.slack.com/apps")}
                className="inline-flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Open Slack App Dashboard
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            </div>
          )}

          {currentStepConfig.action === "manual" && integrationType === "zoom" && currentStep === 0 && (
            <div className="mt-4 p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
              <button
                onClick={() => openUrl("https://marketplace.zoom.us/develop/create")}
                className="inline-flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Open Zoom Marketplace
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            </div>
          )}

          {/* Error message */}
          {error && (
            <div className="mt-4 p-3 text-sm text-red-600 bg-red-50 dark:bg-red-900/20 dark:text-red-400 rounded-lg flex items-start gap-2">
              <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
          <button
            onClick={() => setCurrentStep(Math.max(0, currentStep - 1))}
            disabled={currentStep === 0}
            className="px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Back
          </button>
          <div className="flex items-center gap-2">
            {currentStepConfig.action === "manual" && (
              <button
                onClick={handleMarkComplete}
                className="px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg transition-colors"
              >
                Mark as Done
              </button>
            )}
            {currentStepConfig.action === "api_key" && (
              <button
                onClick={handleMarkComplete}
                disabled={integrationType !== "sheets_relay" ? !clientId || !clientSecret : !baseUrl || !apiKey}
                className="px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Save & Continue
              </button>
            )}
            {currentStepConfig.action === "oauth" && (
              <button
                onClick={handleStartOAuth}
                disabled={isLoading}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                {isLoading ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <ExternalLink className="w-4 h-4" />
                )}
                Authorize {config.name}
              </button>
            )}
            {currentStepConfig.action === "none" && isLastStep && (
              <button
                onClick={handleCreateIntegration}
                disabled={isLoading}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                {isLoading ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Check className="w-4 h-4" />
                )}
                Complete Setup
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
