<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import AiChatPanel from '$lib/components/AiChatPanel.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { getAiChatState, toggleAiChat, closeAiChat } from '$lib/stores/aiChat.svelte';
  import { page } from '$app/state';
  import { getTheme, initTheme, toggleTheme } from '$lib/stores/theme.svelte';
  import { getToasts, showToast } from '$lib/stores/uiNotifications.svelte';
  import FeedbackModal from '$lib/components/FeedbackModal.svelte';
  import CrashReportModal from '$lib/components/CrashReportModal.svelte';
  import AboutModal from '$lib/components/AboutModal.svelte';
  import { getPendingCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import ProfileMenu from '$lib/components/ProfileMenu.svelte';
  import { getFeedbackPromptState } from '$lib/stores/feedbackStore.svelte';
  import { checkForUpdates } from '$lib/stores/updater.svelte';
  import { lockVault } from '$lib/api/vault';
  import { APP_VERSION } from '$lib/appInfo';
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { goto } from '$app/navigation';
  import { openTab } from '$lib/stores/docTabs.svelte';
  import { Button } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
  import { APP_CONFIG } from '$lib/generated/app';
  import { navItems as getNavItems, settingsNavItem } from '$lib/navItems';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import { getPalette, openPalette, closePalette } from '$lib/stores/commandPalette.svelte';

  import AmbientGlow from '$lib/components/AmbientGlow.svelte';
  import { getAmbientStore } from '$lib/stores/ambient.svelte';
  import { getPro, refreshProStatus } from '$lib/stores/pro.svelte';

  let isCollapsed = $state(true);
  let isVaultUnlocked = $state(true); // Open directly without blocking lockscreen
  let pendingCrashReport = $state<ScrubbedCrashReport | null>(null);
  let showAboutModal = $state(false);
  let mobileDrawerOpen = $state(false);

  const theme = getTheme();
  const aiChat = getAiChatState();
  const feedbackPrompt = getFeedbackPromptState();
  const toasts = $derived(getToasts());
  const palette = getPalette();
  const ambient = getAmbientStore();
  const pro = getPro();

  const cardBorderClass = $derived.by(() => {
    if (!ambient.config.enabled || !ambient.config.cardGlowEnabled || !pro.isPro) {
      return 'border border-neutral-200 dark:border-neutral-800 md:border-b-0 md:border-r-0';
    }

    if (ambient.config.cardGlowStyle === 'neon-border') {
      return 'border border-cyan-400 dark:border-cyan-400 md:border-b-0 md:border-r-0 shadow-[inset_0_0_8px_rgba(6,182,212,0.15)]';
    }

    if (ambient.config.cardGlowStyle === 'chroma-beam') {
      return 'border border-cyan-400/50 dark:border-cyan-400/40 md:border-b-0 md:border-r-0';
    }

    // diffused-halo
    return 'border border-cyan-400/30 dark:border-cyan-400/25 md:border-b-0 md:border-r-0';
  });

  const navItems = $derived(getNavItems());
  const settingsItem = $derived(settingsNavItem());

  const currentSection = $derived(
    [...navItems, settingsItem].find((item) => {
      if (item.href === '/') return page.url.pathname === '/';
      return page.url.pathname.startsWith(item.href);
    }) ?? navItems[1]
  );

  function isActive(href: string): boolean {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }

  const isTauri = typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);
  const appWindow = isTauri ? getCurrentWindow() : null;

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch {
      try {
        if (appWindow) await appWindow.minimize();
      } catch (err) {
        console.warn('Failed to minimize window:', err);
      }
    }
  }

  async function maximizeWindow() {
    try {
      await invoke('window_maximize');
    } catch {
      try {
        if (appWindow) await appWindow.toggleMaximize();
      } catch (err) {
        console.warn('Failed to maximize window:', err);
      }
    }
  }

  async function closeWindow() {
    try {
      await invoke('window_close');
    } catch {
      try {
        if (appWindow) await appWindow.close();
      } catch (err) {
        console.warn('Failed to close window:', err);
      }
    }
  }

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      try {
        if (appWindow) await appWindow.startDragging();
      } catch (err) {
        console.warn('Failed to start dragging window:', err);
      }
    }
  }

  function handleLock() {
    closePalette();
    isVaultUnlocked = false;
    lockVault().catch((err) => console.warn('lock_vault failed:', err));
  }

  function toggleSidebar() {
    isCollapsed = !isCollapsed;
  }

  onMount(() => {
    initTheme();
    isVaultUnlocked = true;
    refreshProStatus().catch(() => {});

    if (!import.meta.env.DEV) {
      checkForUpdates({ silent: true }).catch(() => {});
    }

    getPendingCrashReport()
      .then((report) => {
        pendingCrashReport = report;
      })
      .catch(() => {});

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        openPalette('all');
      } else if (e.key === 'Escape') {
        mobileDrawerOpen = false;
        closeAiChat();
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    // Global listener for single-instance / system open-file events: ensure we are on the main editor page
    listen<string>('open-file', (event) => {
      if (event.payload && page.url.pathname !== '/') {
        goto('/');
      }
    }).catch((err) => console.warn('Global open-file listen error:', err));

    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  let { children } = $props();
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') {
      mobileDrawerOpen = false;
      closeAiChat();
    }
  }}
