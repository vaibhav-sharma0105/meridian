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
      <TaskCard task={task} compact />
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

  return (
    <div className="w-72 flex-shrink-0 flex flex-col h-full min-h-0">
      <div className="flex items-center justify-between mb-2 px-1">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-zinc-700 dark:text-zinc-300 uppercase tracking-wide">{column.label}</span>
          <span className="text-xs text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 rounded-full">{tasks.length}</span>
        </div>
        <button onClick={() => setAdding(true)} className="p-0.5 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300">
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      <div
        ref={setNodeRef}
        className={`flex-1 overflow-y-auto rounded-lg p-2 space-y-2 min-h-[80px] transition-colors ${isOver ? "bg-indigo-50 dark:bg-indigo-900/10" : "bg-zinc-50 dark:bg-zinc-800/50"}`}
      >
        {tasks.map((task) => (
          <DraggableCard key={task.id} task={task} />
        ))}

        {adding && (
          <div className="space-y-1">
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
              className="w-full px-2 py-1.5 text-xs rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50"
            />
            <div className="flex gap-1">
              <button onClick={handleAdd} className="px-2 py-1 text-xs bg-indigo-500 text-white rounded">
                {t("common.add")}
              </button>
              <button onClick={() => setAdding(false)} className="px-2 py-1 text-xs text-zinc-500 hover:text-zinc-700">
                {t("common.cancel")}
              </button>
            </div>
          </div>
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
        {activeTask && <TaskCard task={activeTask} compact />}
      </DragOverlay>
    </DndContext>
  );
}
