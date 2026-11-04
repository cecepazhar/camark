import { describe, it, expect, vi } from 'vitest';
import { EditorState, EditorSelection } from '@codemirror/state';
import { CompletionContext } from '@codemirror/autocomplete';
import { renderMarkdownToHtml, extractSection, hashString } from './markdownRender';
import { wikilinkCompletionSource } from './editorCommands';
import { createIdleWatcher, autoLockMinutes } from './idleLock';

describe('note embeds ![[note]]', () => {
  it('renders an embed placeholder (not an image, not "!" + link)', async () => {
    const { html } = await renderMarkdownToHtml('![[prd#Bab Dua]]');
    expect(html).toContain('class="note-embed"');
    expect(html).toContain('data-embed="prd"');
    expect(html).toContain('data-heading="Bab Dua"');
    expect(html).not.toContain('<img');
    expect(html).not.toContain('!<a');
  });
  it('extracts a heading section up to the next same-level heading', () => {
    const doc = '# A\nx\n## B\nisi b\n### B1\nanak\n## C\nisi c';
    expect(extractSection(doc, 'B')).toBe('## B\nisi b\n### B1\nanak');
    expect(extractSection(doc, 'C')).toBe('## C\nisi c');
    expect(extractSection(doc, 'Tidak Ada')).toBeNull();
    expect(extractSection(doc, '')).toBe(doc);
  });
});

describe('image resize ![alt|300](src)', () => {
  it('sets width / height', async () => {
    const { html } = await renderMarkdownToHtml('![Logo|300](a.png) ![B|120x80](b.png)');
    expect(html).toContain('<img src="a.png" alt="Logo" width="300">');
    expect(html).toContain('width="120" height="80"');
  });
  it('leaves normal images alone', async () => {
    const { html } = await renderMarkdownToHtml('![Logo](a.png)');
    expect(html).toContain('alt="Logo"');
    expect(html).not.toContain('width=');
  });
  it('escapes hostile alt/src', async () => {
    const { html } = await renderMarkdownToHtml('![x" onerror="alert(1)|30](a.png)');
    expect(html).not.toContain('onerror="alert');
  });
});

describe('hashString', () => {
  it('is stable and distinguishes inputs', () => {
    expect(hashString('graph TD;A-->B')).toBe(hashString('graph TD;A-->B'));
    expect(hashString('graph TD;A-->B')).not.toBe(hashString('graph TD;A-->C'));
  });
});

describe('[[Name# heading completion', () => {
  const src = wikilinkCompletionSource(
    () => ['prd'],
    (name) => (name === 'prd' ? ['Ringkasan', 'Bab Dua'] : [])
  );
  const ctx = (doc: string) =>
    new CompletionContext(EditorState.create({ doc, selection: EditorSelection.cursor(doc.length) }), doc.length, true);
  it('offers headings of the named note after #', () => {
    const r = src(ctx('lihat [[prd#Ba'));
    expect(r!.from).toBe('lihat [[prd#'.length);
    expect(r!.options.map((o) => o.label)).toEqual(['Ringkasan', 'Bab Dua']);
    expect(r!.options[1].apply).toBe('Bab Dua]]');
  });
  it('returns null when the note has no headings', () => {
    expect(src(ctx('[[lain#'))).toBeNull();
  });
});

describe('auto-lock idle watcher', () => {
  it('fires once after inactivity and restarts on poke', () => {
    vi.useFakeTimers();
    const onIdle = vi.fn();
    const w = createIdleWatcher(1000, onIdle);
    vi.advanceTimersByTime(900);
    w.poke();
    vi.advanceTimersByTime(900);
    expect(onIdle).not.toHaveBeenCalled();
    vi.advanceTimersByTime(200);
    expect(onIdle).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(5000);
    expect(onIdle).toHaveBeenCalledTimes(1);
    w.stop();
    vi.useRealTimers();
  });
  it('is disabled with 0 and stoppable', () => {
    vi.useFakeTimers();
    const onIdle = vi.fn();
    createIdleWatcher(0, onIdle);
    const w = createIdleWatcher(100, onIdle);
    w.stop();
    vi.advanceTimersByTime(1000);
    expect(onIdle).not.toHaveBeenCalled();
    vi.useRealTimers();
  });
  it('parses the minutes setting', () => {
    expect(autoLockMinutes(null)).toBe(10);
    expect(autoLockMinutes('0')).toBe(0);
    expect(autoLockMinutes('999')).toBe(240);
    expect(autoLockMinutes('abc')).toBe(10);
  });
});
