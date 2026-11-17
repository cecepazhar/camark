import { describe, it, expect, beforeEach } from 'vitest';
import {
  splitMarkdownBlocks,
  BlockRenderCache,
  patchDomPreview,
} from './incrementalRender';
import { getKatexCacheStats, clearKatexCache, renderMarkdownToHtml } from '../utils/markdownRender';

describe('Incremental Render Engine (P-03)', () => {
  let cache: BlockRenderCache;

  beforeEach(() => {
    cache = new BlockRenderCache();
    clearKatexCache();
  });

  it('splits markdown into top-level blocks with exact line ranges', () => {
    const doc = [
      '---',
      'title: Test Doc',
      '---',
      '',
      '# Heading 1',
      '',
      'First paragraph with multiple',
      'lines of text.',
      '',
      '```rust',
      'fn main() {',
      '',
      '    println!("inside code block");',
      '}',
      '```',
      '',
      '$$',
      '\\sum_{i=1}^n i = \\frac{n(n+1)}{2}',
      '$$',
      '',
      '> [!NOTE]',
      '> Callout line 1',
      '> Callout line 2',
      '',
      '- Item 1',
      '- Item 2',
      '',
      'Final paragraph.',
    ].join('\n');

    const blocks = splitMarkdownBlocks(doc);

    expect(blocks.length).toBe(8);
    expect(blocks[0].type).toBe('frontmatter');
    expect(blocks[0].startLine).toBe(1);
    expect(blocks[0].endLine).toBe(3);

    expect(blocks[1].type).toBe('heading');
    expect(blocks[1].startLine).toBe(5);

    expect(blocks[2].type).toBe('paragraph');
    expect(blocks[2].startLine).toBe(7);
    expect(blocks[2].endLine).toBe(8);

    expect(blocks[3].type).toBe('code');
    expect(blocks[3].startLine).toBe(10);
    expect(blocks[3].endLine).toBe(15);
    expect(blocks[3].raw).toContain('inside code block');

    expect(blocks[4].type).toBe('math');
    expect(blocks[4].startLine).toBe(17);
    expect(blocks[4].endLine).toBe(19);

    expect(blocks[5].type).toBe('callout');
    expect(blocks[5].startLine).toBe(21);

    expect(blocks[6].type).toBe('list');
    expect(blocks[6].startLine).toBe(25);

    expect(blocks[7].type).toBe('paragraph');
    expect(blocks[7].startLine).toBe(28);
  });

  it('P-03 EVIDENCE: editing one paragraph in a 500-block document re-renders exactly one block', async () => {
    // 1. Build a 500-block document (paragraphs, headings, formulas, and code blocks)
    const blockList: string[] = [];
    for (let i = 0; i < 500; i++) {
      if (i % 10 === 0) {
        blockList.push(`## Section Heading ${i}`);
      } else if (i % 15 === 0) {
        blockList.push(`\`\`\`typescript\nconst blockId = ${i};\nconsole.log(blockId);\n\`\`\``);
      } else if (i % 25 === 0) {
        blockList.push(`$$\\int_0^{${i}} x^2 dx$$`);
      } else {
        blockList.push(`This is content paragraph number ${i}. It discusses distributed system state and invariant properties.`);
      }
    }
    const doc500 = blockList.join('\n\n');

    // 2. Initial render: all 500 blocks must be rendered
    const res1 = await cache.renderDocumentIncremental(doc500);
    expect(res1.totalBlocks).toBe(500);
    expect(res1.reRenderedBlocks).toBe(500);
    expect(res1.cachedBlocks).toBe(0);
    expect(res1.blocks.length).toBe(500);

    // Mount initial DOM container
    const container = document.createElement('div');
    const patch1 = patchDomPreview(container, res1.blocks);
    expect(patch1.insertedCount).toBe(500);
    expect(patch1.updatedCount).toBe(0);
    expect(patch1.reusedCount).toBe(0);
    expect(container.children.length).toBe(500);

    // 3. Edit exactly ONE paragraph in the middle (block #250)
    const editedBlockList = [...blockList];
    const originalBlock250 = editedBlockList[250];
    editedBlockList[250] = `This is content paragraph number 250 with EDITED and MODIFIED text.`;
    const editedDoc500 = editedBlockList.join('\n\n');

    // 4. Incremental render on edited document
    const res2 = await cache.renderDocumentIncremental(editedDoc500);

    // CRITICAL ASSERTION: Exactly 1 block re-rendered, 499 served from cache!
    expect(res2.totalBlocks).toBe(500);
    expect(res2.reRenderedBlocks).toBe(1);
    expect(res2.cachedBlocks).toBe(499);

    // 5. DOM Patching assertion: exactly 1 DOM node updated, 499 nodes preserved untouched!
    const patch2 = patchDomPreview(container, res2.blocks);
    expect(patch2.updatedCount).toBe(1);
    expect(patch2.reusedCount).toBe(499);
    expect(patch2.insertedCount).toBe(0);
    expect(patch2.removedCount).toBe(0);

    // Verify the specific patched node content
    const patchedChild = container.children[250] as HTMLElement;
    expect(patchedChild.innerHTML).toContain('EDITED and MODIFIED');
    expect(patchedChild.getAttribute('data-block-hash')).toBe(res2.blocks[250].hash);

    // Verify unchanged sibling nodes are completely intact
    const siblingChild = container.children[249] as HTMLElement;
    expect(siblingChild.getAttribute('data-block-hash')).toBe(res1.blocks[249].hash);
  });

  it('caches KaTeX output by source hash so unchanged formulas do not re-render', async () => {
    expect(getKatexCacheStats().size).toBe(0);

    const docWithMath = 'Formula 1: $E=mc^2$\n\n$$\\frac{a}{b} = c$$\n\nFormula 2: $E=mc^2$';
    await renderMarkdownToHtml(docWithMath);

    // KaTeX cache should now have entries
    const initialStats = getKatexCacheStats();
    expect(initialStats.size).toBeGreaterThan(0);

    // Render again with same formulas
    await renderMarkdownToHtml(docWithMath);

    // Size should not grow because identical source expressions hit cache
    expect(getKatexCacheStats().size).toBe(initialStats.size);
  });

  it('DOM patcher correctly adds and removes block elements when block count changes', async () => {
    const docA = '# Title\n\nParagraph 1\n\nParagraph 2';
    const docB = '# Title\n\nParagraph 1\n\nParagraph 1.5\n\nParagraph 2\n\nParagraph 3';

    const resA = await cache.renderDocumentIncremental(docA);
    const container = document.createElement('div');
    patchDomPreview(container, resA.blocks);
    expect(container.children.length).toBe(3);

    const resB = await cache.renderDocumentIncremental(docB);
    const patchB = patchDomPreview(container, resB.blocks);
    expect(container.children.length).toBe(5);
    expect(patchB.insertedCount).toBe(2);

    const docC = '# Title';
    const resC = await cache.renderDocumentIncremental(docC);
    const patchC = patchDomPreview(container, resC.blocks);
    expect(container.children.length).toBe(1);
    expect(patchC.removedCount).toBe(4);
  });
});
