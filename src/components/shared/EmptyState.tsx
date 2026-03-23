import { ReactNode } from "react";

interface Props {
  title: string;
  description?: string;
  icon?: ReactNode;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export default function EmptyState({ title, description, icon, action }: Props) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-8 text-center">
      {icon && <div className="mb-4 opacity-40">{icon}</div>}
      <h3 className="text-base font-semibold text-zinc-700 dark:text-zinc-300 mb-1">{title}</h3>
      {description && (
        <p className="text-sm text-zinc-400 dark:text-zinc-500 max-w-xs">{description}</p>
      )}
      {action && (
        <button
          onClick={action.onClick}
          className="mt-4 px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg text-sm font-medium transition-colors"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
