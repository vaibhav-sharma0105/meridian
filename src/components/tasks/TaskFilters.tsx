import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Search, X, Video, Calendar, ChevronDown } from "lucide-react";
import { useTaskStore } from "@/stores/taskStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTasks } from "@/hooks/useTasks";
import { useMeetings } from "@/hooks/useMeetings";
import AssigneeChipInput, { parseAssignees } from "./AssigneeChipInput";
import { KANBAN_COLUMNS } from "@/lib/constants";
import type { Meeting } from "@/lib/tauri";

interface Props {
  showProjectFilter?: boolean;
}

// ── Date helpers ───────────────────────────────────────────────────────────────

function toDateStr(d: Date) {
  return d.toISOString().split("T")[0];
}

function daysAgo(n: number) {
  const d = new Date(); d.setDate(d.getDate() - n); return toDateStr(d);
}
function monthsAgo(n: number) {
  const d = new Date(); d.setMonth(d.getMonth() - n); return toDateStr(d);
}

const DATE_PRESETS = [
  { id: "today",   label: "Today",        from: () => toDateStr(new Date()),  to: () => toDateStr(new Date()) },
  { id: "7d",      label: "Last 7 days",  from: () => daysAgo(7),             to: () => toDateStr(new Date()) },
  { id: "30d",     label: "Last 30 days", from: () => daysAgo(30),            to: () => toDateStr(new Date()) },
  { id: "3mo",     label: "Last 3 months",from: () => monthsAgo(3),           to: () => toDateStr(new Date()) },
  { id: "year",    label: "Last year",    from: () => monthsAgo(12),          to: () => toDateStr(new Date()) },
  { id: "custom",  label: "Custom range", from: () => "",                     to: () => "" },
] as const;

type DatePresetId = typeof DATE_PRESETS[number]["id"] | "";

function derivePresetId(from?: string, to?: string): DatePresetId {
  if (!from && !to) return "";
  const today = toDateStr(new Date());
  for (const p of DATE_PRESETS) {
    if (p.id === "custom") continue;
    if (from === p.from() && to === p.to()) return p.id;
    // Allow ~1 day tolerance for "today" matching
    if (p.id === "today" && from === today && to === today) return "today";
  }
  return "custom";
}

// ── Active filter chip (replaces select when a value is chosen) ───────────────

function ActiveChip({
  label,
  onClear,
  variant = "indigo",
}: {
  label: string;
  onClear: () => void;
  variant?: "indigo" | "red" | "orange";
}) {
  const colors = {
    indigo: "bg-indigo-50 dark:bg-indigo-950/50 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800/60",
    red:    "bg-red-50 dark:bg-red-950/40 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800/60",
    orange: "bg-orange-50 dark:bg-orange-950/40 text-orange-700 dark:text-orange-400 border-orange-200 dark:border-orange-800/60",
  };
  return (
    <span className={`inline-flex items-center gap-1 pl-2 pr-1 py-0.5 text-[12px] font-medium rounded-md border ${colors[variant]}`}>
      {label}
      <button
        onClick={onClear}
        className="p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
        title="Remove filter"
      >
        <X className="w-2.5 h-2.5" />
      </button>
    </span>
  );
}

// ── Shared input style ─────────────────────────────────────────────────────────
const inputCls =
  "px-2 py-1 text-[12px] rounded-md border border-zinc-200 dark:border-zinc-700/60 " +
  "bg-white dark:bg-zinc-800/80 text-zinc-700 dark:text-zinc-300 outline-none " +
  "focus:ring-2 focus:ring-indigo-400/50 focus:border-indigo-400 transition-colors cursor-pointer";

const activeCls =
  "border-indigo-300 dark:border-indigo-700 bg-indigo-50 dark:bg-indigo-900/30 " +
  "text-indigo-700 dark:text-indigo-300";

// ── Meeting multi-select dropdown ──────────────────────────────────────────────

