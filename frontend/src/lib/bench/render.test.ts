import { describe, it, expect } from "vitest";
import {
  benchmarkFixture,
  benchmarkOffThreadFixture,
  countWords,
  measureSingleRender,
  measureOffThreadRender,
} from "./renderBenchmark";
import { RenderCoordinator } from "../render/renderCoordinator";

import text1k from "./fixtures/document-1k.md?raw";
import text10k from "./fixtures/document-10k.md?raw";
import text50k from "./fixtures/document-50k.md?raw";

describe("P-01 & P-02 Render Benchmark Harness and Fixtures", () => {
  it("fixtures have required word count thresholds", () => {
    const words1k = countWords(text1k);
    const words10k = countWords(text10k);
    const words50k = countWords(text50k);

    expect(words1k).toBeGreaterThanOrEqual(1000);
    expect(words10k).toBeGreaterThanOrEqual(10000);
    expect(words50k).toBeGreaterThanOrEqual(50000);
  });

  it("all fixtures contain math, code, and mermaid blocks", () => {
    for (const content of [text1k, text10k, text50k]) {
      expect(content).toContain("$");
      expect(content).toContain("$$");
      expect(content).toMatch(/```(?:rust|typescript|python)/);
      expect(content).toContain("```mermaid");
    }
  });

  it("measures single render correctly with valid parse and sanitize numbers", async () => {
    const m = await measureSingleRender(text1k);

    expect(m.parseMs).toBeGreaterThan(0);
    expect(m.sanitizeMs).toBeGreaterThan(0);
    expect(m.totalMs).toBeCloseTo(m.parseMs + m.sanitizeMs, 2);
    expect(m.rawHtmlLength).toBeGreaterThan(0);
    expect(m.cleanHtmlLength).toBeGreaterThan(0);
  });

  it("runs multi-iteration benchmark and calculates averages", async () => {
    const result = await benchmarkFixture("1k words", text1k, 3, 1);

    expect(result.fixtureName).toBe("1k words");
    expect(result.wordCount).toBeGreaterThanOrEqual(1000);
    expect(result.runs.length).toBe(3);
    expect(result.avgParseMs).toBeGreaterThan(0);
    expect(result.avgSanitizeMs).toBeGreaterThan(0);
    expect(result.avgTotalMs).toBeGreaterThan(0);
  });

  it("measures off-thread render with main-thread dispatch under 16ms (P-02 requirement)", async () => {
    const coordinator = new RenderCoordinator();
    const measurement = await measureOffThreadRender(text1k, coordinator);

    expect(measurement.mainThreadDispatchMs).toBeLessThan(16);
    expect(measurement.mainThreadSanitizeMs).toBeGreaterThan(0);
    expect(measurement.rawHtmlLength).toBeGreaterThan(0);
    expect(measurement.cleanHtmlLength).toBeGreaterThan(0);
    expect(measurement.isStale).toBe(false);
    coordinator.terminate();
  });
});
