/**
 * Pure Markdown-rendering helpers (no DOM, no Tauri) so they can be unit-tested.
 * Fixes audit findings F-07: KaTeX inside code/prose, frontmatter, task toggling.
 */
import { marked, type TokenizerAndRendererExtension } from 'marked';
import katex from 'katex';

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

export interface FrontmatterResult {
  /** Raw YAML between the fences, or null when the document has none. */
  yaml: string | null;
  /** Markdown with the frontmatter block removed. */
  body: string;
}

/** Split a leading `---\n…\n---` YAML block (must start at byte 0). */
export function splitFrontmatter(src: string): FrontmatterResult {
  const m = /^---[ \t]*\r?\n([\s\S]*?)\r?\n(?:---|\.\.\.)[ \t]*(?:\r?\n|$)/.exec(src);
  if (!m) return { yaml: null, body: src };
  return { yaml: m[1], body: src.slice(m[0].length) };
}

function renderMath(tex: string, displayMode: boolean): string {
  try {
    return katex.renderToString(tex, { displayMode, throwOnError: false });
  } catch {
    return escapeHtml(tex);
  }
}

// Inline math follows the Pandoc rule: opening `$` must not be followed by whitespace,
// closing `$` must not be preceded by whitespace nor followed by a digit. This keeps
// "harga $5 dan $10" as plain text. Because these are marked *tokenizers*, they never
// run inside code spans / fenced code (unlike the old global regex replace).
const inlineMath: TokenizerAndRendererExtension = {
  name: 'inlineMath',
  level: 'inline',
  start(src: string) {
    const i = src.indexOf('$');
    return i < 0 ? undefined : i;
  },
  tokenizer(src: string) {
    const m = /^\$(?!\$)(?=\S)((?:\\\$|[^$\n])+?)(?<=\S)\$(?!\d)/.exec(src);
    if (m) return { type: 'inlineMath', raw: m[0], text: m[1] };
    return undefined;
  },
  renderer(token) {
    return renderMath((token as unknown as { text: string }).text, false);
  },
};

const blockMath: TokenizerAndRendererExtension = {
  name: 'blockMath',
  level: 'block',
  start(src: string) {
    const i = src.indexOf('$$');
    return i < 0 ? undefined : i;
  },
  tokenizer(src: string) {
    const m = /^\$\$[ \t]*\n?([\s\S]+?)\n?\$\$[ \t]*(?:\n|$)/.exec(src);
    if (m) return { type: 'blockMath', raw: m[0], text: m[1].trim() };
    return undefined;
  },
  renderer(token) {
    return `<div class="katex-block">${renderMath((token as unknown as { text: string }).text, true)}</div>\n`;
  },
};

// ---------------------------------------------------------------- wikilinks

