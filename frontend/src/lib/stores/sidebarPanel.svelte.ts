// Global active sidebar panel for CAMark (Files / Vault / Search / Bookmarks / Knowledge).
// Synced between the desktop icon rail (+layout.svelte) and the sidebar panel (+page.svelte).
//
// Opsi A (per owner): the Files/Vault tree is ALWAYS visible in the sidebar. The
// Search / Bookmarks / Knowledge panels render as a *card floating above* the tree,
// toggled by the icon rail. `activePanel` records which special card (if any) is
// shown; `files` means "no special card, tree only". Clicking the active icon
// toggles the card back off (back to plain tree).

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
