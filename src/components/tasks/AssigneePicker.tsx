import { useState, useRef, useEffect } from "react";
import {
  ChevronDown,
  User,
  Sparkles,
  AlertTriangle,
  HelpCircle,
  Check,
} from "lucide-react";
import { useAssigneeSuggestions, useTeamMembers } from "@/hooks/useTeam";
import type { AssigneeSuggestion, TeamMember } from "@/lib/tauri";

// ─── Confidence Badge ─────────────────────────────────────────────────────────

function ConfidenceBadge({ confidence }: { confidence: string }) {
  const styles = {
    high: "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400",
    medium: "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400",
    low: "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400",
  };

  return (
    <span
      className={`text-[10px] px-1.5 py-0.5 rounded ${styles[confidence as keyof typeof styles] || styles.low}`}
    >
      {confidence}
    </span>
  );
}

// ─── Factor Tooltip ───────────────────────────────────────────────────────────

function FactorTooltip({ factors }: { factors: AssigneeSuggestion["factors"] }) {
  const formatPercent = (v: number) => `${Math.round(v * 100)}%`;

  return (
    <div className="absolute left-full top-0 ml-2 w-48 p-2 bg-zinc-900 dark:bg-zinc-800 text-white text-xs rounded-lg shadow-lg z-50">
      <div className="font-medium mb-2">Scoring Factors</div>
      <div className="space-y-1">
        <div className="flex justify-between">
          <span>Pattern Match</span>
          <span>{formatPercent(factors.pattern_score)}</span>
        </div>
        <div className="flex justify-between">
          <span>Availability</span>
          <span>{formatPercent(factors.workload_score)}</span>
        </div>
        <div className="flex justify-between">
          <span>Expertise</span>
          <span>{formatPercent(factors.expertise_score)}</span>
        </div>
        <div className="flex justify-between">
          <span>Recent Activity</span>
          <span>{formatPercent(factors.recency_score)}</span>
        </div>
      </div>
    </div>
  );
}

// ─── Workload Warning ─────────────────────────────────────────────────────────

