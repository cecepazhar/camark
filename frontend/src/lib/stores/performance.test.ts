import { describe, expect, it } from 'vitest';
import { DEFAULT_PERFORMANCE_PREFS } from '$lib/api/performance';
import { LARGE_DOC_PAUSE_MS, renderDelayMs } from './performance.svelte';

const base = { ...DEFAULT_PERFORMANCE_PREFS };

describe('renderDelayMs', () => {
  it('keeps the size-based delays for normal documents', () => {
    expect(renderDelayMs(1_000, 200, base)).toBe(80);
    expect(renderDelayMs(50_000, 8_000, base)).toBe(200);
    expect(renderDelayMs(150_000, 19_000, base)).toBe(400);
  });

  it('waits for a typing pause above the large-document threshold', () => {
    expect(renderDelayMs(150_000, 25_000, base)).toBe(LARGE_DOC_PAUSE_MS);
    expect(renderDelayMs(150_000, 25_000, { ...base, largeDocWords: 30_000 })).toBe(400);
  });

  it('treats a threshold of 0 as large-document mode off', () => {
    expect(renderDelayMs(400_000, 60_000, { ...base, largeDocWords: 0 })).toBe(400);
  });

  it('doubles every delay in low power mode', () => {
    const low = { ...base, lowPowerMode: true };
    expect(renderDelayMs(1_000, 200, low)).toBe(160);
    expect(renderDelayMs(150_000, 25_000, low)).toBe(LARGE_DOC_PAUSE_MS * 2);
  });
});
