import { describe, expect, it } from 'vitest';
import { reorderTabs } from './docTabs.svelte';

const tabs = ['a', 'b', 'c', 'd'].map((id) => ({ id }));
const ids = (list: readonly { id: string }[]) => list.map((t) => t.id).join('');

describe('reorderTabs', () => {
  it('moves a tab before another', () => {
    expect(ids(reorderTabs(tabs, 'd', 'b', false))).toBe('adbc');
  });

  it('moves a tab after another', () => {
    expect(ids(reorderTabs(tabs, 'a', 'c', true))).toBe('bcad');
  });

  it('moves to the very end and very start', () => {
    expect(ids(reorderTabs(tabs, 'a', 'd', true))).toBe('bcda');
    expect(ids(reorderTabs(tabs, 'd', 'a', false))).toBe('dabc');
  });

  it('returns the same array when the order would not change', () => {
    expect(reorderTabs(tabs, 'b', 'b', true)).toBe(tabs);
    expect(reorderTabs(tabs, 'b', 'a', true)).toBe(tabs); // already right after a
    expect(reorderTabs(tabs, 'a', 'b', false)).toBe(tabs); // already right before b
  });

  it('ignores unknown ids and never mutates the input', () => {
    expect(reorderTabs(tabs, 'x', 'a', false)).toBe(tabs);
    expect(reorderTabs(tabs, 'a', 'x', false)).toBe(tabs);
    reorderTabs(tabs, 'd', 'a', false);
    expect(ids(tabs)).toBe('abcd');
  });
});
