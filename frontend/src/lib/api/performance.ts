// Settings → Performance. Typed by hand against `caf_core::prefs::PerformancePrefs`
// (serde camelCase); the generated `PerformancePrefs` in `$lib/generated/commands` is stale
// and does not match the Rust struct.
import { invoke } from '@tauri-apps/api/core';

export interface PerformancePrefs {
  gpuAcceleration: boolean;
  backgroundMemorySaving: boolean;
  /** Inherited from CATerm, unused by CAMark; round-tripped unchanged. */
  scrollbackLines: number;
  /** Inherited from CATerm, unused by CAMark; round-tripped unchanged. */
  inactiveSessionSleep: boolean;
  lowPowerMode: boolean;
  /** Word count above which large-document mode applies. 0 = off. */
  largeDocWords: number;
}

export interface MemoryUsage {
  /** PSS summed over CAMark and its WebKit processes. null where not measurable. */
  totalBytes: number | null;
  processCount: number;
}

export const DEFAULT_PERFORMANCE_PREFS: PerformancePrefs = {
  gpuAcceleration: true,
  backgroundMemorySaving: true,
  scrollbackLines: 5000,
  inactiveSessionSleep: true,
  lowPowerMode: false,
  largeDocWords: 20_000
};

export function getPerformancePrefs(): Promise<PerformancePrefs> {
  return invoke<PerformancePrefs>('get_performance_prefs');
}

export function setPerformancePrefs(prefs: PerformancePrefs): Promise<void> {
  return invoke<void>('set_performance_prefs', { prefs });
}

export function getMemoryUsage(): Promise<MemoryUsage> {
  return invoke<MemoryUsage>('performance_memory_usage');
}

/** Returns false on platforms where the cache cannot be cleared on demand. */
export function clearWebCaches(): Promise<boolean> {
  return invoke<boolean>('performance_clear_cache');
}
