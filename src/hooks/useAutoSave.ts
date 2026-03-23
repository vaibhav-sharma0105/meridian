import { useEffect, useRef, useState } from "react";
import { AUTO_SAVE_DEBOUNCE_MS, SAVED_INDICATOR_MS } from "@/lib/constants";

/**
 * Debounced auto-save hook.
 * @param deps  - Values to watch for changes (like a useEffect deps array)
 * @param saveFn - Async function to call when saving
 * @returns { saved } boolean flag to show "Saved" indicator
 */
export function useAutoSave(deps: unknown[], saveFn: () => Promise<void>) {
  const [saved, setSaved] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isFirstRender = useRef(true);

  useEffect(() => {
    // Skip the initial render
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }

    if (timerRef.current) clearTimeout(timerRef.current);

    timerRef.current = setTimeout(async () => {
      try {
        await saveFn();
        setSaved(true);
        setTimeout(() => setSaved(false), SAVED_INDICATOR_MS);
      } catch (e) {
        console.error("Auto-save failed:", e);
      }
    }, AUTO_SAVE_DEBOUNCE_MS);

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, deps);

  return { saved };
}