const wikilink: TokenizerAndRendererExtension = {
  name: 'wikilink',
  level: 'inline',
  start(src: string) {
    const i = src.indexOf('[[');
    return i < 0 ? undefined : i;
  },
  tokenizer(src: string) {
    const m = /^\[\[([^\[\]\n|#]+)(?:#([^\[\]\n|]+))?(?:\|([^\[\]\n]+))?\]\]/.exec(src);
    if (!m) return undefined;
    return {
      type: 'wikilink',
      raw: m[0],
      name: m[1].trim(),
      heading: m[2]?.trim() ?? '',
      alias: m[3]?.trim() ?? '',
    };
  },
  renderer(token) {
    const t = token as unknown as { name: string; heading: string; alias: string };
    const label = t.alias || (t.heading ? `${t.name} › ${t.heading}` : t.name);
    return `<a href="#" class="wikilink" data-wikilink="${escapeHtml(t.name)}" data-heading="${escapeHtml(t.heading)}">${escapeHtml(label)}</a>`;
  },
};

// ---------------------------------------------------------------- embeds ![[note]]

const noteEmbed: TokenizerAndRendererExtension = {
  name: 'noteEmbed',
  level: 'inline',
  start(src: string) {
    const i = src.indexOf('![[');
    return i < 0 ? undefined : i;
  },
  tokenizer(src: string) {
    const m = /^!\[\[([^\[\]\n|#]+)(?:#([^\[\]\n|]+))?(?:\|([^\[\]\n]+))?\]\]/.exec(src);
    if (!m) return undefined;
    return { type: 'noteEmbed', raw: m[0], name: m[1].trim(), heading: m[2]?.trim() ?? '' };
  },
  renderer(token) {
    const t = token as unknown as { name: string; heading: string };
    const label = t.heading ? `${t.name} › ${t.heading}` : t.name;
    // Filled in after render by the page (needs file access); until then it is a plain link.
    return `<span class="note-embed" data-embed="${escapeHtml(t.name)}" data-heading="${escapeHtml(t.heading)}"><a href="#" class="wikilink" data-wikilink="${escapeHtml(t.name)}" data-heading="${escapeHtml(t.heading)}">${escapeHtml(label)}</a></span>`;
  },
};

/** Markdown of the section under `heading` (until the next heading of same or higher level). */
export function extractSection(content: string, heading: string): string | null {
  if (!heading) return content;
  const lines = content.split('\n');
  const want = heading.trim().toLowerCase();
  let start = -1;
  let level = 0;
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    if (/^\s{0,3}(`{3,}|~{3,})/.test(lines[i])) inFence = !inFence;
    if (inFence) continue;
    const m = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(lines[i]);
    if (!m) continue;
    if (start < 0 && m[2].trim().toLowerCase() === want) {
      start = i;
      level = m[1].length;
    } else if (start >= 0 && m[1].length <= level) {
      return lines.slice(start, i).join('\n').trimEnd();
    }
  }
  return start < 0 ? null : lines.slice(start).join('\n').trimEnd();
}

/** Small stable hash (FNV-1a) used to cache rendered Mermaid SVG by source. */
export function hashString(s: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return (h >>> 0).toString(36);
}

// ---------------------------------------------------------------- images ![alt|300](src)

const imageRenderer = {
  image(this: any, token: any): string | false {
    const m = /^(.*?)\|(\d{1,4})(?:x(\d{1,4}))?$/.exec(token.text ?? '');
    if (!m) return false;
    const alt = m[1].trim();
    const w = ` width="${m[2]}"`;
    const h = m[3] ? ` height="${m[3]}"` : '';
    const title = token.title ? ` title="${escapeHtml(token.title)}"` : '';
    return `<img src="${escapeHtml(token.href ?? '')}" alt="${escapeHtml(alt)}"${w}${h}${title}>`;
  },
};

// ---------------------------------------------------------------- footnotes

interface FootnoteState {
  defs: Map<string, string>;
  order: string[];
}
let fnState: FootnoteState = { defs: new Map(), order: [] };

/** Remove `[^id]: text` definitions (outside fenced code) and return them. */
export function extractFootnoteDefs(src: string): { body: string; defs: Map<string, string> } {
  const defs = new Map<string, string>();
  const out: string[] = [];
  const lines = src.split('\n');
  let inFence = false;
  let fenceMark = '';
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const fence = /^\s{0,3}(`{3,}|~{3,})/.exec(line);
    if (fence) {
      if (!inFence) {
        inFence = true;
        fenceMark = fence[1][0];
      } else if (fence[1][0] === fenceMark) {
        inFence = false;
      }
      out.push(line);
      continue;
    }
    const m = !inFence ? /^\[\^([^\]\s]+)\]:[ \t]*(.*)$/.exec(line) : null;
    if (!m) {
      out.push(line);
      continue;
    }
    let text = m[2];
    while (i + 1 < lines.length && /^( {2,}|\t)\S/.test(lines[i + 1])) {
      text += ' ' + lines[++i].trim();
    }
    defs.set(m[1], text);
  }
  return { body: out.join('\n'), defs };
}

const footnoteRef: TokenizerAndRendererExtension = {
  name: 'footnoteRef',
  level: 'inline',
  start(src: string) {
    const i = src.indexOf('[^');
    return i < 0 ? undefined : i;
  },
  tokenizer(src: string) {
    const m = /^\[\^([^\]\s]+)\](?!:)/.exec(src);
    if (m) return { type: 'footnoteRef', raw: m[0], id: m[1] };
    return undefined;
  },
  renderer(token) {
    const id = (token as unknown as { id: string }).id;
    if (!fnState.defs.has(id)) return escapeHtml(`[^${id}]`);
    let n = fnState.order.indexOf(id) + 1;
    if (n === 0) {
      fnState.order.push(id);
      n = fnState.order.length;
    }
    const safe = escapeHtml(id);
    return `<sup class="footnote-ref"><a href="#fn-${safe}" id="fnref-${safe}">${n}</a></sup>`;
  },
};

// ---------------------------------------------------------------- callouts

const CALLOUT_TYPES = new Set([
  'note', 'info', 'tip', 'important', 'warning', 'caution', 'danger', 'success',
  'question', 'bug', 'example', 'quote', 'abstract', 'todo', 'failure',
]);

const calloutRenderer = {
  blockquote(this: any, token: any): string | false {
    const first = token.tokens?.[0];
    if (!first || first.type !== 'paragraph') return false;
    const m = /^\[!(\w+)\][+-]?[ \t]*([^\n]*)(?:\n([\s\S]*))?$/.exec(first.text);
    if (!m) return false;
    const type = m[1].toLowerCase();
    if (!CALLOUT_TYPES.has(type)) return false;
    const title = m[2].trim() || type.charAt(0).toUpperCase() + type.slice(1);
    const restFirst = m[3] ? this.parser.parse(marked.lexer(m[3])) : '';
    const restOthers = token.tokens.length > 1 ? this.parser.parse(token.tokens.slice(1)) : '';
    return `<div class="callout callout-${type}"><div class="callout-title">${escapeHtml(title)}</div><div class="callout-body">${restFirst}${restOthers}</div></div>\n`;
  },
};

let installed = false;
/** Idempotent: registers math, wikilink, footnote and callout support on global `marked`. */
export function installMarkdownExtensions(): void {
  if (installed) return;
  marked.use({ extensions: [blockMath, inlineMath, noteEmbed, wikilink, footnoteRef] });
  marked.use({ renderer: calloutRenderer as any });
  marked.use({ renderer: imageRenderer as any });
  installed = true;
}

export interface RenderOptions {
  /** CommonMark soft line breaks become <br> when true. Default false. */
  breaks?: boolean;
}

/** Full pipeline: frontmatter → footnotes → marked. Returns UNSANITIZED html. */
export async function renderMarkdownToHtml(
  src: string,
  opts: RenderOptions = {}
): Promise<{ html: string; yaml: string | null }> {
  installMarkdownExtensions();
  const { yaml, body } = splitFrontmatter(src);
  const { body: noDefs, defs } = extractFootnoteDefs(body);
  fnState = { defs, order: [] };
  let html = await marked.parse(noDefs, { gfm: true, breaks: opts.breaks ?? false });
  if (fnState.order.length > 0) {
    const items = fnState.order
      .map((id) => {
        const safe = escapeHtml(id);
        const text = marked.parseInline(defs.get(id) ?? '') as string;
        return `<li id="fn-${safe}">${text} <a href="#fnref-${safe}" class="footnote-back">↩</a></li>`;
      })
      .join('');
    html += `<section class="footnotes"><hr><ol>${items}</ol></section>`;
  }
  if (yaml !== null) {
    html = `<details class="frontmatter-block"><summary>Properties</summary><pre>${escapeHtml(yaml)}</pre></details>${html}`;
  }
  return { html, yaml };
}

/**
 * Toggle the n-th GFM task marker (`- [ ]` / `- [x]`) in `src`, ignoring markers inside
 * fenced code blocks. `index` is the position of the checkbox in the rendered preview.
 * Returns the new source, or null when the index does not exist.
 */
export function toggleNthTask(src: string, index: number, checked: boolean): string | null {
  const lines = src.split('\n');
  let inFence = false;
  let fenceMark = '';
  let seen = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const fence = /^\s{0,3}(`{3,}|~{3,})/.exec(line);
    if (fence) {
      if (!inFence) {
        inFence = true;
        fenceMark = fence[1][0];
      } else if (fence[1][0] === fenceMark) {
        inFence = false;
      }
      continue;
    }
    if (inFence) continue;
    const m = /^(\s*(?:[-*+]|\d+[.)])\s+)\[( |x|X)\]/.exec(line);
    if (!m) continue;
    if (seen === index) {
      lines[i] = m[1] + (checked ? '[x]' : '[ ]') + line.slice(m[0].length);
      return lines.join('\n');
    }
    seen++;
  }
  return null;
}
