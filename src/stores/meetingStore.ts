import { create } from "zustand";
import type { Meeting } from "@/lib/tauri";

interface MeetingStore {
  meetingsByProject: Record<string, Meeting[]>;
  // Actions
  setMeetings: (projectId: string, meetings: Meeting[]) => void;
  addMeeting: (meeting: Meeting) => void;
  updateMeeting: (meeting: Meeting) => void;
  removeMeeting: (meetingId: string, projectId: string) => void;
  getMeetingsForProject: (projectId: string) => Meeting[];
}

export const useMeetingStore = create<MeetingStore>((set, get) => ({
  meetingsByProject: {},

  setMeetings: (projectId, meetings) =>
    set((state) => ({
      meetingsByProject: { ...state.meetingsByProject, [projectId]: meetings },
    })),

  addMeeting: (meeting) =>
    set((state) => {
      const existing = state.meetingsByProject[meeting.project_id] || [];
      return {
        meetingsByProject: {
          ...state.meetingsByProject,
          [meeting.project_id]: [meeting, ...existing],
        },
      };
    }),

  updateMeeting: (meeting) =>
    set((state) => {
      const meetings = state.meetingsByProject[meeting.project_id] || [];
      return {
        meetingsByProject: {
          ...state.meetingsByProject,
          [meeting.project_id]: meetings.map((m) =>
            m.id === meeting.id ? meeting : m
          ),
        },
      };
    }),

  removeMeeting: (meetingId, projectId) =>
    set((state) => ({
      meetingsByProject: {
        ...state.meetingsByProject,
        [projectId]: (state.meetingsByProject[projectId] || []).filter(
          (m) => m.id !== meetingId
        ),
      },
    })),

  getMeetingsForProject: (projectId) =>
    get().meetingsByProject[projectId] || [],
}));
