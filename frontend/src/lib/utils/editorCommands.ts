/**
 * Editor commands for CAMark (Phase 1: E-07 hotkeys, E-08 find, E-09 slash commands,
 * K-05 `[[` autocomplete). Spec builders are pure (EditorState in, TransactionSpec out)
 * so they can be unit-tested without a DOM.
 */
import { EditorSelection, type EditorState, type TransactionSpec } from '@codemirror/state';
import type { EditorView, KeyBinding } from '@codemirror/view';
import { indentWithTab } from '@codemirror/commands';
import { searchKeymap } from '@codemirror/search';
import type { CompletionContext, CompletionResult, Completion } from '@codemirror/autocomplete';

/** Wrap / unwrap each selection range. Toggling twice restores the original text. */
export function toggleWrapSpec(
  state: EditorState,
  prefix: string,
  suffix: string = prefix,
  placeholder = 'teks'
): TransactionSpec {
  return state.changeByRange((range) => {
    const text = state.sliceDoc(range.from, range.to);
    const before = state.sliceDoc(Math.max(0, range.from - prefix.length), range.from);
    const after = state.sliceDoc(range.to, Math.min(state.doc.length, range.to + suffix.length));

    // A lone `*` must not unwrap the inner half of `**bold**`.
    const edgeChar = (pos: number) => state.sliceDoc(Math.max(0, pos), pos + 1);
    const ambiguous =
      prefix.length === 1 &&
      prefix === suffix &&
      (edgeChar(range.from - prefix.length - 1) === prefix || edgeChar(range.to + suffix.length) === suffix);

    if (before === prefix && after === suffix && !ambiguous) {
      return {
        changes: [
          { from: range.from - prefix.length, to: range.from },
          { from: range.to, to: range.to + suffix.length },
        ],
        range: EditorSelection.range(range.from - prefix.length, range.to - prefix.length),
      };
    }
    if (
      text.length >= prefix.length + suffix.length &&
      text.startsWith(prefix) &&
      text.endsWith(suffix) &&
      !(prefix === '*' && text.startsWith('**'))
    ) {
      const inner = text.slice(prefix.length, text.length - suffix.length);
      return {
        changes: { from: range.from, to: range.to, insert: inner },
        range: EditorSelection.range(range.from, range.from + inner.length),
      };
    }
    const inner = text || placeholder;
    return {
      changes: { from: range.from, to: range.to, insert: prefix + inner + suffix },
      range: EditorSelection.range(range.from + prefix.length, range.from + prefix.length + inner.length),
    };
  });
}

/** `[text](https://)` with the URL pre-selected. */
export function linkSpec(state: EditorState, url = 'https://'): TransactionSpec {
  return state.changeByRange((range) => {
    const text = state.sliceDoc(range.from, range.to) || 'teks';
    const insert = `[${text}](${url})`;
    const urlFrom = range.from + text.length + 3;
    return {
      changes: { from: range.from, to: range.to, insert },
      range: EditorSelection.range(urlFrom, urlFrom + url.length),
    };
  });
}

