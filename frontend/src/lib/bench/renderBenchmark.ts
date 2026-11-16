/**
 * Repeatable Render Benchmark Harness for CAMark (P-01 & P-02).
 *
 * Measures:
 * 1. Parse time: Markdown AST + KaTeX math tokenization (in-thread vs off-thread worker)
 * 2. Main-thread dispatch duration: Blocking time on UI thread when typing initiates render (<16ms)
 * 3. Sanitize time: DOMPurify HTML sanitization with production allowlist
 * 4. Total time: parse time + sanitize time
 */

import { renderMarkdownToHtml } from '../utils/markdownRender';
import { RenderCoordinator } from '../render/renderCoordinator';
import DOMPurify from 'dompurify';

export const DOMPURIFY_CONFIG = {
  ADD_TAGS: ['button', 'span', 'div', 'svg', 'path'],
  ADD_ATTR: [
    'data-lang',
    'data-code',
    'type',
    'd',
    'stroke',
    'fill',
    'viewBox',
    'stroke-width',
    'stroke-linecap',
    'stroke-linejoin',
  ],
};

export interface BenchmarkMeasurement {
  parseMs: number;
  sanitizeMs: number;
  totalMs: number;
  rawHtmlLength: number;
  cleanHtmlLength: number;
}

export interface FixtureBenchmarkResult {
  fixtureName: string;
  wordCount: number;
  byteSize: number;
  iterations: number;
  runs: BenchmarkMeasurement[];
  avgParseMs: number;
  medianParseMs: number;
  avgSanitizeMs: number;
  medianSanitizeMs: number;
  avgTotalMs: number;
  medianTotalMs: number;
}

export interface OffThreadBenchmarkMeasurement {
  mainThreadDispatchMs: number;
  workerParseMs: number;
  parseMs: number;
  mainThreadSanitizeMs: number;
  sanitizeMs: number;
  totalMs: number;
  rawHtmlLength: number;
  cleanHtmlLength: number;
  seq: number;
  isStale: boolean;
}

export interface OffThreadFixtureBenchmarkResult {
  fixtureName: string;
  wordCount: number;
  byteSize: number;
  iterations: number;
  avgMainThreadDispatchMs: number;
  medianMainThreadDispatchMs: number;
  avgWorkerParseMs: number;
  medianWorkerParseMs: number;
  avgSanitizeMs: number;
  medianSanitizeMs: number;
  avgTotalMs: number;
  medianTotalMs: number;
}

export function countWords(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

export function median(values: number[]): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) {
    return (sorted[mid - 1] + sorted[mid]) / 2;
  }
  return sorted[mid];
}

export function average(values: number[]): number {
  if (values.length === 0) return 0;
  const sum = values.reduce((acc, v) => acc + v, 0);
  return sum / values.length;
}

/**
 * Measure a single in-thread pass of parsing and sanitizing markdown content (P-01 baseline).
 */
export async function measureSingleRender(
  markdownContent: string,
  purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
): Promise<BenchmarkMeasurement> {
  const t0 = performance.now();
  const { html: rawHtml } = await renderMarkdownToHtml(markdownContent, { breaks: false });
  const t1 = performance.now();
  const parseMs = t1 - t0;

  const t2 = performance.now();
  const cleanHtml = purifyInstance.sanitize(rawHtml, DOMPURIFY_CONFIG);
  const t3 = performance.now();
  const sanitizeMs = t3 - t2;

  const totalMs = parseMs + sanitizeMs;

  return {
    parseMs,
    sanitizeMs,
    totalMs,
    rawHtmlLength: rawHtml.length,
    cleanHtmlLength: cleanHtml.length,
  };
}

/**
 * Run a multi-iteration in-thread benchmark on a fixture document with warm-up runs.
 */
