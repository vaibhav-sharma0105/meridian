import { create } from "zustand";
import type { Task, TaskFilters } from "@/lib/tauri";

interface TaskStore {
  tasksByProject: Record<string, Task[]>;
  selectedTaskIds: string[];
  filters: TaskFilters;
  // Actions
  setTasks: (projectId: string, tasks: Task[]) => void;
  updateTaskLocally: (task: Task) => void;
  addTask: (task: Task) => void;
  removeTask: (taskId: string, projectId: string) => void;
  toggleTaskSelection: (taskId: string) => void;
  selectAllTasks: (taskIds: string[]) => void;
  clearSelection: () => void;
  clearTaskSelection: () => void; // alias
  setFilters: (filters: Partial<TaskFilters>) => void;
  clearFilters: () => void;
  getTasksForProject: (projectId: string) => Task[];
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasksByProject: {},
  selectedTaskIds: [],
  filters: {},

  setTasks: (projectId, tasks) =>
    set((state) => ({
      tasksByProject: { ...state.tasksByProject, [projectId]: tasks },
    })),

  updateTaskLocally: (task) =>
    set((state) => {
      const tasks = state.tasksByProject[task.project_id] || [];
      return {
        tasksByProject: {
          ...state.tasksByProject,
          [task.project_id]: tasks.map((t) => (t.id === task.id ? task : t)),
        },
      };
    }),

  addTask: (task) =>
    set((state) => {
      const tasks = state.tasksByProject[task.project_id] || [];
      return {
        tasksByProject: {
          ...state.tasksByProject,
          [task.project_id]: [task, ...tasks],
        },
      };
    }),

  removeTask: (taskId, projectId) =>
    set((state) => {
      const tasks = state.tasksByProject[projectId] || [];
      return {
        tasksByProject: {
          ...state.tasksByProject,
          [projectId]: tasks.filter((t) => t.id !== taskId),
        },
        selectedTaskIds: state.selectedTaskIds.filter((id) => id !== taskId),
      };
    }),

  toggleTaskSelection: (taskId) =>
    set((state) => ({
      selectedTaskIds: state.selectedTaskIds.includes(taskId)
        ? state.selectedTaskIds.filter((id) => id !== taskId)
        : [...state.selectedTaskIds, taskId],
    })),

  selectAllTasks: (taskIds) => set({ selectedTaskIds: taskIds }),

  clearSelection: () => set({ selectedTaskIds: [] }),
  clearTaskSelection: () => set({ selectedTaskIds: [] }),

  setFilters: (filters) =>
    set((state) => ({ filters: { ...state.filters, ...filters } })),

  clearFilters: () => set({ filters: {} }),

  getTasksForProject: (projectId) =>
    get().tasksByProject[projectId] || [],
}));
