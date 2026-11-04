/**
 * Knowledge-graph helpers (Phase 2, frontend-only): link/tag/heading extraction,
 * backlinks, unlinked mentions, rename-aware link rewriting, fuzzy matching and a
 * deterministic force layout. Pure functions — no DOM, no Tauri — so they are unit-tested.
 */

export interface LinkRef {
  target: string; // note name as written (without .md)
  heading: string;
  alias: string;
  embed: boolean;
  line: number; // 0-based
}
export interface HeadingRef {
  level: number;
  text: string;
  slug: string;
  line: number;
}
export interface ParsedNote {
  links: LinkRef[];
  tags: string[];
  headings: HeadingRef[];
}
export interface NoteInput {
  path: string;
  name: string; // file name incl. extension
  content: string;
}
export interface IndexedNote extends ParsedNote {
  path: string;
  name: string;
  key: string;
  lines: string[];
}
export interface Backlink {
  sourcePath: string;
  sourceName: string;
  line: number;
  snippet: string;
}

const MD_EXT = /\.(md|markdown)$/i;

/** Lower-cased base name without extension — the identity used by [[links]]. */
export function noteKey(nameOrPath: string): string {
  const base = nameOrPath.split(/[\\/]/).pop() ?? nameOrPath;
  return base.replace(MD_EXT, '').trim().toLowerCase();
}

