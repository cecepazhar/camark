// Live performance preferences for the running UI. Loaded once at startup by
// +layout.svelte, updated by Settings → Performance, read by the preview renderer.
import {
  DEFAULT_PERFORMANCE_PREFS,
  getPerformancePrefs,
  setPerformancePrefs,
  type PerformancePrefs
} from '$lib/api/performance';

let prefs = $state<PerformancePrefs>({ ...DEFAULT_PERFORMANCE_PREFS });

/** Class on <html> that switches off animations and transitions (see app.css). */
const LOW_POWER_CLASS = 'low-power';

function applyToDocument(p: PerformancePrefs): void {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle(LOW_POWER_CLASS, p.lowPowerMode);
}

export function getPerformance(): { readonly prefs: PerformancePrefs } {
  return {
    get prefs() {
      return prefs;
    }
  };
}

export async function loadPerformancePrefs(): Promise<void> {
  try {
    prefs = { ...DEFAULT_PERFORMANCE_PREFS, ...(await getPerformancePrefs()) };
  } catch {
    // Outside Tauri, or the file is unreadable: defaults (the backend does the same).
  }
  applyToDocument(prefs);
}

/** Saves a change. The UI state is only updated after the backend accepted it. */
export async function updatePerformancePrefs(patch: Partial<PerformancePrefs>): Promise<void> {
  const next = { ...prefs, ...patch };
  await setPerformancePrefs(next);
  prefs = next;
  applyToDocument(prefs);
}

/** Pause after the last keystroke before a large document's preview re-renders. */
export const LARGE_DOC_PAUSE_MS = 800;

/**
 * How long to wait after a keystroke before re-rendering the preview.
 *
 * - Normal documents: 80 / 200 / 400 ms by size, as before.
 * - Large-document mode (word count above `largeDocWords`, when that is not 0): render
 *   only after an 800 ms typing pause.
 * - Low power mode doubles the result.
 */
export function renderDelayMs(chars: number, words: number, p: PerformancePrefs): number {
  let delay = chars > 100_000 ? 400 : chars > 20_000 ? 200 : 80;
  if (p.largeDocWords > 0 && words > p.largeDocWords) {
    delay = Math.max(delay, LARGE_DOC_PAUSE_MS);
  }
  return p.lowPowerMode ? delay * 2 : delay;
}
