import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Search, Filter, Download, ChevronDown, ChevronRight,
  Clock, User, Zap, AlertTriangle, Loader, X, Maximize2, Minimize2
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface AuditLogEntry {
  id: string;
  timestamp: string;
  action_type: string;
  entity_type: string;
  entity_id: string;
  user_id: string | null;
  session_id: string | null;
  summary: string;
  details: string | null;
  before_state: string | null;
  after_state: string | null;
  risk_level: string;
  external_effects: boolean;
  agent_initiated: boolean;
  autonomy_mode: string | null;
}

interface AuditLogFilter {
  action_type?: string;
  entity_type?: string;
  entity_id?: string;
  risk_level?: string;
  agent_initiated?: boolean;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}

const ACTION_TYPES = [
  "create", "update", "delete", "archive", "unarchive",
  "bulk_update", "move", "ingest", "sync", "reorder"
];

const ENTITY_TYPES = ["task", "meeting", "project", "connection", "pending_import"];

const RISK_LEVELS = ["low", "medium", "high", "critical"];

const RISK_COLORS: Record<string, string> = {
  low: "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400",
  medium: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
  high: "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400",
  critical: "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
};

function getDefaultDateFrom(): string {
  const date = new Date();
  date.setDate(date.getDate() - 7);
  return date.toISOString().split('T')[0];
}

