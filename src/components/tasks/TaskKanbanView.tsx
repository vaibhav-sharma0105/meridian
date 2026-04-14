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
import { KANBAN_COLUMNS } from "@/lib/constants";
import TaskCard from "./TaskCard";
import type { Task } from "@/lib/tauri";
import { Plus } from "lucide-react";

// Per-column visual identity — dot color + drop-zone accent
const COLUMN_CHROME: Record<string, { dot: string; over: string; label: string }> = {
  open:        { dot: "bg-zinc-400",    over: "ring-1 ring-zinc-300 dark:ring-zinc-600 bg-zinc-100/60 dark:bg-zinc-700/30",    label: "text-zinc-600 dark:text-zinc-400" },
  in_progress: { dot: "bg-indigo-500",  over: "ring-1 ring-indigo-300 dark:ring-indigo-700/60 bg-indigo-50/60 dark:bg-indigo-950/30", label: "text-indigo-700 dark:text-indigo-400" },
  done:        { dot: "bg-emerald-500", over: "ring-1 ring-emerald-300 dark:ring-emerald-700/60 bg-emerald-50/60 dark:bg-emerald-950/30", label: "text-emerald-700 dark:text-emerald-500" },
  cancelled:   { dot: "bg-zinc-300",    over: "ring-1 ring-zinc-300 dark:ring-zinc-600 bg-zinc-100/60 dark:bg-zinc-700/30",    label: "text-zinc-500 dark:text-zinc-500" },
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
      className={isDragging ? "opacity-40" : ""}
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
}: {
  column: typeof KANBAN_COLUMNS[number];
  tasks: Task[];
  projectId: string;
  onAddTask: (status: string, title: string) => Promise<void>;
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
    <div className="w-72 flex-shrink-0 flex flex-col h-full min-h-0">
      {/* Column header */}
      <div className="flex items-center justify-between mb-2 px-0.5">
        <div className="flex items-center gap-2">
          <span className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${chrome.dot}`} />
          <span className={`text-[11px] font-semibold uppercase tracking-[0.06em] ${chrome.label}`}>
            {column.label}
          </span>
          <span className="text-[11px] tabular-nums text-zinc-400 dark:text-zinc-500 bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 rounded-full leading-none">
            {tasks.length}
          </span>
        </div>
        <button
          onClick={() => setAdding(true)}
          title={t("tasks.addTask")}
          className="p-1 rounded-md text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
        >
          <Plus className="w-3 h-3" />
        </button>
      </div>

      {/* Drop zone */}
      <div
        ref={setNodeRef}
        className={`flex-1 overflow-y-auto rounded-lg p-2 space-y-2 min-h-[80px] transition-all duration-150 ${
          isOver
            ? chrome.over
            : "bg-zinc-50/80 dark:bg-zinc-800/30"
        }`}
      >
        {tasks.map((task) => (
          <DraggableCard key={task.id} task={task} />
        ))}

        {adding ? (
          <div className="space-y-1.5 p-1">
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
              className="w-full px-2.5 py-1.5 text-[12px] rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 outline-none focus:ring-2 focus:ring-indigo-400/40 focus:border-indigo-400"
            />
            <div className="flex gap-1">
              <button onClick={handleAdd} className="px-2.5 py-1 text-[12px] font-medium bg-indigo-500 hover:bg-indigo-600 text-white rounded-md transition-colors">
                {t("common.add")}
              </button>
              <button onClick={() => setAdding(false)} className="px-2.5 py-1 text-[12px] text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors">
                {t("common.cancel")}
              </button>
            </div>
          </div>
        ) : (
          tasks.length === 0 && (
            <button
              onClick={() => setAdding(true)}
              className="w-full py-3 flex items-center justify-center gap-1.5 text-[12px] text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 hover:bg-zinc-100/60 dark:hover:bg-zinc-700/30 rounded-md transition-colors border border-dashed border-zinc-200 dark:border-zinc-700/60"
            >
              <Plus className="w-3 h-3" />
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
      <div className="flex gap-4 p-4 overflow-x-auto h-full">
        {KANBAN_COLUMNS.map((col) => (
          <DroppableColumn
            key={col.id}
            column={col}
            tasks={tasks.filter((t) => t.kanban_column === col.id || t.status === col.id)}
            projectId={projectId}
            onAddTask={handleAddTask}
          />
        ))}
      </div>

      <DragOverlay>
        {activeTask && <TaskCard task={activeTask} />}
      </DragOverlay>
    </DndContext>
  );
}