/** Set the current line to heading `level`; applying the same level again removes it. */
export function headingSpec(state: EditorState, level: number): TransactionSpec {
  const line = state.doc.lineAt(state.selection.main.head);
  const m = /^(#{1,6})\s+/.exec(line.text);
  const clean = line.text.replace(/^#{1,6}\s+/, '');
  const same = m !== null && m[1].length === level;
  const insert = same ? clean : `${'#'.repeat(level)} ${clean}`;
  return { changes: { from: line.from, to: line.to, insert } };
}

function run(spec: (state: EditorState) => TransactionSpec) {
  return (view: EditorView): boolean => {
    view.dispatch(view.state.update(spec(view.state), { scrollIntoView: true, userEvent: 'input' }));
    return true;
  };
}

/** Hotkeys that were missing in the editor. Mod = Ctrl (Linux/Windows) or Cmd (macOS). */
export const markdownKeymap: KeyBinding[] = [
  { key: 'Mod-b', run: run((s) => toggleWrapSpec(s, '**', '**', 'teks tebal')), preventDefault: true },
  { key: 'Mod-i', run: run((s) => toggleWrapSpec(s, '*', '*', 'teks miring')), preventDefault: true },
  { key: 'Mod-k', run: run((s) => linkSpec(s)), preventDefault: true },
  { key: 'Mod-Shift-k', run: run((s) => toggleWrapSpec(s, '`', '`', 'kode')), preventDefault: true },
  { key: 'Mod-Shift-x', run: run((s) => toggleWrapSpec(s, '~~', '~~', 'coret')), preventDefault: true },
  ...[1, 2, 3, 4, 5, 6].map((n) => ({
    key: `Mod-${n}`,
    run: run((s: EditorState) => headingSpec(s, n)),
    preventDefault: true,
  })),
];

/** Tab / Shift-Tab indent + Find & Replace (Mod-f, Mod-h via searchKeymap). */
export const editingKeymap: KeyBinding[] = [
  indentWithTab,
  ...searchKeymap,
  { key: 'Mod-h', run: (view) => searchKeymap.find((k) => k.key === 'Mod-f')?.run?.(view) ?? false },
];

// ---------------------------------------------------------------- slash commands

export interface SlashItem {
  label: string;
  detail: string;
  /** Template text; `|` marks where the cursor ends up (removed from the output). */
  template: string;
}

export const SLASH_ITEMS: SlashItem[] = [
  { label: '/h1', detail: 'Heading 1', template: '# |' },
  { label: '/h2', detail: 'Heading 2', template: '## |' },
  { label: '/h3', detail: 'Heading 3', template: '### |' },
  { label: '/table', detail: 'Tabel 3×3', template: '| Kolom 1 | Kolom 2 | Kolom 3 |\n| --- | --- | --- |\n| | | |\n|' },
  { label: '/code', detail: 'Blok kode', template: '```|\n\n```' },
  { label: '/math', detail: 'Rumus blok (KaTeX)', template: '$$\n|\n$$' },
  { label: '/mermaid', detail: 'Diagram Mermaid', template: '```mermaid\ngraph TD;\n  A-->B;|\n```' },
  { label: '/callout', detail: 'Callout NOTE', template: '> [!NOTE]\n> |' },
  { label: '/warning', detail: 'Callout WARNING', template: '> [!WARNING]\n> |' },
  { label: '/todo', detail: 'Daftar tugas', template: '- [ ] |' },
  { label: '/bullet', detail: 'Daftar poin', template: '- |' },
  { label: '/number', detail: 'Daftar angka', template: '1. |' },
  { label: '/quote', detail: 'Kutipan', template: '> |' },
  { label: '/hr', detail: 'Garis pemisah', template: '---\n|' },
  { label: '/footnote', detail: 'Catatan kaki', template: '[^1]|\n\n[^1]: ' },
];

export function expandTemplate(tpl: string): { text: string; cursor: number } {
  const i = tpl.indexOf('|');
  // A template like the table row legitimately contains `|`: only the LAST one is the cursor.
  const idx = tpl.lastIndexOf('|');
  if (i < 0 || idx < 0) return { text: tpl, cursor: tpl.length };
  return { text: tpl.slice(0, idx) + tpl.slice(idx + 1), cursor: idx };
}

/** Triggers when a line is only `/word`. */
export function slashCompletionSource(context: CompletionContext): CompletionResult | null {
  const line = context.state.doc.lineAt(context.pos);
  const before = line.text.slice(0, context.pos - line.from);
  const m = /^(\s*)\/([\w-]*)$/.exec(before);
  if (!m) return null;
  const from = line.from + m[1].length;
  const options: Completion[] = SLASH_ITEMS.map((item) => ({
    label: item.label,
    detail: item.detail,
    type: 'keyword',
    apply: (view: EditorView, _c: Completion, f: number, t: number) => {
      const { text, cursor } = expandTemplate(item.template);
      view.dispatch({
        changes: { from: f, to: t, insert: text },
        selection: { anchor: f + cursor },
        userEvent: 'input.complete',
      });
    },
  }));
  return { from, options, validFor: /^\/[\w-]*$/ };
}

// ---------------------------------------------------------------- [[wikilink]] completion

/**
 * Autocomplete after `[[`: note names, and after `[[Name#` the headings of that note.
 * CodeMirror's built-in matcher already ranks options fuzzily.
 */
export function wikilinkCompletionSource(
  getNames: () => string[],
  getHeadings: (name: string) => string[] = () => []
) {
  return (context: CompletionContext): CompletionResult | null => {
    const line = context.state.doc.lineAt(context.pos);
    const before = line.text.slice(0, context.pos - line.from);
    const hasClose = context.state.sliceDoc(context.pos, context.pos + 2) === ']]';

    const hm = /\[\[([^\]\n|#]+)#([^\]\n|]*)$/.exec(before);
    if (hm) {
      const from = context.pos - hm[2].length;
      const options: Completion[] = Array.from(new Set(getHeadings(hm[1].trim()))).map((h) => ({
        label: h,
        type: 'property',
        apply: hasClose ? h : `${h}]]`,
      }));
      return options.length ? { from, options, validFor: /^[^\]\n|]*$/ } : null;
    }

    const m = /\[\[([^\]\n|#]*)$/.exec(before);
    if (!m) return null;
    const from = context.pos - m[1].length;
    const options: Completion[] = Array.from(new Set(getNames())).map((name) => ({
      label: name,
      type: 'text',
      apply: hasClose ? name : `${name}]]`,
    }));
    return { from, options, validFor: /^[^\]\n|#]*$/ };
  };
}
