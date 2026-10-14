// Navigation destinations, connected to the code-generated nav items with distinct icons & localized labels.
import { t } from '$lib/i18n/index.svelte';

export interface NavItem {
  key: string;
  href: string;
  label: string;
  /** SVG path for the icon */
  path: string;
}

const NAV_ICONS: Record<string, string> = {
  workspace: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
  editor: 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z',
  vault: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
  ai_copilot: 'M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.286L13 21l-2.286-6.857L5 12l5.714-2.286L13 3z'
};

export function navItems(): NavItem[] {
  return [
    {
      key: 'editor',
      href: '/',
      label: t('nav.editor'),
      path: NAV_ICONS.editor
    },
    {
      key: 'vault',
      href: '/vault',
      label: t('nav.vault'),
      path: NAV_ICONS.vault
    },
    {
      key: 'ai_copilot',
      href: '/copilot',
      label: t('nav.ai_copilot'),
      path: NAV_ICONS.ai_copilot
    }
  ];
}

export function settingsNavItem(): NavItem {
  return {
    key: 'settings',
    href: '/settings',
    label: t('nav.settings'),
    path: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
  };
}
