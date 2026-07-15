import { useEffect, useMemo } from "react";
import { X, FileText, File, ExternalLink, Table } from "lucide-react";
import { format } from "date-fns";
import type { Document } from "@/lib/tauri";

interface Props {
  doc: Document;
  onClose: () => void;
}

const EXT_ICONS: Record<string, typeof FileText> = {
  pdf: FileText,
  docx: FileText,
  txt: File,
  md: File,
  xlsx: Table,
  xls: Table,
  csv: Table,
};

function renderMarkdownTable(content: string): React.ReactNode {
  const lines = content.split("\n");
  const tables: { headers: string[]; rows: string[][] }[] = [];
  let currentTable: { headers: string[]; rows: string[][] } | null = null;
  const nonTableLines: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    if (line.startsWith("|") && line.endsWith("|")) {
      const cells = line.slice(1, -1).split("|").map((c) => c.trim());

      if (!currentTable) {
        currentTable = { headers: cells, rows: [] };
      } else if (line.match(/^\|[-:\s|]+\|$/)) {
        continue;
      } else {
        currentTable.rows.push(cells);
      }
    } else {
      if (currentTable) {
        tables.push(currentTable);
        currentTable = null;
      }
      if (line) {
        nonTableLines.push(line);
      }
    }
  }

  if (currentTable) {
    tables.push(currentTable);
  }

  if (tables.length === 0) {
    return null;
  }

  return (
    <div className="space-y-4">
      {tables.map((table, idx) => (
        <div key={idx} className="overflow-x-auto">
          <table className="min-w-full border-collapse text-sm">
            <thead>
              <tr className="bg-zinc-100 dark:bg-zinc-800">
                {table.headers.map((h, i) => (
                  <th
                    key={i}
                    className="px-3 py-2 text-left text-xs font-semibold text-zinc-600 dark:text-zinc-300 border border-zinc-200 dark:border-zinc-700"
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {table.rows.map((row, ri) => (
                <tr key={ri} className="hover:bg-zinc-50 dark:hover:bg-zinc-800/50">
                  {row.map((cell, ci) => (
                    <td
                      key={ci}
                      className="px-3 py-2 text-zinc-700 dark:text-zinc-300 border border-zinc-200 dark:border-zinc-700"
                    >
                      {cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ))}
      {nonTableLines.length > 0 && (
        <pre className="whitespace-pre-wrap font-sans text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed mt-4">
          {nonTableLines.join("\n")}
        </pre>
      )}
    </div>
  );
}

function ContentRenderer({ content, fileType }: { content: string; fileType: string }) {
  const isTableType = ["xlsx", "xls", "csv"].includes(fileType);
  const hasMarkdownTable = content.includes("|") && content.split("\n").some((l) => l.trim().startsWith("|"));

  if (isTableType || hasMarkdownTable) {
    const tableContent = renderMarkdownTable(content);
    if (tableContent) {
      return <div className="prose prose-sm dark:prose-invert max-w-none">{tableContent}</div>;
    }
  }

  if (fileType === "md") {
    return (
      <div className="prose prose-sm dark:prose-invert max-w-none">
        <MarkdownRenderer content={content} />
      </div>
    );
  }

  return (
    <div className="prose prose-sm dark:prose-invert max-w-none">
      <pre className="whitespace-pre-wrap font-sans text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed">
        {content}
      </pre>
    </div>
  );
}

function MarkdownRenderer({ content }: { content: string }) {
  const lines = content.split("\n");

  return (
    <div className="space-y-2 text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed">
      {lines.map((line, i) => {
        const trimmed = line.trim();

        if (trimmed.startsWith("### ")) {
          return <h3 key={i} className="text-base font-semibold mt-4 mb-2">{trimmed.slice(4)}</h3>;
        }
        if (trimmed.startsWith("## ")) {
          return <h2 key={i} className="text-lg font-semibold mt-5 mb-2">{trimmed.slice(3)}</h2>;
        }
        if (trimmed.startsWith("# ")) {
          return <h1 key={i} className="text-xl font-bold mt-6 mb-3">{trimmed.slice(2)}</h1>;
        }
        if (trimmed.startsWith("- ")) {
          return <li key={i} className="ml-4">{renderInlineMarkdown(trimmed.slice(2))}</li>;
        }
        if (trimmed === "") {
          return <br key={i} />;
        }
        return <p key={i}>{renderInlineMarkdown(trimmed)}</p>;
      })}
    </div>
  );
}

function renderInlineMarkdown(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let remaining = text;
  let key = 0;

  while (remaining.length > 0) {
    const boldMatch = remaining.match(/^\*\*(.+?)\*\*/);
    const italicMatch = remaining.match(/^\*(.+?)\*/);
    const codeMatch = remaining.match(/^`(.+?)`/);

    if (boldMatch) {
      parts.push(<strong key={key++}>{boldMatch[1]}</strong>);
      remaining = remaining.slice(boldMatch[0].length);
    } else if (italicMatch) {
      parts.push(<em key={key++}>{italicMatch[1]}</em>);
      remaining = remaining.slice(italicMatch[0].length);
    } else if (codeMatch) {
      parts.push(
        <code key={key++} className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-800 rounded text-xs">
          {codeMatch[1]}
        </code>
      );
      remaining = remaining.slice(codeMatch[0].length);
    } else {
      const nextSpecial = remaining.search(/\*|`/);
      if (nextSpecial === -1) {
        parts.push(remaining);
        break;
      } else {
        parts.push(remaining.slice(0, nextSpecial));
        remaining = remaining.slice(nextSpecial);
      }
    }
  }

  return parts.length === 1 ? parts[0] : <>{parts}</>;
}

export default function DocPreview({ doc, onClose }: Props) {
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [onClose]);

  const ext = doc.file_type?.toLowerCase() ?? doc.filename?.split(".").pop()?.toLowerCase() ?? "file";
  const Icon = EXT_ICONS[ext] ?? File;
  const displayName = doc.title || doc.filename;
  const uploadedAt = doc.created_at || doc.uploaded_at;

  const content = doc.content_text || "";
  const isUrl = doc.file_type === "url";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-3xl max-h-[85vh] bg-white dark:bg-zinc-900 rounded-xl shadow-2xl flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-100 dark:border-zinc-800">
          <div className="flex items-center gap-3 min-w-0">
            <div className="flex-shrink-0 w-10 h-10 bg-indigo-50 dark:bg-indigo-900/30 rounded-lg flex items-center justify-center">
              <Icon className="w-5 h-5 text-indigo-500" />
            </div>
            <div className="min-w-0">
              <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-50 truncate">
                {displayName}
              </h2>
              <div className="flex items-center gap-2 text-xs text-zinc-500">
                <span className="uppercase">{ext}</span>
                {doc.file_size_bytes && (
                  <>
                    <span>·</span>
                    <span>{formatFileSize(doc.file_size_bytes)}</span>
                  </>
                )}
                {uploadedAt && (
                  <>
                    <span>·</span>
                    <span>{format(new Date(uploadedAt), "MMM d, yyyy")}</span>
                  </>
                )}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {isUrl && doc.source_url && (
              <a
                href={doc.source_url}
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 text-zinc-400 hover:text-indigo-500 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded-lg transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
              </a>
            )}
            <button
              onClick={onClose}
              className="p-2 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {content ? (
            <ContentRenderer content={content} fileType={ext} />
          ) : (
            <div className="flex flex-col items-center justify-center py-12 text-zinc-400">
              <File className="w-12 h-12 mb-3" />
              <p className="text-sm">No preview available</p>
              <p className="text-xs mt-1">Content was not extracted for this document</p>
            </div>
          )}
        </div>

        {/* Embedding status footer */}
        {doc.embeddings_ready && doc.embedding_model && (
          <div className="px-6 py-3 border-t border-zinc-100 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-800/50 rounded-b-xl">
            <p className="text-xs text-zinc-500">
              Embeddings ready · Model: {doc.embedding_model}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
