import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { FileText, File, Trash2, Brain, CheckCircle, AlertCircle, RefreshCw, Clock, Loader, Info } from "lucide-react";
import { deleteDocument, embedDocumentChunks, onEmbedProgress, getDocumentEmbeddingStatus, retryDocumentEmbedding, type DocumentEmbeddingStatus } from "@/lib/tauri";
import ConfirmDialog from "@/components/shared/ConfirmDialog";
import { format } from "date-fns";
import type { Document } from "@/lib/tauri";

interface Props {
  doc: Document;
  onDeleted: () => void;
  onClick?: () => void;
}

const EXT_ICONS: Record<string, typeof FileText> = {
  pdf: FileText,
  docx: FileText,
  txt: File,
  md: File,
};

export default function DocCard({ doc, onDeleted, onClick }: Props) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [embedding, setEmbedding] = useState(false);
  const [embedProgress, setEmbedProgress] = useState<{ progress: number; total: number } | null>(null);
  const [embeddingsReady, setEmbeddingsReady] = useState(doc.embeddings_ready);
  const [embeddingStatus, setEmbeddingStatus] = useState<DocumentEmbeddingStatus | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  useEffect(() => {
    if (embeddingsReady) return;

    const checkStatus = async () => {
      try {
        const status = await getDocumentEmbeddingStatus(doc.id);
        setEmbeddingStatus(status);
        if (status.embeddings_ready) {
          setEmbeddingsReady(true);
        }
      } catch {
        // Ignore errors - status check is optional
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, [doc.id, embeddingsReady]);

  const ext = doc.file_type?.toLowerCase() ?? doc.filename?.split(".").pop()?.toLowerCase() ?? "file";
  const Icon = EXT_ICONS[ext] ?? File;
  const displayName = doc.title || doc.filename;
  const uploadedAt = doc.created_at || doc.uploaded_at;

  const handleDelete = async () => {
    await deleteDocument(doc.id);
    onDeleted();
  };

  const handleEmbed = async () => {
    setEmbedding(true);
    setEmbedProgress(null);

    // Subscribe to progress events for this document
    const unlisten = await onEmbedProgress((data) => {
      if (data.document_id !== doc.id) return;
      setEmbedProgress({ progress: data.progress, total: data.total });
    });
    unlistenRef.current = unlisten;

    try {
      await embedDocumentChunks(doc.id);
      setEmbeddingsReady(true);
    } finally {
      unlisten();
      unlistenRef.current = null;
      setEmbedding(false);
      setEmbedProgress(null);
    }
  };

  const handleRetry = async () => {
    try {
      await retryDocumentEmbedding(doc.id);
      setEmbeddingStatus(prev => prev ? { ...prev, job_status: "pending", job_error: null } : null);
    } catch (e) {
      console.error("Failed to retry embedding:", e);
    }
  };

  const progressPct =
    embedProgress && embedProgress.total > 0
      ? Math.round((embedProgress.progress / embedProgress.total) * 100)
      : 0;

  return (
    <>
      <div className="flex items-center gap-3 p-3 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg hover:shadow-sm transition-shadow group">
        <button
          onClick={onClick}
          className="flex items-center gap-3 flex-1 min-w-0 text-left"
        >
          <div className="flex-shrink-0 w-8 h-8 bg-indigo-50 dark:bg-indigo-900/30 rounded-lg flex items-center justify-center">
            <Icon className="w-4 h-4 text-indigo-500" />
          </div>

          <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-zinc-900 dark:text-zinc-50 truncate">{displayName}</p>
          {embedding && embedProgress ? (
            <div className="mt-1">
              <div className="flex items-center justify-between mb-0.5">
                <span className="text-xs text-indigo-500">Embedding… {progressPct}%</span>
                <span className="text-xs text-zinc-400">{embedProgress.progress}/{embedProgress.total}</span>
              </div>
              <div className="h-1 w-full bg-zinc-200 dark:bg-zinc-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-indigo-500 rounded-full transition-all duration-200"
                  style={{ width: `${progressPct}%` }}
                />
              </div>
            </div>
          ) : embeddingsReady ? (
            <p className="text-xs text-emerald-500 flex items-center gap-1 mt-0.5">
              <CheckCircle className="w-3 h-3" />
              Embeddings ready
            </p>
          ) : embeddingStatus?.job_status === "running" ? (
            <p className="text-xs text-indigo-500 flex items-center gap-1 mt-0.5">
              <Loader className="w-3 h-3 animate-spin" />
              Embedding in progress...
            </p>
          ) : embeddingStatus?.job_status === "pending" ? (
            <p className="text-xs text-amber-500 flex items-center gap-1 mt-0.5">
              <Clock className="w-3 h-3" />
              Queued for embedding
            </p>
          ) : embeddingStatus?.job_error ? (
            <p className="text-xs text-red-500 flex items-center gap-1 mt-0.5">
              <AlertCircle className="w-3 h-3" />
              Embedding failed
            </p>
          ) : !embeddingsReady && embeddingStatus && !embeddingStatus.job_status ? (
            <p className="text-xs text-zinc-400 flex items-center gap-1 mt-0.5">
              <Info className="w-3 h-3" />
              Not indexed · click to embed
            </p>
          ) : (
            <p className="text-xs text-zinc-500">
              {uploadedAt && format(new Date(uploadedAt), "MMM d, yyyy")}
              {ext && ` · .${ext.toUpperCase()}`}
            </p>
          )}
          </div>
        </button>

        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          {embeddingStatus?.job_error ? (
            <button
              onClick={handleRetry}
              title="Retry embedding"
              className="p-1.5 text-zinc-400 hover:text-amber-500 hover:bg-amber-50 dark:hover:bg-amber-900/20 rounded transition-colors"
            >
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
          ) : (
            <button
              onClick={handleEmbed}
              disabled={embedding || embeddingsReady || embeddingStatus?.job_status === "running" || embeddingStatus?.job_status === "pending"}
              title={t("documents.embed")}
              className="p-1.5 text-zinc-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Brain className={`w-3.5 h-3.5 ${embedding ? "animate-pulse text-indigo-500" : ""}`} />
            </button>
          )}
          <button
            onClick={() => setConfirmDelete(true)}
            title={t("common.delete")}
            className="p-1.5 text-zinc-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={confirmDelete}
        title={t("documents.deleteTitle")}
        message={t("documents.deleteMessage")}
        confirmLabel={t("common.delete")}
        onConfirm={handleDelete}
        onCancel={() => setConfirmDelete(false)}
        danger
      />
    </>
  );
}
