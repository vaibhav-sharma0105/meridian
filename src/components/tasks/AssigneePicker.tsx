import { useState, useRef, useEffect } from "react";
import {
  User,
  Sparkles,
  AlertTriangle,
  HelpCircle,
  Check,
  X,
} from "lucide-react";
import { useAssigneeSuggestions, useTeamMembers, useRecordAssigneeSelection } from "@/hooks/useTeam";
import { parseAssignees, serializeAssignees } from "./AssigneeChipInput";
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
//
// Multi-assignee picker: `value` is the same comma-separated string used by
// AssigneeChipInput elsewhere in the app (e.g. "Alice, Bob"). Selecting a
// suggestion or team member ADDS them to the list rather than replacing it;
// clicking an already-selected person removes them.

interface AssigneePickerProps {
  value: string;
  taskTitle: string;
  taskDescription?: string;
  projectId?: string;
  onChange: (value: string, wasOverride: boolean) => void;
}

export function AssigneePicker({
  value,
  taskTitle,
  taskDescription,
  projectId,
  onChange,
}: AssigneePickerProps) {
  const names = parseAssignees(value);
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [hoveredSuggestion, setHoveredSuggestion] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const { data: suggestions = [], isLoading: loadingSuggestions } = useAssigneeSuggestions(
    taskTitle,
    taskDescription,
    projectId
  );
  const { data: allMembers = [] } = useTeamMembers();
  const recordSelection = useRecordAssigneeSelection();

  // Close on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setSearchQuery("");
      }
    }
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  const isSelected = (name: string) => names.some((n) => n.toLowerCase() === name.toLowerCase());

  const addName = (name: string) => {
    const trimmed = name.trim();
    if (!trimmed || isSelected(trimmed)) return;

    // "Override" means: the AI's #1 pick is still on the table (not yet
    // added to this task) and you added someone else instead of it. Once
    // the top suggestion has actually been accepted, adding more people on
    // top of that is just... adding more people, not overriding anything —
    // so it stops counting as an override for the rest of this session.
    const topSuggestion = suggestions[0]?.member.name;
    const topSuggestionAlreadyAdded = topSuggestion ? isSelected(topSuggestion) : false;
    const wasOverride = !!topSuggestion && !topSuggestionAlreadyAdded && trimmed !== topSuggestion;

    onChange(serializeAssignees([...names, trimmed]), wasOverride);
    recordSelection.mutate({ selectedName: trimmed, suggestions, wasOverride });
    setSearchQuery("");
  };

  const removeName = (name: string) => {
    onChange(serializeAssignees(names.filter((n) => n !== name)), false);
  };

  const toggleMember = (name: string) => {
    if (isSelected(name)) {
      removeName(name);
    } else {
      addName(name);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if ((e.key === "Enter" || e.key === ",") && searchQuery.trim()) {
      e.preventDefault();
      const exactMatch = allMembers.find(
        (m) => m.name.toLowerCase() === searchQuery.trim().toLowerCase()
      );
      addName(exactMatch ? exactMatch.name : searchQuery.trim());
    } else if (e.key === "Backspace" && !searchQuery && names.length > 0) {
      removeName(names[names.length - 1]);
    } else if (e.key === "Escape") {
      setIsOpen(false);
      setSearchQuery("");
    }
  };

  const filteredMembers = searchQuery
    ? allMembers.filter(
        (m) =>
          m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          m.email?.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : allMembers;

  const unselectedSuggestions = suggestions.filter((s) => !isSelected(s.member.name));
  const noExactMatch =
    searchQuery.trim() !== "" &&
    !allMembers.some((m) => m.name.toLowerCase() === searchQuery.trim().toLowerCase());

  return (
    <div ref={containerRef} className="relative">
      {/* Trigger — chips + search input */}
      <div
        onClick={() => {
          setIsOpen(true);
          inputRef.current?.focus();
        }}
        className="flex flex-wrap items-center gap-1 px-2 py-1.5 min-h-[38px] w-full text-sm cursor-text
                 bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700
                 rounded-lg hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors"
      >
        <User className="w-4 h-4 text-zinc-400 flex-shrink-0" />
        {names.map((name) => (
          <span
            key={name}
            className="inline-flex items-center gap-1 px-1.5 py-0.5 bg-indigo-100 dark:bg-indigo-900/40
                     text-indigo-700 dark:text-indigo-300 rounded text-xs font-medium"
          >
            {name}
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                removeName(name);
              }}
              className="hover:text-indigo-900 dark:hover:text-indigo-100"
            >
              <X className="w-2.5 h-2.5" />
            </button>
          </span>
        ))}
        <input
          ref={inputRef}
          type="text"
          aria-label="Add assignee"
          value={searchQuery}
          onChange={(e) => {
            setSearchQuery(e.target.value);
            setIsOpen(true);
          }}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsOpen(true)}
          placeholder={names.length === 0 ? "Select assignee..." : ""}
          className="flex-1 min-w-[80px] bg-transparent outline-none text-zinc-900 dark:text-zinc-100 placeholder-zinc-500"
        />
      </div>

      {/* Dropdown */}
      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-zinc-900
                      border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl z-50 overflow-hidden">
          <div className="max-h-64 overflow-y-auto">
            {/* AI Suggestions */}
            {!searchQuery && unselectedSuggestions.length > 0 && (
              <div className="p-2 border-b border-zinc-200 dark:border-zinc-700">
                <div className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-500 font-medium">
                  <Sparkles className="w-3 h-3" />
                  Suggested
                </div>
                {unselectedSuggestions.map((suggestion) => (
                  <div
                    key={suggestion.member.id}
                    className="relative"
                    onMouseEnter={() => setHoveredSuggestion(suggestion.member.id)}
                    onMouseLeave={() => setHoveredSuggestion(null)}
                  >
                    <button
                      type="button"
                      onClick={() => toggleMember(suggestion.member.name)}
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

            {/* Team members (search results, or everyone when not searching) */}
            <div className="p-2">
              <div className="px-2 py-1 text-xs text-zinc-500 font-medium">
                {searchQuery ? "Matches" : "All Team"}
              </div>
              {filteredMembers.length > 0 ? (
                filteredMembers.map((member) => (
                  <button
                    key={member.id}
                    type="button"
                    onClick={() => toggleMember(member.name)}
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
                    {isSelected(member.name) && <Check className="w-4 h-4 text-indigo-600 ml-auto" />}
                  </button>
                ))
              ) : !searchQuery ? (
                <div className="px-2 py-3 text-sm text-zinc-500">No team members yet</div>
              ) : null}

              {/* Free-text add for names not in the roster */}
              {searchQuery && noExactMatch && (
                <button
                  type="button"
                  onClick={() => addName(searchQuery)}
                  className="w-full flex items-center gap-2 px-2 py-2 text-sm text-left
                           text-indigo-600 dark:text-indigo-400
                           hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-colors"
                >
                  <User className="w-4 h-4" />
                  Add "{searchQuery}" as assignee
                </button>
              )}
            </div>

            {loadingSuggestions && !searchQuery && (
              <div className="p-2 text-center text-xs text-zinc-400">
                Loading suggestions...
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
