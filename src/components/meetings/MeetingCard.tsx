import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import {
  ChevronDown,
  ChevronRight,
  Calendar,
  Users,
  Clock,
  Trash2,
  FolderInput,
  Loader2,
  AlertCircle,
} from "lucide-react";
import MeetingHealthBadge from "./MeetingHealthBadge";
import { format } from "date-fns";
import type { Meeting } from "@/lib/tauri";
import * as api from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import toast from "react-hot-toast";

interface Props {
  meeting: Meeting;
  onDelete?: (id: string) => void;
}

export default function MeetingCard({ meeting, onDelete }: Props) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const { projects, fetchProjects } = useProjectStore();

  const [expanded, setExpanded] = useState(false);

  // ── Inline rename state ──────────────────────────────────────────────────
  const [editingTitle, setEditingTitle] = useState(false);
  const [draftTitle, setDraftTitle] = useState(meeting.title);
  const titleInputRef = useRef<HTMLInputElement>(null);
  // Prevent onBlur from committing when Escape was pressed (stale-closure guard)
  const cancelingRef = useRef(false);

  useEffect(() => { setDraftTitle(meeting.title); }, [meeting.title]);

  useEffect(() => {
    if (editingTitle) {
      cancelingRef.current = false;
      titleInputRef.current?.select();
    }
  }, [editingTitle]);

  const commitRename = async () => {
    if (cancelingRef.current) { cancelingRef.current = false; return; }
    const trimmed = draftTitle.trim();
    if (!trimmed) { setDraftTitle(meeting.title); setEditingTitle(false); return; }
    if (trimmed === meeting.title) { setEditingTitle(false); return; }
    try {
      await api.renameMeeting(meeting.id, trimmed);
      // Update the cache immediately so the title reflects without a round-trip refetch
      qc.setQueryData<Meeting[]>(
        ["meetings", meeting.project_id],
        (old) => old?.map((m) => m.id === meeting.id ? { ...m, title: trimmed } : m)
      );
      toast.success("Meeting renamed");
    } catch (err) {
      toast.error(String(err));
      setDraftTitle(meeting.title);
    }
    setEditingTitle(false);
  };

  const cancelRename = () => {
    cancelingRef.current = true;
    setDraftTitle(meeting.title);
    setEditingTitle(false);
  };

  // ── Move-to-project state ────────────────────────────────────────────────
  const [showProjectPicker, setShowProjectPicker] = useState(false);
  const [dropdownPos, setDropdownPos] = useState<{ top: number; right: number } | null>(null);
  const [pendingProject, setPendingProject] = useState<api.Project | null>(null);
  const [taskCount, setTaskCount] = useState<number | null>(null);
  const [isCountLoading, setIsCountLoading] = useState(false);
  const [isMoving, setIsMoving] = useState(false);
  const [moveError, setMoveError] = useState<string | null>(null);
  const pickerRef = useRef<HTMLDivElement>(null);
  const moveButtonRef = useRef<HTMLButtonElement>(null);

  // Other projects (not current, not archived)
  const otherProjects = projects.filter(
    (p) => p.id !== meeting.project_id && !p.archived_at
  );

  // Close the project-picker dropdown on outside click or scroll.
  // Deliberately skipped when the confirm banner is active (pendingProject != null)
  // because the banner sits outside pickerRef — without this guard, mousedown on
  // "Confirm Move" would fire cancelMove() before the click handler runs.
  useEffect(() => {
    if (!showProjectPicker || pendingProject) return;
    function handleClose(e: MouseEvent) {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        cancelMove();
      }
    }
    function handleScroll() { cancelMove(); }
    document.addEventListener("mousedown", handleClose);
    document.addEventListener("scroll", handleScroll, true);
    return () => {
      document.removeEventListener("mousedown", handleClose);
      document.removeEventListener("scroll", handleScroll, true);
    };
  }, [showProjectPicker, pendingProject]);

  function cancelMove() {
    setShowProjectPicker(false);
    setDropdownPos(null);
    setPendingProject(null);
    setTaskCount(null);
    setMoveError(null);
  }

  async function handleProjectSelect(project: api.Project) {
    setPendingProject(project);
    setTaskCount(null);
    setMoveError(null);
    setIsCountLoading(true);
    try {
      const count = await api.countMoveableTasks(meeting.id);
      setTaskCount(count);
    } catch {
      setTaskCount(0);
    } finally {
      setIsCountLoading(false);
    }
  }

  async function confirmMove() {
    if (!pendingProject) return;
    setIsMoving(true);
    setMoveError(null);
    try {
      const result = await api.moveMeetingToProject(meeting.id, pendingProject.id);
      // Invalidate both source and target project caches
      qc.invalidateQueries({ queryKey: ["meetings", result.old_project_id] });
      qc.invalidateQueries({ queryKey: ["tasks", result.old_project_id] });
      qc.invalidateQueries({ queryKey: ["meetings", result.new_project_id] });
      qc.invalidateQueries({ queryKey: ["tasks", result.new_project_id] });
      // Refresh project list so open_task_count badges update in the sidebar
      fetchProjects();

      const taskMsg =
        result.tasks_moved > 0
          ? ` · ${result.tasks_moved} task${result.tasks_moved === 1 ? "" : "s"} moved`
          : "";
      toast.success(`Moved to "${pendingProject.name}"${taskMsg}`);
      cancelMove();
    } catch (err) {
      setMoveError(String(err));
      setIsMoving(false);
    }
  }

  // ── Delete ───────────────────────────────────────────────────────────────
  function handleDelete(e: React.MouseEvent) {
    e.stopPropagation();
    if (window.confirm(t("meetings.confirmDelete", { title: meeting.title }))) {
      onDelete?.(meeting.id);
    }
  }

  const attendees = meeting.attendees
    ? meeting.attendees.split(",").map((a) => a.trim()).filter(Boolean)
    : [];

  return (
    <div className="bg-white dark:bg-zinc-900/80 border border-zinc-100 dark:border-zinc-800/60 rounded-lg overflow-visible transition-shadow hover:shadow-sm">
      {/* ── Card header row ─────────────────────────────────────────────── */}
      <div
        className="flex items-center gap-2.5 px-3.5 py-3 cursor-pointer hover:bg-zinc-50/80 dark:hover:bg-zinc-800/30 transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <button className="text-zinc-300 dark:text-zinc-600 flex-shrink-0 hover:text-zinc-500 dark:hover:text-zinc-400 transition-colors">
          {expanded ? (
            <ChevronDown className="w-3.5 h-3.5" />
          ) : (
            <ChevronRight className="w-3.5 h-3.5" />
          )}
        </button>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 min-w-0">
            {editingTitle ? (
              <input
                ref={titleInputRef}
                value={draftTitle}
                onChange={(e) => setDraftTitle(e.target.value)}
                onBlur={commitRename}
                onKeyDown={(e) => {
                  if (e.key === "Enter") { e.preventDefault(); commitRename(); }
                  if (e.key === "Escape") cancelRename();
                }}
                onClick={(e) => e.stopPropagation()}
                className="flex-1 min-w-0 text-[13px] font-semibold bg-transparent border-b-2 border-indigo-400 outline-none text-zinc-900 dark:text-zinc-100 pb-0.5"
              />
            ) : (
              <h3
                className="text-[13px] font-semibold text-zinc-900 dark:text-zinc-100 truncate cursor-text hover:text-indigo-700 dark:hover:text-indigo-300 transition-colors"
                title="Click to rename"
                onClick={(e) => { e.stopPropagation(); setEditingTitle(true); }}
              >
                {meeting.title}
              </h3>
            )}
            {meeting.health_score !== null && meeting.health_score !== undefined && (
              <MeetingHealthBadge score={meeting.health_score} showLabel />
            )}
          </div>
          <div className="flex items-center gap-3 mt-0.5 flex-wrap">
            <span className="flex items-center gap-1 text-[11px] text-zinc-400 dark:text-zinc-500">
              <Calendar className="w-3 h-3" />
              {(() => {
                try {
                  return format(new Date(meeting.created_at), "MMM d, yyyy");
                } catch {
                  return meeting.created_at;
                }
              })()}
            </span>
            {meeting.platform && (
              <span className="text-[11px] text-zinc-400 dark:text-zinc-500 capitalize">{meeting.platform}</span>
            )}
            {attendees.length > 0 && (
              <span className="flex items-center gap-1 text-[11px] text-zinc-400 dark:text-zinc-500">
                <Users className="w-3 h-3" />
                {attendees.length} attendees
              </span>
            )}
            {meeting.duration_minutes && (
              <span className="flex items-center gap-1 text-[11px] text-zinc-400 dark:text-zinc-500">
                <Clock className="w-3 h-3" />
                {meeting.duration_minutes}m
              </span>
            )}
          </div>
        </div>

        {/* ── Action buttons ──────────────────────────────────────────── */}
        <div
          className="flex items-center gap-1 flex-shrink-0"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Move-to-project button */}
          <div ref={pickerRef}>
            <button
              ref={moveButtonRef}
              onClick={() => {
                if (showProjectPicker) { cancelMove(); return; }
                const rect = moveButtonRef.current?.getBoundingClientRect();
                if (rect) {
                  const DROPDOWN_MAX_H = 260;
                  const spaceBelow = window.innerHeight - rect.bottom - 4;
                  const top = spaceBelow >= DROPDOWN_MAX_H
                    ? rect.bottom + 4
                    : Math.max(8, rect.top - DROPDOWN_MAX_H - 4);
                  setDropdownPos({ top, right: window.innerWidth - rect.right });
                }
                setShowProjectPicker(true);
              }}
              disabled={isMoving}
              title="Move to another project"
              className={`p-1.5 rounded-md transition-colors ${
                showProjectPicker
                  ? "text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-900/30"
                  : "text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800"
              }`}
            >
              {isMoving ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <FolderInput className="w-3.5 h-3.5" />
              )}
            </button>

            {/* Project picker dropdown — fixed positioning escapes overflow-auto containers */}
            {showProjectPicker && !pendingProject && dropdownPos && (
              <div
                style={{ top: dropdownPos.top, right: dropdownPos.right }}
                className="fixed z-50 w-52 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-xl overflow-hidden"
              >
                <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide px-3 py-2 border-b border-zinc-100 dark:border-zinc-700">
                  Move to project
                </p>
                {otherProjects.length === 0 ? (
                  <p className="text-xs text-zinc-400 px-3 py-3">
                    No other active projects available
                  </p>
                ) : (
                  <ul className="py-1 max-h-48 overflow-y-auto">
                    {otherProjects.map((p) => (
                      <li key={p.id}>
                        <button
                          onClick={() => handleProjectSelect(p)}
                          className="w-full text-left px-3 py-2 text-sm text-zinc-700 dark:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-700 flex items-center gap-2 transition-colors"
                        >
                          <span
                            className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                            style={{ backgroundColor: p.color ?? "#6366f1" }}
                          />
                          <span className="truncate">{p.name}</span>
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>

          {/* Delete */}
          {onDelete && (
            <button
              onClick={handleDelete}
              className="p-1.5 rounded-md text-zinc-400 hover:text-red-500 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
              title={t("meetings.delete")}
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* ── Confirm-move banner ───────────────────────────────────────────── */}
      {pendingProject && (
        <div
          className="mx-4 mb-3 p-3 rounded-lg bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800"
          onClick={(e) => e.stopPropagation()}
        >
          {moveError ? (
            <div className="flex items-start gap-2">
              <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <p className="text-xs text-red-600 dark:text-red-400">{moveError}</p>
                <button
                  onClick={cancelMove}
                  className="text-xs text-zinc-500 hover:text-zinc-700 mt-1 underline"
                >
                  Dismiss
                </button>
              </div>
            </div>
          ) : (
            <>
              <p className="text-sm text-indigo-800 dark:text-indigo-200 font-medium mb-1">
                Move to &ldquo;{pendingProject.name}&rdquo;?
              </p>
              <p className="text-xs text-indigo-600 dark:text-indigo-400 mb-3">
                {isCountLoading ? (
                  <span className="flex items-center gap-1">
                    <Loader2 className="w-3 h-3 animate-spin" /> Counting tasks…
                  </span>
                ) : taskCount === null || taskCount === 0 ? (
                  "No open tasks will be moved."
                ) : (
                  <>
                    <strong>{taskCount}</strong> open{" "}
                    {taskCount === 1 ? "task" : "tasks"} from this meeting will
                    also move. Completed tasks and tasks already in other
                    projects stay where they are.
                  </>
                )}
              </p>
              <div className="flex items-center gap-2">
                <button
                  onClick={confirmMove}
                  disabled={isMoving || isCountLoading}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                >
                  {isMoving ? (
                    <Loader2 className="w-3 h-3 animate-spin" />
                  ) : (
                    <FolderInput className="w-3 h-3" />
                  )}
                  {isMoving ? "Moving…" : "Confirm Move"}
                </button>
                <button
                  onClick={cancelMove}
                  disabled={isMoving}
                  className="px-3 py-1.5 text-xs font-medium rounded-md text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </>
          )}
        </div>
      )}

      {/* ── Expanded detail ───────────────────────────────────────────────── */}
      {expanded && (
        <div className="px-4 pb-4 pt-3 border-t border-zinc-100 dark:border-zinc-800/60 space-y-4 animate-fade-in">
          {meeting.summary && (
            <div>
              <p className="text-[10px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-widest mb-1.5">
                {t("meetings.summary")}
              </p>
              <p className="text-[13px] text-zinc-700 dark:text-zinc-300 leading-relaxed">{meeting.summary}</p>
            </div>
          )}

          {meeting.decisions && (
            <div>
              <p className="text-[10px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-widest mb-1.5">
                {t("meetings.decisions")}
              </p>
              <p className="text-[13px] text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap leading-relaxed">
                {meeting.decisions}
              </p>
            </div>
          )}

          {attendees.length > 0 && (
            <div>
              <p className="text-[10px] font-semibold text-zinc-400 dark:text-zinc-500 uppercase tracking-widest mb-1.5">
                {t("meetings.attendees")}
              </p>
              <div className="flex gap-1.5 flex-wrap">
                {attendees.map((a) => (
                  <span
                    key={a}
                    className="text-[11px] bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 px-2 py-0.5 rounded-full"
                  >
                    {a}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
