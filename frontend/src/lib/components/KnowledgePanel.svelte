<script lang="ts">
  import type { Backlink } from '$lib/utils/knowledge';

  interface Props {
    open: boolean;
    noteName: string;
    backlinks: Backlink[];
    mentions: Backlink[];
    tags: { tag: string; paths: string[] }[];
    indexing: boolean;
    onOpen: (path: string, line: number) => void;
    onLinkMention: (m: Backlink) => void;
    onClose: () => void;
  }

  let { open, noteName, backlinks, mentions, tags, indexing, onOpen, onLinkMention, onClose }: Props = $props();
  let tab = $state<'backlinks' | 'mentions' | 'tags'>('backlinks');
  let openTag = $state<string | null>(null);
</script>

{#if open}
  <aside class="knowledge-panel absolute right-0 top-0 bottom-0 z-30 w-72 border-l border-neutral-800 bg-neutral-950/95 backdrop-blur flex flex-col text-xs">
    <header class="flex items-center justify-between px-3 py-2 border-b border-neutral-800">
      <span class="font-semibold text-neutral-200 truncate">Koneksi · {noteName || '—'}</span>
      <button type="button" class="text-neutral-500 hover:text-neutral-200" onclick={onClose} aria-label="Tutup">✕</button>
    </header>
    <nav class="flex border-b border-neutral-800">
      {#each [['backlinks', `Backlinks (${backlinks.length})`], ['mentions', `Mentions (${mentions.length})`], ['tags', `Tags (${tags.length})`]] as [k, label] (k)}
        <button
          type="button"
          class="flex-1 py-1.5 {tab === k ? 'text-cyan-300 border-b-2 border-cyan-400' : 'text-neutral-500 hover:text-neutral-300'}"
          onclick={() => (tab = k as typeof tab)}>{label}</button
        >
      {/each}
    </nav>
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#if indexing}<p class="text-neutral-500 px-1">Mengindeks workspace…</p>{/if}
      {#if tab === 'backlinks'}
        {#each backlinks as b (b.sourcePath + ':' + b.line)}
          <button type="button" class="w-full text-left rounded-md p-2 hover:bg-neutral-800/70" onclick={() => onOpen(b.sourcePath, b.line)}>
            <div class="text-cyan-300 truncate">{b.sourceName}</div>
            <div class="text-neutral-400 line-clamp-2">{b.snippet}</div>
          </button>
        {:else}
          {#if !indexing}<p class="text-neutral-500 px-1">Belum ada catatan yang menautkan ke sini.</p>{/if}
        {/each}
      {:else if tab === 'mentions'}
        {#each mentions as m (m.sourcePath + ':' + m.line)}
          <div class="rounded-md p-2 hover:bg-neutral-800/70">
            <button type="button" class="w-full text-left" onclick={() => onOpen(m.sourcePath, m.line)}>
              <div class="text-neutral-200 truncate">{m.sourceName}</div>
              <div class="text-neutral-400 line-clamp-2">{m.snippet}</div>
            </button>
            <button type="button" class="mt-1 text-[10px] text-cyan-400 hover:underline" onclick={() => onLinkMention(m)}>Jadikan [[link]]</button>
          </div>
        {:else}
          {#if !indexing}<p class="text-neutral-500 px-1">Tidak ada penyebutan tanpa link.</p>{/if}
        {/each}
      {:else}
        {#each tags as t (t.tag)}
          <div>
            <button type="button" class="w-full flex justify-between rounded-md px-2 py-1 hover:bg-neutral-800/70" onclick={() => (openTag = openTag === t.tag ? null : t.tag)}>
              <span class="text-cyan-300">#{t.tag}</span><span class="text-neutral-500">{t.paths.length}</span>
            </button>
            {#if openTag === t.tag}
              {#each t.paths as p (p)}
                <button type="button" class="w-full text-left pl-5 py-0.5 text-neutral-400 hover:text-neutral-100 truncate" onclick={() => onOpen(p, 0)}>{p.split('/').pop()}</button>
              {/each}
            {/if}
          </div>
        {:else}
          {#if !indexing}<p class="text-neutral-500 px-1">Belum ada tag di workspace.</p>{/if}
        {/each}
      {/if}
    </div>
  </aside>
{/if}
