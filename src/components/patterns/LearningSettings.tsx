import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Brain,
  ChevronRight,
  Download,
  Upload,
  RotateCcw,
  Workflow,
  MessageSquare,
  Tag,
  User,
  AlertTriangle,
} from "lucide-react";
import * as api from "../../lib/tauri";

export function LearningSettings() {
  const queryClient = useQueryClient();
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [resetConfirmText, setResetConfirmText] = useState("");

  const { data: summaries = [], isLoading } = useQuery({
    queryKey: ["pattern-summaries"],
    queryFn: () => api.getPatternSummaries(),
  });

  const handleExport = async () => {
    try {
      const data = await api.exportLearningData();
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `meridian-learning-${new Date().toISOString().split("T")[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("Export failed:", e);
    }
  };

  const handleImport = async () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const data = JSON.parse(text) as api.LearningImport;
        await api.importLearningData(data);
        queryClient.invalidateQueries({ queryKey: ["pattern-summaries"] });
      } catch (err) {
        console.error("Import failed:", err);
      }
    };
    input.click();
  };

  const handleResetCategory = async (patternType: string) => {
    try {
      await api.resetPatternCategory(patternType);
      queryClient.invalidateQueries({ queryKey: ["pattern-summaries"] });
      setSelectedCategory(null);
    } catch (e) {
      console.error("Reset failed:", e);
    }
  };

  const handleResetAll = async () => {
    if (resetConfirmText !== "RESET") return;
    try {
      await api.resetAllLearning();
      queryClient.invalidateQueries({ queryKey: ["pattern-summaries"] });
      setShowResetConfirm(false);
      setResetConfirmText("");
    } catch (e) {
      console.error("Reset all failed:", e);
    }
  };

  const getCategoryIcon = (type: string) => {
    switch (type) {
      case "workflow_sequence":
        return Workflow;
      case "communication_style":
        return MessageSquare;
      case "smart_defaults":
        return Tag;
      default:
        return Brain;
    }
  };

  const getCategoryLabel = (type: string) => {
    switch (type) {
      case "workflow_sequence":
        return "Workflow Sequences";
      case "communication_style":
        return "Communication Style";
      case "smart_defaults":
        return "Smart Defaults";
      default:
        return type;
    }
  };

  if (selectedCategory) {
    return (
      <CategoryDetail
        patternType={selectedCategory}
        onBack={() => setSelectedCategory(null)}
        onReset={() => handleResetCategory(selectedCategory)}
      />
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2">
        <Brain className="w-5 h-5 text-indigo-500" />
        <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">Learning</h2>
      </div>

      <p className="text-sm text-zinc-500 dark:text-zinc-400">
        Meridian learns from your patterns to provide better suggestions. View what's been learned and manage your preferences.
      </p>

      {isLoading ? (
        <div className="text-sm text-zinc-400">Loading...</div>
      ) : summaries.length === 0 ? (
        <div className="p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg text-sm text-zinc-500 dark:text-zinc-400 text-center">
          No patterns learned yet. Use the app normally and patterns will emerge over time.
        </div>
      ) : (
        <div className="space-y-2">
          {summaries.map((summary) => {
            const Icon = getCategoryIcon(summary.pattern_type);
            return (
              <button
                key={summary.pattern_type}
                onClick={() => setSelectedCategory(summary.pattern_type)}
                className="w-full flex items-center gap-3 p-3 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors text-left"
              >
                <Icon className="w-5 h-5 text-zinc-400" />
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-zinc-900 dark:text-zinc-100">
                    {getCategoryLabel(summary.pattern_type)}
                  </div>
                  <div className="text-xs text-zinc-500 dark:text-zinc-400">
                    {summary.observation_count} observations · {Math.round(summary.confidence * 100)}% confidence
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <ConfidenceMeter value={summary.confidence} />
                  <ChevronRight className="w-4 h-4 text-zinc-400" />
                </div>
              </button>
            );
          })}
        </div>
      )}

      <div className="border-t border-zinc-200 dark:border-zinc-700 pt-4 space-y-3">
        <div className="flex gap-2">
          <button
            onClick={handleExport}
            className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 rounded-lg hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
          >
            <Download className="w-4 h-4" />
            Export
          </button>
          <button
            onClick={handleImport}
            className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 rounded-lg hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
          >
            <Upload className="w-4 h-4" />
            Import
          </button>
        </div>

        {!showResetConfirm ? (
          <button
            onClick={() => setShowResetConfirm(true)}
            className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 rounded-lg transition-colors"
          >
            <RotateCcw className="w-4 h-4" />
            Reset All Learning
          </button>
        ) : (
          <div className="p-3 bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800 rounded-lg space-y-2">
            <div className="flex items-start gap-2 text-sm text-red-600 dark:text-red-400">
              <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
              <span>This will delete all learned patterns and observations. Type RESET to confirm.</span>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                value={resetConfirmText}
                onChange={(e) => setResetConfirmText(e.target.value)}
                placeholder="Type RESET"
                className="flex-1 px-2 py-1 text-sm border border-red-300 dark:border-red-700 rounded bg-white dark:bg-zinc-900"
              />
              <button
                onClick={handleResetAll}
                disabled={resetConfirmText !== "RESET"}
                className="px-3 py-1 text-sm font-medium text-white bg-red-600 rounded hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Confirm
              </button>
              <button
                onClick={() => {
                  setShowResetConfirm(false);
                  setResetConfirmText("");
                }}
                className="px-3 py-1 text-sm text-zinc-600 dark:text-zinc-400 hover:text-zinc-800 dark:hover:text-zinc-200"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ConfidenceMeter({ value }: { value: number }) {
  const percentage = Math.round(value * 100);
  const color = percentage >= 70 ? "bg-green-500" : percentage >= 40 ? "bg-yellow-500" : "bg-zinc-300";

  return (
    <div className="w-16 h-1.5 bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
      <div className={`h-full ${color} transition-all`} style={{ width: `${percentage}%` }} />
    </div>
  );
}

interface CategoryDetailProps {
  patternType: string;
  onBack: () => void;
  onReset: () => void;
}

function CategoryDetail({ patternType, onBack, onReset }: CategoryDetailProps) {
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  const { data: model } = useQuery({
    queryKey: ["pattern-model", patternType],
    queryFn: () => api.getPatternModel(patternType),
  });

  const getCategoryLabel = (type: string) => {
    switch (type) {
      case "workflow_sequence":
        return "Workflow Sequences";
      case "communication_style":
        return "Communication Style";
      case "smart_defaults":
        return "Smart Defaults";
      default:
        return type;
    }
  };

  const renderModelData = () => {
    if (!model) return null;

    try {
      const data = JSON.parse(model.model_data);

      if (patternType === "workflow_sequence") {
        const sequences = (data as api.WorkflowSequenceModelData).sequences || [];
        return (
          <div className="space-y-2">
            {sequences.length === 0 ? (
              <p className="text-sm text-zinc-500">No sequences learned yet.</p>
            ) : (
              sequences.map((seq, i) => (
                <div key={i} className="p-2 bg-zinc-50 dark:bg-zinc-800 rounded text-sm">
                  <span className="text-zinc-500">After</span>{" "}
                  <span className="font-medium text-zinc-900 dark:text-zinc-100">{seq.trigger_action}</span>
                  <span className="text-zinc-500">, you usually</span>{" "}
                  <span className="font-medium text-zinc-900 dark:text-zinc-100">{seq.follow_action}</span>
                  <span className="text-zinc-400 ml-2">({seq.occurrence_count}x)</span>
                </div>
              ))
            )}
          </div>
        );
      }

      if (patternType === "smart_defaults") {
        const defaults = data as api.SmartDefaultsModelData;
        return (
          <div className="space-y-4">
            {defaults.priority_patterns.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">Priority Patterns</h4>
                <div className="space-y-1">
                  {defaults.priority_patterns.map((p, i) => (
                    <div key={i} className="text-sm">
                      <span className="text-zinc-500">Tasks with</span>{" "}
                      <span className="font-medium">"{p.keyword}"</span>{" "}
                      <span className="text-zinc-500">→</span>{" "}
                      <span className={`font-medium ${p.priority === "high" || p.priority === "critical" ? "text-red-600" : ""}`}>
                        {p.priority}
                      </span>
                      <span className="text-zinc-400 ml-2">({p.occurrence_count}x)</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
            {defaults.assignee_patterns.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">Assignee Patterns</h4>
                <div className="space-y-1">
                  {defaults.assignee_patterns.map((p, i) => (
                    <div key={i} className="text-sm">
                      <span className="text-zinc-500">Tasks with</span>{" "}
                      <span className="font-medium">"{p.keyword}"</span>{" "}
                      <span className="text-zinc-500">→</span>{" "}
                      <span className="font-medium">{p.assignee}</span>
                      <span className="text-zinc-400 ml-2">({p.occurrence_count}x)</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        );
      }

      if (patternType === "communication_style") {
        const style = data as api.CommunicationStyleModelData;
        return (
          <div className="space-y-3 text-sm">
            <div>
              <span className="text-zinc-500">Length preference:</span>{" "}
              <span className="font-medium">{style.length_preference}</span>
            </div>
            <div>
              <span className="text-zinc-500">Formality:</span>{" "}
              <span className="font-medium">{style.formality_level}</span>
            </div>
            {style.common_additions.length > 0 && (
              <div>
                <span className="text-zinc-500">Common additions:</span>{" "}
                {style.common_additions.slice(0, 5).map(([phrase, count], i) => (
                  <span key={i} className="ml-2 px-1.5 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded text-xs">
                    +{phrase} ({count}x)
                  </span>
                ))}
              </div>
            )}
          </div>
        );
      }

      return <pre className="text-xs overflow-auto">{JSON.stringify(data, null, 2)}</pre>;
    } catch {
      return <p className="text-sm text-zinc-500">Unable to parse pattern data.</p>;
    }
  };

  return (
    <div className="space-y-4">
      <button onClick={onBack} className="flex items-center gap-1 text-sm text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300">
        <ChevronRight className="w-4 h-4 rotate-180" />
        Back to Learning
      </button>

      <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">{getCategoryLabel(patternType)}</h2>

      {model && (
        <div className="text-sm text-zinc-500">
          {model.observation_count} observations · {Math.round(model.confidence * 100)}% confidence · Updated{" "}
          {new Date(model.last_updated).toLocaleDateString()}
        </div>
      )}

      <div className="p-4 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg">
        {renderModelData()}
      </div>

      <div className="border-t border-zinc-200 dark:border-zinc-700 pt-4">
        {!showResetConfirm ? (
          <button
            onClick={() => setShowResetConfirm(true)}
            className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 rounded-lg transition-colors"
          >
            <RotateCcw className="w-4 h-4" />
            Reset This Category
          </button>
        ) : (
          <div className="flex items-center gap-3">
            <span className="text-sm text-red-600 dark:text-red-400">Reset all learned patterns in this category?</span>
            <button
              onClick={() => {
                onReset();
                setShowResetConfirm(false);
              }}
              className="px-3 py-1 text-sm font-medium text-white bg-red-600 rounded hover:bg-red-700 transition-colors"
            >
              Yes, Reset
            </button>
            <button onClick={() => setShowResetConfirm(false)} className="text-sm text-zinc-500 hover:text-zinc-700">
              Cancel
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
