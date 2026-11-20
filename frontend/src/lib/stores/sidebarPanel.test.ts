import { beforeEach, describe, expect, it } from 'vitest';
import {
  applyRailClick,
  getActiveSidebarPanel,
  isSidebarVisible,
  setActiveSidebarPanel,
  setSidebarVisible
} from './sidebarPanel.svelte';

describe('sidebar rail clicks', () => {
  beforeEach(() => {
    localStorage.clear();
    setSidebarVisible(true);
    setActiveSidebarPanel('files');
  });

  it('reopens a hidden sidebar on every rail icon instead of doing nothing', () => {
    for (const [panel, isEditor] of [
      ['files', true],
      ['vault', false],
      ['search', false],
      ['bookmarks', false],
      ['knowledge', false]
    ] as const) {
      setSidebarVisible(false);
      const resetRoot = applyRailClick(panel, isEditor);
      expect(isSidebarVisible(), panel).toBe(true);
      expect(getActiveSidebarPanel(), panel).toBe(panel);
      expect(resetRoot, panel).toBe(false);
    }
  });

  it('remembers that a rail click reopened the sidebar', () => {
    setSidebarVisible(false);
    expect(localStorage.getItem('camark_show_file_tree')).toBe('false');
    applyRailClick('search', false);
    expect(localStorage.getItem('camark_show_file_tree')).toBe('true');
  });

  it('keeps the visible-sidebar behaviour: folder resets root, card icons toggle', () => {
    expect(applyRailClick('files', true)).toBe(true);
    expect(getActiveSidebarPanel()).toBe('files');

    applyRailClick('search', false);
    expect(getActiveSidebarPanel()).toBe('search');
    applyRailClick('search', false);
    expect(getActiveSidebarPanel()).toBe('files');
    expect(isSidebarVisible()).toBe(true);
  });

  it('does not remember a temporary hide such as Zen mode', () => {
    setSidebarVisible(false, false);
    expect(isSidebarVisible()).toBe(false);
    expect(localStorage.getItem('camark_show_file_tree')).toBe('true');
  });
});
