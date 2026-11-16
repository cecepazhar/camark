import { describe, it, expect } from "vitest";
import {
  benchmarkFixture,
  benchmarkOffThreadFixture,
} from "./renderBenchmark";

import text1k from "./fixtures/document-1k.md?raw";
import text10k from "./fixtures/document-10k.md?raw";
import text50k from "./fixtures/document-50k.md?raw";

describe("Render Benchmark Runner (P-01 Baseline & P-02 Off-thread)", () => {
  it("runs full render performance benchmarks (in-thread vs off-thread)", async () => {
    console.log("\n========================================================================================");
    console.log("            CAMARK RENDER PERFORMANCE BENCHMARK: P-01 BASELINE VS P-02 OFF-THREAD        ");
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

    // 3. Evidence Assertion: Main thread typing latency must be under 16ms (one frame) on 50k fixture
    const result50k = offThreadResults.find((r) => r.fixtureName.includes("50,000"));
    expect(result50k).toBeDefined();
    console.log(`\n[P-02 Verification] 50k Fixture Main Thread Dispatch Blocking Duration: ${result50k!.avgMainThreadDispatchMs.toFixed(2)} ms (< 16 ms target)`);
    expect(result50k!.avgMainThreadDispatchMs).toBeLessThan(16);
    expect(result50k!.medianMainThreadDispatchMs).toBeLessThan(16);

    console.log("========================================================================================\n");
  }, 180000);
});
