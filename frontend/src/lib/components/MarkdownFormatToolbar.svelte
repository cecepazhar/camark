<script lang="ts">
  import type { EditorView } from 'codemirror';
  import { undo, redo } from '@codemirror/commands';
  import {
    wrapSelection,
    insertHeading,
    insertList,
    insertTableTemplate,
    formatMarkdownTable,
  } from '$lib/utils/markdownTools';
  import { t } from '$lib/i18n/index.svelte';

  interface Props {
    editorView?: EditorView | null;
    isZenMode: boolean;
    isOutlineOpen: boolean;
    onToggleZen: () => void;
    onToggleOutline: () => void;
    onTogglePresentation: () => void;
    onFormatDocumentTable: () => void;
  }

  let {
    editorView,
    isZenMode,
    isOutlineOpen,
    onToggleZen,
    onToggleOutline,
    onTogglePresentation,
    onFormatDocumentTable,
  }: Props = $props();

  function applyBold() {
    if (editorView) wrapSelection(editorView, '**', '**', 'teks tebal');
  }

  function applyItalic() {
    if (editorView) wrapSelection(editorView, '*', '*', 'teks miring');
  }

  function applyStrike() {
    if (editorView) wrapSelection(editorView, '~~', '~~', 'coret');
  }

  function applyH1() {
    if (editorView) insertHeading(editorView, 1);
  }

  function applyH2() {
    if (editorView) insertHeading(editorView, 2);
  }

  function applyH3() {
    if (editorView) insertHeading(editorView, 3);
  }

  function applyBulletList() {
    if (editorView) insertList(editorView, 'bullet');
  }

  function applyNumberList() {
    if (editorView) insertList(editorView, 'number');
  }

  function applyTaskList() {
    if (editorView) insertList(editorView, 'task');
  }

  function applyQuote() {
    if (editorView) wrapSelection(editorView, '> ', '', 'kutipan');
  }

  function applyCode() {
    if (editorView) wrapSelection(editorView, '`', '`', 'kode');
  }

  function applyCodeBlock() {
    if (editorView) wrapSelection(editorView, '```\n', '\n```', 'kode blok');
  }

  function applyLink() {
    if (editorView) wrapSelection(editorView, '[', '](https://)', 'tautan');
  }

  function applyTable() {
    if (editorView) insertTableTemplate(editorView, 3, 3);
  }

  function handleUndo() {
    if (editorView) undo(editorView);
  }

  function handleRedo() {
    if (editorView) redo(editorView);
  }
</script>

<div
  class="h-9 px-2 bg-neutral-100 dark:bg-[#121217] border-b border-neutral-200 dark:border-[#262626] flex items-center justify-between text-neutral-600 dark:text-neutral-400 select-none overflow-x-auto scrollbar-none z-10 shrink-0"
