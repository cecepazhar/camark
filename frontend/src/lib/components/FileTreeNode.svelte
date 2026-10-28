<script lang="ts">
  import FileTreeNode from './FileTreeNode.svelte';
  import type { FileNode } from '$lib/types/fileTree';

  interface Props {
    node: FileNode;
    depth?: number;
    currentFilePath: string | null;
    expandedDirs: Set<string>;
    onOpenFile: (node: FileNode) => void;
    onToggleDir: (path: string) => void;
  }

  let {
    node,
    depth = 0,
    currentFilePath,
    expandedDirs,
    onOpenFile,
    onToggleDir
  }: Props = $props();

  let isExpanded = $derived(expandedDirs.has(node.path));
  let isSelected = $derived(currentFilePath === node.path);
</script>

<div>
  {#if node.is_dir}
    <button
      type="button"
      onclick={() => onToggleDir(node.path)}
      class="w-full flex items-center gap-1.5 py-1.5 rounded-lg text-left transition-colors hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white group select-none"
      style="padding-left: {depth * 14 + 8}px; padding-right: 8px;"
      title={node.path}
      aria-expanded={isExpanded}
    >
      <!-- Chevron expand/collapse -->
      <span class="w-3.5 h-3.5 flex items-center justify-center shrink-0 text-neutral-400 group-hover:text-neutral-600 dark:group-hover:text-neutral-200 transition-transform {isExpanded ? 'rotate-90' : ''}">
        <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
        </svg>
      </span>

      <!-- Folder Open vs Closed Icon -->
      {#if isExpanded}
        <svg class="w-3.5 h-3.5 text-amber-500 shrink-0" fill="currentColor" viewBox="0 0 24 24">
          <path d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h5l2 2h7a2 2 0 012 2v1h-8a2 2 0 00-1.9 1.37L4.1 18.25A2 2 0 005 20zm2-8H7.23a1 1 0 00-.95.68L4 19h16a1 1 0 001-1v-5a1 1 0 00-1-1z" />
        </svg>
      {:else}
        <svg class="w-3.5 h-3.5 text-amber-500 shrink-0" fill="currentColor" viewBox="0 0 24 24">
          <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
        </svg>
      {/if}

      <span class="truncate text-xs font-mono">{node.name}</span>
    </button>

    {#if isExpanded && node.children && node.children.length > 0}
      <div class="flex flex-col space-y-0.5">
        {#each node.children as child (child.path)}
          <FileTreeNode
            node={child}
            depth={depth + 1}
            {currentFilePath}
            {expandedDirs}
            {onOpenFile}
            {onToggleDir}
          />
        {/each}
      </div>
    {:else if isExpanded && (!node.children || node.children.length === 0)}
      <div
        class="text-[11px] text-neutral-400 dark:text-neutral-500 italic py-1"
        style="padding-left: {(depth + 1) * 14 + 18}px;"
      >
        (kosong)
      </div>
    {/if}
  {:else}
    <button
      type="button"
      draggable="true"
      ondragstart={(e) => {
        if (e.dataTransfer) {
          const payload = JSON.stringify({
            path: node.path,
            name: node.name,
            isVault: false,
          });
          e.dataTransfer.setData('application/x-camark-file', payload);
          e.dataTransfer.setData('text/plain', node.path);
          e.dataTransfer.effectAllowed = 'copyMove';
        }
      }}
      onclick={() => onOpenFile(node)}
      class="w-full flex items-center gap-1.5 py-1.5 rounded-lg text-left transition-colors select-none cursor-grab active:cursor-grabbing {isSelected ? 'bg-cyan-500/15 text-cyan-600 dark:text-cyan-300 font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 hover:text-neutral-900 dark:hover:text-white'}"
      style="padding-left: {depth * 14 + 18}px; padding-right: 8px;"
      title={node.path}
    >
      <!-- File Type Icons -->
      {#if node.name.endsWith('.rs')}
        <span class="text-orange-500 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">RS</span>
      {:else if node.name.endsWith('.py') || node.name.endsWith('.pyw')}
        <span class="text-yellow-500 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">PY</span>
      {:else if node.name.endsWith('.js') || node.name.endsWith('.ts') || node.name.endsWith('.jsx') || node.name.endsWith('.tsx')}
        <span class="text-cyan-400 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">JS</span>
      {:else if node.name.endsWith('.json')}
        <span class="text-amber-400 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">{`{}`}</span>
      {:else if node.name.endsWith('.toml') || node.name.endsWith('.yaml') || node.name.endsWith('.yml')}
        <span class="text-purple-400 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">⚙</span>
      {:else if node.name.endsWith('.html') || node.name.endsWith('.css') || node.name.endsWith('.svelte')}
        <span class="text-rose-400 font-bold text-[9px] font-mono shrink-0 w-3.5 text-center">&lt;&gt;</span>
      {:else}
        <svg class="w-3.5 h-3.5 text-cyan-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      {/if}
      <span class="truncate text-xs font-mono">{node.name}</span>
    </button>
  {/if}
</div>
