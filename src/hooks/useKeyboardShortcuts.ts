import { useEffect } from "react";
import { useUIStore } from "@/stores/uiStore";
import { useProjectStore } from "@/stores/projectStore";
import { useTaskStore } from "@/stores/taskStore";

export function useKeyboardShortcuts() {
  const {
    setCommandPaletteOpen,
    toggleRightPanel,
    toggleSidebar,
    setViewMode,
    setIngestModalOpen,
  } = useUIStore();
  const { clearSelection } = useTaskStore();

  const { activeProjectId } = useProjectStore();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const isCtrl = e.ctrlKey || e.metaKey;
      const isShift = e.shiftKey;
      const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
      const isInput = tag === "input" || tag === "textarea" || tag === "select";

      // Ctrl+K — Command palette
      if (isCtrl && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }

      // Escape — close modals / deselect
      if (e.key === "Escape") {
        clearSelection?.();
        setCommandPaletteOpen(false);
        return;
      }

      // Skip shortcuts when typing in inputs
      if (isInput) return;

      // ] — toggle right panel
      if (e.key === "]") {
        e.preventDefault();
        toggleRightPanel();
        return;
      }

      // [ — toggle sidebar
      if (e.key === "[") {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      // Ctrl+N — new task
      if (isCtrl && !isShift && e.key === "n") {
        e.preventDefault();
        // TODO: open new task modal
        return;
      }

      // Ctrl+Shift+N — new project
      if (isCtrl && isShift && e.key === "N") {
        e.preventDefault();
        // TODO: open new project modal
        return;
      }

      // Ctrl+M — new meeting ingest
      if (isCtrl && e.key === "m") {
        e.preventDefault();
        setIngestModalOpen(true);
        return;
      }

      // Ctrl+E — export current project
      if (isCtrl && e.key === "e") {
        e.preventDefault();
        // TODO: open export dialog
        return;
      }

      // Ctrl+1 — List view
      if (isCtrl && e.key === "1") {
        e.preventDefault();
        setViewMode("list");
        return;
      }

      // Ctrl+2 — Kanban view
      if (isCtrl && e.key === "2") {
        e.preventDefault();
        setViewMode("kanban");
        return;
      }

      // Ctrl+3 — Table view
      if (isCtrl && e.key === "3") {
        e.preventDefault();
        setViewMode("table");
        return;
      }

      // Ctrl+/ — focus AI chat
      if (isCtrl && e.key === "/") {
        e.preventDefault();
        const chatInput = document.getElementById("ai-chat-input");
        chatInput?.focus();
        return;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeProjectId]);
}
