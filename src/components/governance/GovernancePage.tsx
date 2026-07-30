import { useState } from "react";
import { Shield, CheckSquare, History, BarChart3, Settings2 } from "lucide-react";
import { AutonomySettings } from "./AutonomySettings";
import { ApprovalQueue } from "./ApprovalQueue";
import { ActionHistoryPanel } from "./ActionHistoryPanel";
import { GovernanceDashboard } from "./GovernanceDashboard";

type Tab = "approvals" | "history" | "dashboard" | "settings";

export function GovernancePage() {
  const [activeTab, setActiveTab] = useState<Tab>("approvals");

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "approvals", label: "Approvals", icon: <CheckSquare className="w-4 h-4" /> },
    { id: "history", label: "History", icon: <History className="w-4 h-4" /> },
    { id: "dashboard", label: "Dashboard", icon: <BarChart3 className="w-4 h-4" /> },
    { id: "settings", label: "Settings", icon: <Settings2 className="w-4 h-4" /> },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-3 px-5 pt-4 pb-3 flex-shrink-0 border-b border-zinc-100 dark:border-zinc-800">
        <div className="p-2 rounded-lg bg-indigo-100 dark:bg-indigo-900/30">
          <Shield className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
        </div>
        <div>
          <h1 className="text-[17px] font-bold tracking-[-0.025em] text-zinc-900 dark:text-zinc-50">
            Governance
          </h1>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            Control agent autonomy, review pending actions, and monitor activity
          </p>
        </div>
      </div>

      {/* Tab bar */}
      <div className="flex gap-1 px-5 pt-3 pb-0 border-b border-zinc-100 dark:border-zinc-800">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-t-lg transition-colors ${
              activeTab === tab.id
                ? "text-indigo-600 dark:text-indigo-400 border-b-2 border-indigo-500 -mb-px bg-indigo-50/50 dark:bg-indigo-900/20"
                : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800/50"
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-5">
        {activeTab === "approvals" && <ApprovalQueue />}
        {activeTab === "history" && <ActionHistoryPanel limit={100} />}
        {activeTab === "dashboard" && <GovernanceDashboard />}
        {activeTab === "settings" && (
          <div className="max-w-lg">
            <AutonomySettings />
          </div>
        )}
      </div>
    </div>
  );
}
