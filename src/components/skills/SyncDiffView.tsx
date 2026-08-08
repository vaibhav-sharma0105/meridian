import { useState } from "react";
import {
  RefreshCw,
  GitCompare,
  Check,
  X,
  AlertTriangle,
  Loader2,
} from "lucide-react";

interface DiffLine {
  type: "add" | "remove" | "context";
  content: string;
  lineNumber: number;
}

interface SyncDiffViewProps {
  skillId: string;
  skillName: string;
  localContent: string;
  remoteContent: string;
  onAcceptRemote: () => void;
  onKeepLocal: () => void;
  onMerge?: (merged: string) => void;
  loading?: boolean;
}

export function SyncDiffView({
  skillName,
  localContent,
  remoteContent,
  onAcceptRemote,
  onKeepLocal,
  loading,
}: SyncDiffViewProps) {
  const [showFullDiff, setShowFullDiff] = useState(false);

  const computeDiff = (): DiffLine[] => {
    const localLines = localContent.split("\n");
    const remoteLines = remoteContent.split("\n");
    const diff: DiffLine[] = [];

    const maxLines = Math.max(localLines.length, remoteLines.length);
    for (let i = 0; i < maxLines; i++) {
      const localLine = localLines[i];
      const remoteLine = remoteLines[i];

      if (localLine === remoteLine) {
        diff.push({ type: "context", content: localLine || "", lineNumber: i + 1 });
      } else {
        if (localLine !== undefined) {
          diff.push({ type: "remove", content: localLine, lineNumber: i + 1 });
        }
        if (remoteLine !== undefined) {
          diff.push({ type: "add", content: remoteLine, lineNumber: i + 1 });
        }
      }
    }

    return diff;
  };

  const diffLines = computeDiff();
  const changedLines = diffLines.filter((d) => d.type !== "context").length;
  const addedLines = diffLines.filter((d) => d.type === "add").length;
  const removedLines = diffLines.filter((d) => d.type === "remove").length;

  const displayDiff = showFullDiff
    ? diffLines
    : diffLines.filter((d) => d.type !== "context").slice(0, 20);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <GitCompare className="h-5 w-5 text-blue-500" />
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">
            Changes for {skillName}
          </h3>
        </div>
        <div className="flex items-center gap-2">
          <span className="px-2 py-0.5 text-xs rounded border border-green-200 text-green-600 dark:border-green-800 dark:text-green-400">
            +{addedLines}
          </span>
          <span className="px-2 py-0.5 text-xs rounded border border-red-200 text-red-600 dark:border-red-800 dark:text-red-400">
            -{removedLines}
          </span>
        </div>
      </div>
      <div className="p-4">
        {changedLines === 0 ? (
          <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg flex items-center gap-2">
            <Check className="h-4 w-4 text-green-500" />
            <p className="text-sm text-green-700 dark:text-green-400">
              No differences found. The skill is up to date.
            </p>
          </div>
        ) : (
          <>
            <div className="rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
              <div className="bg-gray-50 dark:bg-gray-900/50 px-4 py-2 text-sm font-medium border-b border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300">
                {changedLines} lines changed
              </div>
              <div className="max-h-96 overflow-auto font-mono text-sm">
                {displayDiff.map((line, idx) => (
                  <div
                    key={idx}
                    className={`px-4 py-0.5 ${
                      line.type === "add"
                        ? "bg-green-500/10 text-green-700 dark:text-green-400"
                        : line.type === "remove"
                        ? "bg-red-500/10 text-red-700 dark:text-red-400"
                        : "text-gray-500 dark:text-gray-400"
                    }`}
                  >
                    <span className="select-none mr-4 opacity-50">
                      {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                    </span>
                    {line.content}
                  </div>
                ))}
              </div>
              {!showFullDiff && diffLines.length > 20 && (
                <button
                  onClick={() => setShowFullDiff(true)}
                  className="w-full px-4 py-2 text-sm text-center text-gray-500 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                >
                  Show {diffLines.length - displayDiff.length} more lines...
                </button>
              )}
            </div>

            <div className="mt-4 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg flex items-start gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5" />
              <p className="text-sm text-amber-700 dark:text-amber-400">
                Accepting remote changes will overwrite your local modifications.
                Trust will be revoked until you re-grant it.
              </p>
            </div>

            <div className="flex gap-3 mt-4">
              <button
                onClick={onKeepLocal}
                disabled={loading}
                className="flex items-center gap-2 px-4 py-2 text-sm border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 disabled:opacity-50"
              >
                <X className="h-4 w-4" />
                Keep Local
              </button>
              <button
                onClick={onAcceptRemote}
                disabled={loading}
                className="flex items-center gap-2 px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
              >
                {loading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <RefreshCw className="h-4 w-4" />
                )}
                Accept Remote Changes
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
