/**
 * Repeatable Render Benchmark Harness for CAMark (P-01).
 *
 * Measures:
 * 1. Parse time: Markdown AST + KaTeX math tokenization (renderMarkdownToHtml)
 * 2. Sanitize time: DOMPurify HTML sanitization with production allowlist
 * 3. Total time: parse time + sanitize time
 */

import { renderMarkdownToHtml } from '../utils/markdownRender';
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

export function countWords(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

function median(values: number[]): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) {
    return (sorted[mid - 1] + sorted[mid]) / 2;
  }
  return sorted[mid];
}

function average(values: number[]): number {
  if (values.length === 0) return 0;
  const sum = values.reduce((acc, v) => acc + v, 0);
  return sum / values.length;
}

/**
 * Measure a single pass of parsing and sanitizing markdown content.
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
 * Run a multi-iteration benchmark on a fixture document with warm-up runs.
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
