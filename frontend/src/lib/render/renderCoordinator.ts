/**
 * Markdown Render Coordinator (CAMark P-02 & P-03).
 *
 * Coordinates off-thread Markdown + KaTeX parsing using a Web Worker.
 *
 * Sequence Numbering & Stale Result Strategy (P-02):
 * 1. Every parse request receives an auto-incremented sequence number (seq).
 * 2. When worker responses return, any result with seq < latestCompletedSeq is
 *    immediately dropped as stale without touching DOMPurify or the DOM.
 * 3. Only the latest valid response updates latestCompletedSeq.
 *
 * Incremental Block Sanitization (P-03):
 * 1. The worker returns unsanitized HTML blocks with source hashes.
 * 2. The main thread sanitizes ONLY changed blocks with DOMPurify and reuses
 *    cached clean HTML for unchanged blocks.
 * 3. Full sanitized HTML and structured blocks are provided for per-block DOM patching.
 */

import type { RenderOptions } from '../utils/markdownRender';
import DOMPurify from 'dompurify';
import { DOMPURIFY_CONFIG } from '../bench/renderBenchmark';
import {
  defaultBlockCache,
  type RenderedBlock,
  type BlockType,
} from './incrementalRender';
import type {
  MarkdownWorkerRequest,
  MarkdownWorkerResponse,
  WorkerRawBlock,
} from './markdownParser.worker';

export interface RenderResult {
  seq: number;
  html: string;
  yaml: string | null;
  blocks?: RenderedBlock[];
  isStale: boolean;
  error?: string;
  workerDurationMs?: number;
  totalBlocks?: number;
  reRenderedBlocks?: number;
  cachedBlocks?: number;
}

export interface CoordinatorStats {
  dispatchedCount: number;
  completedCount: number;
  droppedStaleCount: number;
  latestDispatchedSeq: number;
  latestCompletedSeq: number;
  lastDispatchMs: number;
}

export type RenderCallback = (result: RenderResult) => void;

export interface IWorkerLike {
  postMessage(message: any, transfer?: Transferable[]): void;
  terminate?(): void;
  onmessage?: ((this: any, ev: MessageEvent) => any) | null;
  addEventListener?(type: string, listener: EventListenerOrEventListenerObject): void;
  removeEventListener?(type: string, listener: EventListenerOrEventListenerObject): void;
}

export class RenderCoordinator {
  private worker: IWorkerLike | null = null;
  private currentSeq = 0;
  private latestCompletedSeq = 0;
  private droppedStaleCount = 0;
  private completedCount = 0;
  private lastDispatchMs = 0;
  private cleanBlockCache = new Map<string, string>();
  private pendingRequests = new Map<
    number,
    {
      resolve: (res: RenderResult) => void;
      startTime: number;
      callback?: RenderCallback;
    }
  >();
  private isFallbackMode = false;

  constructor(customWorker?: IWorkerLike | null) {
    if (customWorker) {
      this.worker = customWorker;
      this.bindWorkerEvents();
    } else if (typeof Worker !== 'undefined') {
      try {
        this.worker = new Worker(new URL('./markdownParser.worker.ts', import.meta.url), {
          type: 'module',
        });
        this.bindWorkerEvents();
      } catch (err) {
        console.warn('RenderCoordinator: Failed to spawn Web Worker, using asynchronous fallback:', err);
        this.isFallbackMode = true;
      }
    } else {
      this.isFallbackMode = true;
    }
  }

  private bindWorkerEvents(): void {
    if (!this.worker) return;

    const handleMessage = (event: MessageEvent<MarkdownWorkerResponse>) => {
      const data = event.data;
      if (!data || data.type !== 'renderResult') return;
      this.handleWorkerResponse(data);
    };

    if (typeof this.worker.addEventListener === 'function') {
      this.worker.addEventListener('message', handleMessage as EventListener);
    } else {
      this.worker.onmessage = handleMessage;
    }
  }

