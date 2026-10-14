<script lang="ts">
  export interface TabItem {
    id: string; // path or vault id
    path: string;
    title: string;
    isVault: boolean;
    isDirty?: boolean;
  }

  interface Props {
    tabs: TabItem[];
    activeTabId: string | null;
    onSelectTab: (id: string) => void;
    onCloseTab: (id: string) => void;
    onNewTab?: () => void;
  }

  let { tabs, activeTabId, onSelectTab, onCloseTab, onNewTab }: Props = $props();
</script>

<div class="h-9 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-100/90 dark:bg-[#141414] px-2 flex items-center gap-1 overflow-x-auto scrollbar-none select-none shrink-0 no-print">
  {#each tabs as tab (tab.id)}
    {@const isActive = tab.id === activeTabId}
    <div
      role="button"
      tabindex="0"
      onclick={() => onSelectTab(tab.id)}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelectTab(tab.id); }}
      class="group relative h-7.5 px-3 rounded-t-md flex items-center gap-2 text-xs font-mono transition-all cursor-pointer border-t border-x {isActive ? 'bg-white dark:bg-[#1a1a1a] text-cyan-600 dark:text-cyan-400 border-neutral-300 dark:border-neutral-700/80 shadow-2xs font-semibold' : 'bg-transparent border-transparent text-neutral-500 hover:text-neutral-800 dark:text-neutral-400 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40'}"
      title={tab.path}
    >
      <!-- Active indicator line at top of tab -->
      {#if isActive}
        <span class="absolute top-0 left-0 right-0 h-[2px] bg-cyan-500 rounded-t" aria-hidden="true"></span>
      {/if}

      <!-- Icon: Vault or Document -->
      <span class="shrink-0 text-[11px]">
        {#if tab.isVault}
          🔒
        {:else}
          <svg class="w-3.5 h-3.5 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        {/if}
      </span>

      <!-- Tab Title -->
      <span class="truncate max-w-[140px] sm:max-w-[180px]">{tab.title}</span>

      <!-- Dirty indicator dot -->
      {#if tab.isDirty}
        <span class="w-1.5 h-1.5 rounded-full bg-cyan-500 shrink-0" title="Ada perubahan belum tersimpan"></span>
      {/if}

      <!-- Close Button -->
      {#if tabs.length > 1}
        <button
          type="button"
          onclick={(e) => {
            e.stopPropagation();
            onCloseTab(tab.id);
          }}
          class="opacity-60 group-hover:opacity-100 p-0.5 rounded-sm hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-400 hover:text-rose-500 dark:hover:text-rose-400 transition-all shrink-0"
          title="Tutup Tab (Ctrl+W)"
          aria-label="Tutup Tab"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>
  {/each}

  {#if onNewTab}
    <button
      type="button"
      onclick={onNewTab}
      class="h-7 w-7 rounded-md flex items-center justify-center text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 transition-colors ml-0.5 shrink-0"
      title="Buka Dokumen Baru (+)"
      aria-label="Buka Dokumen Baru"
    >
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  {/if}
</div>
