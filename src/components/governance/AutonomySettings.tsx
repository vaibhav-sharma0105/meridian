import { useState } from "react";
import { Shield, ChevronDown, Info } from "lucide-react";
import { useAutonomySetting, useSetAutonomySetting } from "@/hooks/useGovernance";
import type { AutonomyMode } from "@/lib/tauri";

const AUTONOMY_MODES: { value: AutonomyMode; label: string; description: string }[] = [
  {
    value: "manual",
    label: "Manual",
    description: "All agent actions require your approval before executing",
  },
  {
    value: "supervised",
    label: "Supervised",
    description: "Low-risk actions execute automatically; high-risk actions require approval",
  },
  {
    value: "autonomous",
    label: "Autonomous",
    description: "Most actions execute automatically; only critical actions require approval",
  },
];

interface AutonomySettingsProps {
  className?: string;
}

export function AutonomySettings({ className }: AutonomySettingsProps) {
  const { data: currentMode } = useAutonomySetting("global");
  const setAutonomy = useSetAutonomySetting();
  const [isOpen, setIsOpen] = useState(false);

  const selectedMode = AUTONOMY_MODES.find(
    (m) => m.value === (currentMode || "supervised")
  ) || AUTONOMY_MODES[1];

  const handleSelect = (mode: AutonomyMode) => {
    setAutonomy.mutate({ key: "global", value: mode });
    setIsOpen(false);
  };

  return (
    <div className={className}>
      <div className="flex items-center gap-2 mb-3">
        <Shield className="w-4 h-4 text-zinc-500" />
        <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
          Global Autonomy Mode
        </h3>
      </div>

      <div className="relative">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="w-full flex items-center justify-between px-3 py-2.5 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors"
        >
          <div className="flex items-center gap-3">
            <div
              className={`w-2 h-2 rounded-full ${
                selectedMode.value === "manual"
                  ? "bg-yellow-500"
                  : selectedMode.value === "supervised"
                  ? "bg-blue-500"
                  : "bg-green-500"
              }`}
            />
            <div className="text-left">
              <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                {selectedMode.label}
              </div>
              <div className="text-xs text-zinc-500 dark:text-zinc-400">
                {selectedMode.description}
              </div>
            </div>
          </div>
          <ChevronDown
            className={`w-4 h-4 text-zinc-400 transition-transform ${
              isOpen ? "rotate-180" : ""
            }`}
          />
        </button>

        {isOpen && (
          <div className="absolute z-10 mt-1 w-full bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg overflow-hidden">
            {AUTONOMY_MODES.map((mode) => (
              <button
                key={mode.value}
                onClick={() => handleSelect(mode.value)}
                className={`w-full flex items-center gap-3 px-3 py-2.5 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors ${
                  mode.value === selectedMode.value
                    ? "bg-zinc-50 dark:bg-zinc-700"
                    : ""
                }`}
              >
                <div
                  className={`w-2 h-2 rounded-full ${
                    mode.value === "manual"
                      ? "bg-yellow-500"
                      : mode.value === "supervised"
                      ? "bg-blue-500"
                      : "bg-green-500"
                  }`}
                />
                <div className="text-left">
                  <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                    {mode.label}
                  </div>
                  <div className="text-xs text-zinc-500 dark:text-zinc-400">
                    {mode.description}
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="mt-4 flex items-start gap-2 p-3 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg">
        <Info className="w-4 h-4 text-zinc-400 mt-0.5 flex-shrink-0" />
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          This setting controls how much oversight you want over agent actions.
          Integrations and skills can override this with their own settings.
        </p>
      </div>
    </div>
  );
}

interface AutonomySelectProps {
  value: string | null | undefined;
  onChange: (value: string | null) => void;
  label?: string;
  inheritLabel?: string;
}

export function AutonomySelect({
  value,
  onChange,
  label = "Autonomy",
  inheritLabel = "Inherit from global",
}: AutonomySelectProps) {
  const [isOpen, setIsOpen] = useState(false);

  const options = [
    { value: null, label: inheritLabel, color: "bg-zinc-400" },
    ...AUTONOMY_MODES.map((m) => ({
      value: m.value,
      label: m.label,
      color:
        m.value === "manual"
          ? "bg-yellow-500"
          : m.value === "supervised"
          ? "bg-blue-500"
          : "bg-green-500",
    })),
  ];

  const selected = options.find((o) => o.value === value) || options[0];

  return (
    <div className="relative">
      {label && (
        <label className="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1">
          {label}
        </label>
      )}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between px-2.5 py-1.5 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-md hover:border-zinc-300 dark:hover:border-zinc-600 text-sm"
      >
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${selected.color}`} />
          <span className="text-zinc-900 dark:text-zinc-100">{selected.label}</span>
        </div>
        <ChevronDown className="w-3.5 h-3.5 text-zinc-400" />
      </button>

      {isOpen && (
        <div className="absolute z-20 mt-1 w-full bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-md shadow-lg overflow-hidden">
          {options.map((opt) => (
            <button
              key={opt.value ?? "inherit"}
              onClick={() => {
                onChange(opt.value);
                setIsOpen(false);
              }}
              className={`w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-zinc-50 dark:hover:bg-zinc-700 text-sm ${
                opt.value === value ? "bg-zinc-50 dark:bg-zinc-700" : ""
              }`}
            >
              <div className={`w-2 h-2 rounded-full ${opt.color}`} />
              <span className="text-zinc-900 dark:text-zinc-100">{opt.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
