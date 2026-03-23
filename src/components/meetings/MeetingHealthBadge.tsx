interface Props {
  score: number;
  showLabel?: boolean;
}

export default function MeetingHealthBadge({ score, showLabel = false }: Props) {
  const color =
    score >= 71 ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
    : score >= 41 ? "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
    : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400";

  const label =
    score >= 71 ? "Healthy"
    : score >= 41 ? "Fair"
    : "Poor";

  return (
    <span className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium ${color}`}>
      {score}
      {showLabel && <span className="opacity-75">· {label}</span>}
    </span>
  );
}