export async function benchmarkFixture(
  fixtureName: string,
  markdownContent: string,
  iterations = 5,
  warmupRuns = 1,
  purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
): Promise<FixtureBenchmarkResult> {
  // Warm-up runs to ensure JIT optimization and marked/KaTeX initialization
  for (let w = 0; w < warmupRuns; w++) {
    await measureSingleRender(markdownContent, purifyInstance);
  }

  const runs: BenchmarkMeasurement[] = [];
  for (let i = 0; i < iterations; i++) {
    const measurement = await measureSingleRender(markdownContent, purifyInstance);
    runs.push(measurement);
  }

  const parseTimes = runs.map((r) => r.parseMs);
  const sanitizeTimes = runs.map((r) => r.sanitizeMs);
  const totalTimes = runs.map((r) => r.totalMs);

  return {
    fixtureName,
    wordCount: countWords(markdownContent),
    byteSize: new TextEncoder().encode(markdownContent).length,
    iterations,
    runs,
    avgParseMs: average(parseTimes),
    medianParseMs: median(parseTimes),
    avgSanitizeMs: average(sanitizeTimes),
    medianSanitizeMs: median(sanitizeTimes),
    avgTotalMs: average(totalTimes),
    medianTotalMs: median(totalTimes),
  };
}

/**
 * Measure off-thread rendering (P-02):
 * Captures main-thread dispatch overhead, worker parse time, and main-thread DOMPurify sanitize time.
 */
export async function measureOffThreadRender(
  markdownContent: string,
  coordinator: RenderCoordinator,
  purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
): Promise<OffThreadBenchmarkMeasurement> {
  const tDispatchStart = performance.now();
  const renderPromise = coordinator.render(markdownContent, { breaks: false });
  const mainThreadDispatchMs = performance.now() - tDispatchStart;

  const res = await renderPromise;
  const workerParseMs = res.workerDurationMs ?? 0;

  const tSanitizeStart = performance.now();
  const cleanHtml = purifyInstance.sanitize(res.html, DOMPURIFY_CONFIG);
  const mainThreadSanitizeMs = performance.now() - tSanitizeStart;

  const totalMs = workerParseMs + mainThreadSanitizeMs;

  return {
    mainThreadDispatchMs,
    workerParseMs,
    parseMs: workerParseMs,
    mainThreadSanitizeMs,
    sanitizeMs: mainThreadSanitizeMs,
    totalMs,
    rawHtmlLength: res.html.length,
    cleanHtmlLength: cleanHtml.length,
    seq: res.seq,
    isStale: res.isStale,
  };
}

/**
 * Run a multi-iteration off-thread benchmark on a fixture document with warm-up runs.
 */
export async function benchmarkOffThreadFixture(
  fixtureName: string,
  markdownContent: string,
  iterations = 5,
  warmupRuns = 1,
  coordinator = new RenderCoordinator(),
  purifyInstance: { sanitize: (html: string, cfg: any) => string } = DOMPurify
): Promise<OffThreadFixtureBenchmarkResult> {
  for (let w = 0; w < warmupRuns; w++) {
    await measureOffThreadRender(markdownContent, coordinator, purifyInstance);
  }

  const runs: OffThreadBenchmarkMeasurement[] = [];
  for (let i = 0; i < iterations; i++) {
    const measurement = await measureOffThreadRender(markdownContent, coordinator, purifyInstance);
    runs.push(measurement);
  }

  const dispatchTimes = runs.map((r) => r.mainThreadDispatchMs);
  const parseTimes = runs.map((r) => r.workerParseMs);
  const sanitizeTimes = runs.map((r) => r.mainThreadSanitizeMs);
  const totalTimes = runs.map((r) => r.totalMs);

  return {
    fixtureName,
    wordCount: countWords(markdownContent),
    byteSize: new TextEncoder().encode(markdownContent).length,
    iterations,
    avgMainThreadDispatchMs: average(dispatchTimes),
    medianMainThreadDispatchMs: median(dispatchTimes),
    avgWorkerParseMs: average(parseTimes),
    medianWorkerParseMs: median(parseTimes),
    avgSanitizeMs: average(sanitizeTimes),
    medianSanitizeMs: median(sanitizeTimes),
    avgTotalMs: average(totalTimes),
    medianTotalMs: median(totalTimes),
  };
}
