<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    EditorView,
    drawSelection,
    dropCursor,
    highlightSpecialChars,
    highlightActiveLine,
    lineNumbers,
    keymap,
  } from '@codemirror/view';
  import {
    history,
    defaultKeymap,
    historyKeymap,
    undo,
  } from '@codemirror/commands';
  import {
    bracketMatching,
    defaultHighlightStyle,
    syntaxHighlighting,
  } from '@codemirror/language';
  import { closeBrackets } from '@codemirror/autocomplete';
  import { Compartment } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import katex from 'katex';
  // mermaid is ~500 KB — lazy-loaded on demand (first mermaid block render) to
  // keep the cold start under a second.
  let mermaid: typeof import('mermaid').default | null = null;
  async function getMermaid() {
    if (!mermaid) {
      mermaid = (await import('mermaid')).default;
    }
    return mermaid;
  }
  let mermaidInitialized = false;
  async function ensureMermaidInitialized() {
    if (mermaidInitialized) return;
    const m = await getMermaid();
    m.initialize({
      startOnLoad: false,
      theme: 'dark',
      securityLevel: 'loose',
    });
    mermaidInitialized = true;
  }
  import { Button, Input, Badge } from '@cecepazhar/caui';

  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { t, getLocale } from '$lib/i18n/index.svelte';
  import { page } from '$app/state';
  import {
    getDocTabs,
    getActiveTabId,
    getActiveTab,
    openTab,
    closeTab,
    closeOtherTabs,
    closeTabsToRight,
    closeAllTabs,
    setActiveTab,
    setDirty,
    restoreTabs,
    getViewMode,
    setViewMode,
    registerExporterHandler,
    setIsSavingDoc,
    type DocTab,
    type ViewMode,
  } from '$lib/stores/docTabs.svelte';
  import {
    updateDocumentContext,
    registerEditorApplyHandler,
    registerEditorUndoHandler,
    type EditorApplyMode,
  } from '$lib/stores/documentContext.svelte';
  import { createLiveMarkdownPlugin } from '$lib/utils/liveMarkdown';

  import FileTreeNode from '$lib/components/FileTreeNode.svelte';
  import MarkdownFormatToolbar from '$lib/components/MarkdownFormatToolbar.svelte';
  import ContextualAiActionBar from '$lib/components/ContextualAiActionBar.svelte';
  import PresentationModal from '$lib/components/PresentationModal.svelte';
  import OutlineDrawer from '$lib/components/OutlineDrawer.svelte';
  import type { FileNode } from '$lib/types/fileTree';
  import {
    formatMarkdownTable,
    htmlTableToMarkdown,
    tsvToMarkdown,
  } from '$lib/utils/markdownTools';
  import { getPro } from '$lib/stores/pro.svelte';
  import { openAiChat } from '$lib/stores/aiChat.svelte';
  import { goto } from '$app/navigation';

  // State
  let workspacePath = $state<string>('.');
  let fileTree = $state<FileNode[]>([]);
  let expandedDirs = $state<Set<string>>(new Set());
  let showFileTree = $state<boolean>(
    typeof localStorage !== 'undefined'
      ? localStorage.getItem('camark_show_file_tree') !== 'false'
      : true
  );
  let currentFilePath = $state<string | null>(null);
  let selectedFileNode = $state<FileNode | null>(null);
  let fileClipboard = $state<{ mode: 'copy' | 'cut'; node: FileNode } | null>(null);

  let fileContextMenu = $state<{
    open: boolean;
    x: number;
    y: number;
    node: FileNode | null;
    isBackground: boolean;
  }>({
    open: false,
    x: 0,
    y: 0,
    node: null,
    isBackground: false,
  });

  let fileModal = $state<{
    open: boolean;
    type: 'file' | 'folder';
    targetDir: string;
    name: string;
    error: string | null;
  }>({
    open: false,
    type: 'file',
    targetDir: '',
    name: '',
    error: null,
  });

  let renameModal = $state<{
    open: boolean;
    node: FileNode | null;
    name: string;
    error: string | null;
  }>({
    open: false,
    node: null,
    name: '',
    error: null,
  });

  let deleteModal = $state<{
    open: boolean;
    node: FileNode | null;
    loading: boolean;
  }>({
    open: false,
    node: null,
    loading: false,
  });
  let isVaultMode = $state<boolean>(false);
  let isVaultUnlocked = $state<boolean>(false);
  let vaultModalOpen = $state<boolean>(false);
  let vaultPassword = $state<string>('');
  let vaultConfirmPassword = $state<string>('');
  let isVaultSetup = $state<boolean>(false);
  let vaultPasswordError = $state<string>('');
  let isVaultModalLoading = $state<boolean>(false);
  let vaultDocs = $state<any[]>([]);
  let currentDocTitle = $state<string>('Untitled Document.md');
  const DEMO_MD_EN = `# Welcome to CAMark Pro

High-performance, privacy-focused, and distraction-free Markdown studio.

## Pro Highlights:
- [x] **GFM Markdown & Code Block Header** with Copy Button
- [x] **Integrated KaTeX Math Engine**: $\\int_{-\\infty}^\\infty e^{-x^2} dx = \\sqrt{\\pi}$
- [x] **Visual Mermaid Flowcharts**
- [x] **Dual Storage**: Local Filesystem & SQLCipher Vault (Argon2id Zero-Knowledge)

\`\`\`mermaid
graph TD;
    A[Open Document] --> B{Select Storage};
    B -->|Direct File| C[Workspace Filesystem];
    B -->|Encrypted| D[SQLCipher Vault];
    C --> E[Live Preview & KaTeX];
    D --> E;
\`\`\`

| Component | Status | Architecture Notes |
| :--- | :--- | :--- |
| UI Shell | Elevated Card | Matching CATerm Pro |
| Encryption | Zero-Knowledge | SQLCipher + Argon2id Hash |
| Typography | Pro Markdown | Clean headings, zebra tables, code pills |
| Rendering | Hybrid Engine | Marked + KaTeX + Mermaid |

> "Privacy and speed are not trade-offs; they are the foundation of modern productivity tools."
`;

  const DEMO_MD_ID = `# Selamat Datang di CAMark Pro

Studio Markdown berkinerja tinggi, berorientasi keamanan, dan tanpa distraksi.

## Fitur Unggulan Pro:
- [x] **GFM Markdown & Code Block Header** dengan Copy Button
- [x] **KaTeX Math Engine Terintegrasi**: $\\int_{-\\infty}^\\infty e^{-x^2} dx = \\sqrt{\\pi}$
- [x] **Diagram Alur Mermaid Visual**
- [x] **Penyimpanan Ganda**: Filesystem Lokal & Brankas SQLCipher (Argon2id Zero-Knowledge)

\`\`\`mermaid
graph TD;
    A[Buka Dokumen] --> B{Pilih Media};
    B -->|Berkas Langsung| C[Filesystem Workspace];
    B -->|Enkripsi| D[Vault SQLCipher];
    C --> E[Live Preview & KaTeX];
    D --> E;
\`\`\`

| Komponen | Status Standar | Keterangan Arsitektur |
| :--- | :--- | :--- |
| UI Shell | Elevated Card | Identik dengan CATerm Pro |
| Enkripsi | Zero-Knowledge | SQLCipher + Argon2id Hash |
| Tipografi | Pro Markdown | Heading bersih, zebra table, code pill |
| Rendering | Hybrid Engine | Marked + KaTeX + Mermaid |

> "Privasi dan kecepatan bukanlah kompromi, melainkan fondasi setiap perkakas produktivitas modern."
`;

  const initialActiveTab = getActiveTab();
  let rawMarkdown = $state<string>(initialActiveTab ? '' : (getLocale() === 'id' ? DEMO_MD_ID : DEMO_MD_EN));
  let renderedHtml = $state<string>('');
  let renderedSvgHtml = $state<string>('');

  // Multi-document tabs reactive state
  const tabs = $derived(getDocTabs());
  const activeTabId = $derived(getActiveTabId());
  const activeTab = $derived(getActiveTab());
  const pro = getPro();

  // View mode: 'editor' | 'live' | 'split' | 'preview' (synchronized via docTabs store)
  const viewMode = $derived(getViewMode());
  let isSaving = $state<boolean>(false);
  let isLoadingTabDoc = false;
  $effect(() => {
    setIsSavingDoc(isSaving);
  });

  // Tabs scroll & overflow container reference
  let tabsContainerEl = $state<HTMLDivElement | null>(null);

  function scrollTabsLeft() {
    if (tabsContainerEl) {
      tabsContainerEl.scrollBy({ left: -160, behavior: 'smooth' });
    }
  }

  function scrollTabsRight() {
    if (tabsContainerEl) {
      tabsContainerEl.scrollBy({ left: 160, behavior: 'smooth' });
    }
  }

  function handleTabsWheel(e: WheelEvent) {
    if (tabsContainerEl && e.deltaY) {
      e.preventDefault();
      tabsContainerEl.scrollLeft += e.deltaY;
    }
  }

  $effect(() => {
    if (activeTabId && tabsContainerEl) {
      const activeEl = tabsContainerEl.querySelector(`[data-tab-id="${CSS.escape(activeTabId)}"]`);
      if (activeEl) {
        activeEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
      }
    }
  });

  // Zen & Focus Mode
  let isZenMode = $state<boolean>(false);
  let isPresentationOpen = $state<boolean>(false);
  let isOutlineOpen = $state<boolean>(false);

  // Text selection & AI Copilot Action Bar
  let selectedText = $state<string>('');
  let selectionRange = $state<{ from: number; to: number } | null>(null);

  // Synchronized scroll state (60fps requestAnimationFrame)
  let isSyncingEditor = false;
  let isSyncingPreview = false;
  let syncRafId: number | null = null;

  // Stats calculation
  let wordCount = $derived(rawMarkdown.trim() ? rawMarkdown.trim().split(/\s+/).length : 0);
  let charCount = $derived(rawMarkdown.length);
  let lineCount = $derived(rawMarkdown.split('\n').length);
  let readingTimeMin = $derived(Math.max(1, Math.ceil(wordCount / 200)));

  let editorElement = $state<HTMLDivElement | null>(null);
  let previewElement = $state<HTMLElement | null>(null);
  let editorView = $state<EditorView | null>(null);

  const languageCompartment = new Compartment();
  const lineWrappingCompartment = new Compartment();
  const liveMarkdownCompartment = new Compartment();
  const lineNumbersCompartment = new Compartment();

  $effect(() => {
    if (editorView) {
      const shouldWrap = isZenMode || viewMode === 'live' || isMarkdownFile(currentFilePath);
      editorView.dispatch({
        effects: lineWrappingCompartment.reconfigure(shouldWrap ? EditorView.lineWrapping : []),
      });
    }
  });

  $effect(() => {
    if (editorView) {
      editorView.dispatch({
        effects: liveMarkdownCompartment.reconfigure(viewMode === 'live' ? createLiveMarkdownPlugin() : []),
      });
    }
  });

  $effect(() => {
    if (editorView) {
      const showNums = !isMarkdownFile(currentFilePath) && viewMode === 'editor';
      editorView.dispatch({
        effects: lineNumbersCompartment.reconfigure(showNums ? lineNumbers() : []),
      });
    }
  });

  // Lazy language modules: import on demand to keep the initial bundle small.
  // The markdown editor is the default (used for the majority of files), so it
  // is the only one loaded synchronously. Code languages load on first use.
  async function getLanguageExtension(filename: string) {
    const lower = (filename || '').toLowerCase();
    if (lower.endsWith('.rs')) {
      const m = await import('@codemirror/lang-rust');
      return m.rust();
    }
    if (lower.endsWith('.py') || lower.endsWith('.pyw')) {
      const m = await import('@codemirror/lang-python');
      return m.python();
    }
    if (
      lower.endsWith('.js') ||
      lower.endsWith('.mjs') ||
      lower.endsWith('.cjs') ||
      lower.endsWith('.ts') ||
      lower.endsWith('.mts') ||
      lower.endsWith('.cts') ||
      lower.endsWith('.jsx') ||
      lower.endsWith('.tsx')
    ) {
      const m = await import('@codemirror/lang-javascript');
      const typescript = lower.endsWith('.ts') || lower.endsWith('.tsx') || lower.endsWith('.mts') || lower.endsWith('.cts');
      const jsx = lower.endsWith('.jsx') || lower.endsWith('.tsx') || lower.endsWith('.js');
      return m.javascript({ typescript, jsx });
    }
    if (lower.endsWith('.json') || lower.endsWith('.jsonc') || lower.endsWith('.json5')) {
      const m = await import('@codemirror/lang-json');
      return m.json();
    }
    if (
      lower.endsWith('.html') ||
      lower.endsWith('.htm') ||
      lower.endsWith('.svg') ||
      lower.endsWith('.svelte') ||
      lower.endsWith('.vue')
    ) {
      const m = await import('@codemirror/lang-html');
      return m.html();
    }
    if (lower.endsWith('.css') || lower.endsWith('.scss') || lower.endsWith('.sass') || lower.endsWith('.less')) {
      const m = await import('@codemirror/lang-css');
      return m.css();
    }
    return markdown();
  }

  function isMarkdownFile(pathOrTitle: string | null): boolean {
    if (!pathOrTitle) return true;
    if (pathOrTitle.startsWith('vault://')) return true;
    const lower = pathOrTitle.toLowerCase();
    return (
      lower.endsWith('.md') ||
      lower.endsWith('.markdown') ||
      lower.endsWith('.mdown') ||
      lower.endsWith('.mkd') ||
      lower.endsWith('.txt') ||
      lower.endsWith('.text')
    );
  }

  function isHtmlFile(pathOrTitle: string | null): boolean {
    if (!pathOrTitle) return false;
    const lower = pathOrTitle.toLowerCase();
    return lower.endsWith('.html') || lower.endsWith('.htm');
  }

  function isSvgFile(pathOrTitle: string | null): boolean {
    if (!pathOrTitle) return false;
    const lower = pathOrTitle.toLowerCase();
    return lower.endsWith('.svg');
  }

  function escapeHtml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  // Custom marked renderer for Pro Markdown (Zebra tables, code block headers with copy buttons)
  const proRenderer = new marked.Renderer();

  // Custom heading renderer: inject id slugs so internal anchor links (e.g. #3-core-architecture)
  // resolve within the scrollable preview container.
  const originalHeading = proRenderer.heading;
  proRenderer.heading = function (token: any) {
    const text = typeof token === 'object' ? token.text : arguments[0];
    const depth = typeof token === 'object' ? token.depth : arguments[1];
    const plain = String(text).replace(/<[^>]+>/g, '');
    const slug = plain
      .toLowerCase()
      .replace(/\/\s*$/, '')
      .replace(/[^\w\s-]/g, '')
      .trim()
      .replace(/\s+/g, '-');
    const html = originalHeading.call(this, token);
    return html.replace(/^<h(\d)[^>]*>/, `<h$1 id="${slug}">`);
  };

  // Custom Table with modern wrapper
  const originalTable = proRenderer.table;
  proRenderer.table = function (token: any) {
    const tableHtml = originalTable.call(this, token);
    return `<div class="markdown-table-wrapper">${tableHtml}</div>`;
  };

  // Custom Code block with header bar + language pill + copy button
  proRenderer.code = function (token: any) {
    const code = typeof token === 'object' ? token.text : arguments[0];
    const lang = (typeof token === 'object' ? token.lang : arguments[1]) || 'text';

    if (lang === 'mermaid') {
      return `<pre class="mermaid-block"><code class="language-mermaid">${code}</code></pre>`;
    }

    const encodedCode = encodeURIComponent(code);
    return `<div class="code-block-wrapper" data-lang="${lang}">
      <div class="code-block-header">
        <span class="code-lang-pill">${lang}</span>
        <button type="button" class="code-copy-btn" data-code="${encodedCode}">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
          ${t('common.copy')}
        </button>
      </div>
      <pre><code class="language-${lang}">${code}</code></pre>
    </div>`;
  };

  marked.use({ renderer: proRenderer });

  function toggleFileTree() {
    showFileTree = !showFileTree;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('camark_show_file_tree', String(showFileTree));
    }
  }

  function toggleDir(path: string) {
    const next = new Set(expandedDirs);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedDirs = next;
  }

  // Synchronized Scrolling Logic
  function syncScrollEditorToPreview() {
    if (isSyncingPreview || !editorView || !previewElement || viewMode !== 'split') return;
    if (syncRafId) cancelAnimationFrame(syncRafId);
    syncRafId = requestAnimationFrame(() => {
      isSyncingEditor = true;
      const scrollDom = editorView!.scrollDOM;
      const editorMax = scrollDom.scrollHeight - scrollDom.clientHeight;
      if (editorMax > 0 && previewElement) {
        const ratio = scrollDom.scrollTop / editorMax;
        const previewMax = previewElement.scrollHeight - previewElement.clientHeight;
        previewElement.scrollTop = ratio * previewMax;
      }
      setTimeout(() => {
        isSyncingEditor = false;
      }, 40);
    });
  }

  function syncScrollPreviewToEditor() {
    if (isSyncingEditor || !editorView || !previewElement || viewMode !== 'split') return;
    if (syncRafId) cancelAnimationFrame(syncRafId);
    syncRafId = requestAnimationFrame(() => {
      isSyncingPreview = true;
      const previewMax = previewElement!.scrollHeight - previewElement!.clientHeight;
      if (previewMax > 0 && editorView) {
        const ratio = previewElement!.scrollTop / previewMax;
        const scrollDom = editorView!.scrollDOM;
        const editorMax = scrollDom.scrollHeight - scrollDom.clientHeight;
        scrollDom.scrollTop = ratio * editorMax;
      }
      setTimeout(() => {
        isSyncingPreview = false;
      }, 40);
    });
  }

  // Smart Paste Handler (converts HTML tables and Excel TSV to Markdown table)
  function handleEditorPaste(e: ClipboardEvent): boolean {
    const html = e.clipboardData?.getData('text/html');
    const text = e.clipboardData?.getData('text/plain');

    if (html && html.includes('<table')) {
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, 'text/html');
      const table = doc.querySelector('table');
      if (table) {
        e.preventDefault();
        const mdTable = htmlTableToMarkdown(table);
        insertTextIntoEditor(mdTable);
        showToast(t('editor.htmlTableConverted'), 'success');
        return true;
      }
    } else if (text && text.includes('\t') && text.includes('\n')) {
      // Spreadsheet TSV
      e.preventDefault();
      const mdTable = tsvToMarkdown(text);
      insertTextIntoEditor(mdTable);
      showToast(t('editor.spreadsheetConverted'), 'success');
      return true;
    }
    return false;
  }

  function insertTextIntoEditor(textToInsert: string) {
    if (!editorView) return;
    const sel = editorView.state.selection.main;
    editorView.dispatch({
      changes: { from: sel.from, to: sel.to, insert: textToInsert },
      selection: { anchor: sel.from + textToInsert.length },
    });
    editorView.focus();
  }

  // Table Auto-Formatter
  function handleFormatDocumentTable() {
    if (!editorView) return;
    const sel = editorView.state.selection.main;
    if (sel.from !== sel.to) {
      const selected = editorView.state.sliceDoc(sel.from, sel.to);
      const formatted = formatMarkdownTable(selected);
      editorView.dispatch({
        changes: { from: sel.from, to: sel.to, insert: formatted },
      });
      showToast(t('editor.selectedTableFormatted'), 'success');
    } else {
      const docText = editorView.state.doc.toString();
      const tableBlockRegex = /((?:^[ \t]*\|.+$\n?){2,})/gm;
      let count = 0;
      const formattedDoc = docText.replace(tableBlockRegex, (match) => {
        count++;
        return formatMarkdownTable(match);
      });
      if (count > 0) {
        editorView.dispatch({
          changes: { from: 0, to: docText.length, insert: formattedDoc },
        });
        showToast(t('editor.alignTablesSuccess', { count }), 'success');
      } else {
        showToast(t('editor.noTablesFound'), 'info');
      }
    }
  }

  function handleToggleZen() {
    if (!pro.isPro && !isZenMode) {
      showToast(t('pro.zenExclusive'), 'info');
      return;
    }
    isZenMode = !isZenMode;
    if (isZenMode) {
      setViewMode('editor');
      showFileTree = false;
      showToast(t('editor.zenModeActive'), 'info');
    } else {
      showFileTree = true;
      setViewMode('preview');
    }
  }

  function handleApplyAiResult(resultText: string) {
    if (!editorView || !selectionRange) return;
    editorView.dispatch({
      changes: { from: selectionRange.from, to: selectionRange.to, insert: resultText },
      selection: { anchor: selectionRange.from + resultText.length },
    });
    selectedText = '';
    selectionRange = null;
    editorView.focus();
  }

  function handleJumpToHeading(heading: { line: number; anchor: string }) {
    if (editorView) {
      const targetLine = Math.min(Math.max(1, heading.line), editorView.state.doc.lines);
      const linePos = editorView.state.doc.line(targetLine).from;
      editorView.dispatch({
        selection: { anchor: linePos },
        effects: EditorView.scrollIntoView(linePos, { y: 'start' }),
      });
      editorView.focus();
    }
    if (previewElement) {
      let target: Element | null = null;
      if (heading.anchor) {
        try {
          target = previewElement.querySelector('#' + CSS.escape(heading.anchor));
        } catch {
          target = null;
        }
      }
      if (!target) {
        const allHeadings = Array.from(previewElement.querySelectorAll('h1, h2, h3, h4, h5, h6'));
        target = allHeadings.find((el) => {
          const text = (el.textContent || '').trim().toLowerCase();
          return text.includes(heading.anchor.replace(/-/g, ' '));
        }) || null;
      }
      if (target) {
        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    }
  }

  onMount(() => {
    const unregisterApply = registerEditorApplyHandler((text: string, mode: EditorApplyMode) => {
      if (!editorView) return;
      const doc = editorView.state.doc;
      if (mode === 'append') {
        const insertPos = doc.length;
        const separator = doc.length > 0 && !doc.toString().endsWith('\n') ? '\n\n' : '';
        editorView.dispatch({
          changes: { from: insertPos, insert: separator + text },
          selection: { anchor: insertPos + separator.length + text.length },
          scrollIntoView: true,
        });
      } else if (mode === 'insert') {
        const sel = editorView.state.selection.main;
        editorView.dispatch({
          changes: { from: sel.from, to: sel.to, insert: text },
          selection: { anchor: sel.from + text.length },
          scrollIntoView: true,
        });
      } else {
        // replace
        editorView.dispatch({
          changes: { from: 0, to: doc.length, insert: text },
          selection: { anchor: text.length },
          scrollIntoView: true,
        });
      }
      editorView.focus();
    });

    const unregisterUndo = registerEditorUndoHandler(() => {
      if (editorView) {
        const undone = undo(editorView);
        editorView.focus();
        return undone;
      }
      return false;
    });

    const unregisterExporter = registerExporterHandler((type) => {
      if (type === 'pdf') {
        exportPdf();
      } else if (type === 'html') {
        exportHtml();
      }
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === 'b' || e.key === 'B')) {
        e.preventDefault();
        toggleFileTree();
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === 'p' || e.key === 'P')) {
        e.preventDefault();
        exportPdf();
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === 'w' || e.key === 'W')) {
        e.preventDefault();
        if (activeTabId) {
          handleCloseTab(activeTabId);
        }
      }

      const isSidebarTarget = Boolean((e.target as HTMLElement)?.closest('.file-tree-sidebar'));
      const isInputTarget = Boolean((e.target as HTMLElement)?.closest('input, textarea, [contenteditable="true"]'));

      if (e.key === 'F2') {
        if (!isInputTarget && selectedFileNode) {
          e.preventDefault();
          triggerRename(selectedFileNode);
        }
      }

      if ((e.ctrlKey || e.metaKey) && (e.key === 'd' || e.key === 'D')) {
        if (!isInputTarget && selectedFileNode) {
          e.preventDefault();
          handleDuplicate(selectedFileNode);
        }
      }

      if (isSidebarTarget && !isInputTarget) {
        if (e.key === 'Delete') {
          if (selectedFileNode) {
            e.preventDefault();
            triggerDelete(selectedFileNode);
          }
        }
        if ((e.ctrlKey || e.metaKey) && (e.key === 'c' || e.key === 'C')) {
          if (selectedFileNode) {
            e.preventDefault();
            handleCopy(selectedFileNode);
          }
        }
        if ((e.ctrlKey || e.metaKey) && (e.key === 'x' || e.key === 'X')) {
          if (selectedFileNode) {
            e.preventDefault();
            handleCut(selectedFileNode);
          }
        }
        if ((e.ctrlKey || e.metaKey) && (e.key === 'v' || e.key === 'V')) {
          if (fileClipboard) {
            e.preventDefault();
            handlePaste(selectedFileNode?.is_dir ? selectedFileNode.path : workspacePath);
          }
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    // Restore saved tabs from local session storage (run first, before editor init)
    restoreTabs();
    const restored = getActiveTab();

    // Initialize CodeMirror 6 with oneDark theme and update listeners
    if (editorElement) {
      const view = new EditorView({
        doc: rawMarkdown,
        extensions: [
          highlightSpecialChars(),
          history(),
          drawSelection(),
          dropCursor(),
          syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
          bracketMatching(),
          closeBrackets(),
          EditorView.domEventHandlers({
            paste(event) {
              return handleEditorPaste(event);
            },
          }),
          keymap.of([
            ...defaultKeymap,
            ...historyKeymap,
          ]),
          highlightActiveLine(),
          lineNumbersCompartment.of(!isMarkdownFile(currentFilePath) && viewMode === 'editor' ? lineNumbers() : []),
          languageCompartment.of(markdown()),
          oneDark,
          lineWrappingCompartment.of(isZenMode || viewMode === 'live' || isMarkdownFile(currentFilePath) ? EditorView.lineWrapping : []),
          liveMarkdownCompartment.of(viewMode === 'live' ? createLiveMarkdownPlugin() : []),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              rawMarkdown = update.state.doc.toString();
              renderMarkdown(rawMarkdown);
              if (!isLoadingTabDoc) {
                if (activeTabId) setDirty(activeTabId, true);
                autoSave();
              }
              updateDocumentContext({ active_doc_content: rawMarkdown });
            }
            if (update.selectionSet) {
              const sel = update.state.selection.main;
              if (sel.from !== sel.to) {
                selectedText = update.state.sliceDoc(sel.from, sel.to);
                selectionRange = { from: sel.from, to: sel.to };
              } else {
                selectedText = '';
                selectionRange = null;
              }
              updateDocumentContext({ selected_text: selectedText });

              // Typewriter scrolling in Zen Mode (center current line)
              if (isZenMode) {
                const head = sel.head;
                const line = update.view.lineBlockAt(head);
                const halfHeight = update.view.scrollDOM.clientHeight / 2;
                update.view.scrollDOM.scrollTop = Math.max(0, line.top - halfHeight);
              }
            }
          }),
        ],
        parent: editorElement,
      });

      editorView = view;
      view.scrollDOM.addEventListener('scroll', syncScrollEditorToPreview, { passive: true });
    }

    // Now that CodeMirror is mounted, load the restored tab if any
    if (restored && editorView) {
      loadTabContent(restored).catch(console.error);
    } else if (!currentFilePath) {
      renderMarkdown(rawMarkdown);
    }

    let lastOpenFileEvent: { path: string; timestamp: number } | null = null;

    // Check initial file passed via CLI or Open With
    invoke<string | null>('get_initial_file_path')
      .then(async (path) => {
        if (path) {
          lastOpenFileEvent = { path, timestamp: Date.now() };
          showFileTree = false;
          const parts = path.split('/');
          const name = parts[parts.length - 1] || 'document.md';
          await openFile({ name, path, is_dir: false });
        }
      })
      .catch((e) => console.warn('No initial file:', e));

    // Listen for single instance open-file events
    listen<string>('open-file', (event) => {
      if (event.payload) {
        const now = Date.now();
        if (
          lastOpenFileEvent &&
          lastOpenFileEvent.path === event.payload &&
          now - lastOpenFileEvent.timestamp < 500
        ) {
          return;
        }
        lastOpenFileEvent = { path: event.payload, timestamp: now };
        showFileTree = false;
        const parts = event.payload.split('/');
        const name = parts[parts.length - 1] || 'document.md';
        openFile({ name, path: event.payload, is_dir: false });
      }
    }).catch((e) => console.warn('Failed to listen to open-file event:', e));

    if (page.url.searchParams.get('mode') === 'vault') {
      isVaultMode = true;
      loadVaultDocs();
    } else {
      loadDirectory(workspacePath);
    }

    updateDocumentContext({
      active_doc_path: currentFilePath,
      active_doc_title: currentDocTitle,
      active_doc_content: rawMarkdown,
      selected_text: '',
      is_vault_doc: isVaultMode,
    });

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      unregisterApply();
      unregisterUndo();
      unregisterExporter();
    };
  });

  async function renderMarkdown(content: string) {
    try {
      if (isHtmlFile(currentFilePath)) {
        return;
      }

      if (isSvgFile(currentFilePath)) {
        renderedSvgHtml = DOMPurify.sanitize(content, {
          USE_PROFILES: { svg: true, svgFilters: true },
        });
        return;
      }

      if (!isMarkdownFile(currentFilePath)) {
        const ext = (currentFilePath?.split('.').pop() || 'text').toLowerCase();
        const encoded = encodeURIComponent(content);
        renderedHtml = DOMPurify.sanitize(
          `
          <div class="code-block-wrapper" data-lang="${ext}">
            <div class="code-block-header">
              <span class="code-lang-pill">${ext}</span>
              <button type="button" class="code-copy-btn" data-code="${encoded}">
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
                ${t('common.copy')}
              </button>
            </div>
            <pre><code class="language-${ext}">${escapeHtml(content)}</code></pre>
          </div>
        `,
          {
            ADD_TAGS: ['button', 'span', 'div', 'svg', 'path', 'pre', 'code'],
            ADD_ATTR: [
              'data-lang',
              'data-code',
              'type',
              'd',
              'stroke',
              'fill',
              'viewBox',
              'stroke-width',
              'stroke-linecap',
              'stroke-linejoin',
            ],
          }
        );
        return;
      }

      // 1. Process math blocks ($$...$$ and $...$)
      let processed = content.replace(/\$\$([\s\S]+?)\$\$/g, (_, math) => {
        try {
          return katex.renderToString(math, { displayMode: true, throwOnError: false });
        } catch {
          return `$$${math}$$`;
        }
      });
      processed = processed.replace(/\$([^\$\n]+?)\$/g, (_, math) => {
        try {
          return katex.renderToString(math, { displayMode: false, throwOnError: false });
        } catch {
          return `$${math}$`;
        }
      });

      // 2. Parse GFM markdown to sanitized HTML
      const rawHtml = await marked.parse(processed, { gfm: true, breaks: true });
      renderedHtml = DOMPurify.sanitize(rawHtml, {
        ADD_TAGS: ['button', 'span', 'div', 'svg', 'path'],
        ADD_ATTR: [
          'data-lang',
          'data-code',
          'type',
          'd',
          'stroke',
          'fill',
          'viewBox',
          'stroke-width',
          'stroke-linecap',
          'stroke-linejoin',
        ],
      });

      // 3. Render Mermaid blocks on next tick (lazy-load mermaid only if needed)
      setTimeout(async () => {
        if (previewElement) {
          const mermaidBlocks = previewElement.querySelectorAll('code.language-mermaid');
          if (mermaidBlocks.length > 0) {
            await ensureMermaidInitialized();
            const m = await getMermaid();
            mermaidBlocks.forEach(async (block, index) => {
              const code = block.textContent || '';
              const id = `mermaid-svg-${index}-${Date.now()}`;
              try {
                const { svg } = await m.render(id, code);
                const container = document.createElement('div');
                container.className = 'my-4 flex justify-center overflow-x-auto';
                container.innerHTML = svg;
                block.parentElement?.replaceWith(container);
              } catch (err) {
                console.error('Mermaid render error:', err);
              }
            });
          }
        }
      }, 50);
    } catch (e) {
      console.error('Markdown rendering failed:', e);
    }
  }

  async function loadDirectory(path: string) {
    workspacePath = path;
    try {
      fileTree = await invoke<FileNode[]>('workspace_list_directory', { dirPath: path });
    } catch (e) {
      console.warn('Cannot list directory:', e);
    }
  }

  async function navigateUp() {
    if (workspacePath === '.' || !workspacePath) {
      // Already at root or relative root, attempt to query canonical parent or keep dot
      return;
    }
    const cleanPath = workspacePath.replace(/\/+$/, '');
    const lastSlash = cleanPath.lastIndexOf('/');
    if (lastSlash === -1) {
      await loadDirectory('.');
    } else if (lastSlash === 0) {
      await loadDirectory('/');
    } else {
      const parent = cleanPath.substring(0, lastSlash);
      await loadDirectory(parent);
    }
  }

  async function openWorkspaceFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t('editor.selectWorkspaceFolder'),
      });
      if (selected && typeof selected === 'string') {
        await loadDirectory(selected);
      }
    } catch (e) {
      console.warn('Failed to select folder:', e);
    }
  }

  async function saveVaultDoc() {
    if (!currentFilePath?.startsWith('vault://') || !isVaultUnlocked) return;
    const docId = currentFilePath.replace('vault://', '');
    const title = currentDocTitle.replace(/^🔒\s*/, '');
    try {
      await invoke('vault_save_document', {
        input: { id: docId, title, content: rawMarkdown, tags: [] },
        callerProfileId: 'default',
      });
      const existing = vaultDocs.find((d) => String(d.id) === String(docId));
      if (existing) {
        existing.content = rawMarkdown;
        existing.title = title;
      }
    } catch (e) {
      console.error('Failed to save vault doc:', e);
    }
  }

  let saveTimeout: any;
  function autoSave() {
    if (!currentFilePath) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      isSaving = true;
      try {
        if (currentFilePath?.startsWith('vault://')) {
          await saveVaultDoc();
          if (activeTabId) setDirty(activeTabId, false);
        } else if (currentFilePath) {
          await invoke('workspace_write_file', { filePath: currentFilePath, content: rawMarkdown });
          if (activeTabId) setDirty(activeTabId, false);
        }
      } catch (e) {
        console.error('Save failed:', e);
      } finally {
        setTimeout(() => {
          isSaving = false;
        }, 300);
      }
    }, 600);
  }

  async function flushCurrentSave() {
    if (saveTimeout) {
      clearTimeout(saveTimeout);
      saveTimeout = null;
    }
    if (currentFilePath?.startsWith('vault://')) {
      await saveVaultDoc();
    } else if (currentFilePath) {
      try {
        await invoke('workspace_write_file', { filePath: currentFilePath, content: rawMarkdown });
      } catch (e) {
        console.error('Failed writing file on flush:', e);
      }
    }
    if (activeTabId) setDirty(activeTabId, false);
  }

  async function loadTabContent(tab: DocTab) {
    // Skip if same file already loaded
    if (currentFilePath === tab.path && editorView && editorView.state.doc.length > 0) {
      // Already showing this file — just make sure scroll is at top
      editorView.dispatch({ effects: EditorView.scrollIntoView(0, { y: 'start' }) });
      return;
    }

    isLoadingTabDoc = true;
    try {
      if (tab.isVault) {
      const docId = tab.path.replace('vault://', '');
      if (!isVaultUnlocked) {
        await enterVaultMode();
        return;
      }
      if (!vaultDocs.find((d) => String(d.id) === String(docId))) {
        vaultDocs = await invoke<any[]>('vault_list_documents', { callerProfileId: 'default' });
      }
      const doc = vaultDocs.find((d) => String(d.id) === String(docId));
      const content = doc?.content ?? '';
      currentFilePath = tab.path;
      currentDocTitle = tab.title;
      rawMarkdown = content;
      await tick();
      if (editorView) {
        // First: update doc without language reconfigure so gutters stay in sync
        editorView.dispatch({
          changes: { from: 0, to: editorView.state.doc.length, insert: content },
        });
        // Second: reconfigure language on separate update so fold-gutter resets
        const langExt = await getLanguageExtension(tab.title || tab.path);
        editorView.dispatch({
          effects: languageCompartment.reconfigure(langExt),
        });
        editorView.dispatch({ effects: EditorView.scrollIntoView(0, { y: 'start' }) });
      }
      renderMarkdown(content);
      updateDocumentContext({
        active_doc_path: tab.path,
        active_doc_title: tab.title,
        active_doc_content: content,
        selected_text: '',
        is_vault_doc: tab.isVault,
      });
    } else {
      try {
        const content = await invoke<string>('workspace_read_file', { filePath: tab.path });
        currentFilePath = tab.path;
        currentDocTitle = tab.title;
        rawMarkdown = content;
        await tick();
        if (editorView) {
          // First: update doc without language reconfigure so gutters stay in sync
          editorView.dispatch({
            changes: { from: 0, to: editorView.state.doc.length, insert: content },
          });
          // Second: reconfigure language on separate update so fold-gutter resets
          const langExt = await getLanguageExtension(tab.title || tab.path);
          editorView.dispatch({
            effects: languageCompartment.reconfigure(langExt),
          });
          editorView.dispatch({ effects: EditorView.scrollIntoView(0, { y: 'start' }) });
        }
        renderMarkdown(content);
        updateDocumentContext({
          active_doc_path: tab.path,
          active_doc_title: tab.title,
          active_doc_content: content,
          selected_text: '',
          is_vault_doc: false,
        });
      } catch (e) {
        console.error('Failed reading tab file:', e);
      }
    }
  } finally {
    isLoadingTabDoc = false;
    setDirty(tab.id, false);
  }
}

  async function handleSelectTab(id: string) {
    setActiveTab(id);
    const target = tabs.find((t) => t.id === id);
    if (!target) return;
    const prevActive = getActiveTab();
    if (prevActive && prevActive.isDirty) {
      await flushCurrentSave();
    }
    await loadTabContent(target);
  }

  // Tab Context Menu State
  let tabContextMenu = $state<{
    open: boolean;
    x: number;
    y: number;
    tab: DocTab | null;
  }>({
    open: false,
    x: 0,
    y: 0,
    tab: null,
  });

  function handleTabContextMenu(e: MouseEvent, tab: DocTab) {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 220;
    const menuHeight = 240;
    const x = Math.min(e.clientX, window.innerWidth - menuWidth - 8);
    const y = Math.min(e.clientY, window.innerHeight - menuHeight - 8);
    tabContextMenu = {
      open: true,
      x: Math.max(8, x),
      y: Math.max(8, y),
      tab,
    };
  }

  function closeTabContextMenu() {
    tabContextMenu.open = false;
    tabContextMenu.tab = null;
  }

  async function handleContextMenuAction(action: 'close' | 'closeOthers' | 'closeRight' | 'closeAll' | 'copyPath' | 'copyName' | 'reveal') {
    const target = tabContextMenu.tab;
    closeTabContextMenu();
    if (!target) return;

    switch (action) {
      case 'close':
        await handleCloseTab(target.id);
        break;
      case 'closeOthers':
        closeOtherTabs(target.id);
        await loadTabContent(target);
        break;
      case 'closeRight':
        closeTabsToRight(target.id);
        const cur = getActiveTab();
        if (cur) await loadTabContent(cur);
        break;
      case 'closeAll':
        closeAllTabs();
        currentFilePath = null;
        currentDocTitle = 'Untitled Document.md';
        rawMarkdown = '';
        if (editorView) {
          editorView.dispatch({
            changes: { from: 0, to: editorView.state.doc.length, insert: '' },
          });
        }
        renderedHtml = '';
        break;
      case 'copyPath':
        await navigator.clipboard.writeText(target.path);
        showToast(t('editor.pathCopied'), 'success');
        break;
      case 'copyName':
        await navigator.clipboard.writeText(target.title.replace(/^🔒\s*/, ''));
        showToast(t('editor.nameCopied'), 'success');
        break;
      case 'reveal':
        if (!target.isVault) {
          try {
            await invoke('workspace_reveal_file', { filePath: target.path });
          } catch (err) {
            console.warn('Failed to reveal file:', err);
            showToast(t('editor.revealFailed'), 'error');
          }
        }
        break;
    }
  }

  async function handleCloseTab(id: string) {
    closeTab(id);
    const nextActive = getActiveTab();
    if (nextActive) {
      await loadTabContent(nextActive);
    } else {
      currentFilePath = null;
      currentDocTitle = t('editor.untitledDocument');
      rawMarkdown = '';
      if (editorView) {
        editorView.dispatch({
          changes: { from: 0, to: editorView.state.doc.length, insert: '' },
        });
      }
      renderedHtml = '';
      updateDocumentContext({
        active_doc_path: null,
        active_doc_title: t('editor.untitledDocument'),
        active_doc_content: '',
        selected_text: '',
        is_vault_doc: false,
      });
    }
  }

  function handleNewTab() {
    createNewFile();
  }

  async function openFile(node: FileNode) {
    if (node.is_dir) return;
    try {
      openTab(node.path, node.name, false);
      const target = getActiveTab();
      if (target) {
        await loadTabContent(target);
      }

      // Don't override workspacePath if opened file is already inside current workspacePath
      const lastSlash = node.path.lastIndexOf('/');
      if (lastSlash > 0) {
        const parentDir = node.path.substring(0, lastSlash);
        const isInsideWorkspace =
          workspacePath !== '.' &&
          (node.path === workspacePath ||
            node.path.startsWith(workspacePath.endsWith('/') ? workspacePath : `${workspacePath}/`));
        if (!isInsideWorkspace && parentDir !== workspacePath) {
          loadDirectory(parentDir);
        }
      }
    } catch (e) {
      console.error('Failed to open file:', e);
    }
  }

  async function enterVaultMode() {
    isVaultMode = true;
    try {
      const initialized = await invoke<boolean>('is_vault_initialized');
      isVaultSetup = !initialized;
      if (!isVaultUnlocked) {
        vaultModalOpen = true;
        vaultPassword = '';
        vaultConfirmPassword = '';
        vaultPasswordError = '';
        return;
      }
      await loadVaultDocs();
    } catch (e) {
      console.warn('Vault status check error:', e);
    }
  }

  async function leaveVaultMode() {
    isVaultMode = false;
    isVaultUnlocked = false;
    vaultDocs = [];
    try {
      await invoke('lock_vault');
    } catch (e) {
      console.warn('Failed to lock vault:', e);
    }
  }

  async function handleVaultPasswordSubmit() {
    vaultPasswordError = '';
    const pass = vaultPassword.trim();
    if (pass.length < 8) {
      vaultPasswordError = t('editor.passwordMinLength');
      return;
    }
    if (isVaultSetup) {
      if (pass !== vaultConfirmPassword.trim()) {
        vaultPasswordError = t('editor.passwordMismatch');
        return;
      }
    }

    isVaultModalLoading = true;
    try {
      await invoke('validate_vault_password', { password: pass });
      isVaultUnlocked = true;
      vaultModalOpen = false;
      vaultPassword = '';
      vaultConfirmPassword = '';
      await loadVaultDocs();
      showToast(isVaultSetup ? 'Master password set & vault unlocked' : 'Vault unlocked', 'success');
    } catch (e: any) {
      vaultPasswordError = typeof e === 'string' ? e : (e?.message || 'Master password salah');
    } finally {
      isVaultModalLoading = false;
    }
  }

  function closeVaultModal() {
    vaultModalOpen = false;
    vaultPassword = '';
    vaultConfirmPassword = '';
    vaultPasswordError = '';
    if (!isVaultUnlocked) {
      isVaultMode = false;
    }
  }

  async function loadVaultDocs() {
    if (!isVaultUnlocked) return;
    try {
      vaultDocs = await invoke<any[]>('vault_list_documents', { callerProfileId: 'default' });
      if (vaultDocs.length > 0 && !currentFilePath?.startsWith('vault://')) {
        openVaultDoc(vaultDocs[0]);
      }
    } catch (e) {
      console.warn('Vault list error:', e);
    }
  }

  async function openVaultDoc(doc: any) {
    if (!isVaultUnlocked) {
      await enterVaultMode();
      return;
    }
    const freshDoc = vaultDocs.find((d) => String(d.id) === String(doc.id)) || doc;
    const tabId = `vault://${freshDoc.id}`;
    const title = `🔒 ${freshDoc.title}`;
    openTab(tabId, title, true);
    currentFilePath = tabId;
    currentDocTitle = title;
    rawMarkdown = freshDoc.content || '';
    await tick();
    if (editorView) {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: rawMarkdown },
      });
    }
    renderMarkdown(rawMarkdown);
    updateDocumentContext({
      active_doc_path: tabId,
      active_doc_title: title,
      active_doc_content: rawMarkdown,
      selected_text: '',
      is_vault_doc: true,
    });
  }

  function getParentPath(filePath: string): string {
    const parts = filePath.split('/');
    if (parts.length <= 1) return '.';
    parts.pop();
    return parts.join('/') || '/';
  }

  function getDuplicateName(originalName: string, isDir: boolean): string {
    if (isDir) {
      return `${originalName} copy`;
    }
    const lastDot = originalName.lastIndexOf('.');
    if (lastDot > 0) {
      const base = originalName.substring(0, lastDot);
      const ext = originalName.substring(lastDot);
      return `${base} copy${ext}`;
    }
    return `${originalName} copy`;
  }

  function openFileContextMenu(e: MouseEvent, node: FileNode) {
    e.preventDefault();
    e.stopPropagation();
    selectedFileNode = node;
    fileContextMenu = {
      open: true,
      x: Math.min(e.clientX, window.innerWidth - 240),
      y: Math.min(e.clientY, window.innerHeight - 380),
      node,
      isBackground: false,
    };
  }

  function openSidebarBackgroundContextMenu(e: MouseEvent) {
    if ((e.target as HTMLElement)?.closest('button, [role="button"], input')) return;
    e.preventDefault();
    fileContextMenu = {
      open: true,
      x: Math.min(e.clientX, window.innerWidth - 240),
      y: Math.min(e.clientY, window.innerHeight - 240),
      node: null,
      isBackground: true,
    };
  }

  function closeFileContextMenu() {
    fileContextMenu.open = false;
    fileContextMenu.node = null;
  }

  function triggerNewFile(targetDir?: string) {
    closeFileContextMenu();
    if (isVaultMode) {
      if (!isVaultUnlocked) {
        enterVaultMode();
        return;
      }
      fileModal = {
        open: true,
        type: 'file',
        targetDir: 'vault',
        name: `files-${vaultDocs.length + 1}.md`,
        error: null,
      };
      return;
    }

    const dir = targetDir || (selectedFileNode?.is_dir ? selectedFileNode.path : workspacePath);
    fileModal = {
      open: true,
      type: 'file',
      targetDir: dir,
      name: 'new_document.md',
      error: null,
    };
  }

  function triggerNewFolder(targetDir?: string) {
    closeFileContextMenu();
    if (isVaultMode) {
      showToast(t('editor.createDirFilesOnly'), 'info');
      return;
    }

    const dir = targetDir || (selectedFileNode?.is_dir ? selectedFileNode.path : workspacePath);
    fileModal = {
      open: true,
      type: 'folder',
      targetDir: dir,
      name: 'new_folder',
      error: null,
    };
  }

  async function submitCreateFileOrFolder() {
    const cleanName = fileModal.name.trim();
    if (!cleanName) {
      fileModal.error = 'Name cannot be empty';
      return;
    }

    if (fileModal.targetDir === 'vault') {
      let title = cleanName;
      if (!title.endsWith('.md')) {
        title = `${title}.md`;
      }
      try {
        const newDoc: any = await invoke('vault_save_document', {
          input: { title, content: '# ' + title + '\n\n', tags: [] },
          callerProfileId: 'default',
        });
        await loadVaultDocs();
        openVaultDoc(newDoc);
        fileModal.open = false;
        showToast(t('editor.vaultNoteCreated', { title }), 'success');
      } catch (e) {
        fileModal.error = `${t('editor.vaultNoteFailed')}${e}`;
      }
      return;
    }

    const dir = fileModal.targetDir || workspacePath;
    const targetPath = dir === '.' ? cleanName : `${dir}/${cleanName}`;

    if (fileModal.type === 'file') {
      try {
        await invoke('workspace_create_file', { filePath: targetPath });
        await loadDirectory(workspacePath);
        if (dir !== '.' && dir !== workspacePath) {
          const next = new Set(expandedDirs);
          next.add(dir);
          expandedDirs = next;
        }
        openFile({ name: cleanName, path: targetPath, is_dir: false });
        fileModal.open = false;
        showToast(t('editor.fileCreated', { name: cleanName }), 'success');
      } catch (e) {
        fileModal.error = `${t('editor.fileFailed')}${e}`;
      }
    } else {
      try {
        await invoke('workspace_create_directory', { dirPath: targetPath });
        await loadDirectory(workspacePath);
        const next = new Set(expandedDirs);
        next.add(targetPath);
        if (dir !== '.' && dir !== workspacePath) {
          next.add(dir);
        }
        expandedDirs = next;
        fileModal.open = false;
        showToast(t('editor.folderCreated', { name: cleanName }), 'success');
      } catch (e) {
        fileModal.error = `${t('editor.folderFailed')}${e}`;
      }
    }
  }

  async function createNewFile() {
    triggerNewFile();
  }

  async function createNewDirectory() {
    triggerNewFolder();
  }

  function triggerRename(node?: FileNode | null) {
    closeFileContextMenu();
    const target = node || selectedFileNode;
    if (!target) return;
    renameModal = {
      open: true,
      node: target,
      name: target.name,
      error: null,
    };
  }

  async function submitRename() {
    if (!renameModal.node) return;
    const target = renameModal.node;
    const cleanName = renameModal.name.trim();
    if (!cleanName) {
      renameModal.error = 'Name cannot be empty';
      return;
    }
    if (cleanName === target.name) {
      renameModal.open = false;
      return;
    }

    const parentDir = getParentPath(target.path);
    const newPath = parentDir === '.' ? cleanName : `${parentDir}/${cleanName}`;

    try {
      await invoke('workspace_rename_file', { oldPath: target.path, newPath });

      // Sync tabs
      if (!target.is_dir) {
        for (const tab of tabs) {
          if (tab.path === target.path) {
            tab.path = newPath;
            tab.title = cleanName;
            if (tab.id === activeTabId) {
              currentFilePath = newPath;
              currentDocTitle = cleanName;
              updateDocumentContext({
                active_doc_path: newPath,
                active_doc_title: cleanName,
              });
            }
          }
        }
      } else {
        const prefix = target.path.endsWith('/') ? target.path : target.path + '/';
        const newPrefix = newPath.endsWith('/') ? newPath : newPath + '/';
        for (const tab of tabs) {
          if (tab.path === target.path || tab.path.startsWith(prefix)) {
            tab.path = newPrefix + tab.path.slice(prefix.length);
            tab.title = tab.path.split('/').pop() || tab.title;
            if (tab.id === activeTabId) {
              currentFilePath = tab.path;
              currentDocTitle = tab.title;
              updateDocumentContext({
                active_doc_path: tab.path,
                active_doc_title: tab.title,
              });
            }
          }
        }
      }

      if (selectedFileNode?.path === target.path) {
        selectedFileNode = { ...target, name: cleanName, path: newPath };
      }

      await loadDirectory(workspacePath);
      renameModal.open = false;
      showToast(`Renamed to "${cleanName}"`, 'success');
    } catch (e) {
      renameModal.error = `Rename failed: ${e}`;
    }
  }

  function triggerDelete(node?: FileNode | null) {
    closeFileContextMenu();
    const target = node || selectedFileNode;
    if (!target) return;
    deleteModal = {
      open: true,
      node: target,
      loading: false,
    };
  }

  async function submitDelete() {
    if (!deleteModal.node) return;
    const target = deleteModal.node;
    deleteModal.loading = true;

    try {
      await invoke('workspace_delete_file', { filePath: target.path });

      if (!target.is_dir) {
        const tabToClose = tabs.find((t) => t.path === target.path);
        if (tabToClose) {
          await handleCloseTab(tabToClose.id);
        }
      } else {
        const prefix = target.path.endsWith('/') ? target.path : target.path + '/';
        const tabsToClose = tabs.filter(
          (t) => t.path === target.path || t.path.startsWith(prefix)
        );
        for (const tab of tabsToClose) {
          await handleCloseTab(tab.id);
        }
      }

      if (selectedFileNode?.path === target.path) {
        selectedFileNode = null;
      }

      await loadDirectory(workspacePath);
      deleteModal.open = false;
      showToast(`Deleted "${target.name}"`, 'success');
    } catch (e) {
      showToast(`Failed to delete: ${e}`, 'error');
    } finally {
      deleteModal.loading = false;
    }
  }

  async function handleDuplicate(node?: FileNode | null) {
    closeFileContextMenu();
    const target = node || selectedFileNode;
    if (!target) return;

    const parentDir = getParentPath(target.path);
    const dupName = getDuplicateName(target.name, target.is_dir);
    const destPath = parentDir === '.' ? dupName : `${parentDir}/${dupName}`;

    try {
      await invoke('workspace_copy_file', {
        srcPath: target.path,
        destPath,
      });
      await loadDirectory(workspacePath);
      showToast(`Duplicated "${dupName}"`, 'success');
    } catch (e) {
      showToast(`Failed to duplicate: ${e}`, 'error');
    }
  }

  function handleCut(node?: FileNode | null) {
    closeFileContextMenu();
    const target = node || selectedFileNode;
    if (!target) return;
    fileClipboard = { mode: 'cut', node: target };
    showToast(`Cut "${target.name}" to clipboard`, 'info');
  }

  function handleCopy(node?: FileNode | null) {
    closeFileContextMenu();
    const target = node || selectedFileNode;
    if (!target) return;
    fileClipboard = { mode: 'copy', node: target };
    showToast(`Copied "${target.name}" to clipboard`, 'info');
  }

  async function handlePaste(destDir?: string) {
    closeFileContextMenu();
    if (!fileClipboard) return;
    const src = fileClipboard.node;
    const targetDir = destDir || (selectedFileNode?.is_dir ? selectedFileNode.path : workspacePath);

    if (src.is_dir) {
      const srcPrefix = src.path.endsWith('/') ? src.path : src.path + '/';
      if (targetDir === src.path || targetDir.startsWith(srcPrefix)) {
        showToast('Cannot copy or move a folder into itself', 'error');
        return;
      }
    }

    const srcParent = getParentPath(src.path);
    let destName = src.name;
    if (fileClipboard.mode === 'copy' && targetDir === srcParent) {
      destName = getDuplicateName(src.name, src.is_dir);
    }

    const targetPath = targetDir === '.' ? destName : `${targetDir}/${destName}`;

    try {
      if (fileClipboard.mode === 'copy') {
        await invoke('workspace_copy_file', {
          srcPath: src.path,
          destPath: targetPath,
        });
        showToast(`Pasted copy "${destName}"`, 'success');
      } else {
        await invoke('workspace_rename_file', {
          oldPath: src.path,
          newPath: targetPath,
        });

        if (!src.is_dir) {
          for (const tab of tabs) {
            if (tab.path === src.path) {
              tab.path = targetPath;
              tab.title = destName;
              if (tab.id === activeTabId) {
                currentFilePath = targetPath;
                currentDocTitle = destName;
                updateDocumentContext({
                  active_doc_path: targetPath,
                  active_doc_title: destName,
                });
              }
            }
          }
        } else {
          const prefix = src.path.endsWith('/') ? src.path : src.path + '/';
          const newPrefix = targetPath.endsWith('/') ? targetPath : targetPath + '/';
          for (const tab of tabs) {
            if (tab.path === src.path || tab.path.startsWith(prefix)) {
              tab.path = newPrefix + tab.path.slice(prefix.length);
              tab.title = tab.path.split('/').pop() || tab.title;
              if (tab.id === activeTabId) {
                currentFilePath = tab.path;
                currentDocTitle = tab.title;
                updateDocumentContext({
                  active_doc_path: tab.path,
                  active_doc_title: tab.title,
                });
              }
            }
          }
        }

        fileClipboard = null;
        showToast(`Moved "${destName}"`, 'success');
      }

      await loadDirectory(workspacePath);
    } catch (e) {
      showToast(`Paste failed: ${e}`, 'error');
    }
  }

  async function handleSendToVault(node: FileNode) {
    closeFileContextMenu();
    if (node.is_dir) {
      showToast('Only markdown documents can be sent to Vault', 'info');
      return;
    }

    let content = '';
    if (currentFilePath === node.path) {
      content = rawMarkdown;
    } else {
      try {
        content = await invoke<string>('workspace_read_file', { filePath: node.path });
      } catch (e) {
        showToast(`Failed to read file: ${e}`, 'error');
        return;
      }
    }

    try {
      await invoke('vault_save_document', {
        input: {
          title: node.name,
          content: content || '# ' + node.name + '\n\n',
          tags: ['workspace-export'],
        },
        callerProfileId: 'default',
      });
      if (isVaultUnlocked) {
        await loadVaultDocs();
      }
      showToast(`Encrypted & saved "${node.name}" to Vault`, 'success');
    } catch (e) {
      showToast(`Failed to save to vault: ${e}`, 'error');
    }
  }

  async function handleSendToHanaAi(node: FileNode) {
    closeFileContextMenu();
    if (node.is_dir) {
      showToast('Please select a file to send to Hana AI', 'info');
      return;
    }

    let content = '';
    if (currentFilePath === node.path) {
      content = rawMarkdown;
    } else {
      try {
        content = await invoke<string>('workspace_read_file', { filePath: node.path });
      } catch (e) {
        showToast(`Failed to read file: ${e}`, 'error');
        return;
      }
    }

    updateDocumentContext({
      active_doc_path: node.path,
      active_doc_title: node.name,
      active_doc_content: content,
      selected_text: '',
      is_vault_doc: false,
    });

    openAiChat();
    showToast(`"${node.name}" context attached to Hana AI`, 'success');
  }

  async function handleCopyFullPath(node: FileNode) {
    closeFileContextMenu();
    try {
      await navigator.clipboard.writeText(node.path);
      showToast(t('editor.pathCopied'), 'success');
    } catch {
      showToast('Failed to copy full path', 'error');
    }
  }

  async function handleCopyRelativePath(node: FileNode) {
    closeFileContextMenu();
    try {
      let rel = node.path;
      if (workspacePath !== '.' && rel.startsWith(workspacePath)) {
        rel = rel.slice(workspacePath.length).replace(/^\/+/, '');
      }
      await navigator.clipboard.writeText(rel);
      showToast(t('editor.copyRelativePath') + ' copied!', 'success');
    } catch {
      showToast('Failed to copy relative path', 'error');
    }
  }

  async function handleRevealFile(filePath: string) {
    closeFileContextMenu();
    try {
      await invoke('workspace_reveal_file', { filePath });
    } catch (err) {
      console.warn('Failed to reveal file:', err);
      showToast(t('editor.revealFailed'), 'error');
    }
  }

  async function handleRefreshWorkspace() {
    closeFileContextMenu();
    if (isVaultMode) {
      if (isVaultUnlocked) {
        await loadVaultDocs();
      }
    } else {
      await loadDirectory(workspacePath);
    }
    showToast(t('editor.refresh') + ' OK', 'success');
  }

  function handleSelectNode(node: FileNode) {
    selectedFileNode = node;
  }

  function handleQuickNewFile(node: FileNode) {
    triggerNewFile(node.path);
  }

  function handleQuickNewFolder(node: FileNode) {
    triggerNewFolder(node.path);
  }

  async function exportHtml() {
    if (!pro.isPro) {
      showToast(t('pro.exportExclusive'), 'error');
      return;
    }
    const defaultName = currentDocTitle.replace(/\.md$/, '.html').replace('🔒 ', '');
    const targetPath = prompt(t('editor.saveHtmlAs'), `/tmp/${defaultName}`);
    if (!targetPath) return;

    try {
      const res = await invoke<any>('export_document_html', {
        htmlContent: renderedHtml,
        docTitle: currentDocTitle,
        targetPath,
      });
      showToast(res.message || t('editor.exportHtmlSuccess'), 'success');
    } catch (e) {
      alert(`${t('editor.exportHtmlFailed')}${e}`);
    }
  }

  async function exportPdf() {
    if (!pro.isPro) {
      showToast(t('pro.exportExclusive'), 'error');
      return;
    }
    // Native print dialog (WebView Print -> Save as PDF)
    window.print();
  }

  async function handlePreviewClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target) return;

    // Handle Code Block Copy Button
    const copyBtn = target.closest('.code-copy-btn') as HTMLButtonElement | null;
    if (copyBtn) {
      const encoded = copyBtn.getAttribute('data-code');
      if (encoded) {
        const codeText = decodeURIComponent(encoded);
        await navigator.clipboard.writeText(codeText);
        copyBtn.classList.add('copied');
        copyBtn.innerHTML = `
          <svg class="w-3 h-3 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/></svg>
          ${t('common.copied')}
        `;
        setTimeout(() => {
          copyBtn.classList.remove('copied');
          copyBtn.innerHTML = `
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
            ${t('common.copy')}
          `;
        }, 2000);
      }
      return;
    }

    // Interactive GFM Task List Checkboxes
    if (target.tagName === 'INPUT' && target.getAttribute('type') === 'checkbox') {
      const checkbox = target as HTMLInputElement;
      const listItem = checkbox.closest('li');
      if (listItem && listItem.textContent) {
        const text = listItem.textContent.trim();
        const checked = checkbox.checked;
        const targetSearch = checked ? `[ ] ${text}` : `[x] ${text}`;
        const targetReplace = checked ? `[x] ${text}` : `[ ] ${text}`;

        if (rawMarkdown.includes(targetSearch)) {
          const updated = rawMarkdown.replace(targetSearch, targetReplace);
          rawMarkdown = updated;
          if (editorView) {
            editorView.dispatch({
              changes: { from: 0, to: editorView.state.doc.length, insert: updated },
            });
          }
          renderMarkdown(updated);
          autoSave();
        }
      }
      return;
    }

    // Markdown Links & Anchors (TOC internal headings, relative .md, external web)
    const anchor = target.closest('a') as HTMLAnchorElement | null;
    if (anchor) {
      const href = anchor.getAttribute('href');
      if (href) {
        e.preventDefault();
        e.stopPropagation();

        if (href.startsWith('#')) {
          const targetId = decodeURIComponent(href.slice(1));
          if (previewElement) {
            let el = previewElement.querySelector(`#${CSS.escape(targetId)}`);
            if (!el) {
              const normalized = targetId.toLowerCase().replace(/[^\w\s-]/g, '').trim().replace(/\s+/g, '-');
              el = previewElement.querySelector(`#${CSS.escape(normalized)}`);
              if (!el) {
                const headings = Array.from(previewElement.querySelectorAll('h1, h2, h3, h4, h5, h6'));
                el = headings.find((h) => {
                  const hText = (h.textContent || '').toLowerCase().replace(/[^\w\s-]/g, '').trim().replace(/\s+/g, '-');
                  return hText === normalized || hText.includes(normalized) || normalized.includes(hText);
                }) || null;
              }
            }
            if (el) {
              el.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
          }
        } else if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
          try {
            await invoke('workspace_reveal_file', { filePath: href });
          } catch {
            window.open(href, '_blank');
          }
        } else if (href.endsWith('.md') || href.includes('.md#') || !href.includes('://')) {
          let targetPath = href.split('#')[0];
          if (currentFilePath && !currentFilePath.startsWith('vault://')) {
            const currentDir = currentFilePath.includes('/') ? currentFilePath.substring(0, currentFilePath.lastIndexOf('/')) : '.';
            if (targetPath.startsWith('./')) {
              targetPath = `${currentDir}/${targetPath.slice(2)}`;
            } else if (!targetPath.startsWith('/')) {
              targetPath = `${currentDir}/${targetPath}`;
            }
          }
          const parts = targetPath.split('/');
          const name = parts[parts.length - 1] || 'document.md';
          openFile({ name, path: targetPath, is_dir: false });
        }
      }
      return;
    }
  }
