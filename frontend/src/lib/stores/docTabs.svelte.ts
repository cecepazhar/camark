// Global multi-document tab state for CAMark Markdown Studio.
// Preserves open docs, active tab, and per-tab viewMode across route changes & session.

export interface DocTab {
  id: string; // unique key: filepath or `vault://id`
  path: string; // filesystem path OR `vault://id`
  title: string;
  isVault: boolean;
  isDirty: boolean;
}

const STORAGE_KEY = 'camark_open_tabs_v1';
const ACTIVE_TAB_KEY = 'camark_active_tab_v1';
const VIEW_MODE_KEY = 'camark_view_mode_v1';

let tabs = $state<DocTab[]>([]);
let activeTabId = $state<string | null>(null);
let viewMode = $state<'editor' | 'split' | 'preview'>('preview');

function persist() {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tabs.map((t) => ({ id: t.id, path: t.path, title: t.title, isVault: t.isVault }))));
    if (activeTabId) localStorage.setItem(ACTIVE_TAB_KEY, activeTabId);
    localStorage.setItem(VIEW_MODE_KEY, viewMode);
  } catch {
    /* ignore quota */
  }
}

export function getDocTabs() {
  return tabs;
}

export function getActiveTabId() {
  return activeTabId;
}

export function getActiveTab(): DocTab | null {
  return tabs.find((t) => t.id === activeTabId) ?? null;
}

export function getViewMode() {
  return viewMode;
}

export function restoreTabs() {
  if (typeof localStorage === 'undefined') return;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const activeRaw = localStorage.getItem(ACTIVE_TAB_KEY);
    const vm = (localStorage.getItem(VIEW_MODE_KEY) || 'preview') as 'editor' | 'split' | 'preview';
    if (raw) {
      const parsed = JSON.parse(raw) as Array<{ id: string; path: string; title: string; isVault: boolean }>;
      tabs = parsed.map((p) => ({ ...p, isDirty: false }));
    }
    activeTabId = activeRaw && tabs.some((t) => t.id === activeRaw) ? activeRaw : (tabs[0]?.id ?? null);
    viewMode = vm === 'editor' || vm === 'split' ? vm : 'preview';
  } catch {
    tabs = [];
    activeTabId = null;
    viewMode = 'preview';
  }
}

export function openTab(path: string, title: string, isVault = false): void {
  const id = path;
  const existing = tabs.find((t) => t.id === id);
  if (existing) {
    // refresh title/content flag
    existing.title = title;
    existing.isVault = isVault;
    activeTabId = id;
    viewMode = 'preview';
    persist();
    return;
  }
  const newTab: DocTab = { id, path, title, isVault, isDirty: false };
  tabs = [...tabs, newTab];
  activeTabId = id;
  viewMode = 'preview';
  persist();
}

export function closeTab(id: string): void {
  const idx = tabs.findIndex((t) => t.id === id);
  if (idx === -1) return;
  tabs = tabs.filter((t) => t.id !== id);
  if (activeTabId === id) {
    const next = tabs[Math.min(idx, tabs.length - 1)];
    activeTabId = next ? next.id : null;
  }
  persist();
}

export function setActiveTab(id: string): void {
  activeTabId = id;
  persist();
}

export function setDirty(id: string, dirty: boolean): void {
  const tab = tabs.find((t) => t.id === id);
  if (tab && tab.isDirty !== dirty) {
    tab.isDirty = dirty;
    tabs = [...tabs]; // trigger reactivity
    persist();
  }
}

export function setViewMode(mode: 'editor' | 'split' | 'preview'): void {
  viewMode = mode;
  persist();
}
