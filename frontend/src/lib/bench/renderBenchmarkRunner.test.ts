import { describe, it, expect } from "vitest";
import {
  benchmarkFixture,
  benchmarkOffThreadFixture,
  benchmarkIncrementalFixture,
} from "./renderBenchmark";

import text1k from "./fixtures/document-1k.md?raw";
import text10k from "./fixtures/document-10k.md?raw";
import text50k from "./fixtures/document-50k.md?raw";

function simulateEditSingleBlock(src: string): string {
  return src.replace('dan ', 'dan [SUNTINGAN_P03] ');
}

describe("Render Benchmark Runner (P-01 Baseline, P-02 Off-thread, P-03 Incremental)", () => {
  it("runs full render performance benchmarks across all optimization stages", async () => {
    console.log("\n========================================================================================");
    console.log("     CAMARK RENDER BENCHMARK: P-01 BASELINE vs P-02 OFF-THREAD vs P-03 INCREMENTAL     ");
    console.log("========================================================================================\n");

    // 1. In-Thread Baseline (P-01)
    console.log("--- PART 1: P-01 IN-THREAD BASELINE (Main Thread Blocking) ---");
    const baselineResults = [
      await benchmarkFixture("1,000 words (1k)", text1k, 5, 2),
      await benchmarkFixture("10,000 words (10k)", text10k, 5, 2),
      await benchmarkFixture("50,000 words (50k)", text50k, 5, 1),
    ];

    console.log("| Fixture Document | Word Count | Byte Size | Parse Time (avg / med) | Sanitize Time (avg / med) | Total Time (avg / med) |");
    console.log("| :--- | :--- | :--- | :--- | :--- | :--- |");
    for (const res of baselineResults) {
      const parseStr = `${res.avgParseMs.toFixed(2)} ms / ${res.medianParseMs.toFixed(2)} ms`;
      const sanitizeStr = `${res.avgSanitizeMs.toFixed(2)} ms / ${res.medianSanitizeMs.toFixed(2)} ms`;
      const totalStr = `${res.avgTotalMs.toFixed(2)} ms / ${res.medianTotalMs.toFixed(2)} ms`;
      console.log(`| ${res.fixtureName.padEnd(18)} | ${String(res.wordCount).padEnd(10)} | ${String(res.byteSize).padEnd(9)} | ${parseStr.padEnd(22)} | ${sanitizeStr.padEnd(25)} | ${totalStr.padEnd(22)} |`);
    }

    // 2. Off-Thread Coordinator (P-02)
    console.log("\n--- PART 2: P-02 OFF-THREAD COORDINATOR (Typing Unblocked) ---");
    const offThreadResults = [
      await benchmarkOffThreadFixture("1,000 words (1k)", text1k, 5, 2),
      await benchmarkOffThreadFixture("10,000 words (10k)", text10k, 5, 2),
      await benchmarkOffThreadFixture("50,000 words (50k)", text50k, 5, 1),
    ];

    console.log("| Fixture Document | Word Count | Main Thread Dispatch (avg/med) | Worker Parse (avg/med) | Sanitize Time (avg/med) | Total End-to-End |");
    console.log("| :--- | :--- | :--- | :--- | :--- | :--- |");
    for (const res of offThreadResults) {
      const dispatchStr = `${res.avgMainThreadDispatchMs.toFixed(2)} ms / ${res.medianMainThreadDispatchMs.toFixed(2)} ms`;
      const parseStr = `${res.avgWorkerParseMs.toFixed(2)} ms / ${res.medianWorkerParseMs.toFixed(2)} ms`;
      const sanitizeStr = `${res.avgSanitizeMs.toFixed(2)} ms / ${res.medianSanitizeMs.toFixed(2)} ms`;
      const totalStr = `${res.avgTotalMs.toFixed(2)} ms`;
      console.log(`| ${res.fixtureName.padEnd(18)} | ${String(res.wordCount).padEnd(10)} | ${dispatchStr.padEnd(30)} | ${parseStr.padEnd(22)} | ${sanitizeStr.padEnd(23)} | ${totalStr.padEnd(16)} |`);
    }

    // 3. Incremental Render (P-03)
    console.log("\n--- PART 3: P-03 INCREMENTAL BLOCK CACHING (Single-Paragraph Edit Re-render) ---");
    const incResults = [
      await benchmarkIncrementalFixture("1,000 words (1k)", text1k, simulateEditSingleBlock(text1k), 5),
      await benchmarkIncrementalFixture("10,000 words (10k)", text10k, simulateEditSingleBlock(text10k), 5),
      await benchmarkIncrementalFixture("50,000 words (50k)", text50k, simulateEditSingleBlock(text50k), 5),
    ];

    console.log("| Fixture Document | Total Blocks | Re-rendered / Cached | Parse Time (avg/med) | Sanitize Time (avg/med) | Total Re-render Time |");
    console.log("| :--- | :--- | :--- | :--- | :--- | :--- |");
    for (const res of incResults) {
      const ratioStr = `${res.reRenderedBlocks} / ${res.cachedBlocks}`;
      const parseStr = `${res.avgParseMs.toFixed(2)} ms / ${res.medianParseMs.toFixed(2)} ms`;
      const sanitizeStr = `${res.avgSanitizeMs.toFixed(2)} ms / ${res.medianSanitizeMs.toFixed(2)} ms`;
      const totalStr = `${res.avgTotalMs.toFixed(2)} ms / ${res.medianTotalMs.toFixed(2)} ms`;
      console.log(`| ${res.fixtureName.padEnd(18)} | ${String(res.totalBlocks).padEnd(12)} | ${ratioStr.padEnd(20)} | ${parseStr.padEnd(20)} | ${sanitizeStr.padEnd(23)} | ${totalStr.padEnd(20)} |`);
    }

    // Evidence Assertions
    const result50kP02 = offThreadResults.find((r) => r.fixtureName.includes("50,000"));
    expect(result50kP02).toBeDefined();
    expect(result50kP02!.avgMainThreadDispatchMs).toBeLessThan(16);

    const result50kP03 = incResults.find((r) => r.fixtureName.includes("50,000"));
    expect(result50kP03).toBeDefined();
    expect(result50kP03!.reRenderedBlocks).toBe(1);
    expect(result50kP03!.cachedBlocks).toBeGreaterThanOrEqual(result50kP03!.totalBlocks - 1);
    console.log(`\n[P-03 Verification] 50k Fixture Incremental Re-render: exactly ${result50kP03!.reRenderedBlocks} block re-rendered, ${result50kP03!.cachedBlocks} cached in ${result50kP03!.avgTotalMs.toFixed(2)} ms (down from ~2,300 ms baseline)`);

    console.log("========================================================================================\n");
  }, 180000);
});
