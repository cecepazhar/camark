/**
 * Idle watcher for vault auto-lock (X-07). Calls `onIdle` once after `timeoutMs` without
 * activity; `poke()` on user activity restarts the countdown. timeoutMs <= 0 disables it.
 */
export interface IdleWatcher {
  poke(): void;
  stop(): void;
}

export function createIdleWatcher(timeoutMs: number, onIdle: () => void): IdleWatcher {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;
  const arm = () => {
    if (timer) clearTimeout(timer);
    timer = null;
    if (stopped || timeoutMs <= 0) return;
    timer = setTimeout(() => {
      timer = null;
      if (!stopped) onIdle();
    }, timeoutMs);
  };
  arm();
  return {
    poke: arm,
    stop() {
      stopped = true;
      if (timer) clearTimeout(timer);
      timer = null;
    },
  };
}

/** Minutes from localStorage key `camark_autolock_min` (default 10, 0 = off, clamped 0..240). */
export function autoLockMinutes(raw: string | null): number {
  if (raw === null || raw.trim() === '') return 10;
  const n = Number(raw);
  if (!Number.isFinite(n)) return 10;
  return Math.min(240, Math.max(0, Math.round(n)));
}
