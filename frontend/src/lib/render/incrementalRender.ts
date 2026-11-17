/**
 * Incremental Markdown & Block-Level Renderer (CAMark P-03).
 *
 * Implements:
 * 1. Top-level block splitting with line numbers (source line -> block mapping for P-04).
 * 2. Source-hash caching of rendered & sanitized HTML per block.
 * 3. DOM patcher: diffs container children against block list, updating ONLY changed blocks.
 * 4. KaTeX & Mermaid caching by source hash with stable element IDs.
 */

import {
  hashString,
  renderMarkdownToHtml,
  splitFrontmatter,
  type RenderOptions,
  escapeHtml,
} from '../utils/markdownRender';
import DOMPurify from 'dompurify';
import { DOMPURIFY_CONFIG } from '../bench/renderBenchmark';

export type BlockType =
  | 'frontmatter'
  | 'heading'
  | 'code'
  | 'mermaid'
  | 'math'
  | 'callout'
  | 'blockquote'
  | 'table'
  | 'list'
  | 'hr'
  | 'paragraph'
  | 'footnotes';

export interface MarkdownBlock {
  id: string;
  raw: string;
  hash: string;
  startLine: number;
  endLine: number;
  type: BlockType;
}

export interface RenderedBlock {
  id: string;
  hash: string;
  startLine: number;
  endLine: number;
  type: BlockType;
  rawHtml: string;
  cleanHtml: string;
  wrappedHtml: string;
  isCached: boolean;
}

export interface IncrementalRenderResult {
  blocks: RenderedBlock[];
  fullHtml: string;
  yaml: string | null;
  totalBlocks: number;
  reRenderedBlocks: number;
  cachedBlocks: number;
  parseMs: number;
  sanitizeMs: number;
}

export interface PatchStats {
  updatedCount: number;
  reusedCount: number;
  insertedCount: number;
  removedCount: number;
}

function detectBlockType(raw: string): BlockType {
  const trimmed = raw.trim();
  if (trimmed.startsWith('---') || trimmed.startsWith('...')) return 'frontmatter';
  if (/^#{1,6}\s/.test(trimmed)) return 'heading';
  if (/^```mermaid\b/.test(trimmed)) return 'mermaid';
  if (/^(```|~~~)/.test(trimmed)) return 'code';
  if (/^\$\$/.test(trimmed)) return 'math';
  if (/^>\s*\[!/.test(trimmed)) return 'callout';
  if (/^>/.test(trimmed)) return 'blockquote';
  if (/^\|/.test(trimmed)) return 'table';
  if (/^(\s*[-*+]|\s*\d+[.)])\s/.test(trimmed)) return 'list';
  if (/^(?:---+|___+|\*\*\*+)$/.test(trimmed)) return 'hr';
  if (/^\[\^[^\]]+\]:/.test(trimmed)) return 'footnotes';
  return 'paragraph';
}

/**
 * Splits a markdown document into top-level blocks with exact line ranges.
 */
