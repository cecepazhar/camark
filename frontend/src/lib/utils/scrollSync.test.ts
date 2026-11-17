import { describe, it, expect } from 'vitest';
import {
  buildBlockMap,
  lineBlockToPreviewScroll,
  previewScrollToLine,
  type BlockLineMapEntry,
} from './scrollSync';

describe('Line-Based Scroll Sync Engine (P-04)', () => {
  describe('buildBlockMap', () => {
    it('returns empty array when container is null or has no blocks', () => {
      expect(buildBlockMap(null)).toEqual([]);
      expect(buildBlockMap(undefined)).toEqual([]);

      const emptyDiv = document.createElement('div');
      expect(buildBlockMap(emptyDiv)).toEqual([]);
    });

    it('extracts startLine, endLine, and bounds from DOM elements', () => {
      const container = document.createElement('div');

      const b1 = document.createElement('div');
      b1.className = 'preview-block';
      b1.setAttribute('data-block-id', 'p1');
      b1.setAttribute('data-line', '1');
      b1.setAttribute('data-line-end', '4');
      Object.defineProperty(b1, 'offsetTop', { value: 0 });
      Object.defineProperty(b1, 'offsetHeight', { value: 60 });

      const b2 = document.createElement('div');
      b2.className = 'preview-block';
      b2.setAttribute('data-block-id', 'p2');
      b2.setAttribute('data-line-start', '6');
      b2.setAttribute('data-line-end', '10');
      Object.defineProperty(b2, 'offsetTop', { value: 80 });
      Object.defineProperty(b2, 'offsetHeight', { value: 120 });

      container.appendChild(b1);
      container.appendChild(b2);

      const map = buildBlockMap(container);
      expect(map.length).toBe(2);
      expect(map[0]).toMatchObject({
        blockId: 'p1',
        startLine: 1,
        endLine: 4,
        top: 0,
        bottom: 60,
        height: 60,
      });
      expect(map[1]).toMatchObject({
        blockId: 'p2',
        startLine: 6,
        endLine: 10,
        top: 80,
        bottom: 200,
        height: 120,
      });
    });
  });

  describe('lineBlockToPreviewScroll & previewScrollToLine mapping', () => {
    const fixtureMap: BlockLineMapEntry[] = [
      {
        blockId: 'intro',
        startLine: 1,
        endLine: 5,
        top: 0,
        bottom: 100,
        height: 100,
      },
      {
        // Tall block: 1,200px Mermaid diagram taking only 5 lines in markdown
        blockId: 'tall-mermaid',
        startLine: 6,
        endLine: 10,
        top: 100,
        bottom: 1300,
        height: 1200,
      },
      {
        // Target section following the tall block
        blockId: 'section-after-diagram',
        startLine: 11,
        endLine: 15,
        top: 1300,
        bottom: 1400,
        height: 100,
      },
      {
        blockId: 'conclusion',
        startLine: 16,
        endLine: 20,
        top: 1400,
        bottom: 1500,
        height: 100,
      },
    ];

    const previewScrollHeight = 2000;
    const previewClientHeight = 500;
    const totalLines = 20;

    it('maps line 1 to top (0px)', () => {
      const scroll = lineBlockToPreviewScroll(1, fixtureMap, previewScrollHeight, previewClientHeight);
      expect(scroll).toBe(0);
      const line = previewScrollToLine(0, fixtureMap, totalLines);
      expect(line).toBe(1);
    });

    it('maps intermediate lines within the first block proportionally', () => {
      // Line 3 of [1..5], span = 5 lines, fraction = (3 - 1) / 5 = 0.4 -> 0.4 * 100 = 40px
      const scroll = lineBlockToPreviewScroll(3, fixtureMap, previewScrollHeight, previewClientHeight);
      expect(scroll).toBe(40);

      const line = previewScrollToLine(40, fixtureMap, totalLines);
      expect(line).toBe(3);
    });

    it('P-04 EVIDENCE: eliminates tall-block drift (ratio sync drifts >400px, line-based sync is exact)', () => {
      // Scenario: User navigates editor to Line 11 (the section right after the 1,200px Mermaid diagram)
      const targetLine = 11;

      // 1. Calculate naive ratio-based sync:
      // Line 11 out of 20 total lines = 55%
      const lineRatio = (targetLine - 1) / (totalLines - 1); // 10/19 ≈ 0.5263
      const maxScroll = previewScrollHeight - previewClientHeight; // 1500px
      const naiveRatioScroll = lineRatio * maxScroll; // ~789.5px

      // The actual top of Line 11's block in preview is 1,300px
      const expectedBlockTop = 1300;
      const naiveDrift = Math.abs(naiveRatioScroll - expectedBlockTop);

      // Verify that naive ratio sync fails drastically (>500px drift)
      expect(naiveDrift).toBeGreaterThan(400);

      // 2. Calculate CAMark line-based sync:
      const lineBasedScroll = lineBlockToPreviewScroll(
        targetLine,
        fixtureMap,
        previewScrollHeight,
        previewClientHeight
      );

      // Line-based sync must pinpoint the exact top of the target block (1,300px)
      expect(lineBasedScroll).toBe(expectedBlockTop);
      const lineBasedDrift = Math.abs(lineBasedScroll - expectedBlockTop);
      expect(lineBasedDrift).toBe(0);

      // 3. Reverse mapping: Preview scroll at 1,300px must map back to line 11
      const mappedLine = previewScrollToLine(1300, fixtureMap, totalLines);
      expect(mappedLine).toBe(11);
    });

    it('handles gap lines smoothly between blocks', () => {
      const gappedMap: BlockLineMapEntry[] = [
        { blockId: 'b1', startLine: 1, endLine: 5, top: 0, bottom: 50, height: 50 },
        { blockId: 'b2', startLine: 10, endLine: 15, top: 150, bottom: 250, height: 100 },
      ];

      // Line 7 is in the gap between endLine 5 (bottom 50) and startLine 10 (top 150)
      const scroll = lineBlockToPreviewScroll(7, gappedMap, 1000, 300);
      expect(scroll).toBeGreaterThan(50);
      expect(scroll).toBeLessThan(150);

      // Reverse mapping
      const line = previewScrollToLine(scroll, gappedMap, 20);
      expect(line).toBe(7);
    });

    it('clamps to maxScroll when scrolling near or at the end', () => {
      const scroll = lineBlockToPreviewScroll(20, fixtureMap, previewScrollHeight, previewClientHeight);
      expect(scroll).toBeLessThanOrEqual(previewScrollHeight - previewClientHeight);
    });
  });
});