  /**
   * Internal handler for results returned by worker or fallback.
   */
  public handleWorkerResponse(data: MarkdownWorkerResponse): void {
    const { seq, html: rawHtmlFallback, yaml, blocks: rawBlocks, error } = data;
    const pending = this.pendingRequests.get(seq);
    const startTime = pending ? pending.startTime : performance.now();
    const workerDurationMs = performance.now() - startTime;

    // Sequence check: drop if an older result arrives after a newer one has already been accepted
    if (seq < this.latestCompletedSeq) {
      this.droppedStaleCount++;
      if (pending) {
        this.pendingRequests.delete(seq);
        const staleResult: RenderResult = {
          seq,
          html: '',
          yaml: null,
          isStale: true,
          workerDurationMs,
        };
        pending.resolve(staleResult);
        if (pending.callback) {
          pending.callback(staleResult);
        }
      }
      return;
    }

    // Accept newer result
    this.latestCompletedSeq = seq;
    this.completedCount++;

    let fullCleanHtml = '';
    let renderedBlocks: RenderedBlock[] | undefined = undefined;

    if (rawBlocks && rawBlocks.length > 0) {
      renderedBlocks = [];
      for (const rb of rawBlocks) {
        let cleanHtml = this.cleanBlockCache.get(rb.hash);
        let isCached = true;
        if (cleanHtml === undefined) {
          cleanHtml = DOMPurify.sanitize(rb.rawHtml, DOMPURIFY_CONFIG);
          if (this.cleanBlockCache.size > 5000) this.cleanBlockCache.clear();
          this.cleanBlockCache.set(rb.hash, cleanHtml);
          isCached = false;
        }

        const wrappedHtml = `<div class="preview-block" data-block-id="${rb.id}" data-block-hash="${rb.hash}" data-line-start="${rb.startLine}" data-line-end="${rb.endLine}">${cleanHtml}</div>`;
        renderedBlocks.push({
          id: rb.id,
          hash: rb.hash,
          startLine: rb.startLine,
          endLine: rb.endLine,
          type: rb.type,
          rawHtml: rb.rawHtml,
          cleanHtml,
          wrappedHtml,
          isCached,
        });
      }
      fullCleanHtml = renderedBlocks.map((b) => b.wrappedHtml).join('\n');
    } else {
      fullCleanHtml = DOMPurify.sanitize(rawHtmlFallback, DOMPURIFY_CONFIG);
    }

    const result: RenderResult = {
      seq,
      html: fullCleanHtml,
      yaml,
      blocks: renderedBlocks,
      totalBlocks: data.totalBlocks ?? renderedBlocks?.length,
      reRenderedBlocks: data.reRenderedBlocks,
      cachedBlocks: data.cachedBlocks,
      isStale: false,
      error,
      workerDurationMs,
    };

    if (pending) {
      this.pendingRequests.delete(seq);
      pending.resolve(result);
      if (pending.callback) {
        pending.callback(result);
      }
    }
  }

  /**
   * Dispatches a render request off-thread and returns a Promise.
   * If a newer request completes first, this promise resolves with `isStale: true`.
   */
  public render(
    content: string,
    options: RenderOptions = {}
  ): Promise<RenderResult> {
    const seq = ++this.currentSeq;
    const t0 = performance.now();

    const knownFilesList = options.knownFiles
      ? Array.from(options.knownFiles)
      : undefined;

    const request: MarkdownWorkerRequest = {
      type: 'render',
      seq,
      content,
      options: {
        breaks: options.breaks ?? false,
        knownFiles: knownFilesList,
        currentFilePath: options.currentFilePath ?? null,
      },
    };

    return new Promise<RenderResult>((resolve) => {
      this.pendingRequests.set(seq, {
        resolve,
        startTime: t0,
      });

      if (!this.isFallbackMode && this.worker) {
        this.worker.postMessage(request);
        this.lastDispatchMs = performance.now() - t0;
      } else {
        // Fallback for testing environments (Vitest jsdom)
        this.lastDispatchMs = performance.now() - t0;
        setTimeout(async () => {
          try {
            const inc = await defaultBlockCache.renderDocumentIncremental(content, options);
            const rawBlocks: WorkerRawBlock[] = inc.blocks.map((b) => ({
              id: b.id,
              hash: b.hash,
              startLine: b.startLine,
              endLine: b.endLine,
              type: b.type,
              rawHtml: b.rawHtml,
            }));
            this.handleWorkerResponse({
              type: 'renderResult',
              seq,
              html: inc.fullHtml,
              yaml: inc.yaml,
              blocks: rawBlocks,
              totalBlocks: inc.totalBlocks,
              reRenderedBlocks: inc.reRenderedBlocks,
              cachedBlocks: inc.cachedBlocks,
            });
          } catch (err: unknown) {
            const errMsg = err instanceof Error ? err.message : String(err);
            this.handleWorkerResponse({
              type: 'renderResult',
              seq,
              html: '',
              yaml: null,
              error: errMsg,
            });
          }
        }, 0);
      }
    });
  }

