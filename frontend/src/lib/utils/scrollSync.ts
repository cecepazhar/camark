/**
 * Line-Based Synchronized Scrolling (CAMark P-04).
 *
 * Replaces naive ratio-based scroll synchronization with an exact
 * source-line -> preview-block map using `data-line` / `data-line-start` attributes.
 *
 * Ensures accurate scrolling alignment even when documents contain tall blocks
 * (e.g. 1,000px diagrams, images, or large tables) above the viewport.
 */

export interface BlockLineMapEntry {
  blockId: string;
  startLine: number;
  endLine: number;
  top: number;
  bottom: number;
  height: number;
  element?: HTMLElement;
}

/**
 * Builds a sorted map of rendered preview blocks with their line numbers and vertical bounds.
 * @param container The element containing rendered blocks (or the preview element itself).
 * @param scrollContainer Optional scrollable parent container for accurate relative coordinate calculation.
 */
export function buildBlockMap(
  container: HTMLElement | null | undefined,
  scrollContainer?: HTMLElement | null
): BlockLineMapEntry[] {
  if (!container) return [];

  // Query all blocks that carry line information
  const blockElements = Array.from(
    container.querySelectorAll<HTMLElement>('[data-line], [data-line-start], .preview-block')
  );

  const entries: BlockLineMapEntry[] = [];
  const sc = scrollContainer || container;
  const containerRect = typeof sc.getBoundingClientRect === 'function' ? sc.getBoundingClientRect() : null;
  const containerScrollTop = sc.scrollTop || 0;

  for (let i = 0; i < blockElements.length; i++) {
    const el = blockElements[i];
    const lineAttr = el.getAttribute('data-line') || el.getAttribute('data-line-start');
    if (!lineAttr) continue;

    const startLine = parseInt(lineAttr, 10);
    if (isNaN(startLine)) continue;

    const endLineAttr = el.getAttribute('data-line-end');
    const endLine = endLineAttr ? parseInt(endLineAttr, 10) : startLine;
    const blockId = el.getAttribute('data-block-id') || `block-${i}`;

    // Calculate vertical offset relative to container's scroll content
    let top = el.offsetTop ?? 0;
    let height = el.offsetHeight ?? 0;

    // Use getBoundingClientRect for layout-accurate offset relative to container scroll context
    if (containerRect && (containerRect.height > 0 || containerRect.top > 0) && typeof el.getBoundingClientRect === 'function') {
      const elRect = el.getBoundingClientRect();
      if (elRect.height > 0 || elRect.top !== 0) {
        top = elRect.top - containerRect.top + containerScrollTop;
        height = elRect.height;
      }
    }

    const bottom = top + height;

    entries.push({
      blockId,
      startLine,
      endLine: isNaN(endLine) ? startLine : endLine,
      top,
      bottom,
      height,
      element: el,
    });
  }

  // Sort strictly by startLine ascending
  entries.sort((a, b) => a.startLine - b.startLine);
  return entries;
}

/**
 * Maps a source line number in the editor to the exact vertical scroll position in the preview.
 */
export function lineBlockToPreviewScroll(
  line: number,
  blockMap: BlockLineMapEntry[],
  previewScrollHeight: number,
  previewClientHeight: number
): number {
  const maxScroll = Math.max(0, previewScrollHeight - previewClientHeight);
  if (line <= 1 || blockMap.length === 0) return 0;

  // 1. Check if line is before the first block
  if (line < blockMap[0].startLine) {
    return 0;
  }

  // 2. Check if line is on or past the last block
  const lastBlock = blockMap[blockMap.length - 1];
  if (line >= lastBlock.endLine) {
    return Math.min(maxScroll, lastBlock.top);
  }

  // 3. Search block map
  for (let i = 0; i < blockMap.length; i++) {
    const b = blockMap[i];

    // Inside block [startLine, endLine]
    if (line >= b.startLine && line <= b.endLine) {
      const lineSpan = Math.max(1, b.endLine - b.startLine + 1);
      const fraction = (line - b.startLine) / lineSpan;
      const target = b.top + fraction * b.height;
      return Math.min(maxScroll, Math.max(0, target));
    }

    // Between current block and next block (e.g. empty lines between blocks)
    if (i < blockMap.length - 1) {
      const nextB = blockMap[i + 1];
      if (line > b.endLine && line < nextB.startLine) {
        const gapLines = Math.max(1, nextB.startLine - b.endLine);
        const fraction = (line - b.endLine) / gapLines;
        const target = b.bottom + fraction * (nextB.top - b.bottom);
        return Math.min(maxScroll, Math.max(0, target));
      }
    }
  }

  return 0;
}

/**
 * Maps a vertical scroll position in the preview back to the corresponding editor line number.
 */
export function previewScrollToLine(
  scrollTop: number,
  blockMap: BlockLineMapEntry[],
  totalLines: number
): number {
  if (scrollTop <= 0 || blockMap.length === 0) return 1;

  // 1. If scroll is before first block top
  if (scrollTop <= blockMap[0].top) {
    return blockMap[0].startLine;
  }

  // 2. If scroll is past or at last block bottom
  const lastBlock = blockMap[blockMap.length - 1];
  if (scrollTop >= lastBlock.bottom) {
    return Math.min(totalLines, lastBlock.endLine);
  }

  // 3. Search block map
  for (let i = 0; i < blockMap.length; i++) {
    const b = blockMap[i];

    // Inside block bounds
    if (scrollTop >= b.top && scrollTop <= b.bottom) {
      const height = Math.max(1, b.height);
      const fraction = (scrollTop - b.top) / height;
      const lineSpan = Math.max(1, b.endLine - b.startLine + 1);
      const calculatedLine = Math.round(b.startLine + fraction * lineSpan);
      return Math.min(totalLines, Math.max(1, calculatedLine));
    }

    // Between current block and next block
    if (i < blockMap.length - 1) {
      const nextB = blockMap[i + 1];
      if (scrollTop > b.bottom && scrollTop < nextB.top) {
        const gapHeight = Math.max(1, nextB.top - b.bottom);
        const fraction = (scrollTop - b.bottom) / gapHeight;
        const gapLines = Math.max(1, nextB.startLine - b.endLine);
        const calculatedLine = Math.round(b.endLine + fraction * gapLines);
        return Math.min(totalLines, Math.max(1, calculatedLine));
      }
    }
  }

  return Math.min(totalLines, lastBlock.endLine);
}
