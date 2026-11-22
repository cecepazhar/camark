import { describe, it, expect } from 'vitest';
import {
  parseNote,
  buildIndex,
  getBacklinks,
  findUnlinkedMentions,
  renameLinksInContent,
  graphData,
} from './knowledge';

describe('Wikilink, Backlinks, and Knowledge Graph Integration (F4-01 / W-01..W-04)', () => {
  const notes = [
    {
      path: '/vault/arsitektur.md',
      name: 'arsitektur.md',
      content: `# Arsitektur Sistem\n\nLihat [[panduan-instalasi]] dan [[modul-rust#Fungsi Inti|Modul Core]].\n\n## Detail\nImplementasi berbasis Tauri v2.`,
    },
    {
      path: '/vault/panduan-instalasi.md',
      name: 'panduan-instalasi.md',
      content: `# Panduan Instalasi\n\nMerujuk ke [[arsitektur]].\nCatatan ini juga membahas tentang arsitektur secara umum.`,
    },
    {
      path: '/vault/modul-rust.md',
      name: 'modul-rust.md',
      content: `# Modul Core Rust\n\n## Fungsi Inti\nLogika inti aplikasi CAMark.`,
    },
  ];

  it('parses wikilinks with headings and custom aliases (W-01)', () => {
    const parsed = parseNote(notes[0].content);
    expect(parsed.links).toHaveLength(2);
    expect(parsed.links[0]).toMatchObject({ target: 'panduan-instalasi', heading: '', alias: '' });
    expect(parsed.links[1]).toMatchObject({ target: 'modul-rust', heading: 'Fungsi Inti', alias: 'Modul Core' });
  });

  it('indexes backlinks across workspace documents with line context (W-02)', () => {
    const index = buildIndex(notes);
    const backlinksToArsitektur = getBacklinks(index, '/vault/arsitektur.md');
    expect(backlinksToArsitektur).toHaveLength(1);
    expect(backlinksToArsitektur[0].sourceName).toBe('panduan-instalasi.md');
    expect(backlinksToArsitektur[0].snippet).toContain('[[arsitektur]]');
  });

  it('detects unlinked mentions in documents (W-03)', () => {
    const index = buildIndex(notes);
    const unlinked = findUnlinkedMentions(index, 'arsitektur');
    expect(unlinked.length).toBeGreaterThan(0);
    expect(unlinked.some((m) => m.sourceName === 'panduan-instalasi.md')).toBe(true);
  });

  it('renames links across content automatically when note title changes', () => {
    const res = renameLinksInContent(notes[0].content, 'panduan-instalasi', 'panduan-pemasangan-baru');
    expect(res.text).toContain('[[panduan-pemasangan-baru]]');
    expect(res.text).not.toContain('[[panduan-instalasi]]');
    expect(res.count).toBe(1);
  });

  it('builds graph representation with nodes and directional edges (W-04)', () => {
    const index = buildIndex(notes);
    const graph = graphData(index);
    expect(graph.nodes).toHaveLength(3);
    expect(graph.edges.length).toBeGreaterThanOrEqual(2);
  });
});