/>

<NotificationCenter />
{#if isVaultUnlocked}
  <CommandPalette onLock={handleLock} />
{/if}

{#if !isVaultUnlocked}
  <LockScreen onUnlocked={() => (isVaultUnlocked = true)} />
{:else}
  <!-- Pro Shell Frameless Layout Matching CATerm -->
  <div class="flex flex-col h-screen w-screen overflow-hidden bg-neutral-100 dark:bg-[#0e0e0e] text-neutral-800 dark:text-neutral-300 font-sans transition-colors duration-150 pb-[env(safe-area-inset-bottom,0px)]">
    <!-- Sleek Frameless Unified Header (h-12 / 48px) -->
    <header
      class="min-h-[3rem] h-[calc(3rem+env(safe-area-inset-top,0px))] pt-[env(safe-area-inset-top,0px)] flex items-center gap-1 md:gap-2 pl-2 pr-1 md:pl-3 shrink-0 select-none cursor-default border-b border-transparent"
      data-tauri-drag-region
      onmousedown={startDragging}
      ondblclick={(e) => {
        const target = e.target as HTMLElement | null;
        if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
          maximizeWindow();
        }
      }}
    >
      <!-- Left: Mobile nav toggle + Section breadcrumb -->
      <div class="flex items-center gap-1 min-w-0 flex-1" data-tauri-drag-region>
        <button
          type="button"
          onclick={() => mobileDrawerOpen = true}
          class="md:hidden p-1.5 rounded-lg text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors shrink-0"
          title={t('shell.openNav') || 'Open Navigation'}
          aria-label="Open Navigation"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>

        <div class="flex gap-1.5 text-xs items-center overflow-x-auto scrollbar-none min-w-0" data-tauri-drag-region>
          <span class="px-2 py-1 text-xs font-semibold shrink-0 hidden sm:flex items-center gap-2 text-neutral-900 dark:text-white" data-tauri-drag-region>
            <svg class="w-3.5 h-3.5 shrink-0 text-cyan-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={currentSection.path} />
            </svg>
            <span>{currentSection.label}</span>
          </span>
        </div>

        <!-- Draggable blank space spanning center area -->
        <div class="flex-1 h-full min-w-[20px]" data-tauri-drag-region></div>
      </div>

      <!-- Right: Live sync dot, Hana AI shortcut, Language, Theme & Frameless Window Controls -->
      <div class="flex items-center gap-1.5 text-neutral-500 dark:text-neutral-400 shrink-0 no-drag" data-tauri-drag-region>
        <!-- Live Status Pulse Indicator -->
        <div class="hidden sm:flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border border-emerald-500/20" title="Brankas Terenkripsi (SQLCipher) & Local-First">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_6px_rgba(16,185,129,0.8)]"></span>
          <span class="text-[11px] font-mono hidden md:inline">SQLCipher Vault</span>
        </div>

        <!-- Pro Edition Status Pill -->
        {#if pro.isPro}
          <a
            href="/settings?tab=pro"
            class="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold text-amber-300 bg-amber-500/15 border border-amber-500/30 shadow-xs hover:bg-amber-500/25 transition-colors"
            title="CAMark Pro Edition Aktif"
          >
            <span>★</span>
            <span>PRO</span>
          </a>
        {:else}
          <a
            href="/settings?tab=pro"
            class="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold text-neutral-400 bg-neutral-800 hover:text-white border border-neutral-700 transition-colors"
            title="Aktifkan Lisensi Pro"
          >
            <span>★ Pro</span>
          </a>
        {/if}

        <Button
          variant="ghost"
          size="icon"
          onclick={() => openPalette('all')}
          class="!p-1.5 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 rounded-md transition-colors"
          title="Search / Command Palette (Ctrl+K)"
          aria-label="Command Palette"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </Button>

        <!-- Language Switcher -->
        <LanguageSwitcher />

        <!-- Theme Toggle (Dark / Light) -->
        <Button
          variant="ghost"
          size="icon"
          onclick={() => toggleTheme()}
          class="!p-1.5 rounded-md text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
          title={theme.name === 'dark' ? t('shell.lightTheme') : t('shell.darkTheme')}
          aria-label={t('shell.theme')}
        >
          {#if theme.name === 'dark'}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
            </svg>
          {/if}
        </Button>

        <!-- Divider -->
        <div class="h-3.5 w-px bg-neutral-300 dark:bg-neutral-800 mx-0.5"></div>

        <!-- Frameless Window Controls -->
        <div class="flex items-center">
          <Button
            variant="ghost"
            size="icon"
            onclick={minimizeWindow}
            class="!p-1.5 rounded-md text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
            title="Minimize"
            aria-label="Minimize"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onclick={maximizeWindow}
            class="!p-1.5 rounded-md text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
            title="Maximize"
            aria-label="Maximize"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2" />
            </svg>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onclick={closeWindow}
            class="!p-1.5 rounded-md text-neutral-500 hover:text-white dark:text-neutral-400 hover:!bg-rose-600 transition-colors"
            title="Close"
            aria-label="Close"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </Button>
        </div>
      </div>
    </header>

    <!-- Shell Body: Sidebar + Main Content Card -->
    <div class="flex flex-1 min-h-0">
      <!-- Mobile Drawer Backdrop -->
      {#if mobileDrawerOpen}
        <button
          type="button"
          onclick={() => (mobileDrawerOpen = false)}
          class="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm md:hidden transition-opacity border-0 p-0 cursor-default"
          aria-label="Close Menu"
        ></button>
      {/if}

      <!-- Mobile Drawer Navigation -->
      <aside
        class="fixed inset-y-0 left-0 z-50 w-72 max-w-[85vw] bg-neutral-100 dark:bg-[#0e0e0e] border-r border-neutral-200 dark:border-neutral-800 flex flex-col justify-between shadow-2xl md:hidden transform transition-transform duration-200 ease-in-out {mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'}"
        aria-label="Mobile Navigation"
      >
        <div class="min-h-0 flex flex-col">
          <div class="min-h-[3rem] h-[calc(3rem+env(safe-area-inset-top,0px))] pt-[env(safe-area-inset-top,0px)] flex items-center justify-between px-4">
            <div class="flex items-center gap-2">
              <Logo size={22} mode="brand" />
              <span class="font-bold text-neutral-900 dark:text-white text-base tracking-tight">{APP_CONFIG.name}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 font-mono">v{APP_VERSION}</span>
            </div>
            <button
              onclick={() => (mobileDrawerOpen = false)}
              class="p-1.5 rounded-lg hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors"
              aria-label="Close Navigation"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <p class="px-5 pt-2 pb-1.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-neutral-500">{t('shell.workspace') || 'Navigasi'}</p>
          <nav class="px-2 space-y-0.5 overflow-y-auto scrollbar-none text-sm">
            {#each navItems as item}
              {#if item.key === 'ai_copilot'}
                <button
                  type="button"
                  onclick={() => {
                    mobileDrawerOpen = false;
                    toggleAiChat();
                  }}
                  title={item.label}
                  class="w-full px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors text-left {aiChat.open ? 'bg-cyan-500/15 text-cyan-600 dark:text-cyan-400 font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
                >
                  <svg class="w-[18px] h-[18px] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
                  </svg>
                  <span class="truncate">{item.label}</span>
                </button>
              {:else}
                <a
                  href={item.href}
                  onclick={() => (mobileDrawerOpen = false)}
                  title={item.label}
                  aria-current={isActive(item.href) ? 'page' : undefined}
                  class="px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors {isActive(item.href) ? 'bg-neutral-200/80 dark:bg-neutral-800/80 text-neutral-900 dark:text-white font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
                >
                  <svg class="w-[18px] h-[18px] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
                  </svg>
                  <span class="truncate">{item.label}</span>
                </a>
              {/if}
            {/each}
          </nav>
        </div>

        <div class="p-2 pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]">
          <ProfileMenu onLock={handleLock} onSignOut={handleLock} onNavigate={() => (mobileDrawerOpen = false)} />
        </div>
      </aside>

      <!-- Desktop Sleek Sidebar with Collapsible Rail -->
      <aside
        class="hidden md:flex flex-col shrink-0 will-change-[width] {isCollapsed ? 'w-16' : 'w-60'}"
        aria-label="Desktop Sidebar"
      >
        <!-- Brand & Sidebar Toggle -->
        <div class="h-14 flex items-center shrink-0 {isCollapsed ? 'justify-center px-2' : 'justify-between pl-4 pr-3'}">
          {#if isCollapsed}
            <button
              type="button"
              onclick={toggleSidebar}
              title="Perluas sidebar"
              aria-label="Expand sidebar"
              class="p-1.5 rounded-lg hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
            >
              <Logo size={22} mode="brand" />
            </button>
          {:else}
            <div class="flex items-center gap-2.5 min-w-0">
              <Logo size={22} mode="brand" />
              <span class="font-bold text-neutral-900 dark:text-white text-base tracking-tight truncate">{APP_CONFIG.name}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 font-mono shrink-0">v{APP_VERSION}</span>
            </div>
            <button
              type="button"
              onclick={toggleSidebar}
              title="Kecilkan sidebar"
              aria-label="Collapse sidebar"
              class="p-1 rounded-md hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors shrink-0"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
              </svg>
            </button>
          {/if}
        </div>

        <!-- Sidebar Navigation List -->
        <nav class="flex-1 min-h-0 overflow-y-auto scrollbar-none px-2 space-y-0.5 text-sm pt-1">
          {#each navItems as item}
            {@const active = item.key === 'ai_copilot' ? aiChat.open : isActive(item.href)}
            {#if item.key === 'ai_copilot'}
              <!-- AI Assistant: opens the floating Hana AI drawer without leaving the current document -->
              <button
                type="button"
                onclick={toggleAiChat}
                title={item.label}
                aria-pressed={aiChat.open}
                class="relative w-full px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors overflow-hidden {active ? 'bg-cyan-500/15 text-cyan-600 dark:text-cyan-400 font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
              >
                {#if active}
                  <span class="absolute -left-2 top-1.5 bottom-1.5 w-[3px] rounded-r bg-cyan-500" aria-hidden="true"></span>
                {/if}
                <div class="w-[20px] h-[20px] flex items-center justify-center shrink-0">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
                  </svg>
                </div>
                <span class="truncate whitespace-nowrap transition-all duration-150 {isCollapsed ? 'opacity-0 w-0 pointer-events-none hidden' : 'opacity-100 min-w-0'}">
                  {item.label}
                </span>
              </button>
            {:else}
              <a
                href={item.href}
                title={isCollapsed ? item.label : ''}
                aria-current={active ? 'page' : undefined}
                class="relative px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors overflow-hidden {active ? 'bg-neutral-200/80 dark:bg-neutral-800/80 text-neutral-900 dark:text-white font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
              >
                {#if active}
                  <!-- Active marker pinned to the window left edge -->
                  <span class="absolute -left-2 top-1.5 bottom-1.5 w-[3px] rounded-r bg-cyan-500" aria-hidden="true"></span>
                {/if}
                <div class="w-[20px] h-[20px] flex items-center justify-center shrink-0">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
                  </svg>
                </div>
                <span class="truncate whitespace-nowrap transition-all duration-150 {isCollapsed ? 'opacity-0 w-0 pointer-events-none hidden' : 'opacity-100 min-w-0'}">
                  {item.label}
                </span>
              </a>
            {/if}
          {/each}
        </nav>

        <!-- Sidebar Footer with ProfileMenu & Active Avatar Halo -->
        <div class="p-2 border-t border-neutral-200/60 dark:border-neutral-800/60">
          <ProfileMenu collapsed={isCollapsed} onLock={handleLock} onSignOut={handleLock} />
        </div>
      </aside>

      <!-- Main Content Card with Ambient Halo Underglow & CATerm Grid Flow Canvas -->
      <div class="flex-1 min-w-0 flex relative overflow-visible pl-2 pt-2 pr-0 pb-0 md:p-0">
        <AmbientGlow defaultAccent="#06b6d4" />

        <main class="flex-1 min-w-0 flex flex-col overflow-hidden relative z-10 bg-white dark:bg-[#161616] {cardBorderClass} rounded-none rounded-tl-2xl md:rounded-none md:rounded-tl-xl transition-colors duration-150">
          <!-- Page Router Container -->
          <div class="flex-1 min-w-0 flex flex-col overflow-hidden relative z-20">
            {@render children?.()}
          </div>
        </main>
      </div>
    </div>

    <!-- Modals & Global Overlays -->
    <AiChatPanel isOpen={aiChat.open} onClose={() => closeAiChat()} />

    <!-- Sleek Floating Action Button (FAB) Hana AI in bottom right corner -->
    {#if !aiChat.open}
      <button
        type="button"
        onclick={() => toggleAiChat()}
        class="fixed bottom-5 right-5 z-40 flex items-center gap-2.5 px-4 py-2.5 rounded-full bg-neutral-900/90 hover:bg-neutral-800 text-neutral-100 border border-neutral-700/80 shadow-2xl shadow-black/60 hover:scale-105 active:scale-95 transition-all text-xs font-semibold cursor-pointer group"
        title="Hana AI (Markdown & Writing Copilot)"
        aria-label="Open Hana AI"
      >
        <span class="text-rose-500 group-hover:rotate-12 transition-transform text-sm">🌸</span>
        <span class="font-medium tracking-tight">Hana AI</span>
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
      </button>
    {/if}

    {#if pendingCrashReport}
      <CrashReportModal
        report={pendingCrashReport}
        onClose={() => (pendingCrashReport = null)}
      />
    {/if}

    {#if feedbackPrompt.show}
      <FeedbackModal onClose={() => feedbackPrompt.close()} />
    {/if}

    {#if showAboutModal}
      <AboutModal onClose={() => (showAboutModal = false)} />
    {/if}
  </div>
{/if}
