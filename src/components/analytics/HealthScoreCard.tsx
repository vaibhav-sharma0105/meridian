interface Props {
  score: number;
  label?: string;
}

export default function HealthScoreCard({ score, label = "Meeting Health" }: Props) {
  const color =
    score >= 71 ? "text-green-600 dark:text-green-400"
    : score >= 41 ? "text-amber-600 dark:text-amber-400"
    : "text-red-600 dark:text-red-400";

  const bg =
    score >= 71 ? "bg-green-50 dark:bg-green-900/20"
    : score >= 41 ? "bg-amber-50 dark:bg-amber-900/20"
    : "bg-red-50 dark:bg-red-900/20";

  const arc = Math.round((score / 100) * 251.2); // circumference of r=40

  return (
    <div className={`${bg} rounded-xl p-5 flex flex-col items-center`}>
      <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-3">{label}</p>
      <div className="relative w-24 h-24">
        <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
          <circle cx="50" cy="50" r="40" fill="none" stroke="currentColor" strokeWidth="8" className="text-zinc-200 dark:text-zinc-700" />
          <circle
            cx="50" cy="50" r="40" fill="none" strokeWidth="8"
            stroke="currentColor"
            className={color}
            strokeLinecap="round"
            strokeDasharray={`${arc} 251.2`}
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className={`text-2xl font-bold ${color}`}>{score}</span>
        </div>
      </div>
    </div>
  );
}