/** Same slug rule as the preview's heading ids (+page.svelte proRenderer.heading). */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/\/\s*$/, '')
    .replace(/[^\w\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-');
}

/** Replace inline code spans with spaces so their contents are ignored but columns stay. */
function blankInlineCode(line: string): string {
  return line.replace(/(`+)([\s\S]*?)\1/g, (m) => ' '.repeat(m.length));
}

/** Iterate lines that are outside fenced code blocks and outside a leading frontmatter. */
function* proseLines(content: string): Generator<{ text: string; index: number }> {
  const lines = content.split('\n');
  let i = 0;
  if (lines[0]?.trim() === '---') {
    const end = lines.findIndex((l, k) => k > 0 && (l.trim() === '---' || l.trim() === '...'));
    if (end > 0) i = end + 1;
  }
  let inFence = false;
  let mark = '';
  for (; i < lines.length; i++) {
    const fence = /^\s{0,3}(`{3,}|~{3,})/.exec(lines[i]);
    if (fence) {
      if (!inFence) {
        inFence = true;
        mark = fence[1][0];
      } else if (fence[1][0] === mark) {
        inFence = false;
      }
      continue;
    }
    if (!inFence) yield { text: blankInlineCode(lines[i]), index: i };
  }
}

const LINK_RE = /(!?)\[\[([^\[\]\n|#]+)(?:#([^\[\]\n|]+))?(?:\|([^\[\]\n]+))?\]\]/g;
const TAG_RE = /(^|[\s(])#([\p{L}\p{N}_][\p{L}\p{N}_/-]*)/gu;

function frontmatterTags(content: string): string[] {
  const m = /^---\r?\n([\s\S]*?)\r?\n(?:---|\.\.\.)/.exec(content);
  if (!m) return [];
  const inline = /^tags:\s*\[([^\]]*)\]/m.exec(m[1]);
  if (inline) return inline[1].split(',').map((t) => t.trim().replace(/^['"#]|['"]$/g, '')).filter(Boolean);
  const block = /^tags:\s*\n((?:\s*-\s*.+\n?)+)/m.exec(m[1]);
  if (block) return block[1].split('\n').map((l) => l.replace(/^\s*-\s*/, '').trim().replace(/^['"#]|['"]$/g, '')).filter(Boolean);
  const single = /^tags:\s*(\S.*)$/m.exec(m[1]);
  return single ? single[1].split(/[,\s]+/).map((t) => t.replace(/^#/, '')).filter(Boolean) : [];
}

export function parseNote(content: string): ParsedNote {
  const links: LinkRef[] = [];
  const headings: HeadingRef[] = [];
  const tags = new Set<string>(frontmatterTags(content).map((t) => t.toLowerCase()));
  for (const { text, index } of proseLines(content)) {
    const h = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(text);
    if (h) {
      const clean = h[2].replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
      headings.push({ level: h[1].length, text: clean, slug: slugify(clean), line: index });
    }
    for (const m of text.matchAll(LINK_RE)) {
      links.push({
        target: m[2].trim(),
        heading: m[3]?.trim() ?? '',
        alias: m[4]?.trim() ?? '',
        embed: m[1] === '!',
        line: index,
      });
    }
    if (!h) {
      for (const m of text.matchAll(TAG_RE)) {
        if (!/^\d+$/.test(m[2])) tags.add(m[2].toLowerCase());
      }
    }
  }
  return { links, tags: [...tags].sort(), headings };
}

export interface NoteIndex {
  notes: Map<string, IndexedNote>; // key -> note (first wins on duplicate names)
  backlinks: Map<string, Backlink[]>; // target key -> sources
  tags: Map<string, string[]>; // tag -> note paths
}

export function buildIndex(inputs: NoteInput[]): NoteIndex {
  const notes = new Map<string, IndexedNote>();
  for (const n of inputs) {
    if (!MD_EXT.test(n.name)) continue;
    const key = noteKey(n.name);
    if (notes.has(key)) continue;
    notes.set(key, { ...parseNote(n.content), path: n.path, name: n.name, key, lines: n.content.split('\n') });
  }
  const backlinks = new Map<string, Backlink[]>();
  const tags = new Map<string, string[]>();
  for (const note of notes.values()) {
    const seen = new Set<string>();
    for (const l of note.links) {
      const target = noteKey(l.target);
      if (target === note.key) continue;
      const dedupe = `${target}:${l.line}`;
      if (seen.has(dedupe)) continue;
      seen.add(dedupe);
      const list = backlinks.get(target) ?? [];
      list.push({
        sourcePath: note.path,
        sourceName: note.name,
        line: l.line,
        snippet: (note.lines[l.line] ?? '').trim().slice(0, 200),
      });
      backlinks.set(target, list);
    }
    for (const t of note.tags) {
      const list = tags.get(t) ?? [];
      list.push(note.path);
      tags.set(t, list);
    }
  }
  return { notes, backlinks, tags };
}

export function getBacklinks(index: NoteIndex, nameOrPath: string): Backlink[] {
  return index.backlinks.get(noteKey(nameOrPath)) ?? [];
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** Plain-text mentions of a note's name that are not already wikilinks. */
export function findUnlinkedMentions(index: NoteIndex, nameOrPath: string): Backlink[] {
  const key = noteKey(nameOrPath);
  const target = index.notes.get(key);
  const display = target ? target.name.replace(MD_EXT, '') : nameOrPath.replace(MD_EXT, '');
  if (display.trim().length < 3) return [];
  const re = new RegExp(`(^|[^\\p{L}\\p{N}_])${escapeRegExp(display)}(?=$|[^\\p{L}\\p{N}_])`, 'iu');
  const out: Backlink[] = [];
  for (const note of index.notes.values()) {
    if (note.key === key) continue;
    for (const { text, index: line } of proseLines(note.lines.join('\n'))) {
      const withoutLinks = text.replace(/!?\[\[[^\]]*\]\]/g, (m) => ' '.repeat(m.length)).replace(/\[[^\]]*\]\([^)]*\)/g, (m) => ' '.repeat(m.length));
      if (re.test(withoutLinks)) {
        out.push({ sourcePath: note.path, sourceName: note.name, line, snippet: note.lines[line].trim().slice(0, 200) });
      }
    }
  }
  return out;
}

/** Rewrite [[old]], [[old|a]], [[old#h]] and ![[old]] to the new name. Skips code. */
export function renameLinksInContent(content: string, oldName: string, newName: string): { text: string; count: number } {
  const oldKey = noteKey(oldName);
  const newBase = (newName.split(/[\\/]/).pop() ?? newName).replace(MD_EXT, '');
  const lines = content.split('\n');
  let count = 0;
  const prose = new Set<number>();
  for (const { index } of proseLines(content)) prose.add(index);
  for (let i = 0; i < lines.length; i++) {
    if (!prose.has(i)) continue;
    // Protect inline code spans while rewriting.
    const spans: string[] = [];
    const masked = lines[i].replace(/(`+)[\s\S]*?\1/g, (m) => {
      spans.push(m);
      return `\u0000${spans.length - 1}\u0000`;
    });
    const replaced = masked.replace(LINK_RE, (full, bang: string, target: string, heading?: string, alias?: string) => {
      if (noteKey(target) !== oldKey) return full;
      count++;
      return `${bang}[[${newBase}${heading ? `#${heading}` : ''}${alias ? `|${alias}` : ''}]]`;
    });
    lines[i] = replaced.replace(/\u0000(\d+)\u0000/g, (_, n: string) => spans[Number(n)]);
  }
  return { text: lines.join('\n'), count };
}

// ---------------------------------------------------------------- fuzzy

/** Subsequence fuzzy score; higher is better, -1 = no match. */
export function fuzzyScore(query: string, candidate: string): number {
  const q = query.toLowerCase().trim();
  const c = candidate.toLowerCase();
  if (!q) return 0;
  let score = 0;
  let ci = 0;
  let prev = -2;
  for (let qi = 0; qi < q.length; qi++) {
    const ch = q[qi];
    if (ch === ' ') continue;
    const found = c.indexOf(ch, ci);
    if (found < 0) return -1;
    score += 1;
    if (found === prev + 1) score += 5; // consecutive
    if (found === 0 || /[\s/_\-.]/.test(c[found - 1])) score += 8; // word start
    prev = found;
    ci = found + 1;
  }
  if (c.startsWith(q)) score += 20;
  if (c.includes(q)) score += 10;
  score -= c.length * 0.05; // prefer shorter
  return score;
}

export function fuzzyFilter<T>(query: string, items: T[], text: (t: T) => string, limit = 50): T[] {
  if (!query.trim()) return items.slice(0, limit);
  return items
    .map((item) => ({ item, s: fuzzyScore(query, text(item)) }))
    .filter((x) => x.s >= 0)
    .sort((a, b) => b.s - a.s)
    .slice(0, limit)
    .map((x) => x.item);
}

// ---------------------------------------------------------------- graph

export interface GraphNode {
  id: string; // note key
  label: string;
  path: string | null; // null = unresolved link
  degree: number;
  x: number;
  y: number;
}
export interface GraphEdge {
  source: string;
  target: string;
}

export function graphData(index: NoteIndex, includeUnresolved = false): { nodes: GraphNode[]; edges: GraphEdge[] } {
  const nodes = new Map<string, GraphNode>();
  for (const n of index.notes.values()) {
    nodes.set(n.key, { id: n.key, label: n.name.replace(MD_EXT, ''), path: n.path, degree: 0, x: 0, y: 0 });
  }
  const edges: GraphEdge[] = [];
  const seen = new Set<string>();
  for (const n of index.notes.values()) {
    for (const l of n.links) {
      const t = noteKey(l.target);
      if (t === n.key) continue;
      if (!nodes.has(t)) {
        if (!includeUnresolved) continue;
        nodes.set(t, { id: t, label: l.target, path: null, degree: 0, x: 0, y: 0 });
      }
      const k = n.key < t ? `${n.key}\u0001${t}` : `${t}\u0001${n.key}`;
      if (seen.has(k)) continue;
      seen.add(k);
      edges.push({ source: n.key, target: t });
      nodes.get(n.key)!.degree++;
      nodes.get(t)!.degree++;
    }
  }
  return { nodes: [...nodes.values()], edges };
}

/** Deterministic Fruchterman–Reingold layout in a width×height box. Mutates node x/y. */
export function forceLayout(nodes: GraphNode[], edges: GraphEdge[], width: number, height: number, iterations = 300): void {
  const n = nodes.length;
  if (n === 0) return;
  const cx = width / 2;
  const cy = height / 2;
  // Deterministic initial placement on a circle.
  nodes.forEach((node, i) => {
    const a = (2 * Math.PI * i) / n;
    const r = Math.min(width, height) * 0.35;
    node.x = cx + r * Math.cos(a);
    node.y = cy + r * Math.sin(a);
  });
  if (n === 1) return;
  const area = width * height;
  const k = Math.sqrt(area / n) * 0.75;
  const pos = new Map(nodes.map((nd) => [nd.id, nd]));
  let temp = Math.min(width, height) / 8;
  const cool = temp / (iterations + 1);
  const dx = new Float64Array(n);
  const dy = new Float64Array(n);
  const idx = new Map(nodes.map((nd, i) => [nd.id, i]));
  for (let it = 0; it < iterations; it++) {
    dx.fill(0);
    dy.fill(0);
    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        let ddx = nodes[i].x - nodes[j].x;
        let ddy = nodes[i].y - nodes[j].y;
        let d = Math.hypot(ddx, ddy);
        if (d < 0.01) {
          ddx = 0.01 * (i - j);
          ddy = 0.01;
          d = Math.hypot(ddx, ddy);
        }
        const f = (k * k) / d;
        dx[i] += (ddx / d) * f;
        dy[i] += (ddy / d) * f;
        dx[j] -= (ddx / d) * f;
        dy[j] -= (ddy / d) * f;
      }
    }
    for (const e of edges) {
      const a = pos.get(e.source);
      const b = pos.get(e.target);
      if (!a || !b) continue;
      const ia = idx.get(e.source)!;
      const ib = idx.get(e.target)!;
      const ddx = a.x - b.x;
      const ddy = a.y - b.y;
      const d = Math.max(0.01, Math.hypot(ddx, ddy));
      const f = (d * d) / k;
      dx[ia] -= (ddx / d) * f;
      dy[ia] -= (ddy / d) * f;
      dx[ib] += (ddx / d) * f;
      dy[ib] += (ddy / d) * f;
    }
    for (let i = 0; i < n; i++) {
      // gentle gravity keeps disconnected nodes on screen
      dx[i] += (cx - nodes[i].x) * 0.02 * k / 10;
      dy[i] += (cy - nodes[i].y) * 0.02 * k / 10;
      const d = Math.hypot(dx[i], dy[i]);
      if (d > 0) {
        const step = Math.min(d, temp);
        nodes[i].x += (dx[i] / d) * step;
        nodes[i].y += (dy[i] / d) * step;
      }
      nodes[i].x = Math.min(width - 10, Math.max(10, nodes[i].x));
      nodes[i].y = Math.min(height - 10, Math.max(10, nodes[i].y));
    }
    temp -= cool;
  }
}