function MeetingFilter({
  meetings,
  selectedIds,
  onChange,
}: {
  meetings: Meeting[];
  selectedIds: string[];
  onChange: (ids: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const toggle = (id: string) => {
    const next = selectedIds.includes(id)
      ? selectedIds.filter((s) => s !== id)
      : [...selectedIds, id];
    onChange(next);
  };

  const isActive = selectedIds.length > 0;

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className={`flex items-center gap-1.5 px-2 py-1 text-[12px] rounded-md border transition-colors ${
          isActive ? activeCls : "border-zinc-200 dark:border-zinc-700/60 bg-white dark:bg-zinc-800/80 text-zinc-700 dark:text-zinc-300"
        }`}
      >
        <Video className="w-3 h-3 flex-shrink-0" />
        <span>
          {isActive
            ? `${selectedIds.length} meeting${selectedIds.length > 1 ? "s" : ""}`
            : "By meeting"}
        </span>
        <ChevronDown className={`w-3 h-3 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>

      {open && (
        <div className="absolute top-full left-0 mt-1 w-64 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl z-50 overflow-hidden animate-fade-in">
          <div className="px-3 py-2 border-b border-zinc-100 dark:border-zinc-700 flex items-center justify-between">
            <span className="text-[11px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider">
              Filter by meeting
            </span>
            {isActive && (
              <button
                onClick={() => onChange([])}
                className="text-[11px] text-indigo-500 hover:text-indigo-700 dark:hover:text-indigo-300"
              >
                Clear
              </button>
            )}
          </div>
          <ul className="py-1 max-h-48 overflow-y-auto">
            {meetings.map((m) => (
              <li key={m.id}>
                <label className="flex items-center gap-2.5 px-3 py-2 hover:bg-zinc-50 dark:hover:bg-zinc-700/50 cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    checked={selectedIds.includes(m.id)}
                    onChange={() => toggle(m.id)}
                    className="w-3.5 h-3.5 rounded accent-indigo-500 flex-shrink-0"
                  />
                  <span className="text-[12px] text-zinc-700 dark:text-zinc-300 truncate leading-snug">
                    {m.title}
                  </span>
                </label>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

// ── Date filter popover ───────────────────────────────────────────────────────

function DateFilter({
  dateFrom,
  dateTo,
  onChange,
}: {
  dateFrom?: string;
  dateTo?: string;
  onChange: (from?: string, to?: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [customFrom, setCustomFrom] = useState(dateFrom ?? "");
  const [customTo, setCustomTo]     = useState(dateTo   ?? "");
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const activePresetId = derivePresetId(dateFrom, dateTo);
  const isActive = activePresetId !== "";
  const activeLabel = DATE_PRESETS.find(p => p.id === activePresetId)?.label;

  const selectPreset = (id: DatePresetId) => {
    if (id === "") { onChange(undefined, undefined); setOpen(false); return; }
    const p = DATE_PRESETS.find(x => x.id === id)!;
    if (id === "custom") {
      setCustomFrom(dateFrom ?? "");
      setCustomTo(dateTo ?? "");
      return; // keep popover open for custom inputs
    }
    onChange(p.from(), p.to());
    setOpen(false);
  };

  const applyCustom = () => {
    onChange(customFrom || undefined, customTo || undefined);
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(o => !o)}
        className={`flex items-center gap-1.5 px-2 py-1 text-[12px] rounded-md border transition-colors ${
          isActive ? activeCls : "border-zinc-200 dark:border-zinc-700/60 bg-white dark:bg-zinc-800/80 text-zinc-700 dark:text-zinc-300"
        }`}
      >
        <Calendar className="w-3 h-3 flex-shrink-0" />
        <span>{isActive ? activeLabel : "Created date"}</span>
        {isActive
          ? <button onClick={(e) => { e.stopPropagation(); onChange(undefined, undefined); }} className="p-0.5 rounded hover:bg-black/10 dark:hover:bg-white/10"><X className="w-2.5 h-2.5" /></button>
          : <ChevronDown className={`w-3 h-3 transition-transform ${open ? "rotate-180" : ""}`} />
        }
      </button>

      {open && (
        <div className="absolute top-full left-0 mt-1 w-52 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl z-50 py-1 overflow-hidden animate-fade-in">
          {/* Clear */}
          {isActive && (
            <button
              onClick={() => { onChange(undefined, undefined); setOpen(false); }}
              className="w-full text-left px-3 py-1.5 text-[12px] text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-700/50 transition-colors"
            >
              Clear filter
            </button>
          )}
          {isActive && <div className="h-px bg-zinc-100 dark:bg-zinc-700 mx-2 my-1" />}

          {/* Presets */}
          {DATE_PRESETS.filter(p => p.id !== "custom").map(p => (
            <button
              key={p.id}
              onClick={() => selectPreset(p.id)}
              className={`w-full text-left px-3 py-1.5 text-[12px] transition-colors flex items-center justify-between ${
                activePresetId === p.id
                  ? "text-indigo-700 dark:text-indigo-300 bg-indigo-50 dark:bg-indigo-950/50"
                  : "text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-700/50"
              }`}
            >
              {p.label}
              {activePresetId === p.id && <span className="w-1.5 h-1.5 rounded-full bg-indigo-500 flex-shrink-0" />}
            </button>
          ))}

          {/* Custom range */}
          <div className="h-px bg-zinc-100 dark:bg-zinc-700 mx-2 my-1" />
          <div className="px-3 py-2 space-y-2">
            <p className="text-[11px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider">Custom range</p>
            <div className="flex flex-col gap-1.5">
              <input
                type="date"
                value={customFrom}
                onChange={(e) => setCustomFrom(e.target.value)}
                className={`${inputCls} w-full`}
                placeholder="From"
              />
              <input
                type="date"
                value={customTo}
                onChange={(e) => setCustomTo(e.target.value)}
                className={`${inputCls} w-full`}
                placeholder="To"
              />
            </div>
            <button
              onClick={applyCustom}
              disabled={!customFrom && !customTo}
              className="w-full py-1 text-[12px] font-medium bg-indigo-500 hover:bg-indigo-600 disabled:opacity-40 text-white rounded-md transition-colors"
            >
              Apply
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// ── Main filter bar ────────────────────────────────────────────────────────────

export default function TaskFilters({ showProjectFilter = false }: Props) {
  const { t } = useTranslation();
  const { filters, setFilters } = useTaskStore();
  const { activeProjectId, projects } = useProjectStore();
  const { meetings } = useMeetings(activeProjectId);

  const { tasks: allTasks } = useTasks(activeProjectId, {});
  const assignees = Array.from(
    new Set(allTasks.flatMap((task) => parseAssignees(task.assignee ?? "")))
  );

  const hasFilters =
    filters.search_query ||
    filters.status ||
    filters.priority ||
    filters.assignee ||
    filters.project_id ||
    filters.meeting_ids?.length ||
    filters.date_from ||
    filters.date_to;

  const clearAll = () =>
    setFilters({
      search_query: undefined,
      status: undefined,
      priority: undefined,
      assignee: undefined,
      project_id: undefined,
      meeting_ids: undefined,
      date_from: undefined,
      date_to: undefined,
    });

  return (
    <div className="flex items-center gap-1.5 flex-wrap">
      {/* Search */}
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3 h-3 text-zinc-400 pointer-events-none" />
        <input
          type="text"
          value={filters.search_query ?? ""}
          onChange={(e) => setFilters({ search_query: e.target.value || undefined })}
          placeholder={t("tasks.searchPlaceholder")}
          className="pl-7 pr-3 py-1 text-[12px] rounded-md border border-zinc-200 dark:border-zinc-700/60 bg-white dark:bg-zinc-800/80 text-zinc-700 dark:text-zinc-300 w-40 outline-none focus:ring-2 focus:ring-indigo-400/50 focus:border-indigo-400 transition-colors placeholder:text-zinc-400"
        />
      </div>

      {/* Status */}
      {filters.status ? (
        <ActiveChip
          label={KANBAN_COLUMNS.find(c => c.id === filters.status)?.label ?? filters.status}
          onClear={() => setFilters({ status: undefined })}
        />
      ) : (
        <select
          value=""
          onChange={(e) => setFilters({ status: e.target.value || undefined })}
          className={inputCls}
        >
          <option value="">{t("tasks.allStatuses")}</option>
          {KANBAN_COLUMNS.map((col) => (
            <option key={col.id} value={col.id}>{col.label}</option>
          ))}
          <option value="cancelled">Cancelled</option>
        </select>
      )}

      {/* Priority */}
      {filters.priority ? (
        <ActiveChip
          label={filters.priority.charAt(0).toUpperCase() + filters.priority.slice(1)}
          onClear={() => setFilters({ priority: undefined })}
          variant={filters.priority === "critical" ? "red" : filters.priority === "high" ? "orange" : "indigo"}
        />
      ) : (
        <select
          value=""
          onChange={(e) => setFilters({ priority: e.target.value || undefined })}
          className={inputCls}
        >
          <option value="">All priorities</option>
          <option value="critical">Critical</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
      )}

      {/* Assignee */}
      <div className="w-40">
        <AssigneeChipInput
          value={filters.assignee ?? ""}
          onChange={(v) => setFilters({ assignee: v || undefined })}
          suggestions={assignees}
          placeholder={t("tasks.filterAssignee")}
          showHint={false}
        />
      </div>

      {/* Meeting multi-select — project-scoped only */}
      {activeProjectId && meetings.length > 0 && (
        <MeetingFilter
          meetings={meetings}
          selectedIds={filters.meeting_ids ?? []}
          onChange={(ids) => setFilters({ meeting_ids: ids.length ? ids : undefined })}
        />
      )}

      {/* Date filter */}
      <DateFilter
        dateFrom={filters.date_from}
        dateTo={filters.date_to}
        onChange={(from, to) => setFilters({ date_from: from, date_to: to })}
      />

      {/* Project — All Tasks view only */}
      {showProjectFilter && (
        <select
          value={filters.project_id ?? ""}
          onChange={(e) => setFilters({ project_id: e.target.value || undefined })}
          className={`${inputCls} max-w-[130px] ${filters.project_id ? activeCls : ""}`}
        >
          <option value="">All projects</option>
          {projects.filter((p) => !p.archived_at).map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      )}

      {/* Clear all */}
      {hasFilters && (
        <button
          onClick={clearAll}
          className="flex items-center gap-1 px-2 py-1 text-[12px] text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <X className="w-3 h-3" />
          {t("tasks.clearFilters")}
        </button>
      )}
    </div>
  );
}