export default function AuditLogViewer() {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [filters, setFilters] = useState<AuditLogFilter>({
    limit: 50,
    date_from: getDefaultDateFrom(),
  });
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [total, setTotal] = useState(0);
  const [fullscreen, setFullscreen] = useState(false);

  const fetchLogs = useCallback(async (appendMode = false) => {
    if (appendMode) {
      setLoadingMore(true);
    } else {
      setLoading(true);
    }
    try {
      const result = await invoke<{ entries: AuditLogEntry[]; total: number; has_more: boolean }>("get_audit_log", {
        filter: {
          ...filters,
          offset: appendMode ? entries.length : 0,
        }
      });
      if (appendMode) {
        setEntries(prev => [...prev, ...result.entries]);
      } else {
        setEntries(result.entries);
      }
      setHasMore(result.has_more);
      setTotal(result.total);
    } catch (e) {
      console.error("Failed to fetch audit log:", e);
      if (!appendMode) setEntries([]);
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, [filters, entries.length]);

  useEffect(() => {
    fetchLogs();
  }, [filters.action_type, filters.entity_type, filters.risk_level, filters.agent_initiated, filters.date_from, filters.date_to]);

  const handleExport = async (format: "json" | "csv") => {
    try {
      const result = await invoke<string>("export_audit_log", { filter: filters, format });
      // Result is the file path - could show a toast here
      console.log("Exported to:", result);
    } catch (e) {
      console.error("Export failed:", e);
    }
  };

  const formatDate = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const formatActionType = (action: string) => {
    return action.split("_").map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
  };

  const content = (
    <div className={fullscreen ? "h-full flex flex-col" : "space-y-4"}>
      {/* Header */}
      <div className={`flex items-center justify-between ${fullscreen ? "px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex-shrink-0" : ""}`}>
        <h3 className={`font-semibold text-zinc-900 dark:text-zinc-50 ${fullscreen ? "text-base" : "text-sm"}`}>
          Activity Log
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`flex items-center gap-1.5 px-2.5 py-1.5 text-xs rounded-lg border transition-colors ${
              showFilters
                ? "border-indigo-200 bg-indigo-50 text-indigo-600 dark:border-indigo-800 dark:bg-indigo-900/20 dark:text-indigo-400"
                : "border-zinc-200 text-zinc-500 hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
            }`}
          >
            <Filter className="w-3.5 h-3.5" />
            Filters
          </button>
          <div className="relative group">
            <button className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700">
              <Download className="w-3.5 h-3.5" />
              Export
            </button>
            <div className="absolute right-0 top-full mt-1 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
              <button
                onClick={() => handleExport("json")}
                className="block w-full px-3 py-2 text-xs text-left text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-700 rounded-t-lg"
              >
                Export as JSON
              </button>
              <button
                onClick={() => handleExport("csv")}
                className="block w-full px-3 py-2 text-xs text-left text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-700 rounded-b-lg"
              >
                Export as CSV
              </button>
            </div>
          </div>
          <button
            onClick={() => setFullscreen(!fullscreen)}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700"
            title={fullscreen ? "Exit fullscreen" : "Fullscreen"}
          >
            {fullscreen ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Filters */}
      {showFilters && (
        <div className={`p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg space-y-3 flex-shrink-0 ${fullscreen ? "mx-6" : ""}`}>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs text-zinc-500 mb-1">Action Type</label>
              <select
                value={filters.action_type || ""}
                onChange={(e) => setFilters(f => ({ ...f, action_type: e.target.value || undefined }))}
                className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
              >
                <option value="">All actions</option>
                {ACTION_TYPES.map(a => (
                  <option key={a} value={a}>{formatActionType(a)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs text-zinc-500 mb-1">Entity Type</label>
              <select
                value={filters.entity_type || ""}
                onChange={(e) => setFilters(f => ({ ...f, entity_type: e.target.value || undefined }))}
                className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
              >
                <option value="">All entities</option>
                {ENTITY_TYPES.map(e => (
                  <option key={e} value={e}>{e.charAt(0).toUpperCase() + e.slice(1)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs text-zinc-500 mb-1">Risk Level</label>
              <select
                value={filters.risk_level || ""}
                onChange={(e) => setFilters(f => ({ ...f, risk_level: e.target.value || undefined }))}
                className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
              >
                <option value="">All levels</option>
                {RISK_LEVELS.map(r => (
                  <option key={r} value={r}>{r.charAt(0).toUpperCase() + r.slice(1)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs text-zinc-500 mb-1">Source</label>
              <select
                value={filters.agent_initiated === undefined ? "" : filters.agent_initiated ? "agent" : "user"}
                onChange={(e) => setFilters(f => ({
                  ...f,
                  agent_initiated: e.target.value === "" ? undefined : e.target.value === "agent"
                }))}
                className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
              >
                <option value="">All sources</option>
                <option value="user">User actions</option>
                <option value="agent">Agent actions</option>
              </select>
            </div>
          </div>
          <div className="pt-2 border-t border-zinc-200 dark:border-zinc-700 space-y-2">
            <div className="flex items-center gap-2">
              <label className="text-xs text-zinc-500">Quick:</label>
              {[
                { label: "Today", days: 0 },
                { label: "7 days", days: 7 },
                { label: "30 days", days: 30 },
                { label: "All", days: null },
              ].map(({ label, days }) => {
                const isActive = days === null
                  ? !filters.date_from
                  : filters.date_from === (() => {
                      const d = new Date();
                      d.setDate(d.getDate() - days);
                      return d.toISOString().split('T')[0];
                    })();
                return (
                  <button
                    key={label}
                    onClick={() => {
                      if (days === null) {
                        setFilters(f => ({ ...f, date_from: undefined, date_to: undefined }));
                      } else {
                        const d = new Date();
                        d.setDate(d.getDate() - days);
                        setFilters(f => ({ ...f, date_from: d.toISOString().split('T')[0], date_to: undefined }));
                      }
                    }}
                    className={`px-2 py-1 text-[10px] rounded ${
                      isActive
                        ? "bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300"
                        : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200 dark:bg-zinc-700 dark:text-zinc-400 dark:hover:bg-zinc-600"
                    }`}
                  >
                    {label}
                  </button>
                );
              })}
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs text-zinc-500 mb-1">From Date</label>
                <input
                  type="date"
                  value={filters.date_from || ""}
                  onChange={(e) => setFilters(f => ({ ...f, date_from: e.target.value || undefined }))}
                  className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
                />
              </div>
              <div>
                <label className="block text-xs text-zinc-500 mb-1">To Date</label>
                <input
                  type="date"
                  value={filters.date_to || ""}
                  onChange={(e) => setFilters(f => ({ ...f, date_to: e.target.value || undefined }))}
                  className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Log entries */}
      <div className={`space-y-1 overflow-y-auto ${fullscreen ? "flex-1 px-6" : "max-h-[400px]"}`}>
        {loading && entries.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <Loader className="w-5 h-5 animate-spin text-zinc-400" />
          </div>
        ) : entries.length === 0 ? (
          <div className="text-center py-8 text-sm text-zinc-400">
            No activity logged yet
          </div>
        ) : (
          entries.map((entry) => (
            <div
              key={entry.id}
              className="border border-zinc-100 dark:border-zinc-800 rounded-lg overflow-hidden"
            >
              <button
                onClick={() => setExpandedId(expandedId === entry.id ? null : entry.id)}
                className="w-full px-3 py-2 flex items-center gap-3 text-left hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
              >
                {expandedId === entry.id ? (
                  <ChevronDown className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
                ) : (
                  <ChevronRight className="w-3.5 h-3.5 text-zinc-400 flex-shrink-0" />
                )}

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium text-zinc-900 dark:text-zinc-50 truncate">
                      {entry.summary}
                    </span>
                    {entry.agent_initiated && (
                      <span title="Agent action">
                        <Zap className="w-3 h-3 text-indigo-500 flex-shrink-0" />
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2 mt-0.5">
                    <span className="text-[10px] text-zinc-400">
                      {formatDate(entry.timestamp)}
                    </span>
                    <span className="text-[10px] text-zinc-300 dark:text-zinc-600">•</span>
                    <span className="text-[10px] text-zinc-400 capitalize">
                      {entry.entity_type}
                    </span>
                  </div>
                </div>

                <span className={`px-1.5 py-0.5 text-[10px] font-medium rounded ${RISK_COLORS[entry.risk_level] || RISK_COLORS.low}`}>
                  {entry.risk_level}
                </span>
              </button>

              {expandedId === entry.id && (
                <div className="px-3 py-2 bg-zinc-50 dark:bg-zinc-800/30 border-t border-zinc-100 dark:border-zinc-800 text-xs space-y-2">
                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <span className="text-zinc-400">Action:</span>{" "}
                      <span className="text-zinc-700 dark:text-zinc-300">{formatActionType(entry.action_type)}</span>
                    </div>
                    <div>
                      <span className="text-zinc-400">Entity ID:</span>{" "}
                      <span className="text-zinc-700 dark:text-zinc-300 font-mono">{entry.entity_id.slice(0, 8)}...</span>
                    </div>
                    {entry.session_id && (
                      <div>
                        <span className="text-zinc-400">Session:</span>{" "}
                        <span className="text-zinc-700 dark:text-zinc-300 font-mono">{entry.session_id.slice(0, 8)}...</span>
                      </div>
                    )}
                    {entry.external_effects && (
                      <div className="flex items-center gap-1 text-amber-600 dark:text-amber-400">
                        <AlertTriangle className="w-3 h-3" />
                        External effects
                      </div>
                    )}
                  </div>
                  {entry.details && (
                    <div className="pt-2 border-t border-zinc-200 dark:border-zinc-700">
                      <span className="text-zinc-400">Details:</span>
                      <pre className="mt-1 p-2 bg-zinc-100 dark:bg-zinc-800 rounded text-[10px] overflow-x-auto">
                        {JSON.stringify(JSON.parse(entry.details), null, 2)}
                      </pre>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))
        )}

        {hasMore && (
          <button
            onClick={() => fetchLogs(true)}
            disabled={loadingMore}
            className="w-full py-2 text-xs text-indigo-500 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded-lg transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {loadingMore ? (
              <>
                <Loader className="w-3 h-3 animate-spin" />
                Loading...
              </>
            ) : (
              `Load more (showing ${entries.length} of ${total})`
            )}
          </button>
        )}
      </div>

      {/* Footer with count and retention notice */}
      <div className={`pt-3 border-t border-zinc-100 dark:border-zinc-800 flex items-center justify-between ${fullscreen ? "px-6 pb-4 flex-shrink-0" : ""}`}>
        <p className="text-xs text-zinc-400">
          {entries.length > 0 && `Showing ${entries.length}${total > entries.length ? ` of ${total}` : ""} entries • `}
          Last 7 days by default
        </p>
        <p className="text-xs text-zinc-400">
          Retained for 2 years
        </p>
      </div>
    </div>
  );

  if (fullscreen) {
    return (
      <div className="fixed inset-0 z-[60] flex items-center justify-center">
        <div className="absolute inset-0 bg-black/50" onClick={() => setFullscreen(false)} />
        <div className="relative bg-white dark:bg-zinc-900 rounded-xl shadow-2xl w-[90vw] h-[90vh] max-w-6xl border border-zinc-200 dark:border-zinc-700 flex flex-col overflow-hidden">
          {content}
        </div>
      </div>
    );
  }

  return content;
}
