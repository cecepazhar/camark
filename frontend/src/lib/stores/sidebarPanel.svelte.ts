// Global active sidebar panel for CAMark (Files / Vault / Search / Bookmarks / Knowledge).
// Synced between the desktop icon rail (+layout.svelte) and the sidebar panel (+page.svelte).
//
// Opsi A (per owner): the Files/Vault tree is ALWAYS visible in the sidebar. The
// Search / Bookmarks / Knowledge panels render as a *card floating above* the tree,
// toggled by the icon rail. `activePanel` records which special card (if any) is
// shown; `files` means "no special card, tree only". Clicking the active icon
// toggles the card back off (back to plain tree).
//
// `sidebarVisible` is whether the whole sidebar column is shown at all. It lives here,
// not in +page.svelte, so a rail click on any route can reopen a hidden sidebar:
// before, the rail only switched `activePanel` inside a column that stayed hidden,
// so clicking an icon looked like it did nothing.

export type SidebarPanel = 'files' | 'vault' | 'search' | 'bookmarks' | 'knowledge';

const VISIBLE_KEY = 'camark_show_file_tree';

function readVisible(): boolean {
  try {
    return typeof localStorage === 'undefined' || localStorage.getItem(VISIBLE_KEY) !== 'false';
  } catch {
    return true;
  }
}

function persistVisible(visible: boolean): void {
  try {
    localStorage.setItem(VISIBLE_KEY, String(visible));
  } catch {
    // Storage unavailable (private mode, quota): the choice just isn't remembered.
  }
}

let activePanel = $state<SidebarPanel>('files');
let sidebarVisible = $state<boolean>(readVisible());

export function getSidebarPanelStore() {
  return {
    get activePanel() {
      return activePanel;
    },
    setActivePanel(p: SidebarPanel) {
      activePanel = p;
    },
    get visible() {
      return sidebarVisible;
    }
  };
}

export function setActiveSidebarPanel(p: SidebarPanel): void {
  activePanel = p;
}

export function getActiveSidebarPanel(): SidebarPanel {
  return activePanel;
}

export function isSidebarVisible(): boolean {
  return sidebarVisible;
}

/**
 * Shows or hides the sidebar column. `persist: false` is for temporary layouts such as
 * Zen mode, which must not overwrite the user's remembered choice.
 */
export function setSidebarVisible(visible: boolean, persist = true): void {
  sidebarVisible = visible;
  if (persist) persistVisible(visible);
}

export function toggleSidebarVisible(): void {
  setSidebarVisible(!sidebarVisible);
}

/**
 * What a click on an icon-rail entry does to the sidebar. Returns true when the file
 * explorer should also be reset to the workspace root.
 *
 * - Sidebar hidden: show it with the clicked panel. A click is never a silent no-op.
 * - Folder (editor) icon on a visible sidebar: back to the plain tree, reset to root.
 * - Card icon that is already open: close the card (back to the tree).
 * - Any other card icon: open that card.
 */
export function applyRailClick(panel: SidebarPanel, isEditorIcon: boolean): boolean {
  if (!sidebarVisible) {
    setSidebarVisible(true);
    activePanel = panel;
    return false;
  }
  if (isEditorIcon) {
    activePanel = 'files';
    return true;
  }
  activePanel = activePanel === panel ? 'files' : panel;
  return false;
}
