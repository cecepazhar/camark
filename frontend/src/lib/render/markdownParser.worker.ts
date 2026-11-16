/**
 * Web Worker for off-thread Markdown + KaTeX parsing (CAMark P-02).
 *
 * Runs Marked lexing/parsing, KaTeX math typesetting, footnotes,
 * frontmatter separation, callout rendering, and wikilink autolinking
 * off the browser main thread to eliminate typing latency on large documents.
 *
 * Returns raw, UNSANITIZED HTML with sequence numbers.
 * Sanitization with DOMPurify strictly remains on the main thread.
 */

import { renderMarkdownToHtml, type RenderOptions } from '../utils/markdownRender';

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
  error?: string;
}

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
    const { html, yaml } = await renderMarkdownToHtml(content, opts);
    return {
      type: 'renderResult',
      seq,
      html,
      yaml,
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
