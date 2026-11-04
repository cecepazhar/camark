import { describe, it, expect } from 'vitest';
import { EditorState, EditorSelection } from '@codemirror/state';
import { CompletionContext } from '@codemirror/autocomplete';
import {
  toggleWrapSpec,
  linkSpec,
  headingSpec,
  expandTemplate,
  slashCompletionSource,
  wikilinkCompletionSource,
  SLASH_ITEMS,
  markdownKeymap,
  editingKeymap,
} from './editorCommands';

const st = (doc: string, from: number, to = from) =>
  EditorState.create({ doc, selection: EditorSelection.single(from, to) });
const apply = (s: EditorState, spec: ReturnType<typeof toggleWrapSpec>) => s.update(spec).state;

describe('toggleWrapSpec (Ctrl+B / Ctrl+I)', () => {
  it('wraps a selection in bold', () => {
    const s = apply(st('halo dunia', 5, 10), toggleWrapSpec(st('halo dunia', 5, 10), '**'));
    expect(s.doc.toString()).toBe('halo **dunia**');
    expect(s.sliceDoc(s.selection.main.from, s.selection.main.to)).toBe('dunia');
  });
  it('toggling twice restores the original text', () => {
    const s1 = apply(st('halo dunia', 5, 10), toggleWrapSpec(st('halo dunia', 5, 10), '**'));
    const s2 = apply(s1, toggleWrapSpec(s1, '**'));
    expect(s2.doc.toString()).toBe('halo dunia');
  });
  it('inserts a selected placeholder when nothing is selected', () => {
    const base = st('', 0);
    const s = apply(base, toggleWrapSpec(base, '*', '*', 'teks miring'));
    expect(s.doc.toString()).toBe('*teks miring*');
    expect(s.sliceDoc(s.selection.main.from, s.selection.main.to)).toBe('teks miring');
  });
  it('italic inside bold does not strip the bold', () => {
    const base = st('**a**', 2, 3);
    expect(apply(base, toggleWrapSpec(base, '*')).doc.toString()).toBe('***a***');
  });
  it('works for every cursor in a multi-selection', () => {
    const base = EditorState.create({
      doc: 'ab cd',
      selection: EditorSelection.create([EditorSelection.range(0, 2), EditorSelection.range(3, 5)]),
      extensions: [EditorState.allowMultipleSelections.of(true)],
    });
    expect(apply(base, toggleWrapSpec(base, '~~')).doc.toString()).toBe('~~ab~~ ~~cd~~');
  });
});

describe('linkSpec (Ctrl+K)', () => {
  it('wraps selection and selects the URL', () => {
    const base = st('lihat docs', 6, 10);
    const s = apply(base, linkSpec(base));
    expect(s.doc.toString()).toBe('lihat [docs](https://)');
    expect(s.sliceDoc(s.selection.main.from, s.selection.main.to)).toBe('https://');
  });
});

describe('headingSpec (Ctrl+1..6)', () => {
  it('sets, switches and toggles off a heading', () => {
    const a = apply(st('Judul', 0), headingSpec(st('Judul', 0), 2));
    expect(a.doc.toString()).toBe('## Judul');
    const b = apply(a, headingSpec(a, 3));
    expect(b.doc.toString()).toBe('### Judul');
    const c = apply(b, headingSpec(b, 3));
    expect(c.doc.toString()).toBe('Judul');
  });
});

describe('keymaps', () => {
  const keys = markdownKeymap.map((k) => k.key);
  it('binds bold/italic/link/code/strike and 6 heading levels', () => {
    for (const k of ['Mod-b', 'Mod-i', 'Mod-k', 'Mod-Shift-k', 'Mod-Shift-x', 'Mod-1', 'Mod-6']) {
      expect(keys).toContain(k);
    }
  });
  it('adds Tab indent and find/replace', () => {
    const ek = editingKeymap.map((k) => k.key);
    expect(ek).toContain('Tab');
    expect(ek).toContain('Mod-f');
    expect(ek).toContain('Mod-h');
  });
});

describe('slash commands', () => {
  const ctx = (doc: string) => new CompletionContext(st(doc, doc.length), doc.length, true);
  it('offers commands when a line is just /word', () => {
    const r = slashCompletionSource(ctx('/ta'));
    expect(r).not.toBeNull();
    expect(r!.from).toBe(0);
    expect(r!.options.map((o) => o.label)).toContain('/table');
  });
  it('is silent in the middle of a sentence or a URL', () => {
    expect(slashCompletionSource(ctx('lihat /ta'))).toBeNull();
    expect(slashCompletionSource(ctx('https://x.com/ta'))).toBeNull();
  });
  it('expands templates and places the cursor at the last |', () => {
    expect(expandTemplate('## |')).toEqual({ text: '## ', cursor: 3 });
    const t = expandTemplate(SLASH_ITEMS.find((i) => i.label === '/table')!.template);
    expect(t.text.startsWith('| Kolom 1 |')).toBe(true);
    expect(t.cursor).toBe(t.text.length);
    expect(expandTemplate('```|\n\n```')).toEqual({ text: '```\n\n```', cursor: 3 });
  });
});

describe('[[ wikilink completion', () => {
  const src = wikilinkCompletionSource(() => ['prd', 'task', 'prd']);
  const ctx = (doc: string) => new CompletionContext(st(doc, doc.length), doc.length, true);
  it('suggests unique note names after [[', () => {
    const r = src(ctx('lihat [[pr'));
    expect(r!.from).toBe(8);
    expect(r!.options.map((o) => o.label)).toEqual(['prd', 'task']);
  });
  it('does nothing outside a [[ context', () => {
    expect(src(ctx('lihat [pr'))).toBeNull();
    expect(src(ctx('[[sudah]] lalu'))).toBeNull();
  });
});
