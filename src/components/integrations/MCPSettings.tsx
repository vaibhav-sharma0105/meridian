import { useState, useEffect } from "react";
import {
  X,
  Server,
  Check,
  Shield,
  AlertTriangle,
  Info,
  Copy,
  RefreshCw,
  Eye,
  Edit3,
  Trash2,
  Zap,
  Clock,
} from "lucide-react";
import * as api from "@/lib/tauri";
import toast from "react-hot-toast";

interface MCPSettingsProps {
  onClose: () => void;
}

import type { McpPermissions } from "@/lib/tauri";

const DEFAULT_PERMISSIONS: McpPermissions = {
  read_tasks: true,
  read_meetings: true,
  read_projects: true,
  create_task: false,
  update_task: false,
  delete_task: false,
  create_meeting_note: false,
  run_skill: false,
  rate_limit_per_minute: 100,
};

const PERMISSION_GROUPS = [
  {
    title: "Read Operations",
    description: "Allow AI tools to read data from Meridian",
    icon: <Eye className="w-4 h-4" />,
    permissions: [
      { key: "read_tasks", label: "Read Tasks", risk: "low" },
      { key: "read_meetings", label: "Read Meetings", risk: "low" },
      { key: "read_projects", label: "Read Projects", risk: "low" },
    ],
  },
  {
    title: "Write Operations",
    description: "Allow AI tools to create and modify data",
    icon: <Edit3 className="w-4 h-4" />,
    permissions: [
      { key: "create_task", label: "Create Tasks", risk: "medium" },
      { key: "update_task", label: "Update Tasks", risk: "medium" },
      { key: "create_meeting_note", label: "Create Meeting Notes", risk: "medium" },
    ],
  },
  {
    title: "Destructive Operations",
    description: "High-risk operations requiring extra caution",
    icon: <Trash2 className="w-4 h-4" />,
    permissions: [
      { key: "delete_task", label: "Delete Tasks", risk: "high" },
    ],
  },
  {
    title: "Automation",
    description: "Allow AI tools to trigger automations",
    icon: <Zap className="w-4 h-4" />,
    permissions: [
      { key: "run_skill", label: "Run Skills", risk: "high" },
    ],
  },
];