function WorkloadWarning({ member }: { member: TeamMember }) {
  const workload = member.workload_score ?? 0;
  if (workload < 0.8) return null;

  return (
    <div className="flex items-center gap-1 text-[10px] text-amber-600 dark:text-amber-400">
      <AlertTriangle className="w-3 h-3" />
      <span>High workload</span>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

interface AssigneePickerProps {
  value?: string;
  taskTitle: string;
  taskDescription?: string;
  projectId?: string;
  onChange: (assignee: string | undefined, wasOverride: boolean) => void;
}

export function AssigneePicker({
  value,
  taskTitle,
  taskDescription,
  projectId,
  onChange,
}: AssigneePickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [hoveredSuggestion, setHoveredSuggestion] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const { data: suggestions = [], isLoading: loadingSuggestions } = useAssigneeSuggestions(
    taskTitle,
    taskDescription,
    projectId
  );
  const { data: allMembers = [] } = useTeamMembers();

  // Close on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  // Filter members for search
  const filteredMembers = searchQuery
    ? allMembers.filter(
        (m) =>
          m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          m.email?.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : [];

  const handleSelect = (name: string) => {
    const topSuggestion = suggestions[0]?.member.name;
    const wasOverride = topSuggestion && name !== topSuggestion;
    onChange(name, !!wasOverride);
    setIsOpen(false);
    setSearchQuery("");
  };

  const handleClear = () => {
    onChange(undefined, false);
    setIsOpen(false);
  };

  return (
    <div ref={containerRef} className="relative">
      {/* Trigger Button */}
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm text-left w-full
                 bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700
                 rounded-lg hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors"
      >
        <User className="w-4 h-4 text-zinc-400" />
        <span className={value ? "text-zinc-900 dark:text-zinc-100" : "text-zinc-500"}>
          {value || "Select assignee..."}
        </span>
        <ChevronDown className="w-4 h-4 text-zinc-400 ml-auto" />
      </button>

      {/* Dropdown */}
      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-zinc-900
                      border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl z-50 overflow-hidden">
          {/* Search */}
          <div className="p-2 border-b border-zinc-200 dark:border-zinc-700">
            <input
              type="text"
              placeholder="Search team members..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              autoFocus
              className="w-full px-3 py-1.5 text-sm bg-zinc-50 dark:bg-zinc-800
                       border border-zinc-200 dark:border-zinc-700 rounded
                       focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
            />
          </div>

          <div className="max-h-64 overflow-y-auto">
            {/* AI Suggestions */}
            {!searchQuery && suggestions.length > 0 && (
              <div className="p-2 border-b border-zinc-200 dark:border-zinc-700">
                <div className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-500 font-medium">
                  <Sparkles className="w-3 h-3" />
                  Suggested
                </div>
                {suggestions.map((suggestion) => (
                  <div
                    key={suggestion.member.id}
                    className="relative"
                    onMouseEnter={() => setHoveredSuggestion(suggestion.member.id)}
                    onMouseLeave={() => setHoveredSuggestion(null)}
                  >
                    <button
                      type="button"
                      onClick={() => handleSelect(suggestion.member.name)}
                      className="w-full flex items-center gap-3 px-2 py-2 text-sm text-left
                               hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
                    >
                      {suggestion.member.avatar_url ? (
                        <img
                          src={suggestion.member.avatar_url}
                          alt=""
                          className="w-6 h-6 rounded-full"
                        />
                      ) : (
                        <div className="w-6 h-6 rounded-full bg-indigo-100 dark:bg-indigo-900/30
                                      flex items-center justify-center">
                          <span className="text-xs font-medium text-indigo-600 dark:text-indigo-400">
                            {suggestion.member.name.charAt(0)}
                          </span>
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-zinc-900 dark:text-zinc-100 truncate">
                            {suggestion.member.name}
                          </span>
                          <ConfidenceBadge confidence={suggestion.confidence} />
                          {value === suggestion.member.name && (
                            <Check className="w-4 h-4 text-indigo-600" />
                          )}
                        </div>
                        <div className="text-xs text-zinc-500 truncate">
                          {suggestion.reason}
                        </div>
                        <WorkloadWarning member={suggestion.member} />
                      </div>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          setHoveredSuggestion(
                            hoveredSuggestion === suggestion.member.id ? null : suggestion.member.id
                          );
                        }}
                        className="p-1 text-zinc-400 hover:text-zinc-600"
                      >
                        <HelpCircle className="w-3.5 h-3.5" />
                      </button>
                    </button>

                    {/* Factor Tooltip */}
                    {hoveredSuggestion === suggestion.member.id && (
                      <FactorTooltip factors={suggestion.factors} />
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* Search Results */}
            {searchQuery && filteredMembers.length > 0 && (
              <div className="p-2">
                {filteredMembers.map((member) => (
                  <button
                    key={member.id}
                    type="button"
                    onClick={() => handleSelect(member.name)}
                    className="w-full flex items-center gap-3 px-2 py-2 text-sm text-left
                             hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
                  >
                    {member.avatar_url ? (
                      <img src={member.avatar_url} alt="" className="w-6 h-6 rounded-full" />
                    ) : (
                      <div className="w-6 h-6 rounded-full bg-zinc-100 dark:bg-zinc-800
                                    flex items-center justify-center">
                        <span className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                          {member.name.charAt(0)}
                        </span>
                      </div>
                    )}
                    <span className="text-zinc-900 dark:text-zinc-100">{member.name}</span>
                    {value === member.name && <Check className="w-4 h-4 text-indigo-600 ml-auto" />}
                  </button>
                ))}
              </div>
            )}

            {/* All Members (no search) */}
            {!searchQuery && allMembers.length > 0 && (
              <div className="p-2">
                <div className="px-2 py-1 text-xs text-zinc-500 font-medium">All Team</div>
                {allMembers
                  .filter((m) => !suggestions.some((s) => s.member.id === m.id))
                  .map((member) => (
                    <button
                      key={member.id}
                      type="button"
                      onClick={() => handleSelect(member.name)}
                      className="w-full flex items-center gap-3 px-2 py-2 text-sm text-left
                               hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
                    >
                      {member.avatar_url ? (
                        <img src={member.avatar_url} alt="" className="w-6 h-6 rounded-full" />
                      ) : (
                        <div className="w-6 h-6 rounded-full bg-zinc-100 dark:bg-zinc-800
                                      flex items-center justify-center">
                          <span className="text-xs font-medium text-zinc-600 dark:text-zinc-400">
                            {member.name.charAt(0)}
                          </span>
                        </div>
                      )}
                      <span className="text-zinc-900 dark:text-zinc-100">{member.name}</span>
                      {value === member.name && (
                        <Check className="w-4 h-4 text-indigo-600 ml-auto" />
                      )}
                    </button>
                  ))}
              </div>
            )}

            {/* No Results */}
            {searchQuery && filteredMembers.length === 0 && (
              <div className="p-4 text-center text-sm text-zinc-500">
                No team members match "{searchQuery}"
              </div>
            )}

            {/* Loading */}
            {loadingSuggestions && !searchQuery && (
              <div className="p-4 text-center text-sm text-zinc-500">
                Loading suggestions...
              </div>
            )}
          </div>

          {/* Clear Button */}
          {value && (
            <div className="p-2 border-t border-zinc-200 dark:border-zinc-700">
              <button
                type="button"
                onClick={handleClear}
                className="w-full px-3 py-1.5 text-sm text-zinc-500 hover:text-zinc-700
                         hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
              >
                Clear assignee
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
