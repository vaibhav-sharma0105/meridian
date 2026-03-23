import { create } from "zustand";
import * as api from "@/lib/tauri";
import type { Project, CreateProjectInput, UpdateProjectInput } from "@/lib/tauri";

interface ProjectStore {
  projects: Project[];
  activeProjectId: string | null;
  loading: boolean;
  error: string | null;
  // Actions
  fetchProjects: () => Promise<void>;
  loadProjects: () => Promise<void>; // alias for fetchProjects
  setActiveProject: (id: string | null) => void;
  createProject: (input: CreateProjectInput) => Promise<Project>;
  updateProject: (input: UpdateProjectInput) => Promise<Project>;
  archiveProject: (id: string) => Promise<void>;
  getActiveProject: () => Project | null;
}

export const useProjectStore = create<ProjectStore>((set, get) => ({
  projects: [],
  activeProjectId: null,
  loading: false,
  error: null,

  loadProjects: async () => {
    const store = get();
    return store.fetchProjects();
  },

  fetchProjects: async () => {
    set({ loading: true, error: null });
    try {
      const projects = await api.getProjects();
      set({ projects, loading: false });
      // Set first project as active if none selected
      if (!get().activeProjectId && projects.length > 0) {
        set({ activeProjectId: projects[0].id });
      }
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  setActiveProject: (id) => set({ activeProjectId: id }),

  createProject: async (input) => {
    const project = await api.createProject(input);
    set((state) => ({
      projects: [project, ...state.projects],
      activeProjectId: project.id,
    }));
    return project;
  },

  updateProject: async (input) => {
    const updated = await api.updateProject(input);
    set((state) => ({
      projects: state.projects.map((p) =>
        p.id === updated.id ? updated : p
      ),
    }));
    return updated;
  },

  archiveProject: async (id) => {
    await api.archiveProject(id);
    set((state) => {
      const remaining = state.projects.filter((p) => p.id !== id);
      const newActive =
        state.activeProjectId === id
          ? remaining[0]?.id ?? null
          : state.activeProjectId;
      return { projects: remaining, activeProjectId: newActive };
    });
  },

  getActiveProject: () => {
    const { projects, activeProjectId } = get();
    return projects.find((p) => p.id === activeProjectId) ?? null;
  },
}));
