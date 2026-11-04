import { describe, it, expect } from 'vitest';
import {
  noteKey,
  parseNote,
  buildIndex,
  getBacklinks,
  findUnlinkedMentions,
  renameLinksInContent,
  fuzzyScore,
  fuzzyFilter,
  graphData,
  forceLayout,
} from './knowledge';

const vault = [
  { path: '/v/prd.md', name: 'prd.md', content: '---\ntags: [produk, spec]\n---\n# PRD\nLihat [[task]] dan [[Roadmap#Q1|rencana]].\n## Bab Dua\n#draft' },
  { path: '/v/task.md', name: 'task.md', content: '# Task\nDari [[prd]]. Juga ![[prd#Bab Dua]].\n```\n[[bukan-link]] #bukan-tag\n```\nSebut prd di sini dan `[[kode]]`.' },
  { path: '/v/notes/roadmap.md', name: 'roadmap.md', content: 'Roadmap tanpa link. Membahas PRD secara singkat. #produk' },
  { path: '/v/img.png', name: 'img.png', content: '' },
];

describe('parseNote', () => {
  it('extracts links, embeds, headings and tags; ignores code', () => {
    const p = parseNote(vault[1].content);
    expect(p.links.map((l) => [l.target, l.heading, l.embed])).toEqual([
      ['prd', '', false],
      ['prd', 'Bab Dua', true],
    ]);
    expect(p.headings[0]).toMatchObject({ level: 1, text: 'Task', slug: 'task', line: 0 });
    expect(p.tags).toEqual([]);
  });
  it('reads frontmatter and inline tags, not headings', () => {
    expect(parseNote(vault[0].content).tags).toEqual(['draft', 'produk', 'spec']);
  });
  it('keeps alias and heading', () => {
    const l = parseNote(vault[0].content).links[1];
    expect(l).toMatchObject({ target: 'Roadmap', heading: 'Q1', alias: 'rencana' });
  });
});

describe('index & backlinks', () => {
  const idx = buildIndex(vault);
  it('indexes only markdown files', () => {
    expect([...idx.notes.keys()].sort()).toEqual(['prd', 'roadmap', 'task']);
  });
  it('finds backlinks case-insensitively with line + snippet', () => {
    const bl = getBacklinks(idx, '/v/prd.md');
    expect(bl).toHaveLength(1); // two links on the same line count once
    expect(bl[0]).toMatchObject({ sourceName: 'task.md', line: 1 });
    expect(bl[0].snippet).toContain('[[prd]]');
    expect(getBacklinks(idx, 'roadmap')[0].sourceName).toBe('prd.md');
  });
  it('builds a tag index', () => {
    expect(idx.tags.get('produk')?.sort()).toEqual(['/v/notes/roadmap.md', '/v/prd.md']);
  });
  it('noteKey normalises paths and extensions', () => {
    expect(noteKey('/a/b/Roadmap.MD')).toBe('roadmap');
  });
});

describe('unlinked mentions', () => {
  it('finds plain-text mentions but not existing links or code', () => {
    const m = findUnlinkedMentions(buildIndex(vault), 'prd');
    expect(m.map((x) => x.sourceName).sort()).toEqual(['roadmap.md', 'task.md']);
    expect(m.find((x) => x.sourceName === 'task.md')!.line).toBe(5);
  });
});

describe('renameLinksInContent', () => {
  it('rewrites every link form and preserves heading/alias', () => {
    const src = 'A [[prd]] B [[PRD|alias]] C ![[prd#Bab]] D [[prd-lain]]';
    const r = renameLinksInContent(src, 'prd.md', 'spesifikasi.md');
    expect(r.text).toBe('A [[spesifikasi]] B [[spesifikasi|alias]] C ![[spesifikasi#Bab]] D [[prd-lain]]');
    expect(r.count).toBe(3);
  });
  it('does not touch code blocks or inline code', () => {
    const src = '```\n[[prd]]\n```\n`[[prd]]` [[prd]]';
    const r = renameLinksInContent(src, 'prd', 'x');
    expect(r.text).toBe('```\n[[prd]]\n```\n`[[prd]]` [[x]]');
    expect(r.count).toBe(1);
  });
});

describe('fuzzy', () => {
  it('ranks prefix and word-start matches first', () => {
    const items = ['notes/roadmap.md', 'prd.md', 'project-review.md', 'xyz.md'];
    expect(fuzzyFilter('prd', items, (s) => s)[0]).toBe('prd.md');
    expect(fuzzyFilter('pr rev', items, (s) => s)[0]).toBe('project-review.md');
    expect(fuzzyScore('zzz', 'prd.md')).toBe(-1);
  });
});

describe('graph', () => {
  it('builds unique undirected edges between resolved notes', () => {
    const g = graphData(buildIndex(vault));
    expect(g.edges).toHaveLength(2); // prd–task (both directions) + prd–roadmap
    expect(g.nodes.find((n) => n.id === 'prd')!.degree).toBe(2);
    expect(graphData(buildIndex(vault), true).nodes.length).toBe(3);
  });
  it('lays out deterministically inside the box; linked nodes end up closer', () => {
    const notes = Array.from({ length: 12 }, (_, i) => ({
      path: `/n${i}.md`,
      name: `n${i}.md`,
      content: i < 6 ? `[[n${(i + 1) % 6}]]` : '',
    }));
    const g1 = graphData(buildIndex(notes));
    const g2 = graphData(buildIndex(notes));
    forceLayout(g1.nodes, g1.edges, 800, 600);
    forceLayout(g2.nodes, g2.edges, 800, 600);
    expect(g1.nodes.map((n) => [n.x, n.y])).toEqual(g2.nodes.map((n) => [n.x, n.y]));
    for (const n of g1.nodes) {
      expect(n.x).toBeGreaterThanOrEqual(10);
      expect(n.x).toBeLessThanOrEqual(790);
    }
    const p = new Map(g1.nodes.map((n) => [n.id, n]));
    const dist = (a: string, b: string) => Math.hypot(p.get(a)!.x - p.get(b)!.x, p.get(a)!.y - p.get(b)!.y);
    const linked = g1.edges.map((e) => dist(e.source, e.target));
    const avgLinked = linked.reduce((s, d) => s + d, 0) / linked.length;
    const avgUnlinked = dist('n6', 'n9') + dist('n7', 'n10') + dist('n8', 'n11');
    expect(avgLinked).toBeLessThan(avgUnlinked / 3);
  });
});
