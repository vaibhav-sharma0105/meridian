import { useState } from "react";
import {
  FileText,
  FileImage,
  FileJson,
  FileCode,
  Download,
  ExternalLink,
  Eye,
  X,
} from "lucide-react";

interface OutputFile {
  name: string;
  path: string;
  size: number;
  mime_type: string | null;
}

interface FilePreviewCardProps {
  file: OutputFile;
  onDownload?: (file: OutputFile) => void;
  onOpen?: (file: OutputFile) => void;
}

export function FilePreviewCard({ file, onDownload, onOpen }: FilePreviewCardProps) {
  const [showPreview, setShowPreview] = useState(false);
  const [previewContent, setPreviewContent] = useState<string | null>(null);

  const getFileIcon = () => {
    const mime = file.mime_type || "";
    if (mime.startsWith("image/")) return <FileImage className="h-8 w-8 text-blue-500" />;
    if (mime.includes("json")) return <FileJson className="h-8 w-8 text-amber-500" />;
    if (mime.includes("javascript") || mime.includes("python") || mime.includes("code"))
      return <FileCode className="h-8 w-8 text-green-500" />;
    return <FileText className="h-8 w-8 text-gray-400" />;
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const canPreview = () => {
    const mime = file.mime_type || "";
    return (
      mime.startsWith("text/") ||
      mime.includes("json") ||
      mime.includes("javascript") ||
      mime.includes("markdown")
    );
  };

  const handlePreview = async () => {
    if (!canPreview()) return;

    try {
      setPreviewContent(`Preview of ${file.name} would be shown here.`);
      setShowPreview(true);
    } catch (err) {
      console.error("Failed to preview file:", err);
    }
  };

  return (
    <>
      <div className="group bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:shadow-md transition-shadow">
        <div className="p-4">
          <div className="flex items-start gap-3">
            {getFileIcon()}
            <div className="flex-1 min-w-0">
              <h4 className="font-medium truncate text-gray-900 dark:text-gray-100">{file.name}</h4>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {formatSize(file.size)}
                {file.mime_type && ` · ${file.mime_type.split("/")[1]}`}
              </p>
            </div>
          </div>

          <div className="flex gap-2 mt-3 opacity-0 group-hover:opacity-100 transition-opacity">
            {canPreview() && (
              <button
                onClick={handlePreview}
                className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
              >
                <Eye className="h-4 w-4" />
                Preview
              </button>
            )}
            <button
              onClick={() => onDownload?.(file)}
              className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
            >
              <Download className="h-4 w-4" />
              Download
            </button>
            <button
              onClick={() => onOpen?.(file)}
              className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
            >
              <ExternalLink className="h-4 w-4" />
              Open
            </button>
          </div>
        </div>
      </div>

      {showPreview && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="fixed inset-0 bg-black/50" onClick={() => setShowPreview(false)} />
          <div className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden">
            <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">{file.name}</h3>
              <button
                onClick={() => setShowPreview(false)}
                className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <div className="p-4 overflow-auto max-h-[60vh]">
              <pre className="text-sm whitespace-pre-wrap font-mono bg-gray-50 dark:bg-gray-900/50 p-4 rounded-lg text-gray-800 dark:text-gray-200">
                {previewContent}
              </pre>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
