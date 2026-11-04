<script lang="ts">
  import { tick } from 'svelte';
  import { fuzzyFilter } from '$lib/utils/knowledge';

  interface SwitcherItem {
    name: string;
    path: string;
  }

  interface Props {
    open: boolean;
    items: SwitcherItem[];
    recent?: string[];
    onPick: (item: SwitcherItem) => void;
    onCreate?: (name: string) => void;
    onClose: () => void;
  }

  let { open, items, recent = [], onPick, onCreate, onClose }: Props = $props();

  let query = $state('');
  let active = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);

  const ordered = $derived.by(() => {
    if (query.trim()) return fuzzyFilter(query, items, (i) => i.path.split('/').slice(-2).join('/'), 50);
    const rank = new Map(recent.map((p, i) => [p, i]));
    return [...items]
      .sort((a, b) => (rank.get(a.path) ?? 1e9) - (rank.get(b.path) ?? 1e9) || a.name.localeCompare(b.name))
      .slice(0, 50);
  });

  $effect(() => {
    if (open) {
      query = '';
      active = 0;
      tick().then(() => inputEl?.focus());
    }
  });

  $effect(() => {
    if (active >= ordered.length) active = Math.max(0, ordered.length - 1);
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      active = (active + 1) % Math.max(1, ordered.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      active = (active - 1 + ordered.length) % Math.max(1, ordered.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const pick = ordered[active];
      if (pick) onPick(pick);
      else if (query.trim() && onCreate) onCreate(query.trim());
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-[60] flex items-start justify-center pt-[12vh] bg-black/50 backdrop-blur-xs" role="presentation" onclick={onClose}>
    <div
      class="w-[560px] max-w-[92vw] rounded-xl border border-neutral-700/70 bg-neutral-900 shadow-2xl overflow-hidden"
      role="dialog"
      aria-modal="true"
      aria-label="Quick Switcher"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={onKey}
    >
      <input
        bind:this={inputEl}
        bind:value={query}
        oninput={() => (active = 0)}
        placeholder="Cari catatan… (Enter untuk membuka)"
        class="w-full px-4 py-3 bg-transparent text-sm text-neutral-100 outline-none border-b border-neutral-800"
      />
      <ul class="max-h-[50vh] overflow-y-auto py-1">
        {#each ordered as item, i (item.path)}
          <li>
            <button
              type="button"
              class="w-full text-left px-4 py-2 flex flex-col {i === active ? 'bg-cyan-500/15 text-cyan-200' : 'text-neutral-300 hover:bg-neutral-800'}"
              onmouseenter={() => (active = i)}
              onclick={() => onPick(item)}
            >
              <span class="text-sm truncate">{item.name}</span>
              <span class="text-[10px] text-neutral-500 truncate">{item.path}</span>
            </button>
          </li>
        {/each}
        {#if ordered.length === 0}
          <li class="px-4 py-3 text-xs text-neutral-500">
            {#if query.trim() && onCreate}Tidak ada hasil. Enter untuk membuat "{query.trim()}.md"{:else}Tidak ada catatan.{/if}
          </li>
        {/if}
      </ul>
      <div class="px-4 py-1.5 text-[10px] text-neutral-500 border-t border-neutral-800">↑↓ pilih · Enter buka · Esc tutup</div>
    </div>
  </div>
{/if}
