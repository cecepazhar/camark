/**
 * Web Worker for off-thread Markdown + KaTeX parsing (CAMark P-02 & P-03).
 *
 * Runs Marked lexing/parsing, KaTeX math typesetting, footnotes,
 * frontmatter separation, callout rendering, and wikilink autolinking
 * off the browser main thread to eliminate typing latency on large documents.
 *
 * P-03 Incremental parsing:
 * Splits documents into top-level blocks and caches raw HTML by block source hash.
 * Only changed blocks are parsed on edits.
 *
 * Returns raw, UNSANITIZED HTML with sequence numbers and block metadata.
 * Sanitization with DOMPurify strictly remains on the main thread.
 */

import {
  splitMarkdownBlocks,
  BlockRenderCache,
  type BlockType,
} from './incrementalRender';
import { splitFrontmatter, type RenderOptions } from '../utils/markdownRender';

export interface WorkerRawBlock {
  id: string;
  hash: string;
  startLine: number;
  endLine: number;
  type: BlockType;
  rawHtml: string;
}

export interface MarkdownWorkerRequest {
  type: 'render';
  seq: number;
  content: string;
  options?: {
    breaks?: boolean;
    knownFiles?: string[];
    currentFilePath?: string | null;
  };
}

export interface MarkdownWorkerResponse {
  type: 'renderResult';
  seq: number;
  html: string;
  yaml: string | null;
  blocks?: WorkerRawBlock[];
  totalBlocks?: number;
  reRenderedBlocks?: number;
  cachedBlocks?: number;
  error?: string;
}

const workerBlockCache = new BlockRenderCache();

/**
 * Pure handler for a render request, directly testable in any environment.
 */
export async function processRenderRequest(
  request: MarkdownWorkerRequest
): Promise<MarkdownWorkerResponse> {
  const { seq, content, options } = request;
  try {
    const opts: RenderOptions = {
      breaks: options?.breaks ?? false,
      knownFiles: options?.knownFiles ? new Set(options.knownFiles) : undefined,
      currentFilePath: options?.currentFilePath ?? null,
    };

    const { yaml } = splitFrontmatter(content);
    const parsedBlocks = splitMarkdownBlocks(content);

    const blocks: WorkerRawBlock[] = [];
    let cachedBlocks = 0;
    let reRenderedBlocks = 0;

    for (const pb of parsedBlocks) {
      const { rawHtml, isCached } = await workerBlockCache.renderBlockRaw(pb, opts);
      if (isCached) {
        cachedBlocks++;
      } else {
        reRenderedBlocks++;
      }
      blocks.push({
        id: pb.id,
        hash: pb.hash,
        startLine: pb.startLine,
        endLine: pb.endLine,
        type: pb.type,
        rawHtml,
      });
    }

    // Assemble raw full HTML string as fallback
    const rawFullHtml = blocks
      .map(
        (b) =>
          `<div class="preview-block" data-block-id="${b.id}" data-block-hash="${b.hash}" data-line-start="${b.startLine}" data-line-end="${b.endLine}">${b.rawHtml}</div>`
      )
      .join('\n');

    return {
      type: 'renderResult',
      seq,
      html: rawFullHtml,
      yaml,
      blocks,
      totalBlocks: blocks.length,
      reRenderedBlocks,
      cachedBlocks,
    };
  } catch (err: unknown) {
    const errMsg = err instanceof Error ? err.message : String(err);
    return {
      type: 'renderResult',
      seq,
      html: '',
      yaml: null,
      error: errMsg,
    };
  }
}

// Attach listener in Web Worker context
const ctx: any = typeof self !== 'undefined' ? self : null;
if (ctx && typeof ctx.postMessage === 'function') {
  ctx.onmessage = async (event: MessageEvent<MarkdownWorkerRequest>) => {
    const data = event.data;
    if (!data || data.type !== 'render') return;
    const response = await processRenderRequest(data);
    ctx.postMessage(response);
  };
}
