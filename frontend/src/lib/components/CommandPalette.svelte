<script lang="ts">
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import { getPalette, closePalette } from '$lib/stores/commandPalette.svelte';
  import { navItems, settingsNavItem } from '$lib/navItems';
  import { toggleTheme } from '$lib/stores/theme.svelte';
  import { toggleAiChat } from '$lib/stores/aiChat.svelte';

  let { onLock }: { onLock?: () => void } = $props();

  const palette = getPalette();
  let query = $state('');
  let activeIndex = $state(0);
  let inputElement: HTMLInputElement | undefined = $state();
  let listElement: HTMLUListElement | undefined = $state();

  interface CommandItem {
    id: string;
    group: 'pages' | 'actions';
    label: string;
    hint?: string;
    run: () => void;
  }

  $effect(() => {
    if (!palette.open) return;
    query = '';
    activeIndex = 0;
    requestAnimationFrame(() => inputElement?.focus());
  });

  const baseItems = $derived.by((): CommandItem[] => {
    const pages = [...navItems(), settingsNavItem()].map((item) => ({
      id: `nav:${item.href}`,
      group: 'pages' as const,
      label: item.label,
      hint: item.href,
      run: () => void goto(item.href)
    }));

    const actions: CommandItem[] = [
      {
        id: 'action:ai',
        group: 'actions',
        label: t('ai.askAi') || 'Hana AI Copilot',
        hint: '🌸 AI Markdown',
        run: () => toggleAiChat()
      },
      {
        id: 'action:theme',
        group: 'actions',
        label: t('shell.theme') || 'Ganti Tema (Light/Dark)',
        hint: 'Appearance',
        run: () => toggleTheme()
      }
    ];

    return [...pages, ...actions];
  });

  const filteredItems = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return baseItems;
    return baseItems.filter(
      (item) => item.label.toLowerCase().includes(q) || (item.hint && item.hint.toLowerCase().includes(q))
    );
  });

  $effect(() => {
    if (activeIndex >= filteredItems.length) {
      activeIndex = Math.max(0, filteredItems.length - 1);
    }
  });

  function selectItem(item?: CommandItem) {
    if (!item) return;
    closePalette();
    item.run();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      closePalette();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % Math.max(1, filteredItems.length);
      scrollActive();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + filteredItems.length) % Math.max(1, filteredItems.length);
      scrollActive();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      selectItem(filteredItems[activeIndex]);
    }
  }

  function scrollActive() {
    requestAnimationFrame(() => {
      listElement?.querySelector(`[data-index="${activeIndex}"]`)?.scrollIntoView({ block: 'nearest' });
    });
  }
</script>

{#if palette.open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[300] flex items-start justify-center pt-[12vh] bg-black/50 backdrop-blur-xs p-4"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) closePalette();
    }}
  >
    <div
      class="w-full max-w-xl bg-white dark:bg-[#141414] border border-neutral-200 dark:border-neutral-800 rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
      aria-label="Command Palette"
    >
      <!-- Search Input Header -->
      <div class="flex items-center gap-2.5 px-4 border-b border-neutral-200 dark:border-neutral-800">
        <svg class="w-4 h-4 text-neutral-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          bind:this={inputElement}
          type="text"
          bind:value={query}
          onkeydown={handleKeyDown}
          placeholder="Cari menu, perintah, atau aksi..."
          class="w-full py-3.5 bg-transparent text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none"
        />
        <kbd class="px-1.5 py-0.5 rounded border border-neutral-200 dark:border-neutral-700 text-[10px] font-mono text-neutral-500">Esc</kbd>
      </div>

      <!-- Filtered List -->
      <ul bind:this={listElement} class="max-h-80 overflow-y-auto p-2" role="listbox">
        {#if filteredItems.length === 0}
          <li class="p-6 text-center text-xs text-neutral-500">
            Tidak ada perintah yang sesuai.
          </li>
        {:else}
          {#each filteredItems as cmd, idx (cmd.id)}
            <li
              role="option"
              data-index={idx}
              aria-selected={activeIndex === idx}
              onclick={() => selectItem(cmd)}
              onmouseenter={() => (activeIndex = idx)}
              class="w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-left text-xs transition-colors cursor-pointer {activeIndex === idx ? 'bg-cyan-600 text-white shadow-xs' : 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60'}"
            >
              <div class="flex items-center gap-2 truncate">
                <span class="truncate font-medium">{cmd.label}</span>
              </div>
              {#if cmd.hint}
                <span class="text-[10px] px-2 py-0.5 rounded font-mono {activeIndex === idx ? 'bg-cyan-700 text-cyan-100' : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400'}">
                  {cmd.hint}
                </span>
              {/if}
            </li>
          {/each}
        {/if}
      </ul>
    </div>
  </div>
{/if}