>
  <!-- Left Formatting Tools -->
  <div class="flex items-center gap-0.5">
    <!-- Undo / Redo -->
    <button
      type="button"
      onclick={handleUndo}
      title={t('toolbar.undo')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 7v6h6"></path>
        <path d="M21 17a9 9 0 00-9-9 9 9 0 00-6 2.3L3 13"></path>
      </svg>
    </button>
    <button
      type="button"
      onclick={handleRedo}
      title={t('toolbar.redo')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 7v6h-6"></path>
        <path d="M3 17a9 9 0 019-9 9 9 0 016 2.3l3 2.7"></path>
      </svg>
    </button>

    <span class="w-[1px] h-3.5 bg-neutral-300 dark:bg-[#262626] mx-1"></span>

    <!-- Headings -->
    <button
      type="button"
      onclick={applyH1}
      title={t('toolbar.h1')}
      class="px-1.5 py-0.5 rounded text-xs font-bold hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      H1
    </button>
    <button
      type="button"
      onclick={applyH2}
      title={t('toolbar.h2')}
      class="px-1.5 py-0.5 rounded text-xs font-bold hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      H2
    </button>
    <button
      type="button"
      onclick={applyH3}
      title={t('toolbar.h3')}
      class="px-1.5 py-0.5 rounded text-xs font-bold hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      H3
    </button>

    <span class="w-[1px] h-3.5 bg-neutral-300 dark:bg-[#262626] mx-1"></span>

    <!-- Text Formatting -->
    <button
      type="button"
      onclick={applyBold}
      title={t('toolbar.bold')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white font-serif font-black text-xs transition-colors"
    >
      B
    </button>
    <button
      type="button"
      onclick={applyItalic}
      title={t('toolbar.italic')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white italic font-serif text-xs transition-colors"
    >
      I
    </button>
    <button
      type="button"
      onclick={applyStrike}
      title={t('toolbar.strikethrough')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white line-through text-xs transition-colors"
    >
      S
    </button>

    <span class="w-[1px] h-3.5 bg-neutral-300 dark:bg-[#262626] mx-1"></span>

    <!-- Lists -->
    <button
      type="button"
      onclick={applyBulletList}
      title={t('toolbar.bulletList')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="8" y1="6" x2="21" y2="6"></line>
        <line x1="8" y1="12" x2="21" y2="12"></line>
        <line x1="8" y1="18" x2="21" y2="18"></line>
        <line x1="3" y1="6" x2="3.01" y2="6"></line>
        <line x1="3" y1="12" x2="3.01" y2="12"></line>
        <line x1="3" y1="18" x2="3.01" y2="18"></line>
      </svg>
    </button>
    <button
      type="button"
      onclick={applyNumberList}
      title={t('toolbar.numberedList')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="10" y1="6" x2="21" y2="6"></line>
        <line x1="10" y1="12" x2="21" y2="12"></line>
        <line x1="10" y1="18" x2="21" y2="18"></line>
        <path d="M4 6h1v4"></path>
        <path d="M4 10h2"></path>
        <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"></path>
      </svg>
    </button>
    <button
      type="button"
      onclick={applyTaskList}
      title={t('toolbar.taskList')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="5" width="6" height="6" rx="1"></rect>
        <line x1="13" y1="8" x2="21" y2="8"></line>
        <line x1="13" y1="16" x2="21" y2="16"></line>
        <path d="M3 17l2 2 4-4"></path>
      </svg>
    </button>

    <span class="w-[1px] h-3.5 bg-neutral-300 dark:bg-[#262626] mx-1"></span>

    <!-- Blockquote & Code -->
    <button
      type="button"
      onclick={applyQuote}
      title={t('toolbar.quote')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"></path>
        <path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"></path>
      </svg>
    </button>
    <button
      type="button"
      onclick={applyCode}
      title={t('toolbar.inlineCode')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white font-mono text-[11px] transition-colors"
    >
      &lt;/&gt;
    </button>
    <button
      type="button"
      onclick={applyCodeBlock}
      title={t('toolbar.codeBlock')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white font-mono text-[11px] transition-colors"
    >
      {'{ }'}
    </button>
    <button
      type="button"
      onclick={applyLink}
      title={t('toolbar.link')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white transition-colors"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
      </svg>
    </button>

    <span class="w-[1px] h-3.5 bg-neutral-300 dark:bg-[#262626] mx-1"></span>

    <!-- Table Tools -->
    <button
      type="button"
      onclick={applyTable}
      title={t('toolbar.table')}
      class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-[#1f1f28] hover:text-neutral-900 dark:hover:text-white flex items-center gap-1 transition-colors"
    >
      <svg class="w-3.5 h-3.5 text-cyan-500 dark:text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2"></rect>
        <path d="M3 9h18"></path>
        <path d="M3 15h18"></path>
        <path d="M9 3v18"></path>
        <path d="M15 3v18"></path>
      </svg>
    </button>
    <button
      type="button"
      onclick={onFormatDocumentTable}
      title={t('toolbar.alignTable')}
      class="px-2 py-0.5 rounded text-[10px] font-semibold bg-neutral-200/80 dark:bg-[#18181F] hover:bg-cyan-500/20 text-cyan-700 dark:text-cyan-400 border border-cyan-500/30 flex items-center gap-1 transition-colors"
    >
      <span>{t('toolbar.alignTable')}</span>
    </button>
  </div>

  <!-- Right: Outline, Presentation & Zen Mode Controls -->
  <div class="flex items-center gap-1.5 shrink-0">
    <!-- Document Outline Drawer Toggle -->
    <button
      type="button"
      onclick={onToggleOutline}
      class="px-2 py-0.5 rounded text-[11px] font-medium flex items-center gap-1 transition-colors {isOutlineOpen ? 'bg-cyan-500/20 text-cyan-700 dark:text-cyan-300 border border-cyan-500/30' : 'hover:bg-neutral-200 dark:hover:bg-[#1f1f28] text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
      title={t('toolbar.outlineDesc')}
    >
      <span>📑</span>
      <span class="hidden sm:inline">{t('toolbar.outline')}</span>
    </button>

    <!-- Presentation / Slideshow Toggle -->
    <button
      type="button"
      onclick={onTogglePresentation}
      class="px-2 py-0.5 rounded text-[11px] font-bold flex items-center gap-1 bg-gradient-to-r from-amber-500/20 to-orange-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/40 hover:from-amber-500/30 hover:to-orange-500/30 transition-all shadow-xs"
      title={t('toolbar.presentationDesc')}
    >
      <span>📽️</span>
      <span>{t('toolbar.presentation')}</span>
    </button>

    <!-- Zen Mode Switch -->
    <button
      type="button"
      onclick={onToggleZen}
      class="px-2 py-0.5 rounded text-[11px] font-medium flex items-center gap-1 transition-colors {isZenMode ? 'bg-cyan-500 text-black font-bold' : 'hover:bg-neutral-200 dark:hover:bg-[#1f1f28] text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
      title={t('toolbar.zenDesc')}
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M3 12h1m16 0h1m-9-9v1m0 16v1"></path>
      </svg>
      <span>{isZenMode ? t('toolbar.zenActive') : t('toolbar.zen')}</span>
    </button>
  </div>
</div>
