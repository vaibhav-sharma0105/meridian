interface Props {
  confidence: number;
  size?: "sm" | "md";
}

export default function TaskConfidenceBadge({ confidence, size = "sm" }: Props) {
  const pct = Math.round(confidence * 100);
  const color =
    pct >= 80 ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
    : pct >= 50 ? "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
    : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400";

  return (
    <span className={`inline-flex items-center rounded px-1.5 py-0.5 font-medium ${color} ${size === "sm" ? "text-xs" : "text-sm"}`}>
      {pct}%
    </span>
  );
}