  /**
   * Dispatches a render request with a callback triggered only when complete.
   * Stale results have isStale: true.
   */
  public requestRender(
    content: string,
    options: RenderOptions = {},
    callback?: RenderCallback
  ): number {
    const seq = ++this.currentSeq;
    const t0 = performance.now();

    const knownFilesList = options.knownFiles
      ? Array.from(options.knownFiles)
      : undefined;

    const request: MarkdownWorkerRequest = {
      type: 'render',
      seq,
      content,
      options: {
        breaks: options.breaks ?? false,
        knownFiles: knownFilesList,
        currentFilePath: options.currentFilePath ?? null,
      },
    };

    this.pendingRequests.set(seq, {
      resolve: () => {},
      startTime: t0,
      callback,
    });

    if (!this.isFallbackMode && this.worker) {
      this.worker.postMessage(request);
      this.lastDispatchMs = performance.now() - t0;
    } else {
      this.lastDispatchMs = performance.now() - t0;
      setTimeout(async () => {
        try {
          const inc = await defaultBlockCache.renderDocumentIncremental(content, options);
          const rawBlocks: WorkerRawBlock[] = inc.blocks.map((b) => ({
            id: b.id,
            hash: b.hash,
            startLine: b.startLine,
            endLine: b.endLine,
            type: b.type,
            rawHtml: b.rawHtml,
          }));
          this.handleWorkerResponse({
            type: 'renderResult',
            seq,
            html: inc.fullHtml,
            yaml: inc.yaml,
            blocks: rawBlocks,
            totalBlocks: inc.totalBlocks,
            reRenderedBlocks: inc.reRenderedBlocks,
            cachedBlocks: inc.cachedBlocks,
          });
        } catch (err: unknown) {
          const errMsg = err instanceof Error ? err.message : String(err);
          this.handleWorkerResponse({
            type: 'renderResult',
            seq,
            html: '',
            yaml: null,
            error: errMsg,
          });
        }
      }, 0);
    }

    return seq;
  }

  /**
   * Returns telemetry regarding requests, completed renders, and dropped stale parses.
   */
  public getStats(): CoordinatorStats {
    return {
      dispatchedCount: this.currentSeq,
      completedCount: this.completedCount,
      droppedStaleCount: this.droppedStaleCount,
      latestDispatchedSeq: this.currentSeq,
      latestCompletedSeq: this.latestCompletedSeq,
      lastDispatchMs: this.lastDispatchMs,
    };
  }

  /**
   * Resets sequence counters (e.g. when opening a completely new document).
   */
  public reset(initialSeq = 0): void {
    this.currentSeq = initialSeq;
    this.latestCompletedSeq = initialSeq;
    this.droppedStaleCount = 0;
    this.completedCount = 0;
    this.cleanBlockCache.clear();
    this.pendingRequests.clear();
  }

  /**
   * Terminates the worker thread and cleans up all pending requests.
   */
  public terminate(): void {
    if (this.worker && typeof this.worker.terminate === 'function') {
      this.worker.terminate();
    }
    this.worker = null;
    this.cleanBlockCache.clear();
    this.pendingRequests.clear();
  }
}

/**
 * Global singleton instance for app-wide coordination.
 */
let defaultCoordinator: RenderCoordinator | null = null;

export function getRenderCoordinator(): RenderCoordinator {
  if (!defaultCoordinator) {
    defaultCoordinator = new RenderCoordinator();
  }
  return defaultCoordinator;
}
