import { describe, it } from "vitest";
import { benchmarkFixture } from "./renderBenchmark";

import text1k from "./fixtures/document-1k.md?raw";
import text10k from "./fixtures/document-10k.md?raw";
import text50k from "./fixtures/document-50k.md?raw";

describe("P-01 Baseline Render Benchmark Runner", () => {
  it("runs full baseline performance benchmark (1k, 10k, 50k words)", async () => {
    console.log("\n========================================================================================");
    console.log("              P-01 MARKDOWN RENDER PERFORMANCE BENCHMARK (BASELINE)                     ");
    console.log("========================================================================================\n");

    const results = [
      await benchmarkFixture("1,000 words (1k)", text1k, 5, 2),
      await benchmarkFixture("10,000 words (10k)", text10k, 5, 2),
      await benchmarkFixture("50,000 words (50k)", text50k, 5, 1),
    ];

    console.log("| Fixture Document | Word Count | Byte Size | Parse Time (avg / med) | Sanitize Time (avg / med) | Total Time (avg / med) |");
    console.log("| :--- | :--- | :--- | :--- | :--- | :--- |");

    for (const res of results) {
      const parseStr = `${res.avgParseMs.toFixed(2)} ms / ${res.medianParseMs.toFixed(2)} ms`;
      const sanitizeStr = `${res.avgSanitizeMs.toFixed(2)} ms / ${res.medianSanitizeMs.toFixed(2)} ms`;
      const totalStr = `${res.avgTotalMs.toFixed(2)} ms / ${res.medianTotalMs.toFixed(2)} ms`;
      console.log(`| ${res.fixtureName.padEnd(18)} | ${String(res.wordCount).padEnd(10)} | ${String(res.byteSize).padEnd(9)} | ${parseStr.padEnd(22)} | ${sanitizeStr.padEnd(25)} | ${totalStr.padEnd(22)} |`);
    }

    console.log("\n----------------------------------------------------------------------------------------");
    console.log("Detail Iterations per Fixture:");
    for (const res of results) {
      console.log(`\n> [${res.fixtureName}] (${res.iterations} iterations)`);
      res.runs.forEach((run, idx) => {
        console.log(`  Run #${idx + 1}: Parse = ${run.parseMs.toFixed(2)} ms | Sanitize = ${run.sanitizeMs.toFixed(2)} ms | Total = ${run.totalMs.toFixed(2)} ms`);
      });
    }
    console.log("========================================================================================\n");
  }, 120000);
});
