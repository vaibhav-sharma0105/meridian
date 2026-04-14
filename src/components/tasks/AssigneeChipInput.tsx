import { useState, useRef, KeyboardEvent } from "react";
import { X, User } from "lucide-react";

interface Props {
  value: string;          // comma-separated, e.g. "Alice, Bob"
  onChange: (value: string) => void;
  suggestions?: string[]; // existing names to autocomplete
  placeholder?: string;
  showHint?: boolean;     // show "Press Enter or comma" hint (default true)
}

/** Parse a comma-separated assignee string into a trimmed array, skipping blanks. */
export function parseAssignees(value: string): string[] {
  return value
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

/** Serialise an array of names back to comma-separated. */
export function serializeAssignees(names: string[]): string {
  return names.join(", ");
}

export default function AssigneeChipInput({
  value,
  onChange,
  suggestions = [],
  placeholder = "Add assignee…",
  showHint = true,
}: Props) {
  const names = parseAssignees(value);
  const [input, setInput] = useState("");
  const [showSuggestions, setShowSuggestions] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const add = (name: string) => {
    const trimmed = name.trim();
    if (!trimmed || names.includes(trimmed)) return;
    onChange(serializeAssignees([...names, trimmed]));
    setInput("");
    setShowSuggestions(false);
  };

  const remove = (name: string) => {
    onChange(serializeAssignees(names.filter((n) => n !== name)));
  };

  const handleKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if ((e.key === "Enter" || e.key === ",") && input.trim()) {
      e.preventDefault();
      add(input);
    } else if (e.key === "Backspace" && !input && names.length > 0) {
      remove(names[names.length - 1]);
    }
  };

  const filtered = suggestions.filter(
    (s) => s.toLowerCase().includes(input.toLowerCase()) && !names.includes(s)
  );

  return (
    <div className="relative">
      <div
        className="flex flex-wrap gap-1 items-center px-2 py-1 min-h-[30px] rounded border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 cursor-text"
        onClick={() => inputRef.current?.focus()}
      >
        {names.map((name) => (
          <span
            key={name}
            className="inline-flex items-center gap-1 px-1.5 py-0.5 bg-indigo-100 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 rounded text-xs font-medium"
          >
            <User className="w-2.5 h-2.5" />
            {name}
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); remove(name); }}
              className="hover:text-indigo-900 dark:hover:text-indigo-100"
            >
              <X className="w-2.5 h-2.5" />
            </button>
          </span>
        ))}
        <input
          ref={inputRef}
          value={input}
          onChange={(e) => { setInput(e.target.value); setShowSuggestions(true); }}
          onKeyDown={handleKey}
          onFocus={() => setShowSuggestions(true)}
          onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
          placeholder={names.length === 0 ? placeholder : ""}
          className="flex-1 min-w-[80px] text-xs bg-transparent outline-none text-zinc-900 dark:text-zinc-50 placeholder-zinc-400"
        />
      </div>

      {showSuggestions && input && filtered.length > 0 && (
        <ul className="absolute z-50 top-full left-0 right-0 mt-0.5 bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-md shadow-lg max-h-32 overflow-y-auto">
          {filtered.slice(0, 6).map((s) => (
            <li key={s}>
              <button
                type="button"
                onMouseDown={() => add(s)}
                className="w-full text-left px-3 py-1.5 text-xs text-zinc-700 dark:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-700 flex items-center gap-1.5"
              >
                <User className="w-3 h-3 text-zinc-400" />
                {s}
              </button>
            </li>
          ))}
        </ul>
      )}

      {showHint && <p className="text-[10px] text-zinc-400 mt-0.5">Press Enter or comma to add</p>}
    </div>
  );
}
