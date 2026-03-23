import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, ChevronRight, Calendar, Users, Clock, Trash2 } from "lucide-react";
import MeetingHealthBadge from "./MeetingHealthBadge";
import { format } from "date-fns";
import type { Meeting } from "@/lib/tauri";

interface Props {
  meeting: Meeting;
  onDelete?: (id: string) => void;
}

export default function MeetingCard({ meeting, onDelete }: Props) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  function handleDelete(e: React.MouseEvent) {
    e.stopPropagation();
    if (window.confirm(t("meetings.confirmDelete", { title: meeting.title }))) {
      onDelete?.(meeting.id);
    }
  }

  const attendees = meeting.attendees ? meeting.attendees.split(",").map((a) => a.trim()).filter(Boolean) : [];

  return (
    <div className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg overflow-hidden">
      <div
        className="flex items-center gap-3 p-4 cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <button className="text-zinc-400 flex-shrink-0">
          {expanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
        </button>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-50 truncate">{meeting.title}</h3>
            {meeting.health_score !== null && meeting.health_score !== undefined && (
              <MeetingHealthBadge score={meeting.health_score} showLabel />
            )}
          </div>
          <div className="flex items-center gap-3 mt-1 flex-wrap">
            <span className="flex items-center gap-1 text-xs text-zinc-500">
              <Calendar className="w-3 h-3" />
              {format(new Date(meeting.created_at), "MMM d, yyyy")}
            </span>
            {meeting.platform && (
              <span className="text-xs text-zinc-400 capitalize">{meeting.platform}</span>
            )}
            {attendees.length > 0 && (
              <span className="flex items-center gap-1 text-xs text-zinc-500">
                <Users className="w-3 h-3" />
                {attendees.length} attendees
              </span>
            )}
            {meeting.duration_minutes && (
              <span className="flex items-center gap-1 text-xs text-zinc-500">
                <Clock className="w-3 h-3" />
                {meeting.duration_minutes}m
              </span>
            )}
          </div>
        </div>

        {onDelete && (
          <button
            onClick={handleDelete}
            className="flex-shrink-0 p-1.5 text-zinc-400 hover:text-red-500 dark:hover:text-red-400 rounded transition-colors"
            title={t("meetings.delete")}
          >
            <Trash2 className="w-4 h-4" />
          </button>
        )}
      </div>

      {expanded && (
        <div className="px-4 pb-4 border-t border-zinc-100 dark:border-zinc-800 pt-3 space-y-3">
          {meeting.summary && (
            <div>
              <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-1">{t("meetings.summary")}</p>
              <p className="text-sm text-zinc-700 dark:text-zinc-300">{meeting.summary}</p>
            </div>
          )}

          {meeting.decisions && (
            <div>
              <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-1">{t("meetings.decisions")}</p>
              <p className="text-sm text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap">{meeting.decisions}</p>
            </div>
          )}

          {attendees.length > 0 && (
            <div>
              <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-1">{t("meetings.attendees")}</p>
              <div className="flex gap-2 flex-wrap">
                {attendees.map((a) => (
                  <span key={a} className="text-xs bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 px-2 py-0.5 rounded-full">
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
