import { useTranslation } from "react-i18next";
import {
  DndContext,
  DragEndEvent,
  DragOverlay,
  useDraggable,
  useDroppable,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { useState } from "react";
import { useTasks } from "@/hooks/useTasks";
import { useUIStore } from "@/stores/uiStore";
import { KANBAN_COLUMNS } from "@/lib/constants";
import TaskCard from "./TaskCard";
import type { Task } from "@/lib/tauri";
import { Plus } from "lucide-react";

// Per-column visual identity — richer Material You tonal surfaces
const COLUMN_CHROME: Record<string, {
  dot: string;
  header: string;
  label: string;
  count: string;
  zone: string;
  zoneOver: string;
  addBtn: string;
}> = {
  open: {
    dot: "bg-zinc-400",
    header: "bg-zinc-50 dark:bg-zinc-900/40 border-zinc-200 dark:border-zinc-800/60",
    label: "text-zinc-600 dark:text-zinc-400",
    count: "bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400",
    zone: "bg-zinc-50/60 dark:bg-zinc-800/20",
    zoneOver: "ring-2 ring-zinc-300 dark:ring-zinc-600 bg-zinc-100/80 dark:bg-zinc-700/30",
    addBtn: "border-zinc-200 dark:border-zinc-700/60 text-zinc-400 hover:text-zinc-600 hover:border-zinc-300 dark:hover:border-zinc-600 hover:bg-zinc-50 dark:hover:bg-zinc-800/40",
  },
  in_progress: {
    dot: "bg-indigo-500",
    header: "bg-indigo-50/80 dark:bg-indigo-950/40 border-indigo-200/60 dark:border-indigo-900/50",
    label: "text-indigo-700 dark:text-indigo-400",
    count: "bg-indigo-100 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-400",
    zone: "bg-indigo-50/30 dark:bg-indigo-950/20",
    zoneOver: "ring-2 ring-indigo-300 dark:ring-indigo-700/60 bg-indigo-50/70 dark:bg-indigo-950/40",
    addBtn: "border-indigo-200/60 dark:border-indigo-800/40 text-indigo-400 hover:text-indigo-600 hover:border-indigo-300 dark:hover:border-indigo-700 hover:bg-indigo-50/60 dark:hover:bg-indigo-950/30",
  },
  done: {
    dot: "bg-emerald-500",
    header: "bg-emerald-50/80 dark:bg-emerald-950/30 border-emerald-200/60 dark:border-emerald-900/50",
    label: "text-emerald-700 dark:text-emerald-500",
    count: "bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-500",
    zone: "bg-emerald-50/30 dark:bg-emerald-950/10",
    zoneOver: "ring-2 ring-emerald-300 dark:ring-emerald-700/60 bg-emerald-50/70 dark:bg-emerald-950/30",
    addBtn: "border-emerald-200/60 dark:border-emerald-800/40 text-emerald-400 hover:text-emerald-600 hover:border-emerald-300 dark:hover:border-emerald-700 hover:bg-emerald-50/60 dark:hover:bg-emerald-950/20",
  },
  cancelled: {
    dot: "bg-zinc-300",
    header: "bg-zinc-50 dark:bg-zinc-900/40 border-zinc-200 dark:border-zinc-800/60",
    label: "text-zinc-400 dark:text-zinc-500",
    count: "bg-zinc-100 dark:bg-zinc-800 text-zinc-400 dark:text-zinc-500",
    zone: "bg-zinc-50/40 dark:bg-zinc-800/10",
    zoneOver: "ring-2 ring-zinc-300 dark:ring-zinc-600 bg-zinc-100/60 dark:bg-zinc-700/20",
    addBtn: "border-zinc-200 dark:border-zinc-700/60 text-zinc-300 hover:text-zinc-500 hover:border-zinc-300 dark:hover:border-zinc-600 hover:bg-zinc-50 dark:hover:bg-zinc-800/30",
  },
};

interface Props {
  projectId: string | null;
}

function DraggableCard({ task }: { task: Task }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: task.id });
  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      className={isDragging ? "opacity-35 scale-[0.98]" : ""}
      style={{ transition: "opacity 120ms, transform 120ms" }}
    >
      <TaskCard task={task} />
    </div>
  );
}

