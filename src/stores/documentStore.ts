import { create } from "zustand";
import type { Document } from "@/lib/tauri";

interface DocumentStore {
  documentsByProject: Record<string, Document[]>;
  // Actions
  setDocuments: (projectId: string, docs: Document[]) => void;
  addDocument: (doc: Document) => void;
  removeDocument: (docId: string, projectId: string) => void;
  getDocumentsForProject: (projectId: string) => Document[];
}

export const useDocumentStore = create<DocumentStore>((set, get) => ({
  documentsByProject: {},

  setDocuments: (projectId, docs) =>
    set((state) => ({
      documentsByProject: { ...state.documentsByProject, [projectId]: docs },
    })),

  addDocument: (doc) =>
    set((state) => {
      const existing = state.documentsByProject[doc.project_id] || [];
      return {
        documentsByProject: {
          ...state.documentsByProject,
          [doc.project_id]: [doc, ...existing],
        },
      };
    }),

  removeDocument: (docId, projectId) =>
    set((state) => ({
      documentsByProject: {
        ...state.documentsByProject,
        [projectId]: (state.documentsByProject[projectId] || []).filter(
          (d) => d.id !== docId
        ),
      },
    })),

  getDocumentsForProject: (projectId) =>
    get().documentsByProject[projectId] || [],
}));
