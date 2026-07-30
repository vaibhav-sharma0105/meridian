import { MoreVertical, Edit2, Trash2, Mail, Briefcase } from "lucide-react";
import { useState, useRef, useEffect } from "react";
import type { TeamMember } from "@/lib/tauri";

// ─── Source Badge Colors ──────────────────────────────────────────────────────

const SOURCE_BADGES: Record<string, { bg: string; text: string; label: string }> = {
  manual: { bg: "bg-zinc-100 dark:bg-zinc-800", text: "text-zinc-600 dark:text-zinc-400", label: "Manual" },
  slack: { bg: "bg-purple-100 dark:bg-purple-900/30", text: "text-purple-600 dark:text-purple-400", label: "Slack" },
  google: { bg: "bg-blue-100 dark:bg-blue-900/30", text: "text-blue-600 dark:text-blue-400", label: "Google" },
};

// ─── Workload Indicator ───────────────────────────────────────────────────────

function WorkloadIndicator({ score }: { score?: number }) {
  if (score === undefined || score === null) return null;

  const percentage = Math.round(score * 100);
  let color = "bg-green-500";
  let label = "Available";

  if (score > 0.8) {
    color = "bg-red-500";
    label = "Overloaded";
  } else if (score > 0.5) {
    color = "bg-yellow-500";
    label = "Busy";
  }

  return (
    <div className="flex items-center gap-2">
      <div className="w-16 h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
        <div className={`h-full ${color} rounded-full`} style={{ width: `${percentage}%` }} />
      </div>
      <span className="text-xs text-zinc-500">{label}</span>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

interface TeamMemberCardProps {
  member: TeamMember;
  onEdit: () => void;
  onDelete: () => void;
}

export function TeamMemberCard({ member, onEdit, onDelete }: TeamMemberCardProps) {
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const badge = SOURCE_BADGES[member.source] || SOURCE_BADGES.manual;

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setShowMenu(false);
      }
    }
    if (showMenu) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [showMenu]);

  return (
    <div className="group relative bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800
                    rounded-lg p-4 hover:border-zinc-300 dark:hover:border-zinc-700 transition-colors">
      {/* Menu Button */}
      <div className="absolute top-3 right-3" ref={menuRef}>
        <button
          onClick={() => setShowMenu(!showMenu)}
          className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300
                   opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <MoreVertical className="w-4 h-4" />
        </button>

        {showMenu && (
          <div className="absolute right-0 top-6 w-32 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700
                         rounded-lg shadow-lg py-1 z-10">
            <button
              onClick={() => {
                setShowMenu(false);
                onEdit();
              }}
              className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-700 dark:text-zinc-300
                       hover:bg-zinc-100 dark:hover:bg-zinc-700"
            >
              <Edit2 className="w-3.5 h-3.5" />
              Edit
            </button>
            {member.source === "manual" && (
              <button
                onClick={() => {
                  setShowMenu(false);
                  onDelete();
                }}
                className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-red-600 dark:text-red-400
                         hover:bg-red-50 dark:hover:bg-red-900/20"
              >
                <Trash2 className="w-3.5 h-3.5" />
                Remove
              </button>
            )}
          </div>
        )}
      </div>

      {/* Avatar & Name */}
      <div className="flex items-start gap-3 mb-3">
        {member.avatar_url ? (
          <img
            src={member.avatar_url}
            alt={member.name}
            className="w-10 h-10 rounded-full object-cover"
          />
        ) : (
          <div className="w-10 h-10 rounded-full bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center">
            <span className="text-sm font-medium text-indigo-600 dark:text-indigo-400">
              {member.name.charAt(0).toUpperCase()}
            </span>
          </div>
        )}
        <div className="flex-1 min-w-0">
          <h3 className="font-medium text-zinc-900 dark:text-zinc-100 truncate">
            {member.name}
          </h3>
          <div className="flex items-center gap-2 mt-0.5">
            <span className={`text-xs px-2 py-0.5 rounded-full ${badge.bg} ${badge.text}`}>
              {badge.label}
            </span>
            {member.role && member.role !== "member" && (
              <span className="text-xs text-zinc-500 capitalize">{member.role}</span>
            )}
          </div>
        </div>
      </div>

      {/* Email */}
      {member.email && (
        <div className="flex items-center gap-2 text-sm text-zinc-500 mb-2">
          <Mail className="w-3.5 h-3.5" />
          <span className="truncate">{member.email}</span>
        </div>
      )}

      {/* Expertise */}
      {member.expertise && member.expertise.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-3">
          {member.expertise.slice(0, 3).map((skill) => (
            <span
              key={skill}
              className="text-xs px-2 py-0.5 bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 rounded"
            >
              {skill}
            </span>
          ))}
          {member.expertise.length > 3 && (
            <span className="text-xs text-zinc-400">+{member.expertise.length - 3} more</span>
          )}
        </div>
      )}

      {/* Workload */}
      <WorkloadIndicator score={member.workload_score} />
    </div>
  );
}