function DroppableColumn({
  column,
  tasks,
  projectId,
  onAddTask,
  fluid,
}: {
  column: typeof KANBAN_COLUMNS[number];
  tasks: Task[];
  projectId: string;
  onAddTask: (status: string, title: string) => Promise<void>;
  fluid: boolean;
}) {
  const { t } = useTranslation();
  const { setNodeRef, isOver } = useDroppable({ id: column.id });
  const [adding, setAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");

  const handleAdd = async () => {
    if (!newTitle.trim()) return;
    await onAddTask(column.id, newTitle.trim());
    setNewTitle("");
    setAdding(false);
  };

  const chrome = COLUMN_CHROME[column.id] ?? COLUMN_CHROME.open;

  return (
    <div className={`flex flex-col h-full min-h-0 ${fluid ? "flex-1 min-w-[220px]" : "w-80 flex-shrink-0"}`}>
      {/* Column header — tonal pill */}
      <div className={`flex items-center justify-between mb-3 px-3 py-2 rounded-xl border ${chrome.header}`}>
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full flex-shrink-0 ${chrome.dot}`} />
          <span className={`text-[12px] font-semibold uppercase tracking-[0.06em] ${chrome.label}`}>
            {column.label}
          </span>
          <span className={`text-[11.5px] tabular-nums px-1.5 py-0.5 rounded-full leading-none font-medium ${chrome.count}`}>
            {tasks.length}
          </span>
        </div>
        <button
          onClick={() => setAdding(true)}
          title={t("tasks.addTask")}
          className={`p-1 rounded-lg transition-all duration-150 ${chrome.label} hover:bg-white/50 dark:hover:bg-white/5`}
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Drop zone */}
      <div
        ref={setNodeRef}
        className={`flex-1 overflow-y-auto rounded-xl p-2.5 space-y-2.5 min-h-[80px] transition-all duration-150 ${
          isOver ? chrome.zoneOver : chrome.zone
        }`}
      >
        {tasks.map((task) => (
          <DraggableCard key={task.id} task={task} />
        ))}

        {adding ? (
          <div className="space-y-2 p-1.5 animate-fade-in">
            <input
              type="text"
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              placeholder={t("tasks.titlePlaceholder")}
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
                if (e.key === "Escape") setAdding(false);
              }}
              className="w-full px-3 py-2 text-[13.5px] rounded-lg border border-[#e2e2e8] dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 outline-none focus-visible:ring-2 focus-visible:ring-indigo-400/40 focus:border-indigo-400 transition-all placeholder:text-zinc-400"
            />
            <div className="flex gap-1.5">
              <button onClick={handleAdd} className="px-3 py-1.5 text-[13px] font-medium bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg transition-colors shadow-sm">
                {t("common.add")}
              </button>
              <button onClick={() => setAdding(false)} className="px-3 py-1.5 text-[13px] text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors">
                {t("common.cancel")}
              </button>
            </div>
          </div>
        ) : (
          tasks.length === 0 && (
            <button
              onClick={() => setAdding(true)}
              className={`w-full py-4 flex items-center justify-center gap-2 text-[13px] rounded-xl transition-all duration-150 border border-dashed ${chrome.addBtn}`}
            >
              <Plus className="w-3.5 h-3.5" />
              {t("tasks.addTask")}
            </button>
          )
        )}
      </div>
    </div>
  );
}

export default function TaskKanbanView({ projectId }: Props) {
  const { tasks, updateTask, createTask, refetch } = useTasks(projectId);
  const { rightPanelOpen, activeView } = useUIStore();
  const fluid = !(rightPanelOpen && activeView !== "chat");
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveTask(null);
    if (!over || !projectId) return;
    const task = tasks.find((t) => t.id === active.id);
    if (!task || task.kanban_column === over.id) return;
    await updateTask({ id: task.id, kanban_column: String(over.id), status: String(over.id) });
  };

  const handleAddTask = async (status: string, title: string) => {
    if (!projectId) return;
    await createTask({ project_id: projectId, title, kanban_column: status });
  };

  if (!projectId) return null;

  return (
    <DndContext
      sensors={sensors}
      onDragStart={({ active }) => setActiveTask(tasks.find((t) => t.id === active.id) ?? null)}
      onDragEnd={handleDragEnd}
    >
      <div className="flex gap-5 p-5 overflow-x-auto h-full">
        {KANBAN_COLUMNS.map((col) => (
          <DroppableColumn
            key={col.id}
            column={col}
            tasks={tasks.filter((t) => t.kanban_column === col.id || t.status === col.id)}
            projectId={projectId}
            onAddTask={handleAddTask}
            fluid={fluid}
          />
        ))}
      </div>

      <DragOverlay>
        {activeTask && (
          <div className="rotate-1 shadow-xl opacity-95 scale-[1.02] transition-transform">
            <TaskCard task={activeTask} />
          </div>
        )}
      </DragOverlay>
    </DndContext>
  );
}
