<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { EditorView, basicSetup } from 'codemirror';
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
    type DocTab,
  } from '$lib/stores/docTabs.svelte';

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

  // Multi-document tabs reactive state
  const tabs = $derived(getDocTabs());
  const activeTabId = $derived(getActiveTabId());
  const activeTab = $derived(getActiveTab());
  const pro = getPro();

  // View mode: 'editor' | 'split' | 'preview' (synchronized via docTabs store)
  let viewMode = $state<'editor' | 'split' | 'preview'>(getViewMode());
  let isSaving = $state<boolean>(false);

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

  $effect(() => {
    if (editorView) {
      editorView.dispatch({
        effects: lineWrappingCompartment.reconfigure(isZenMode ? EditorView.lineWrapping : []),
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
    return html.replace(/^<h(\d)/, `<h$1 id="${slug}">`);
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
      viewMode = 'editor';
      showFileTree = false;
      showToast(t('editor.zenModeActive'), 'info');
    } else {
      showFileTree = true;
      viewMode = 'preview';
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
        scrollIntoView: true,
      });
      editorView.focus();
    }
    if (previewElement) {
      const allHeadings = Array.from(previewElement.querySelectorAll('h1, h2, h3, h4, h5, h6'));
      const target = allHeadings.find((el) => {
        const text = (el.textContent || '').trim().toLowerCase();
        return text.includes(heading.anchor.replace(/-/g, ' '));
      });
      if (target) {
        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    }
  }

  onMount(() => {
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
    };
    window.addEventListener('keydown', handleKeyDown);

    // Restore saved tabs from local session storage (run first, before editor init)
    restoreTabs();
    viewMode = getViewMode();
    const restored = getActiveTab();

    // Initialize CodeMirror 6 with oneDark theme and update listeners
    if (editorElement) {
      const view = new EditorView({
        doc: rawMarkdown,
        extensions: [
          basicSetup,
          languageCompartment.of(markdown()),
          oneDark,
          lineWrappingCompartment.of(isZenMode ? EditorView.lineWrapping : []),
          EditorView.domEventHandlers({
            paste(event) {
              return handleEditorPaste(event);
            },
          }),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              rawMarkdown = update.state.doc.toString();
              renderMarkdown(rawMarkdown);
              if (activeTabId) setDirty(activeTabId, true);
              autoSave();
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

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  async function renderMarkdown(content: string) {
    try {
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
      } catch (e) {
        console.error('Failed reading tab file:', e);
      }
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
  }

  async function createNewFile() {
    if (isVaultMode) {
      if (!isVaultUnlocked) {
        await enterVaultMode();
        return;
      }
      const nextIndex = vaultDocs.length + 1;
      const defaultVaultTitle = `files-${nextIndex}.md`;
      let title = prompt(t('editor.vaultNotePrompt'), defaultVaultTitle);
      if (!title) return;
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
        showToast(t('editor.vaultNoteCreated', { title }), 'success');
      } catch (e) {
        alert(`${t('editor.vaultNoteFailed')}${e}`);
      }
      return;
    }

    const fileName = prompt(t('editor.createNewFilePrompt'), t('editor.defaultNewFileName'));
    if (!fileName) return;
    const targetPath = workspacePath === '.' ? fileName : `${workspacePath}/${fileName}`;
    try {
      await invoke('workspace_create_file', { filePath: targetPath });
      await loadDirectory(workspacePath);
      openFile({ name: fileName, path: targetPath, is_dir: false });
      showToast(t('editor.fileCreated', { name: fileName }), 'success');
    } catch (e) {
      alert(`${t('editor.fileFailed')}${e}`);
    }
  }

  async function createNewDirectory() {
    if (isVaultMode) {
      showToast(t('editor.createDirFilesOnly'), 'info');
      return;
    }
    const dirName = prompt(t('editor.createNewFolderPrompt'), t('editor.defaultNewFolderName'));
    if (!dirName) return;
    const targetPath = workspacePath === '.' ? dirName : `${workspacePath}/${dirName}`;
    try {
      await invoke('workspace_create_directory', { dirPath: targetPath });
      await loadDirectory(workspacePath);
      const next = new Set(expandedDirs);
      next.add(targetPath);
      expandedDirs = next;
      showToast(t('editor.folderCreated', { name: dirName }), 'success');
    } catch (e) {
      alert(`${t('editor.folderFailed')}${e}`);
    }
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

    <!-- Center/Fill: Document Tabs Strip -->
    <div class="flex-1 min-w-0 flex items-center gap-1 overflow-x-auto scrollbar-none h-full pt-1">
      {#each tabs as tab (tab.id)}
        {@const isActive = tab.id === activeTabId}
        <div
          role="button"
          tabindex="0"
          onclick={() => handleSelectTab(tab.id)}
          oncontextmenu={(e) => handleTabContextMenu(e, tab)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelectTab(tab.id); }}
          class="group relative h-8 px-3 rounded-t-md flex items-center gap-2 text-xs font-mono transition-all cursor-pointer border-t border-x shrink-0 {isActive ? 'bg-white dark:bg-[#1a1a1a] text-cyan-600 dark:text-cyan-400 border-neutral-300 dark:border-neutral-700/80 shadow-2xs font-semibold' : 'bg-transparent border-transparent text-neutral-500 hover:text-neutral-800 dark:text-neutral-400 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40'}"
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

          <span class="truncate max-w-[120px] sm:max-w-[180px]">{tab.title}</span>

          {#if tab.isDirty}
            <span class="w-1.5 h-1.5 rounded-full bg-cyan-500 shrink-0" title={t('editor.unsavedChanges')}></span>
          {/if}

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

      <button
        type="button"
        onclick={handleNewTab}
        class="h-7 w-7 rounded-md flex items-center justify-center text-neutral-400 hover:text-neutral-700 dark:hover:text-white hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 transition-colors shrink-0"
        title={t('editor.openNewDocShortcut')}
        aria-label={t('editor.newDocument')}
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>

    <!-- Right: Exporters & Mode Switcher -->
    <div class="flex items-center gap-2 shrink-0">
      {#if isSaving}
        <span class="text-[10px] px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-600 dark:text-cyan-400 border border-cyan-500/20 shrink-0 font-mono animate-pulse">
          {t('editor.saving')}
        </span>
      {/if}

      <!-- PDF Print Exporter -->
      <Button
        variant="secondary"
        size="sm"
        onclick={exportPdf}
        class="flex items-center gap-1.5"
        title={t('editor.printPdf')}
      >
        <svg class="w-3.5 h-3.5 text-rose-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 9V2h12v7"></path>
          <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"></path>
          <path d="M6 14h12v8H6z"></path>
        </svg>
        <span class="hidden sm:inline">PDF</span>
      </Button>

      <!-- HTML Exporter -->
      <Button
        variant="secondary"
        size="sm"
        onclick={exportHtml}
        class="flex items-center gap-1.5"
        title={t('editor.exportHtml')}
      >
        <svg class="w-3.5 h-3.5 text-cyan-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        <span class="hidden sm:inline">HTML</span>
      </Button>

      <!-- Editor / Split / Preview View Switcher -->
      <div class="flex items-center p-0.5 bg-neutral-200/70 dark:bg-neutral-900 rounded-lg border border-neutral-300 dark:border-neutral-800 text-xs">
        <button
          onclick={() => { viewMode = 'editor'; setViewMode('editor'); }}
          class="px-2.5 py-1 rounded-md transition-all {viewMode === 'editor' ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-2xs' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >
          {t('editor.editorMode')}
        </button>
        <button
          onclick={() => { viewMode = 'split'; setViewMode('split'); }}
          class="px-2.5 py-1 rounded-md transition-all {viewMode === 'split' ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-2xs' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >
          {t('editor.splitMode')}
        </button>
        <button
          onclick={() => { viewMode = 'preview'; setViewMode('preview'); }}
          class="px-2.5 py-1 rounded-md transition-all {viewMode === 'preview' ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-medium shadow-2xs' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >
          {t('editor.previewMode')}
        </button>
      </div>
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

      <!-- Action Sub-header: Title & New File/Folder Buttons -->
      <div class="p-2.5 px-3 border-b border-neutral-200 dark:border-neutral-800/80 flex items-center justify-between">
        <span class="text-[10px] font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          {isVaultMode ? t('editor.vaultNotes') : t('editor.filesAndFolders')}
        </span>
        <div class="flex items-center gap-1">
          {#if !isVaultMode}
            <button
              onclick={createNewDirectory}
              class="p-1 rounded-md hover:bg-neutral-200 dark:hover:bg-neutral-800 text-amber-600 dark:text-amber-400 transition-colors"
              title={t('editor.newFolder')}
              aria-label={t('editor.newFolder')}
            >
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
              </svg>
            </button>
          {/if}
          <button
            onclick={createNewFile}
            class="p-1 rounded-md hover:bg-neutral-200 dark:hover:bg-neutral-800 {isVaultMode ? 'text-amber-600 dark:text-amber-400' : 'text-cyan-600 dark:text-cyan-400'} transition-colors"
            title={isVaultMode ? t('editor.createNewVaultNoteTitle') : t('editor.createNewFileTitle')}
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
      <div class="flex-1 overflow-y-auto p-2 space-y-0.5 text-xs font-mono scrollbar-none">
        {#if isVaultMode}
          {#each vaultDocs as doc (doc.id)}
            <button
              onclick={() => openVaultDoc(doc)}
              class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left transition-colors {currentFilePath === `vault://${doc.id}` ? 'bg-amber-500/15 text-amber-600 dark:text-amber-300 font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60 hover:text-neutral-900 dark:hover:text-white'}"
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
              {expandedDirs}
              onOpenFile={openFile}
              onToggleDir={toggleDir}
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
      class="flex-1 flex flex-col bg-neutral-50 dark:bg-[#0f141c] overflow-hidden relative {viewMode === 'split' ? 'border-r border-neutral-200 dark:border-neutral-800' : ''} {viewMode === 'preview' ? 'hidden' : 'flex'}"
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

      <!-- CodeMirror Editor Viewport -->
      <div
        bind:this={editorElement}
        class="flex-1 overflow-auto font-mono text-sm leading-relaxed p-2 {isZenMode ? 'max-w-3xl mx-auto w-full pt-12 pb-32' : ''}"
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
      class="preview-scroll-accel flex-1 overflow-y-auto bg-white dark:bg-[#0c1017] p-8 sm:p-10 markdown-preview max-w-none select-text overscroll-contain will-change-[scroll-position] bg-[linear-gradient(to_right,#00000006_1px,transparent_1px),linear-gradient(to_bottom,#00000006_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px),linear-gradient(to_bottom,#ffffff05_1px,transparent_1px)] bg-[size:56px_56px] {viewMode === 'editor' || isZenMode ? 'hidden' : 'block'}"
    >
      {@html renderedHtml}
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
    background-color: transparent !important;
  }
  :global(.cm-scroller) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace !important;
    font-size: 13px !important;
    line-height: 1.65 !important;
  }
</style>
