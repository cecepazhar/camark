// Global active sidebar panel for CAMark (Files / Vault / Search / Bookmarks / Knowledge).
// Synced between the desktop icon rail (+layout.svelte) and the sidebar panel (+page.svelte).

export type SidebarPanel = 'files' | 'vault' | 'search' | 'bookmarks' | 'knowledge';

let activePanel = $state<SidebarPanel>('files');

export function getSidebarPanelStore() {
  return {
    get activePanel() {
      return activePanel;
    },
    setActivePanel(p: SidebarPanel) {
      activePanel = p;
    }
  };
}

export function setActiveSidebarPanel(p: SidebarPanel): void {
  activePanel = p;
}

export function getActiveSidebarPanel(): SidebarPanel {
  return activePanel;
}