export function splitMarkdownBlocks(src: string): MarkdownBlock[] {
  if (!src) return [];

  const lines = src.split('\n');
  const blocks: MarkdownBlock[] = [];
  let currentLines: string[] = [];
  let inFence = false;
  let fenceChar = '';
  let inMath = false;
  let inFrontmatter = false;
  let startLine = 1;
  let blockIndex = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    const lineNum = i + 1;

    // 1. YAML Frontmatter check at byte 0
    if (i === 0 && /^---[ \t]*$/.test(trimmed)) {
      inFrontmatter = true;
      currentLines.push(line);
      continue;
    }
    if (inFrontmatter) {
      currentLines.push(line);
      if (/^(?:---|\\.\\.\\.)[ \t]*$/.test(trimmed)) {
        inFrontmatter = false;
        const raw = currentLines.join('\n');
        blocks.push({
          id: `b-${blockIndex++}`,
          raw,
          hash: hashString(raw),
          startLine: 1,
          endLine: lineNum,
          type: 'frontmatter',
        });
        currentLines = [];
        startLine = lineNum + 1;
      }
      continue;
    }

    // 2. Fenced Code Blocks (``` or ~~~)
    const fenceMatch = /^(\`\`\`+|~~~+)/.exec(trimmed);
    if (fenceMatch) {
      if (!inFence) {
        inFence = true;
        fenceChar = fenceMatch[1][0];
      } else if (trimmed.startsWith(fenceChar)) {
        inFence = false;
        fenceChar = '';
      }
    }

    // 3. Block Math ($$)
    if (!inFence) {
      if (trimmed === '$$') {
        inMath = !inMath;
      } else if (trimmed.startsWith('$$') && !trimmed.endsWith('$$')) {
        inMath = true;
      } else if (inMath && trimmed.endsWith('$$')) {
        inMath = false;
      }
    }

    // 4. Blank line outside fences and display math splits blocks
    if (!inFence && !inMath && trimmed === '') {
      if (currentLines.length > 0) {
        const raw = currentLines.join('\n');
        blocks.push({
          id: `b-${blockIndex++}`,
          raw,
          hash: hashString(raw),
          startLine,
          endLine: i,
          type: detectBlockType(raw),
        });
        currentLines = [];
      }
      startLine = lineNum + 1;
    } else {
      if (currentLines.length === 0) {
        startLine = lineNum;
      }
      currentLines.push(line);
    }
  }

  if (currentLines.length > 0) {
    const raw = currentLines.join('\n');
    blocks.push({
      id: `b-${blockIndex++}`,
      raw,
      hash: hashString(raw),
      startLine,
      endLine: lines.length,
      type: detectBlockType(raw),
    });
  }

  return blocks;
}

/**
 * Cache for rendered block HTML (both raw Marked and clean DOMPurify).
 */
export class BlockRenderCache {
  private rawCache = new Map<string, string>();
  private cleanCache = new Map<string, string>();

  public getCacheSize(): { raw: number; clean: number } {
    return {
      raw: this.rawCache.size,
      clean: this.cleanCache.size,
    };
  }

  public clear(): void {
    this.rawCache.clear();
    this.cleanCache.clear();
  }

  /**
   * Render a single block's raw HTML (Marked + KaTeX).
   */
  public async renderBlockRaw(
    block: MarkdownBlock,
    options: RenderOptions = {}
  ): Promise<{ rawHtml: string; isCached: boolean }> {
    const cached = this.rawCache.get(block.hash);
    if (cached !== undefined) {
      return { rawHtml: cached, isCached: true };
    }

    let rawHtml = '';
    if (block.type === 'frontmatter') {
      const { yaml } = splitFrontmatter(block.raw);
      if (yaml !== null) {
        rawHtml = `<details class="frontmatter-block"><summary>Properties</summary><pre>${escapeHtml(yaml)}</pre></details>`;
      }
    } else {
      const { html } = await renderMarkdownToHtml(block.raw, options);
      rawHtml = html;
    }

    if (this.rawCache.size > 5000) this.rawCache.clear();
    this.rawCache.set(block.hash, rawHtml);
    return { rawHtml, isCached: false };
  }

  /**
   * Sanitize a single block's raw HTML with DOMPurify on the main thread.
   */
  public sanitizeBlock(
    rawHtml: string,
    hash: string,
    purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
  ): { cleanHtml: string; isCached: boolean } {
    const cached = this.cleanCache.get(hash);
    if (cached !== undefined) {
      return { cleanHtml: cached, isCached: true };
    }

    const cleanHtml = purifyInstance.sanitize(rawHtml, DOMPURIFY_CONFIG);
    if (this.cleanCache.size > 5000) this.cleanCache.clear();
    this.cleanCache.set(hash, cleanHtml);
    return { cleanHtml, isCached: false };
  }

  /**
   * Incremental render pipeline:
   * 1. Splits source into blocks.
   * 2. Reuses cached HTML for unchanged blocks (0 ms).
   * 3. Renders & sanitizes only changed or new blocks.
   * 4. Assembles fullHtml and returns structured blocks for DOM patching.
   */
  public async renderDocumentIncremental(
    src: string,
    options: RenderOptions = {},
    purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
  ): Promise<IncrementalRenderResult> {
    const t0 = performance.now();
    const blocks = splitMarkdownBlocks(src);
    const { yaml } = splitFrontmatter(src);

    const renderedBlocks: RenderedBlock[] = [];
    let cachedBlocks = 0;
    let reRenderedBlocks = 0;
    let totalParseMs = 0;
    let totalSanitizeMs = 0;

    for (const block of blocks) {
      // Check clean cache first
      const cleanHit = this.cleanCache.get(block.hash);
      const rawHit = this.rawCache.get(block.hash);

      if (cleanHit !== undefined && rawHit !== undefined) {
        cachedBlocks++;
        const wrappedHtml = `<div class="preview-block" data-block-id="${block.id}" data-block-hash="${block.hash}" data-line-start="${block.startLine}" data-line-end="${block.endLine}">${cleanHit}</div>`;
        renderedBlocks.push({
          id: block.id,
          hash: block.hash,
          startLine: block.startLine,
          endLine: block.endLine,
          type: block.type,
          rawHtml: rawHit,
          cleanHtml: cleanHit,
          wrappedHtml,
          isCached: true,
        });
      } else {
        reRenderedBlocks++;
        const tp0 = performance.now();
        const { rawHtml } = await this.renderBlockRaw(block, options);
        totalParseMs += performance.now() - tp0;

        const ts0 = performance.now();
        const { cleanHtml } = this.sanitizeBlock(rawHtml, block.hash, purifyInstance);
        totalSanitizeMs += performance.now() - ts0;

        const wrappedHtml = `<div class="preview-block" data-block-id="${block.id}" data-block-hash="${block.hash}" data-line-start="${block.startLine}" data-line-end="${block.endLine}">${cleanHtml}</div>`;
        renderedBlocks.push({
          id: block.id,
          hash: block.hash,
          startLine: block.startLine,
          endLine: block.endLine,
          type: block.type,
          rawHtml,
          cleanHtml,
          wrappedHtml,
          isCached: false,
        });
      }
    }

    const fullHtml = renderedBlocks.map((b) => b.wrappedHtml).join('\n');

    return {
      blocks: renderedBlocks,
      fullHtml,
      yaml,
      totalBlocks: blocks.length,
      reRenderedBlocks,
      cachedBlocks,
      parseMs: totalParseMs,
      sanitizeMs: totalSanitizeMs,
    };
  }
}

/**
 * Global singleton cache instance.
 */
export const defaultBlockCache = new BlockRenderCache();

/**
 * Patches the live DOM in `container` per block:
 * - Unchanged blocks (`data-block-hash` matches) are left 100% untouched.
 * - Changed blocks have their innerHTML and dataset updated in-place.
 * - Added blocks are appended.
 * - Deleted blocks are removed.
 */
export function patchDomPreview(
  container: HTMLElement,
  blocks: RenderedBlock[]
): PatchStats {
  let updatedCount = 0;
  let reusedCount = 0;
  let insertedCount = 0;
  let removedCount = 0;

  const existingChildren = Array.from(container.children) as HTMLElement[];
  const minLen = Math.min(existingChildren.length, blocks.length);

  // 1. In-place comparison and patching of existing block elements
  for (let i = 0; i < minLen; i++) {
    const child = existingChildren[i];
    const block = blocks[i];
    const currentHash = child.getAttribute('data-block-hash');

    if (currentHash === block.hash) {
      reusedCount++;
      // Node is completely preserved
    } else {
      updatedCount++;
      child.innerHTML = block.cleanHtml;
      child.setAttribute('data-block-hash', block.hash);
      child.setAttribute('data-block-id', block.id);
      child.setAttribute('data-line-start', String(block.startLine));
      child.setAttribute('data-line-end', String(block.endLine));
      child.setAttribute('data-block-type', block.type);
      child.setAttribute('data-dirty', '1');
    }
  }

  // 2. Append new blocks if new document has more blocks
  if (blocks.length > existingChildren.length) {
    for (let i = existingChildren.length; i < blocks.length; i++) {
      insertedCount++;
      const block = blocks[i];
      const el = document.createElement('div');
      el.className = 'preview-block';
      el.setAttribute('data-block-id', block.id);
      el.setAttribute('data-block-hash', block.hash);
      el.setAttribute('data-line-start', String(block.startLine));
      el.setAttribute('data-line-end', String(block.endLine));
      el.setAttribute('data-block-type', block.type);
      el.setAttribute('data-dirty', '1');
      el.innerHTML = block.cleanHtml;
      container.appendChild(el);
    }
  }

  // 3. Remove trailing children if new document has fewer blocks
  if (existingChildren.length > blocks.length) {
    for (let i = blocks.length; i < existingChildren.length; i++) {
      removedCount++;
      existingChildren[i].remove();
    }
  }

  return {
    updatedCount,
    reusedCount,
    insertedCount,
    removedCount,
  };
}
