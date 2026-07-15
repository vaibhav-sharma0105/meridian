import { useState, useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Sparkles, ChevronDown, ChevronUp, Plus, Trash2, Loader2, Check, AlertTriangle, RefreshCw } from "lucide-react";
import type { TaskPlan, Task } from "@/lib/tauri";
import * as api from "@/lib/tauri";

interface PlanSectionProps {
  taskId: string;
  taskTitle: string;
  onSubtasksCreated?: (tasks: Task[]) => void;
}

const complexityColors = {
  simple: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
  medium: "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
  complex: "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400",
};

const complexityLabels = {
  simple: "Simple",
  medium: "Medium",
  complex: "Complex",
};

export function PlanSection({ taskId, taskTitle, onSubtasksCreated }: PlanSectionProps) {
  const queryClient = useQueryClient();
  const [expanded, setExpanded] = useState(false);
  const [editedSubtasks, setEditedSubtasks] = useState<string[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isCreating, setIsCreating] = useState(false);

  const { data: plan, isLoading } = useQuery({
    queryKey: ["taskPlan", taskId],
    queryFn: () => api.getTaskPlan(taskId),
  });

  useEffect(() => {
    if (plan?.suggested_subtasks) {
      setEditedSubtasks([...plan.suggested_subtasks]);
    }
  }, [plan]);

  const handleGeneratePlan = async () => {
    setIsGenerating(true);
    try {
      await api.evaluateTaskPlan(taskId);
      queryClient.invalidateQueries({ queryKey: ["taskPlan", taskId] });
    } finally {
      setIsGenerating(false);
    }
  };

  const handleCreateSubtasks = async () => {
    if (editedSubtasks.length === 0) return;

    setIsCreating(true);
    try {
      if (plan && editedSubtasks.join(",") !== plan.suggested_subtasks.join(",")) {
        await api.recordPlanCorrection(
          taskId,
          plan.suggested_subtasks,
          editedSubtasks,
          "create_with_edits"
        );
      }

      const newTasks = await api.acceptPlan(taskId, editedSubtasks);
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      onSubtasksCreated?.(newTasks);
    } finally {
      setIsCreating(false);
    }
  };

  const updateSubtask = (index: number, value: string) => {
    const updated = [...editedSubtasks];
    updated[index] = value;
    setEditedSubtasks(updated);
  };

  const removeSubtask = (index: number) => {
    setEditedSubtasks(editedSubtasks.filter((_, i) => i !== index));
  };

  const addSubtask = () => {
    setEditedSubtasks([...editedSubtasks, ""]);
  };

  if (isLoading) {
    return (
      <div className="p-3 text-center text-sm text-zinc-500">
        <Loader2 className="w-4 h-4 mx-auto animate-spin" />
      </div>
    );
  }

  if (!plan) {
    return (
      <div className="border border-dashed border-zinc-200 dark:border-zinc-700 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-sm text-zinc-500 dark:text-zinc-400">
            <Sparkles className="w-4 h-4" />
            <span>AI can analyze this task's complexity</span>
          </div>
          <button
            onClick={handleGeneratePlan}
            disabled={isGenerating}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded-lg disabled:opacity-50"
          >
            {isGenerating ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Sparkles className="w-3 h-3" />
            )}
            Analyze
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex-1 flex items-center gap-3"
        >
          <Sparkles className="w-4 h-4 text-indigo-500" />
          <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
            Task Plan
          </span>
          <span className={`px-2 py-0.5 text-xs font-medium rounded ${complexityColors[plan.complexity as keyof typeof complexityColors]}`}>
            {complexityLabels[plan.complexity as keyof typeof complexityLabels]}
          </span>
        </button>
        <div className="flex items-center gap-2">
          <button
            onClick={handleGeneratePlan}
            disabled={isGenerating}
            title="Re-analyze task"
            className="p-1.5 text-zinc-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded disabled:opacity-50"
          >
            {isGenerating ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <RefreshCw className="w-3.5 h-3.5" />
            )}
          </button>
          <button onClick={() => setExpanded(!expanded)}>
            {expanded ? (
              <ChevronUp className="w-4 h-4 text-zinc-400" />
            ) : (
              <ChevronDown className="w-4 h-4 text-zinc-400" />
            )}
          </button>
        </div>
      </div>

      {expanded && (
        <div className="px-4 pb-4 space-y-4 border-t border-zinc-100 dark:border-zinc-800">
          <p className="text-xs text-zinc-500 dark:text-zinc-400 pt-3">
            {plan.reasoning}
          </p>

          {plan.complexity === "complex" && (
            <div className="flex items-start gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 rounded-lg">
              <AlertTriangle className="w-4 h-4 text-amber-500 mt-0.5 flex-shrink-0" />
              <div>
                <p className="text-sm font-medium text-amber-700 dark:text-amber-300">
                  This task needs breakdown
                </p>
                <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                  Consider splitting into smaller tasks before starting.
                </p>
              </div>
            </div>
          )}

          {plan.suggested_action && plan.complexity === "simple" && (
            <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <p className="text-sm text-green-700 dark:text-green-300">
                {plan.suggested_action}
              </p>
            </div>
          )}

          {editedSubtasks.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wide">
                  Suggested Subtasks
                </span>
                <button
                  onClick={addSubtask}
                  className="flex items-center gap-1 text-xs text-zinc-500 hover:text-indigo-500"
                >
                  <Plus className="w-3 h-3" />
                  Add
                </button>
              </div>
              <div className="pl-3 border-l-2 border-zinc-200 dark:border-zinc-700 space-y-1">
                {editedSubtasks.map((subtask, index) => (
                  <div key={index} className="flex items-center gap-2 group hover:bg-zinc-50 dark:hover:bg-zinc-800/50 rounded px-1 -mx-1 py-0.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-zinc-300 dark:bg-zinc-600 flex-shrink-0" />
                    <input
                      type="text"
                      value={subtask}
                      onChange={(e) => updateSubtask(index, e.target.value)}
                      className="flex-1 px-2 py-1 text-[13px] border-0 border-b border-transparent focus:border-zinc-200 dark:focus:border-zinc-700 bg-transparent text-zinc-700 dark:text-zinc-300 focus:outline-none"
                      placeholder="Subtask..."
                    />
                    <button
                      onClick={() => removeSubtask(index)}
                      className="p-1 text-zinc-400 opacity-0 group-hover:opacity-100 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all"
                      title="Remove subtask"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>

              <button
                onClick={handleCreateSubtasks}
                disabled={isCreating || editedSubtasks.filter((s) => s.trim()).length === 0}
                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-white bg-indigo-500 hover:bg-indigo-600 rounded disabled:opacity-50"
              >
                {isCreating ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <Check className="w-3 h-3" />
                )}
                Create {editedSubtasks.filter((s) => s.trim()).length} Subtasks
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