</script>

<div class="h-full flex flex-col bg-white dark:bg-[#161616] text-neutral-800 dark:text-neutral-200 overflow-hidden font-sans relative">
  <!-- Unified Studio Header & Tab Bar (h-10 / 40px) -->
  <header class="h-10 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-100/90 dark:bg-[#121212] px-2 flex items-center justify-between gap-2 z-10 shrink-0 no-print select-none">
    <!-- Left: File Tree Toggle & Saving State -->
    <div class="flex items-center gap-1.5 shrink-0">
      <button
        onclick={toggleFileTree}
        class="p-1.5 rounded-lg transition-colors {showFileTree ? 'bg-cyan-500/10 text-cyan-600 dark:text-cyan-400 border border-cyan-500/20' : 'text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800'}"
        title={t('editor.toggleFilePanel')}
        aria-label="Toggle Files Explorer Rail"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
      </button>

      <div class="h-4 w-px bg-neutral-300 dark:bg-neutral-800 mx-0.5"></div>
    </div>

    <!-- Scroll Left Button -->
    <button
      type="button"
      onclick={scrollTabsLeft}
      class="h-7 w-5 rounded flex items-center justify-center text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 transition-colors shrink-0 select-none cursor-pointer"
      title="Scroll tabs left"
      aria-label="Scroll tabs left"
    >
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
    </button>

    <!-- Center/Fill: Document Tabs Strip -->
    <div
      bind:this={tabsContainerEl}
      onwheel={handleTabsWheel}
      class="flex-1 min-w-0 flex items-center gap-1 overflow-x-auto scrollbar-none h-full pt-1 scroll-smooth"
    >
      {#each tabs as tab (tab.id)}
        {@const isActive = tab.id === activeTabId}
        <div
          role="button"
          tabindex="0"
          data-tab-id={tab.id}
          onclick={() => handleSelectTab(tab.id)}
          oncontextmenu={(e) => handleTabContextMenu(e, tab)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelectTab(tab.id); }}
          class="group relative h-8 px-2.5 rounded-t-md flex items-center gap-1.5 text-xs font-mono transition-all cursor-pointer border-t border-x flex-shrink min-w-[70px] max-w-[140px] sm:max-w-[190px] {isActive ? 'bg-white dark:bg-[#1a1a1a] text-cyan-600 dark:text-cyan-400 border-neutral-300 dark:border-neutral-700/80 shadow-2xs font-semibold' : 'bg-transparent border-transparent text-neutral-500 hover:text-neutral-800 dark:text-neutral-400 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40'}"
          title={tab.path}
        >
          {#if isActive}
            <span class="absolute top-0 left-0 right-0 h-[2px] bg-cyan-500 rounded-t" aria-hidden="true"></span>
          {/if}

          <span class="shrink-0 text-[11px] flex items-center justify-center">
            {#if tab.isVault}
              🔒
            {:else if tab.path.endsWith('.rs')}
              <span class="text-orange-500 font-bold text-[9px] font-mono">RS</span>
            {:else if tab.path.endsWith('.py') || tab.path.endsWith('.pyw')}
              <span class="text-yellow-500 font-bold text-[9px] font-mono">PY</span>
            {:else if tab.path.endsWith('.js') || tab.path.endsWith('.ts') || tab.path.endsWith('.jsx') || tab.path.endsWith('.tsx')}
              <span class="text-cyan-400 font-bold text-[9px] font-mono">JS</span>
            {:else if tab.path.endsWith('.json')}
              <span class="text-amber-400 font-bold text-[9px] font-mono">{`{}`}</span>
            {:else if tab.path.endsWith('.toml') || tab.path.endsWith('.yaml') || tab.path.endsWith('.yml')}
              <span class="text-purple-400 font-bold text-[9px] font-mono">⚙</span>
            {:else if tab.path.endsWith('.html') || tab.path.endsWith('.css') || tab.path.endsWith('.svelte')}
              <span class="text-rose-400 font-bold text-[9px] font-mono">&lt;&gt;</span>
            {:else}
              <svg class="w-3.5 h-3.5 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            {/if}
          </span>

          <span class="truncate flex-1 min-w-0">{tab.title}</span>

          {#if tabs.length > 1}
            <button
              type="button"
              onclick={(e) => {
                e.stopPropagation();
                handleCloseTab(tab.id);
              }}
              class="opacity-60 group-hover:opacity-100 p-0.5 rounded-sm hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-400 hover:text-rose-500 dark:hover:text-rose-400 transition-all shrink-0"
              title={t('editor.closeTabShortcut')}
              aria-label={t('editor.closeTab')}
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Scroll Right Button & New Tab Button -->
    <div class="flex items-center gap-0.5 shrink-0">
      <button
        type="button"
        onclick={scrollTabsRight}
        class="h-7 w-5 rounded flex items-center justify-center text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 transition-colors select-none cursor-pointer"
        title="Scroll tabs right"
        aria-label="Scroll tabs right"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>

      <button
        type="button"
        onclick={handleNewTab}
        class="h-7 w-7 rounded-md flex items-center justify-center text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 transition-colors shrink-0 cursor-pointer"
        title={t('editor.openNewDocShortcut')}
        aria-label={t('editor.newDocument')}
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
  </header>

  <!-- Main Multi-Pane Body -->
  <div class="flex-1 flex overflow-hidden min-h-0 relative">
    <!-- Workspace Files / Vault Explorer Sidebar -->
    <aside
      oncontextmenu={(e) => e.preventDefault()}
      class="{showFileTree ? 'w-60 sm:w-64' : 'hidden'} border-r border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-[#111111] flex flex-col shrink-0 select-none file-tree-sidebar no-print"
    >
      <!-- Mode Switcher Tabs: Files vs Encrypted Vault -->
      <div class="grid grid-cols-2 p-1.5 border-b border-neutral-200 dark:border-neutral-800/80 gap-1 bg-neutral-100/80 dark:bg-neutral-950/60">
        <button
          onclick={leaveVaultMode}
          class="py-1 px-2 rounded-md text-[11px] font-semibold transition-all {(!isVaultMode) ? 'bg-white dark:bg-neutral-800 text-cyan-600 dark:text-cyan-400 shadow-2xs' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >
          📁 {t('editor.filesTab')}
        </button>
        <button
          onclick={enterVaultMode}
          class="py-1 px-2 rounded-md text-[11px] font-semibold transition-all {isVaultMode ? 'bg-white dark:bg-neutral-800 text-amber-600 dark:text-amber-400 shadow-2xs' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >
          🔒 {t('editor.vaultTab')}
        </button>
      </div>

      <!-- Action Sub-header: Title & Top Action Toolbar (New File, New Folder, Refresh) -->
      <div class="p-2.5 px-3 border-b border-neutral-200 dark:border-neutral-800/80 flex items-center justify-between">
        <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          {isVaultMode ? t('editor.vaultNotes') : t('editor.filesAndFolders')}
        </span>
        <div class="flex items-center gap-1">
          <!-- Refresh Button -->
          <button
            type="button"
            onclick={handleRefreshWorkspace}
            class="p-1 rounded-md hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors"
            title={t('editor.refresh')}
            aria-label={t('editor.refresh')}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>

          {#if !isVaultMode}
            <!-- New Folder Button (+ Folder) -->
            <button
              type="button"
              onclick={() => triggerNewFolder()}
              class="p-1 rounded-md hover:bg-neutral-200 dark:hover:bg-neutral-800 text-amber-600 dark:text-amber-400 transition-colors"
              title="{t('editor.newFolder')} (+ Folder)"
              aria-label={t('editor.newFolder')}
            >
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
              </svg>
            </button>
          {/if}

          <!-- New File Button (+ File) -->
          <button
            type="button"
            onclick={() => triggerNewFile()}
            class="p-1 rounded-md hover:bg-neutral-200 dark:hover:bg-neutral-800 {isVaultMode ? 'text-amber-600 dark:text-amber-400' : 'text-cyan-600 dark:text-cyan-400'} transition-colors"
            title="{isVaultMode ? t('editor.createNewVaultNoteTitle') : t('editor.createNewFileTitle')} (+ File)"
            aria-label={t('editor.newFile')}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
      </div>

      {#if !isVaultMode}
        <!-- Breadcrumb / Active Folder Bar with Parent and Open Folder actions -->
        <div class="px-2.5 py-1.5 border-b border-neutral-200 dark:border-neutral-800/80 bg-neutral-100/50 dark:bg-neutral-900/40 flex items-center justify-between gap-1 text-[11px] font-mono">
          <div class="flex items-center gap-1.5 min-w-0 flex-1" title={workspacePath}>
            <button
              onclick={navigateUp}
              disabled={workspacePath === '.' || workspacePath === '/'}
              class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-300 disabled:opacity-30 disabled:pointer-events-none transition-colors shrink-0"
              title={t('editor.parentDirUp')}
              aria-label={t('editor.parentDir')}
            >
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
              </svg>
            </button>
            <span class="truncate text-neutral-700 dark:text-neutral-300 font-medium">
              {workspacePath === '.' ? './ (Workspace)' : workspacePath.split('/').filter(Boolean).pop() || '/'}
            </span>
          </div>

          <button
            onclick={openWorkspaceFolder}
            class="px-1.5 py-0.5 rounded text-[10px] bg-neutral-200/70 hover:bg-neutral-300/80 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-200 transition-colors shrink-0 flex items-center gap-1"
            title={t('editor.selectFolderOther')}
          >
            <svg class="w-3 h-3 text-cyan-600 dark:text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
            </svg>
            <span>{t('editor.openFolder')}</span>
          </button>
        </div>
      {/if}

      <!-- Scrollable File / Vault Tree -->
      <div
        class="flex-1 overflow-y-auto p-2 space-y-0.5 text-xs font-mono scrollbar-none"
        oncontextmenu={openSidebarBackgroundContextMenu}
        role="presentation"
      >
        {#if isVaultMode}
          {#each vaultDocs as doc (doc.id)}
            <button
              draggable="true"
              ondragstart={(e) => {
                if (e.dataTransfer) {
                  const payload = JSON.stringify({
                    path: `vault://${doc.id}`,
                    name: doc.title,
                    isVault: true,
                    content: doc.content || '',
                  });
                  e.dataTransfer.setData('application/x-camark-file', payload);
                  e.dataTransfer.setData('text/plain', `vault://${doc.id}`);
                  e.dataTransfer.effectAllowed = 'copyMove';
                }
              }}
              onclick={() => openVaultDoc(doc)}
              class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left transition-colors cursor-grab active:cursor-grabbing {currentFilePath === `vault://${doc.id}` ? 'bg-amber-500/15 text-amber-600 dark:text-amber-300 font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 hover:text-neutral-900 dark:hover:text-white'}"
            >
              <svg class="w-3.5 h-3.5 text-amber-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              <span class="truncate">{doc.title}</span>
            </button>
          {/each}
          {#if vaultDocs.length === 0}
            <div class="p-4 text-center text-neutral-400 dark:text-neutral-500 text-[11px]">
              {t('editor.noVaultNotes')}
            </div>
          {/if}
        {:else}
          {#each fileTree as node (node.path)}
            <FileTreeNode
              {node}
              depth={0}
              {currentFilePath}
              selectedNodePath={selectedFileNode?.path}
              {expandedDirs}
              onOpenFile={openFile}
              onToggleDir={toggleDir}
              onSelectNode={handleSelectNode}
              onContextMenu={openFileContextMenu}
              onQuickNewFile={handleQuickNewFile}
              onQuickNewFolder={handleQuickNewFolder}
            />
          {/each}
          {#if fileTree.length === 0}
            <div class="p-4 text-center text-neutral-400 dark:text-neutral-500 text-[11px]">
              {t('editor.noFilesFound')}
            </div>
          {/if}
        {/if}
      </div>
    </aside>

    <!-- Editor Pane (CodeMirror 6 with Format Toolbar & Zen Mode) -->
    <main
      class="flex-1 min-h-0 flex flex-col overflow-hidden relative {viewMode === 'live' ? 'bg-white dark:bg-[#0c1017] bg-[linear-gradient(to_right,#00000006_1px,transparent_1px),linear-gradient(to_bottom,#00000006_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px),linear-gradient(to_bottom,#ffffff05_1px,transparent_1px)] bg-[size:56px_56px]' : 'bg-neutral-50 dark:bg-[#0f141c]'} {viewMode === 'split' ? 'border-r border-neutral-200 dark:border-neutral-800' : ''} {viewMode === 'preview' ? 'hidden' : 'flex'}"
    >
      <!-- Format Toolbar Above Editor -->
      <MarkdownFormatToolbar
        {editorView}
        {isZenMode}
        {isOutlineOpen}
        onToggleZen={handleToggleZen}
        onToggleOutline={() => (isOutlineOpen = !isOutlineOpen)}
        onTogglePresentation={() => {
          if (!pro.isPro) {
            showToast(t('pro.presentationExclusive'), 'error');
            return;
          }
          isPresentationOpen = true;
        }}
        onFormatDocumentTable={handleFormatDocumentTable}
      />

      <!-- CodeMirror Editor Viewport (no overflow-auto or padding that would push gutters out of alignment) -->
      <div
        bind:this={editorElement}
        class="flex-1 min-h-0 h-full w-full overflow-hidden {viewMode === 'live' ? 'cm-live-mode max-w-4xl w-full mx-auto' : ''} {isZenMode ? 'max-w-3xl mx-auto w-full pt-12 pb-32' : ''}"
      ></div>

      <!-- Floating Contextual AI Action Bar on Text Selection -->
      {#if selectedText.trim().length > 0}
        <div class="absolute bottom-6 left-1/2 -translate-x-1/2 z-30">
          <ContextualAiActionBar
            {selectedText}
            onApplyResult={handleApplyAiResult}
            onClose={() => {
              selectedText = '';
              selectionRange = null;
            }}
          />
        </div>
      {/if}
    </main>

    <!-- Live Preview Pane with Synchronized Scrolling (hardware-accelerated) -->
    <section
      bind:this={previewElement}
      onscroll={viewMode === 'split' ? syncScrollPreviewToEditor : undefined}
      onclick={handlePreviewClick}
      class="preview-scroll-accel flex-1 overflow-y-auto bg-white dark:bg-[#0c1017] {isHtmlFile(currentFilePath) ? '!p-0 overflow-hidden' : isSvgFile(currentFilePath) ? 'p-4 sm:p-6' : 'p-8 sm:p-10'} markdown-preview max-w-none select-text overscroll-contain will-change-[scroll-position] bg-[linear-gradient(to_right,#00000006_1px,transparent_1px),linear-gradient(to_bottom,#00000006_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px),linear-gradient(to_bottom,#ffffff05_1px,transparent_1px)] bg-[size:56px_56px] {viewMode === 'editor' || viewMode === 'live' || isZenMode ? 'hidden' : 'block'}"
    >
      {#if isHtmlFile(currentFilePath)}
        <div class="w-full h-full flex flex-col bg-white dark:bg-[#111111] overflow-hidden">
          <div class="h-8 px-3 bg-neutral-100 dark:bg-[#161616] border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between text-xs text-neutral-500 font-mono select-none shrink-0 no-print">
            <span class="flex items-center gap-1.5 text-cyan-600 dark:text-cyan-400 font-semibold text-[11px]">
              <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
              <span>LIVE WEB PREVIEW</span>
            </span>
            <span class="text-[10px] text-neutral-400">Sandboxed HTML Canvas</span>
          </div>
          <iframe
            title="Live Web Preview"
            srcdoc={rawMarkdown}
            sandbox="allow-scripts allow-forms allow-same-origin allow-modals"
            class="w-full flex-1 border-0 bg-white"
          ></iframe>
        </div>
      {:else if isSvgFile(currentFilePath)}
        <div class="w-full h-full flex flex-col items-center justify-center min-h-[400px]">
          <div class="p-6 sm:p-8 rounded-2xl bg-neutral-100 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 shadow-xl max-w-2xl w-full flex flex-col items-center gap-4">
            <div class="w-full flex items-center justify-between text-xs font-mono text-neutral-500 pb-2 border-b border-neutral-200 dark:border-neutral-800">
              <span class="font-semibold text-cyan-600 dark:text-cyan-400 flex items-center gap-1.5">
                <svg class="w-4 h-4 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
                  <polyline points="2 17 12 22 22 17"></polyline>
                  <polyline points="2 12 12 17 22 12"></polyline>
                </svg>
                <span>Vector SVG Canvas</span>
              </span>
              <span class="text-[10px] px-2 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/20 font-bold">SVG</span>
            </div>

            <!-- Checkerboard Transparent Backdrop Canvas for Graphic Preview -->
            <div class="w-full flex items-center justify-center p-8 rounded-xl border border-neutral-300/80 dark:border-neutral-800 bg-[linear-gradient(45deg,#ccc_25%,transparent_25%),linear-gradient(-45deg,#ccc_25%,transparent_25%),linear-gradient(45deg,transparent_75%,#ccc_75%),linear-gradient(-45deg,transparent_75%,#ccc_75%)] dark:bg-[linear-gradient(45deg,#1f1f1f_25%,transparent_25%),linear-gradient(-45deg,#1f1f1f_25%,transparent_25%),linear-gradient(45deg,transparent_75%,#1f1f1f_75%),linear-gradient(-45deg,transparent_75%,#1f1f1f_75%)] bg-[size:16px_16px] bg-[position:0_0,0_8px,8px_-8px,-8px_0] overflow-auto">
              <div class="max-w-full max-h-[500px] flex items-center justify-center drop-shadow-md svg-graphic-wrapper">
                {@html renderedSvgHtml}
              </div>
            </div>
          </div>
        </div>
      {:else}
        {@html renderedHtml}
      {/if}
    </section>

    <!-- Live Document Outline Drawer -->
    <OutlineDrawer
      isOpen={isOutlineOpen && !isZenMode}
      {rawMarkdown}
      onJumpToHeading={handleJumpToHeading}
      onClose={() => (isOutlineOpen = false)}
    />
  </div>

  <!-- Fullscreen Presentation Slides Modal (PRO) -->
  <PresentationModal
    isOpen={isPresentationOpen}
    {rawMarkdown}
    docTitle={currentDocTitle}
    onClose={() => (isPresentationOpen = false)}
  />

  <!-- Sleek Status Bar in Card Footer -->
  <footer class="h-7 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-100 dark:bg-[#111111] px-3 sm:px-4 flex items-center justify-between text-[11px] font-mono text-neutral-500 dark:text-neutral-400 z-10 shrink-0 select-none no-print">
    <div class="flex items-center gap-3 sm:gap-4 truncate">
      <span>{t('editor.words')}: <strong class="text-neutral-800 dark:text-neutral-200">{wordCount}</strong></span>
      <span class="hidden sm:inline">{t('editor.chars')}: <strong class="text-neutral-800 dark:text-neutral-200">{charCount}</strong></span>
      <span>{t('editor.lines')}: <strong class="text-neutral-800 dark:text-neutral-200">{lineCount}</strong></span>
      <span class="hidden md:inline">{t('editor.readTime')}: ~{readingTimeMin} {t('editor.minutes')}</span>
    </div>

    <div class="flex items-center gap-3 shrink-0">
      <span class="px-2 py-0.5 rounded text-[10px] bg-neutral-200 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300">
        {isVaultMode ? t('editor.vaultStorageLabel') : t('editor.fileStorageLabel')}
      </span>
      <span class="hidden sm:inline">UTF-8</span>
      <span class="flex items-center gap-1.5" title="Local-First Storage Engine">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_5px_rgba(16,185,129,0.8)]"></span>
        <span class="text-emerald-600 dark:text-emerald-400 font-medium">Local-First</span>
      </span>
    </div>
  </footer>

  <!-- Custom Floating Tab Context Menu -->
  {#if tabContextMenu.open && tabContextMenu.tab}
    <!-- Backdrop for dismissing -->
    <div
      role="presentation"
      class="fixed inset-0 z-50 bg-transparent cursor-default"
      onclick={closeTabContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeTabContextMenu(); }}
    ></div>

    <!-- Menu Panel (CAUI Theme) -->
    <div
      role="menu"
      tabindex="-1"
      class="fixed z-50 w-52 py-1 bg-neutral-900 border border-neutral-700/80 rounded-xl shadow-2xl shadow-black/80 text-xs font-sans text-neutral-300 select-none animate-in fade-in duration-75 overflow-hidden"
      style="left: {tabContextMenu.x}px; top: {tabContextMenu.y}px;"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => { if (e.key === 'Escape') closeTabContextMenu(); }}
    >
      <button
        type="button"
        role="menuitem"
        onclick={() => handleContextMenuAction('close')}
        class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
      >
        <span>{t('editor.closeTab')}</span>
        <span class="text-[10px] text-neutral-500 font-mono">Ctrl+W</span>
      </button>

      {#if tabs.length > 1}
        <button
          type="button"
          role="menuitem"
          onclick={() => handleContextMenuAction('closeOthers')}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.closeOtherTabs')}</span>
        </button>

        <button
          type="button"
          role="menuitem"
          onclick={() => handleContextMenuAction('closeRight')}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.closeTabsToRight')}</span>
        </button>
      {/if}

      <button
        type="button"
        role="menuitem"
        onclick={() => handleContextMenuAction('closeAll')}
        class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-rose-500/15 hover:text-rose-400 transition-colors text-rose-300"
      >
        <span>{t('editor.closeAllTabs')}</span>
      </button>

      <div class="h-px bg-neutral-800 my-1"></div>

      <button
        type="button"
        role="menuitem"
        onclick={() => handleContextMenuAction('copyPath')}
        class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
      >
        <span>{t('editor.copyPath')}</span>
      </button>

      <button
        type="button"
        role="menuitem"
        onclick={() => handleContextMenuAction('copyName')}
        class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
      >
        <span>{t('editor.copyName')}</span>
      </button>

      {#if !tabContextMenu.tab.isVault}
        <button
          type="button"
          role="menuitem"
          onclick={() => handleContextMenuAction('reveal')}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.revealInFileManager')}</span>
        </button>
      {/if}
    </div>
  {/if}

  <!-- Files & Folders Right-Click Context Menu -->
  {#if fileContextMenu.open}
    <!-- Backdrop for dismissing -->
    <div
      role="presentation"
      class="fixed inset-0 z-50 bg-transparent cursor-default"
      onclick={closeFileContextMenu}
      oncontextmenu={(e) => { e.preventDefault(); closeFileContextMenu(); }}
    ></div>

    <!-- Menu Panel (CAUI Theme) -->
    <div
      role="menu"
      tabindex="-1"
      class="fixed z-50 w-56 py-1 bg-neutral-900 border border-neutral-700/80 rounded-xl shadow-2xl shadow-black/80 text-xs font-sans text-neutral-300 select-none animate-in fade-in duration-75 overflow-hidden"
      style="left: {fileContextMenu.x}px; top: {fileContextMenu.y}px;"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => { if (e.key === 'Escape') closeFileContextMenu(); }}
    >
      {#if fileContextMenu.node}
        {@const node = fileContextMenu.node}
        <div class="px-3 py-1.5 text-[11px] font-mono font-semibold text-neutral-400 border-b border-neutral-800 truncate flex items-center gap-1.5">
          {#if node.is_dir}
            <svg class="w-3.5 h-3.5 text-amber-500 shrink-0" fill="currentColor" viewBox="0 0 24 24"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" /></svg>
          {:else}
            <svg class="w-3.5 h-3.5 text-cyan-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
          {/if}
          <span class="truncate">{node.name}</span>
        </div>

        {#if node.is_dir}
          <button
            type="button"
            role="menuitem"
            onclick={() => triggerNewFile(node.path)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 text-cyan-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
              <span>{t('editor.newFile')}</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">+ File</span>
          </button>

          <button
            type="button"
            role="menuitem"
            onclick={() => triggerNewFolder(node.path)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-amber-500/15 hover:text-amber-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" /></svg>
              <span>{t('editor.newFolder')}</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">+ Folder</span>
          </button>

          <div class="h-px bg-neutral-800 my-1"></div>
        {:else}
          <button
            type="button"
            role="menuitem"
            onclick={() => { closeFileContextMenu(); openFile(node); }}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
              <span>Open</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">Enter</span>
          </button>

          <div class="h-px bg-neutral-800 my-1"></div>
        {/if}

        <button
          type="button"
          role="menuitem"
          onclick={() => handleCut(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879a3 3 0 11-4.242-4.242 3 3 0 014.242 0z" /></svg>
            <span>{t('editor.cut')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">Ctrl+X</span>
        </button>

        <button
          type="button"
          role="menuitem"
          onclick={() => handleCopy(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
            <span>{t('editor.copy')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">Ctrl+C</span>
        </button>

        {#if node.is_dir}
          <button
            type="button"
            role="menuitem"
            disabled={!fileClipboard}
            onclick={() => handlePaste(node.path)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left disabled:opacity-30 disabled:pointer-events-none hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" /></svg>
              <span>{t('editor.paste')}</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">Ctrl+V</span>
          </button>
        {/if}

        <button
          type="button"
          role="menuitem"
          onclick={() => handleDuplicate(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2" /></svg>
            <span>{t('editor.duplicate')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">Ctrl+D</span>
        </button>

        <div class="h-px bg-neutral-800 my-1"></div>

        <button
          type="button"
          role="menuitem"
          onclick={() => triggerRename(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
            <span>{t('editor.rename')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">F2</span>
        </button>

        <button
          type="button"
          role="menuitem"
          onclick={() => triggerDelete(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-rose-500/15 hover:text-rose-400 text-rose-300 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5 text-rose-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
            <span>{t('editor.delete')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">Del</span>
        </button>

        <div class="h-px bg-neutral-800 my-1"></div>

        <button
          type="button"
          role="menuitem"
          onclick={() => handleCopyFullPath(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.copyFullPath')}</span>
        </button>

        <button
          type="button"
          role="menuitem"
          onclick={() => handleCopyRelativePath(node)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.copyRelativePath')}</span>
        </button>

        {#if !node.is_dir}
          <button
            type="button"
            role="menuitem"
            onclick={() => handleSendToVault(node)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-amber-500/15 hover:text-amber-400 text-amber-300 transition-colors"
          >
            <span class="flex items-center gap-2">
              <span>🔒</span>
              <span>{t('editor.sendToVault')}</span>
            </span>
          </button>

          <button
            type="button"
            role="menuitem"
            onclick={() => handleSendToHanaAi(node)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-rose-500/15 hover:text-rose-400 text-rose-300 transition-colors"
          >
            <span class="flex items-center gap-2">
              <span>🌸</span>
              <span>{t('editor.sendToAiChat')}</span>
            </span>
          </button>
        {/if}

        <button
          type="button"
          role="menuitem"
          onclick={() => handleRevealFile(node.path)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span>{t('editor.revealInFileManager')}</span>
        </button>
      {:else}
        <!-- Background Context Menu (Clicked on empty sidebar space) -->
        <button
          type="button"
          role="menuitem"
          onclick={() => triggerNewFile(workspacePath)}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5 text-cyan-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
            <span>{t('editor.newFile')}</span>
          </span>
          <span class="text-[10px] text-neutral-500 font-mono">+ File</span>
        </button>

        {#if !isVaultMode}
          <button
            type="button"
            role="menuitem"
            onclick={() => triggerNewFolder(workspacePath)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-amber-500/15 hover:text-amber-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" /></svg>
              <span>{t('editor.newFolder')}</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">+ Folder</span>
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!fileClipboard}
            onclick={() => handlePaste(workspacePath)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left disabled:opacity-30 disabled:pointer-events-none hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" /></svg>
              <span>{t('editor.paste')}</span>
            </span>
            <span class="text-[10px] text-neutral-500 font-mono">Ctrl+V</span>
          </button>
        {/if}

        <div class="h-px bg-neutral-800 my-1"></div>

        <button
          type="button"
          role="menuitem"
          onclick={handleRefreshWorkspace}
          class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
        >
          <span class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5 text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
            <span>{t('editor.refresh')}</span>
          </span>
        </button>

        {#if !isVaultMode}
          <button
            type="button"
            role="menuitem"
            onclick={() => handleRevealFile(workspacePath)}
            class="w-full px-3 py-1.5 flex items-center justify-between text-left hover:bg-cyan-500/15 hover:text-cyan-400 transition-colors"
          >
            <span>{t('editor.revealInFileManager')}</span>
          </button>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- New File / Folder Modal -->
  {#if fileModal.open}
    <div
      role="presentation"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 select-none animate-in fade-in duration-100"
      onclick={(e) => {
        if (e.target === e.currentTarget) fileModal.open = false;
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        class="w-full max-w-sm rounded-2xl bg-neutral-900 border border-neutral-700/80 p-5 shadow-2xl shadow-black text-neutral-200"
      >
        <div class="flex items-center gap-2 mb-2 {fileModal.type === 'folder' ? 'text-amber-400' : 'text-cyan-400'}">
          {#if fileModal.type === 'folder'}
            <svg class="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 24 24"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" /></svg>
            <h2 class="text-sm font-bold text-white tracking-wide">{t('editor.newFolderModalTitle')}</h2>
          {:else}
            <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
            <h2 class="text-sm font-bold text-white tracking-wide">{t('editor.newFileModalTitle')}</h2>
          {/if}
        </div>

        <p class="text-xs text-neutral-400 mb-3 truncate">
          {fileModal.targetDir === 'vault' ? 'Secret SQLCipher Vault' : fileModal.targetDir}
        </p>

        <form
          onsubmit={(e) => {
            e.preventDefault();
            submitCreateFileOrFolder();
          }}
          class="space-y-3"
        >
          <div>
            <Input
              type="text"
              bind:value={fileModal.name}
              placeholder={t('editor.namePlaceholder')}
              class="w-full font-mono text-xs"
              autofocus
            />
          </div>

          {#if fileModal.error}
            <div class="text-[11px] text-rose-400 font-mono">
              {fileModal.error}
            </div>
          {/if}

          <div class="flex items-center justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onclick={() => fileModal.open = false}
              class="text-neutral-400 hover:text-white"
            >
              {t('editor.cancelBtn')}
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              class="{fileModal.type === 'folder' ? '!bg-amber-600 hover:!bg-amber-500' : '!bg-cyan-600 hover:!bg-cyan-500'} text-white"
            >
              {t('editor.createBtn')}
            </Button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Rename Modal -->
  {#if renameModal.open && renameModal.node}
    <div
      role="presentation"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 select-none animate-in fade-in duration-100"
      onclick={(e) => {
        if (e.target === e.currentTarget) renameModal.open = false;
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        class="w-full max-w-sm rounded-2xl bg-neutral-900 border border-neutral-700/80 p-5 shadow-2xl shadow-black text-neutral-200"
      >
        <div class="flex items-center gap-2 mb-2 text-cyan-400">
          <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
          <h2 class="text-sm font-bold text-white tracking-wide">
            {t('editor.renameModalTitle', { type: renameModal.node.is_dir ? 'Folder' : 'File' })}
          </h2>
        </div>

        <p class="text-xs text-neutral-400 mb-3 truncate">
          {renameModal.node.path}
        </p>

        <form
          onsubmit={(e) => {
            e.preventDefault();
            submitRename();
          }}
          class="space-y-3"
        >
          <div>
            <label for="rename-input" class="block text-[11px] text-neutral-400 mb-1">{t('editor.newNameLabel')}</label>
            <Input
              id="rename-input"
              type="text"
              bind:value={renameModal.name}
              placeholder={t('editor.namePlaceholder')}
              class="w-full font-mono text-xs"
              autofocus
            />
          </div>

          {#if renameModal.error}
            <div class="text-[11px] text-rose-400 font-mono">
              {renameModal.error}
            </div>
          {/if}

          <div class="flex items-center justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onclick={() => renameModal.open = false}
              class="text-neutral-400 hover:text-white"
            >
              {t('editor.cancelBtn')}
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              class="!bg-cyan-600 hover:!bg-cyan-500 text-white"
            >
              {t('editor.renameBtn')}
            </Button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Delete Confirmation Modal -->
  {#if deleteModal.open && deleteModal.node}
    <div
      role="presentation"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 select-none animate-in fade-in duration-100"
      onclick={(e) => {
        if (e.target === e.currentTarget && !deleteModal.loading) deleteModal.open = false;
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        class="w-full max-w-sm rounded-2xl bg-neutral-900 border border-neutral-700/80 p-5 shadow-2xl shadow-black text-neutral-200"
      >
        <div class="flex items-center gap-2 mb-2 text-rose-400">
          <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
          <h2 class="text-sm font-bold text-white tracking-wide">
            {t('editor.deleteModalTitle', { type: deleteModal.node.is_dir ? 'Folder' : 'File' })}
          </h2>
        </div>

        <p class="text-xs text-neutral-300 mb-2 leading-relaxed">
          {t('editor.deleteModalDesc', { name: deleteModal.node.name })}
        </p>

        <div class="p-2.5 rounded-lg bg-neutral-950/80 border border-neutral-800 text-[11px] font-mono text-neutral-400 mb-4 truncate">
          {deleteModal.node.path}
        </div>

        <div class="flex items-center justify-end gap-2">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            disabled={deleteModal.loading}
            onclick={() => deleteModal.open = false}
            class="text-neutral-400 hover:text-white"
          >
            {t('editor.cancelBtn')}
          </Button>
          <Button
            type="button"
            variant="danger"
            size="sm"
            disabled={deleteModal.loading}
            loading={deleteModal.loading}
            onclick={submitDelete}
            class="!bg-rose-600 hover:!bg-rose-500 text-white"
          >
            {t('editor.deleteBtn')}
          </Button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Secret Vault Setup / Unlock Password Modal -->
  {#if vaultModalOpen}
    <div
      role="presentation"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 select-none animate-in fade-in duration-100"
      onclick={(e) => {
        if (e.target === e.currentTarget) closeVaultModal();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="vault-modal-title"
        class="w-full max-w-sm rounded-2xl bg-neutral-900 border border-neutral-700/80 p-5 shadow-2xl shadow-black text-neutral-200"
      >
        <div class="flex items-center gap-2 mb-2 text-amber-400">
          <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          <h2 id="vault-modal-title" class="text-sm font-bold text-white tracking-wide">
            {isVaultSetup ? t('editor.setupVaultTitle') : t('editor.unlockVaultTitle')}
          </h2>
        </div>

        <p class="text-xs text-neutral-400 mb-4 leading-relaxed">
          {isVaultSetup ? t('editor.setupVaultDesc') : t('editor.unlockVaultDesc')}
        </p>

        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleVaultPasswordSubmit();
          }}
          class="space-y-3"
        >
          <div>
            <Input
              type="password"
              bind:value={vaultPassword}
              placeholder={t('editor.masterPasswordPlaceholder')}
              class="w-full font-mono text-xs"
            />
          </div>

          {#if isVaultSetup}
            <div>
              <Input
                type="password"
                bind:value={vaultConfirmPassword}
                placeholder={t('editor.confirmPasswordPlaceholder')}
                class="w-full font-mono text-xs"
              />
            </div>
          {/if}

          {#if vaultPasswordError}
            <div class="text-[11px] text-rose-400 font-mono">
              {vaultPasswordError}
            </div>
          {/if}

          <div class="flex items-center justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onclick={closeVaultModal}
              class="text-neutral-400 hover:text-white"
            >
              {t('editor.cancelBtn')}
            </Button>
            <Button
              type="submit"
              variant="warning"
              size="sm"
              disabled={isVaultModalLoading || !vaultPassword.trim()}
              loading={isVaultModalLoading}
              class="bg-amber-500 hover:bg-amber-400 text-neutral-950 font-semibold"
            >
              {isVaultSetup ? t('editor.setupVaultBtn') : t('editor.unlockVaultBtn')}
            </Button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(.cm-editor) {
    height: 100% !important;
    max-height: 100% !important;
    background-color: transparent !important;
    font-size: 13px !important;
  }
  :global(.cm-editor.cm-focused) {
    outline: none !important;
  }
  :global(.cm-scroller) {
    height: 100% !important;
    max-height: 100% !important;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace !important;
    line-height: 1.65 !important;
    overflow: auto !important;
  }
  :global(.cm-gutters) {
    background-color: transparent !important;
    border-right: 1px solid rgba(255, 255, 255, 0.08) !important;
    color: #71717a !important;
  }
  :global(.dark .cm-gutters) {
    border-right-color: rgba(255, 255, 255, 0.08) !important;
    color: #71717a !important;
  }
  :global(:not(.dark) .cm-gutters) {
    border-right-color: rgba(0, 0, 0, 0.08) !important;
    color: #a1a1aa !important;
  }
  :global(.cm-lineNumbers .cm-gutterElement) {
    padding: 0 10px 0 6px !important;
    min-width: 28px !important;
    text-align: right !important;
    line-height: 1.65 !important;
  }
  :global(.cm-foldGutter .cm-gutterElement) {
    padding: 0 4px !important;
    line-height: 1.65 !important;
  }
  :global(.cm-content) {
    padding: 8px 14px !important;
    line-height: 1.65 !important;
  }
  :global(.cm-line) {
    line-height: 1.65 !important;
  }

  /* Live Smart WYSIWYG Mode Canvas: 100% Matches Preview Typography */
  :global(.cm-live-mode) {
    font-size: 0.95rem !important;
    background: transparent !important;
  }
  :global(.cm-live-mode .cm-editor) {
    background: transparent !important;
    background-color: transparent !important;
  }
  :global(.cm-live-mode .cm-gutters) {
    display: none !important;
    width: 0 !important;
    border: none !important;
    background: transparent !important;
  }
  :global(.cm-live-mode .cm-activeLine) {
    background: transparent !important;
    background-color: transparent !important;
  }
  :global(.cm-live-mode .cm-scroller) {
    height: 100% !important;
    max-height: 100% !important;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif !important;
    font-size: 0.95rem !important;
    line-height: 1.75 !important;
    overflow: auto !important;
    background: transparent !important;
  }
  :global(.cm-live-mode .cm-content) {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif !important;
    font-size: 0.95rem !important;
    line-height: 1.75 !important;
    padding: 2.5rem 2.5rem 8rem 2.5rem !important;
    max-width: 54rem !important;
    margin: 0 auto !important;
    background: transparent !important;
  }
  :global(html.dark .cm-live-mode .cm-content) {
    color: #d4d4d8 !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-content) {
    color: #18181b !important;
  }
  :global(.cm-live-mode .cm-line) {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif !important;
    line-height: 1.75 !important;
    padding: 2px 0 !important;
  }

  /* Live Smart Markdown Tokens (Smooth inline hiding and reveal on line cursor) */
  :global(.cm-live-token-hidden) {
    display: inline-block !important;
    width: 0 !important;
    max-width: 0 !important;
    font-size: 0 !important;
    opacity: 0 !important;
    overflow: hidden !important;
    vertical-align: middle !important;
    user-select: none !important;
  }
  :global(.cm-live-token-active) {
    opacity: 0.45 !important;
    font-size: 0.85em !important;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
    color: #06b6d4 !important;
  }

  /* Scalable Vector SVG Graphic Canvas Preview */
  :global(.svg-graphic-wrapper svg) {
    max-width: 100% !important;
    max-height: 480px !important;
    width: auto !important;
    height: auto !important;
    display: block !important;
  }

  /* Live Smart Headings (Identical to Preview H1, H2, H3, H4) */
  :global(.cm-live-mode .cm-live-h1) {
    font-size: 1.85rem !important;
    font-weight: 700 !important;
    line-height: 1.3 !important;
    letter-spacing: -0.02em !important;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
    padding-bottom: 0.5rem !important;
    padding-top: 1.25rem !important;
  }
  :global(html.dark .cm-live-mode .cm-live-h1) {
    color: #fafafa !important;
    border-bottom-color: rgba(255, 255, 255, 0.08) !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-h1) {
    color: #09090b !important;
    border-bottom-color: rgba(0, 0, 0, 0.08) !important;
  }

  :global(.cm-live-mode .cm-live-h2) {
    font-size: 1.45rem !important;
    font-weight: 700 !important;
    line-height: 1.3 !important;
    letter-spacing: -0.02em !important;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05) !important;
    padding-bottom: 0.35rem !important;
    padding-top: 1rem !important;
  }
  :global(html.dark .cm-live-mode .cm-live-h2) {
    color: #fafafa !important;
    border-bottom-color: rgba(255, 255, 255, 0.05) !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-h2) {
    color: #09090b !important;
    border-bottom-color: rgba(0, 0, 0, 0.05) !important;
  }

  :global(.cm-live-mode .cm-live-h3) {
    font-size: 1.25rem !important;
    font-weight: 700 !important;
    line-height: 1.3 !important;
    letter-spacing: -0.02em !important;
    padding-top: 0.75rem !important;
  }
  :global(html.dark .cm-live-mode .cm-live-h3) {
    color: #fafafa !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-h3) {
    color: #09090b !important;
  }

  :global(.cm-live-mode .cm-live-h4) {
    font-size: 1.05rem !important;
    font-weight: 700 !important;
    letter-spacing: -0.02em !important;
    padding-top: 0.5rem !important;
  }

  /* Bold, Italic, Code, Blockquote: Identical to Preview */
  :global(.cm-live-mode .cm-live-bold) {
    font-weight: 600 !important;
  }
  :global(html.dark .cm-live-mode .cm-live-bold) {
    color: #fafafa !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-bold) {
    color: #09090b !important;
  }

  :global(.cm-live-mode .cm-live-italic) {
    font-style: italic !important;
  }
  :global(html.dark .cm-live-mode .cm-live-italic) {
    color: #f4f4f5 !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-italic) {
    color: #27272a !important;
  }

  :global(.cm-live-mode .cm-live-inlinecode) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace !important;
    font-size: 0.85em !important;
    border-radius: 4px !important;
    padding: 0.15rem 0.35rem !important;
  }
  :global(html.dark .cm-live-mode .cm-live-inlinecode) {
    background-color: rgba(255, 255, 255, 0.08) !important;
    color: #22d3ee !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-inlinecode) {
    background-color: #f4f4f5 !important;
    color: #0891b2 !important;
  }

  :global(.cm-live-mode .cm-live-strike) {
    text-decoration: line-through !important;
    opacity: 0.65 !important;
  }

  :global(.cm-live-mode .cm-live-quote) {
    border-left: 4px solid #06b6d4 !important;
    padding-left: 1rem !important;
    border-radius: 0 6px 6px 0 !important;
    font-style: italic !important;
  }
  :global(html.dark .cm-live-mode .cm-live-quote) {
    background: rgba(6, 182, 212, 0.06) !important;
    color: #a1a1aa !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-quote) {
    background: rgba(6, 182, 212, 0.03) !important;
    color: #52525b !important;
  }

  :global(.cm-live-mode .cm-live-listmark) {
    color: #06b6d4 !important;
    font-weight: 700 !important;
    margin-right: 0.25rem !important;
  }

  :global(.cm-live-mode .cm-live-list-item) {
    padding-left: 1.5rem !important;
    text-indent: -1.25rem !important;
  }
  :global(.cm-live-mode .cm-live-list-item-indent-1) {
    padding-left: 2.75rem !important;
    text-indent: -1.25rem !important;
  }
  :global(.cm-live-mode .cm-live-list-item-indent-2) {
    padding-left: 4rem !important;
    text-indent: -1.25rem !important;
  }

  :global(.cm-live-mode .cm-live-bullet) {
    font-size: 0 !important;
    color: transparent !important;
    display: inline-block !important;
    width: 1.25rem !important;
    vertical-align: middle !important;
    user-select: none !important;
  }
  :global(.cm-live-mode .cm-live-bullet::before) {
    content: "•" !important;
    font-size: 1.25rem !important;
    color: #06b6d4 !important;
    font-weight: 700 !important;
    display: inline-block !important;
    line-height: 1 !important;
  }

  :global(.cm-live-mode .cm-live-ascii) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace !important;
    font-size: 0.88em !important;
    line-height: 1.25 !important;
    letter-spacing: 0 !important;
    white-space: pre !important;
  }

  :global(.cm-live-mode .cm-live-link) {
    color: #22d3ee !important;
    text-decoration: underline !important;
    text-underline-offset: 3px !important;
    cursor: pointer !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-link) {
    color: #0891b2 !important;
  }

  :global(.cm-live-mode .cm-live-codeblock) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
    font-size: 0.88em !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  :global(html.dark .cm-live-mode .cm-live-codeblock) {
    background: rgba(0, 0, 0, 0.35) !important;
    color: #93c5fd !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-codeblock) {
    background: #f1f5f9 !important;
    color: #1e293b !important;
  }

  :global(.cm-live-mode .cm-live-hr) {
    border-top: 1px solid rgba(255, 255, 255, 0.12) !important;
    padding-top: 0.75rem !important;
    margin-top: 0.75rem !important;
  }
  :global(html:not(.dark) .cm-live-mode .cm-live-hr) {
    border-top-color: rgba(0, 0, 0, 0.1) !important;
  }
</style>