export function MCPSettings({ onClose }: MCPSettingsProps) {
  const [permissions, setPermissions] = useState<McpPermissions>(DEFAULT_PERMISSIONS);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [copied, setCopied] = useState(false);

  const mcpServerCommand = "cd src-tauri/meridian-mcp && cargo run -- --port 3100";
  const mcpEndpoint = "http://localhost:3100";

  useEffect(() => {
    loadPermissions();
  }, []);

  const loadPermissions = async () => {
    try {
      const perms = await api.getMcpPermissions();
      setPermissions({ ...DEFAULT_PERMISSIONS, ...perms });
    } catch (e) {
      console.error("Failed to load MCP permissions:", e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await api.setMcpPermissions(permissions);
      toast.success("MCP permissions saved");
    } catch (e) {
      toast.error("Failed to save permissions");
    } finally {
      setIsSaving(false);
    }
  };

  const togglePermission = (key: keyof McpPermissions) => {
    if (typeof permissions[key] === "boolean") {
      setPermissions((prev) => ({ ...prev, [key]: !prev[key] }));
    }
  };

  const handleCopy = async (text: string) => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    toast.success("Copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  const getRiskBadge = (risk: string) => {
    const styles = {
      low: "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300",
      medium: "bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300",
      high: "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300",
    };
    return (
      <span className={`px-1.5 py-0.5 text-[10px] font-medium rounded ${styles[risk as keyof typeof styles]}`}>
        {risk}
      </span>
    );
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 overflow-y-auto py-8">
      <div
        className="w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-200 dark:border-zinc-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center">
              <Server className="w-5 h-5 text-purple-600 dark:text-purple-400" />
            </div>
            <div>
              <h3 className="font-semibold text-zinc-900 dark:text-zinc-100">
                MCP Server Settings
              </h3>
              <p className="text-xs text-zinc-500">
                Configure AI tool access via Model Context Protocol
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

        {isLoading ? (
          <div className="p-6 flex items-center justify-center">
            <RefreshCw className="w-5 h-5 animate-spin text-zinc-400" />
          </div>
        ) : (
          <div className="p-6 space-y-6">
            {/* Server Info */}
            <div className="space-y-3">
              <div className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                Server Command
              </div>
              <div className="flex items-center gap-2">
                <code className="flex-1 px-3 py-2 text-xs bg-zinc-100 dark:bg-zinc-800 rounded-lg font-mono text-zinc-800 dark:text-zinc-200 overflow-x-auto">
                  {mcpServerCommand}
                </code>
                <button
                  onClick={() => handleCopy(mcpServerCommand)}
                  className="p-2 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded"
                >
                  {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
                </button>
              </div>
              <div className="flex items-center gap-2 text-xs text-zinc-500">
                <Info className="w-3 h-3" />
                Endpoint: <code className="font-mono">{mcpEndpoint}</code>
              </div>
            </div>

            {/* Rate Limiting */}
            <div>
              <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                Rate Limit
              </label>
              <div className="flex items-center gap-3">
                <input
                  type="number"
                  min={10}
                  max={1000}
                  value={permissions.rate_limit_per_minute}
                  onChange={(e) =>
                    setPermissions((prev) => ({
                      ...prev,
                      rate_limit_per_minute: Number(e.target.value),
                    }))
                  }
                  className="w-24 px-3 py-2 text-sm border border-zinc-300 dark:border-zinc-600 rounded-lg bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                />
                <span className="text-sm text-zinc-500">requests per minute</span>
              </div>
              <p className="text-xs text-zinc-500 mt-1 flex items-center gap-1">
                <Clock className="w-3 h-3" />
                Sliding window rate limiter protects against runaway requests
              </p>
            </div>

            {/* Permission Groups */}
            <div className="space-y-4">
              {PERMISSION_GROUPS.map((group) => (
                <div
                  key={group.title}
                  className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden"
                >
                  <div className="flex items-center gap-2 px-4 py-3 bg-zinc-50 dark:bg-zinc-800/50">
                    <span className="text-zinc-500">{group.icon}</span>
                    <div>
                      <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                        {group.title}
                      </div>
                      <div className="text-xs text-zinc-500">{group.description}</div>
                    </div>
                  </div>
                  <div className="divide-y divide-zinc-200 dark:divide-zinc-700">
                    {group.permissions.map((perm) => (
                      <label
                        key={perm.key}
                        className="flex items-center justify-between px-4 py-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 cursor-pointer"
                      >
                        <div className="flex items-center gap-3">
                          <input
                            type="checkbox"
                            checked={permissions[perm.key as keyof McpPermissions] as boolean}
                            onChange={() => togglePermission(perm.key as keyof McpPermissions)}
                            className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600 text-indigo-600 focus:ring-indigo-500"
                          />
                          <span className="text-sm text-zinc-700 dark:text-zinc-300">
                            {perm.label}
                          </span>
                        </div>
                        {getRiskBadge(perm.risk)}
                      </label>
                    ))}
                  </div>
                </div>
              ))}
            </div>

            {/* Warning */}
            <div className="flex items-start gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
              <AlertTriangle className="w-4 h-4 text-amber-500 mt-0.5 flex-shrink-0" />
              <div className="text-xs text-amber-700 dark:text-amber-300">
                <strong>Security note:</strong> Write and delete operations allow AI tools
                to modify your data. All MCP operations are logged to the audit log with{" "}
                <code className="font-mono">agent_initiated: true</code>.
              </div>
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
            onClick={handleSave}
            disabled={isSaving}
            className="px-4 py-2 text-sm font-medium bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isSaving ? "Saving..." : "Save Permissions"}
          </button>
        </div>
      </div>
    </div>
  );
}
