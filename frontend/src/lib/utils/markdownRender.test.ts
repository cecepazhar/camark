import { describe, it, expect, beforeAll } from 'vitest';
import { marked } from 'marked';
import {
  installMarkdownExtensions,
  splitFrontmatter,
  toggleNthTask,
  escapeHtml,
} from './markdownRender';

beforeAll(() => installMarkdownExtensions());
const render = (s: string) => marked.parse(s, { gfm: true }) as string;

describe('math (F-07 / T1, T2)', () => {
  it('does not render $VAR inside fenced code', () => {
    const html = render('```bash\necho $HOME dan $PATH\n```');
    expect(html).not.toContain('katex');
    expect(html).toContain('$HOME');
  });
  it('does not render $VAR inside inline code', () => {
    expect(render('jalankan `echo $A dan $B`')).not.toContain('katex');
  });
  it('keeps prices as plain text', () => {
    const html = render('Harga paket $5 dan paket pro $10 per bulan.');
    expect(html).not.toContain('katex');
    expect(html).toContain('$5');
  });
  it('still renders real inline math', () => {
    expect(render('rumus $x^2$ ok')).toContain('katex');
  });
  it('renders block math', () => {
    expect(render('$$\nE = mc^2\n$$')).toContain('katex-display');
  });
});

describe('frontmatter (T4)', () => {
  it('splits a leading YAML block', () => {
    const r = splitFrontmatter('---\ntitle: Laporan\ntags: [a, b]\n---\n\nIsi');
    expect(r.yaml).toBe('title: Laporan\ntags: [a, b]');
    expect(r.body).toBe('\nIsi');
  });
  it('ignores --- that is not at the start', () => {
    const src = 'Teks\n\n---\n\nLagi';
    expect(splitFrontmatter(src)).toEqual({ yaml: null, body: src });
  });
  it('a horizontal rule document is untouched', () => {
    expect(splitFrontmatter('---\n\nIsi').yaml).toBeNull();
  });
});

describe('task toggle (T8)', () => {
  const doc = '- [ ] Review\n- [x] Deploy\n- [ ] Review';
  it('toggles the 3rd item, not the first duplicate', () => {
    expect(toggleNthTask(doc, 2, true)).toBe('- [ ] Review\n- [x] Deploy\n- [x] Review');
  });
  it('unchecks', () => {
    expect(toggleNthTask(doc, 1, false)).toBe('- [ ] Review\n- [ ] Deploy\n- [ ] Review');
  });
  it('skips markers inside code fences', () => {
    const src = '```md\n- [ ] contoh\n```\n\n- [ ] nyata';
    expect(toggleNthTask(src, 0, true)).toBe('```md\n- [ ] contoh\n```\n\n- [x] nyata');
  });
  it('returns null for a missing index', () => {
    expect(toggleNthTask(doc, 9, true)).toBeNull();
  });
});

describe('escapeHtml (T3)', () => {
  it('escapes tags', () => {
    expect(escapeHtml('<b>x</b>')).toBe('&lt;b&gt;x&lt;/b&gt;');
  });
});

import { renderMarkdownToHtml, extractFootnoteDefs } from './markdownRender';
import { highlightToHtml } from './highlight';

describe('callouts (F-07 / T6)', () => {
  it('renders > [!WARNING] as a callout', async () => {
    const { html } = await renderMarkdownToHtml('> [!WARNING]\n> Hati-hati');
    expect(html).toContain('callout callout-warning');
    expect(html).toContain('Hati-hati');
    expect(html).not.toContain('[!WARNING]');
  });
  it('supports a custom title and multi-paragraph body', async () => {
    const { html } = await renderMarkdownToHtml('> [!tip] Judul Saya\n> baris satu\n>\n> baris dua');
    expect(html).toContain('callout-tip');
    expect(html).toContain('Judul Saya');
    expect(html).toContain('baris dua');
  });
  it('leaves normal blockquotes alone', async () => {
    const { html } = await renderMarkdownToHtml('> kutipan biasa');
    expect(html).toContain('<blockquote>');
    expect(html).not.toContain('callout');
  });
});

describe('footnotes (T5)', () => {
  it('renders refs in order and a footnote section', async () => {
    const { html } = await renderMarkdownToHtml('A[^b] dan B[^a].\n\n[^a]: Catatan A\n[^b]: Catatan B');
    expect(html).toContain('href="#fn-b"');
    expect(html.indexOf('fn-b')).toBeLessThan(html.indexOf('fn-a'));
    expect(html).toContain('class="footnotes"');
    expect(html).toContain('Catatan B');
    expect(html).not.toContain('[^b]:');
  });
  it('keeps an undefined ref as plain text', async () => {
    const { html } = await renderMarkdownToHtml('Teks[^x] saja');
    expect(html).toContain('[^x]');
    expect(html).not.toContain('footnotes');
  });
  it('ignores definitions inside code fences', () => {
    const r = extractFootnoteDefs('```\n[^1]: contoh\n```');
    expect(r.defs.size).toBe(0);
  });
  it('does not leak state between renders', async () => {
    await renderMarkdownToHtml('A[^1]\n\n[^1]: satu');
    const { html } = await renderMarkdownToHtml('tanpa catatan');
    expect(html).not.toContain('footnotes');
  });
});

describe('wikilinks (T7)', () => {
  it('renders [[Nama]], alias and heading', async () => {
    const { html } = await renderMarkdownToHtml('[[Catatan Lain]] [[prd|Dokumen PRD]] [[prd#Bab 2]]');
    expect(html).toContain('data-wikilink="Catatan Lain"');
    expect(html).toContain('>Dokumen PRD</a>');
    expect(html).toContain('data-heading="Bab 2"');
  });
  it('escapes hostile names', async () => {
    const { html } = await renderMarkdownToHtml('[[<img src=x onerror=1>]]');
    expect(html).not.toContain('<img');
  });
  it('is not applied inside code', async () => {
    const { html } = await renderMarkdownToHtml('`[[a]]`');
    expect(html).not.toContain('wikilink');
  });
});

describe('pipeline options', () => {
  it('CommonMark soft breaks by default, <br> when breaks=true', async () => {
    expect((await renderMarkdownToHtml('a\nb')).html).not.toContain('<br');
    expect((await renderMarkdownToHtml('a\nb', { breaks: true })).html).toContain('<br');
  });
  it('frontmatter becomes a collapsed properties block, not a heading', async () => {
    const { html, yaml } = await renderMarkdownToHtml('---\ntitle: Laporan\n---\n\nIsi');
    expect(yaml).toBe('title: Laporan');
    expect(html).toContain('frontmatter-block');
    expect(html).not.toContain('<h2');
  });
});

describe('syntax highlight', () => {
  it('highlights supported languages with escaped output', async () => {
    const out = await highlightToHtml('const a = "<b>";', 'ts');
    expect(out).toContain('tok-');
    expect(out).not.toContain('<b>');
    expect(out).toContain('&lt;b&gt;');
  });
  it('returns null for unsupported languages', async () => {
    expect(await highlightToHtml('echo hi', 'bash')).toBeNull();
  });
  it('round-trips the text exactly', async () => {
    const code = 'fn main() {\n    println!("hi");\n}\n';
    const html = (await highlightToHtml(code, 'rust'))!;
    expect(html.replace(/<[^>]+>/g, '').replace(/&quot;/g, '"').replace(/&#039;/g, "'")).toBe(code);
  });
});
