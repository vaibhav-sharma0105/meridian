import { useState } from "react";
import {
  Users,
  UserPlus,
  RefreshCw,
  Gauge,
  Search,
  MoreVertical,
  Edit2,
  Trash2,
  AlertCircle,
} from "lucide-react";
import { useTeamMembers, useSyncTeamFromSlack, useComputeTeamWorkloads, useDeleteTeamMember } from "@/hooks/useTeam";
import { TeamMemberForm } from "./TeamMemberForm";
import { TeamMemberCard } from "./TeamMemberCard";
import type { TeamMember } from "@/lib/tauri";

// ─── Source Badge Colors ──────────────────────────────────────────────────────

const SOURCE_BADGES: Record<string, { bg: string; text: string; label: string }> = {
  manual: { bg: "bg-zinc-100 dark:bg-zinc-800", text: "text-zinc-600 dark:text-zinc-400", label: "Manual" },
  slack: { bg: "bg-purple-100 dark:bg-purple-900/30", text: "text-purple-600 dark:text-purple-400", label: "Slack" },
  google: { bg: "bg-blue-100 dark:bg-blue-900/30", text: "text-blue-600 dark:text-blue-400", label: "Google" },
};

// ─── Main Component ───────────────────────────────────────────────────────────

export function TeamSettings() {
  const { data: members = [], isLoading } = useTeamMembers();
  const syncSlack = useSyncTeamFromSlack();
  const computeWorkloads = useComputeTeamWorkloads();
  const deleteMember = useDeleteTeamMember();

  const [searchQuery, setSearchQuery] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingMember, setEditingMember] = useState<TeamMember | null>(null);
  const [sourceFilter, setSourceFilter] = useState<string | null>(null);

  // Filter members
  const filteredMembers = members.filter((member) => {
    const matchesSearch = !searchQuery ||
      member.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      member.email?.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesSource = !sourceFilter || member.source === sourceFilter;
    return matchesSearch && matchesSource;
  });

  // Group by source for stats
  const sourceStats = members.reduce((acc, m) => {
    acc[m.source] = (acc[m.source] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const handleSyncSlack = () => {
    syncSlack.mutate();
  };

  const handleRecomputeWorkloads = () => {
    computeWorkloads.mutate();
  };

  const handleDeleteMember = (id: string) => {
    if (confirm("Remove this team member?")) {
      deleteMember.mutate(id);
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex-shrink-0 px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg">
              <Users className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
                Team Roster
              </h2>
              <p className="text-sm text-zinc-500">
                {members.length} members across {Object.keys(sourceStats).length} sources
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleSyncSlack}
              disabled={syncSlack.isPending}
              className="flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400
                       hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
            >
              <RefreshCw className={`w-4 h-4 ${syncSlack.isPending ? "animate-spin" : ""}`} />
              Sync Slack
            </button>
            <button
              onClick={handleRecomputeWorkloads}
              disabled={computeWorkloads.isPending || members.length === 0}
              title="Recompute workload from each member's open task count"
              className="flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-600 dark:text-zinc-400
                       hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors
                       disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Gauge className={`w-4 h-4 ${computeWorkloads.isPending ? "animate-pulse" : ""}`} />
              Recompute Workloads
            </button>
            <button
              onClick={() => setShowAddForm(true)}
              className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-white
                       bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors"
            >
              <UserPlus className="w-4 h-4" />
              Add Member
            </button>
          </div>
        </div>

        {/* Search & Filters */}
        <div className="flex items-center gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" />
            <input
              type="text"
              placeholder="Search by name or email..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-4 py-2 text-sm bg-zinc-50 dark:bg-zinc-800
                       border border-zinc-200 dark:border-zinc-700 rounded-lg
                       focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
            />
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setSourceFilter(null)}
              className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
                sourceFilter === null
                  ? "bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100"
                  : "text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800"
              }`}
            >
              All
            </button>
            {Object.entries(sourceStats).map(([source, count]) => {
              const badge = SOURCE_BADGES[source] || SOURCE_BADGES.manual;
              return (
                <button
                  key={source}
                  onClick={() => setSourceFilter(sourceFilter === source ? null : source)}
                  className={`flex items-center gap-1 px-3 py-1.5 text-sm rounded-lg transition-colors ${
                    sourceFilter === source
                      ? "bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100"
                      : "text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800"
                  }`}
                >
                  {badge.label}
                  <span className="text-xs text-zinc-400">({count})</span>
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* Member List */}
      <div className="flex-1 overflow-y-auto p-6">
        {isLoading ? (
          <div className="flex items-center justify-center h-32 text-zinc-500">
            Loading team members...
          </div>
        ) : filteredMembers.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-32 text-zinc-500">
            <Users className="w-8 h-8 mb-2 text-zinc-300 dark:text-zinc-600" />
            <p className="text-sm">
              {searchQuery || sourceFilter
                ? "No members match your filters"
                : "No team members yet"}
            </p>
            {!searchQuery && !sourceFilter && (
              <button
                onClick={() => setShowAddForm(true)}
                className="mt-2 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Add your first team member
              </button>
            )}
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredMembers.map((member) => (
              <TeamMemberCard
                key={member.id}
                member={member}
                onEdit={() => setEditingMember(member)}
                onDelete={() => handleDeleteMember(member.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Sync Status */}
      {syncSlack.isSuccess && (
        <div className="flex-shrink-0 px-6 py-3 bg-green-50 dark:bg-green-900/20 border-t border-green-200 dark:border-green-800">
          <p className="text-sm text-green-700 dark:text-green-400">
            Synced {syncSlack.data?.added || 0} new members, updated {syncSlack.data?.updated || 0}
          </p>
        </div>
      )}

      {syncSlack.isError && (
        <div className="flex-shrink-0 px-6 py-3 bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-800">
          <div className="flex items-center gap-2 text-sm text-red-700 dark:text-red-400">
            <AlertCircle className="w-4 h-4" />
            <p>Failed to sync: {(syncSlack.error as Error)?.message || "Unknown error"}</p>
          </div>
        </div>
      )}

      {/* Add/Edit Form Modal */}
      {(showAddForm || editingMember) && (
        <TeamMemberForm
          member={editingMember}
          onClose={() => {
            setShowAddForm(false);
            setEditingMember(null);
          }}
        />
      )}
    </div>
  );
}
