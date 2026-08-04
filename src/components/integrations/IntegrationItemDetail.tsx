import type { IntegrationCache } from "@/lib/tauri";

interface Props {
  item: IntegrationCache;
}

export function IntegrationItemDetail({ item }: Props) {
  const data =
    typeof item.data === "string" ? JSON.parse(item.data) : item.data;

  return (
    <div className="px-4 pb-4 pt-2 border-t border-zinc-100 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-800/30">
      {data.description && (
        <div className="mb-3">
          <div className="text-xs font-medium text-zinc-500 mb-1">
            Description
          </div>
          <div className="text-sm text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap">
            {data.description}
          </div>
        </div>
      )}
      {data.message && (
        <div className="mb-3">
          <div className="text-xs font-medium text-zinc-500 mb-1">
            Commit Message
          </div>
          <div className="text-sm text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap">
            {data.message}
          </div>
        </div>
      )}
      {data.body && (
        <div className="mb-3">
          <div className="text-xs font-medium text-zinc-500 mb-1">Body</div>
          <div className="text-sm text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap line-clamp-6">
            {data.body}
          </div>
        </div>
      )}
      {data.files && data.files.length > 0 && (
        <div className="mb-3">
          <div className="text-xs font-medium text-zinc-500 mb-1">
            Files Changed
          </div>
          <div className="text-sm text-zinc-600 dark:text-zinc-400">
            {data.files.slice(0, 5).join(", ")}
            {data.files.length > 5 && ` (+${data.files.length - 5} more)`}
          </div>
        </div>
      )}
      {data.labels && data.labels.length > 0 && (
        <div className="flex items-center gap-2 flex-wrap">
          {data.labels.map((label: string) => (
            <span
              key={label}
              className="px-2 py-0.5 text-xs bg-zinc-200 dark:bg-zinc-700 rounded-full"
            >
              {label}
            </span>
          ))}
        </div>
      )}
      {data.state && (
        <div className="mt-2 text-xs text-zinc-500">State: {data.state}</div>
      )}
    </div>
  );
}
