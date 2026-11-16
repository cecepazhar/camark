import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RenderCoordinator, type IWorkerLike } from './renderCoordinator';
import { processRenderRequest, type MarkdownWorkerRequest, type MarkdownWorkerResponse } from './markdownParser.worker';

describe('RenderCoordinator (P-02 Off-thread Parsing)', () => {
  it('increments sequence numbers monotonically for each request', async () => {
    const coordinator = new RenderCoordinator();
    const p1 = coordinator.render('# First');
    const p2 = coordinator.render('# Second');
    const p3 = coordinator.render('# Third');

    const [r1, r2, r3] = await Promise.all([p1, p2, p3]);

    expect(r1.seq).toBe(1);
    expect(r2.seq).toBe(2);
    expect(r3.seq).toBe(3);
    expect(coordinator.getStats().latestDispatchedSeq).toBe(3);
    coordinator.terminate();
  });

  it('correctly drops stale results when older responses arrive after newer ones', () => {
    // Custom mock worker where we manually control response order
    let messageHandler: ((event: MessageEvent<any>) => void) | null = null;
    const mockWorker: IWorkerLike = {
      postMessage: vi.fn(),
      addEventListener: (type: string, listener: any) => {
        if (type === 'message') messageHandler = listener;
      },
      terminate: vi.fn(),
    };

    const coordinator = new RenderCoordinator(mockWorker);

    // Simulate 3 requests dispatched
    const r1Promise = coordinator.render('Doc v1'); // seq: 1
    const r2Promise = coordinator.render('Doc v2'); // seq: 2
    const r3Promise = coordinator.render('Doc v3'); // seq: 3

    expect(coordinator.getStats().latestDispatchedSeq).toBe(3);
    expect(messageHandler).toBeTruthy();

    // Now simulate Worker completing seq 3 FIRST (e.g. quick partial parse or thread race)
    messageHandler!({
      data: {
        type: 'renderResult',
        seq: 3,
        html: '<p>Content 3</p>',
        yaml: null,
      },
    } as MessageEvent);

    // seq 3 should be accepted
    expect(coordinator.getStats().latestCompletedSeq).toBe(3);
    expect(coordinator.getStats().completedCount).toBe(1);
    expect(coordinator.getStats().droppedStaleCount).toBe(0);

    // Now simulate Worker completing seq 1 (which is older than seq 3)
    messageHandler!({
      data: {
        type: 'renderResult',
        seq: 1,
        html: '<p>Content 1 (Stale)</p>',
        yaml: null,
      },
    } as MessageEvent);

    // seq 1 MUST be dropped as stale
    expect(coordinator.getStats().latestCompletedSeq).toBe(3); // untouched
    expect(coordinator.getStats().droppedStaleCount).toBe(1);

    // Now simulate Worker completing seq 2 (which is also older than seq 3)
    messageHandler!({
      data: {
        type: 'renderResult',
        seq: 2,
        html: '<p>Content 2 (Stale)</p>',
        yaml: null,
      },
    } as MessageEvent);

    // seq 2 MUST also be dropped as stale
    expect(coordinator.getStats().latestCompletedSeq).toBe(3);
    expect(coordinator.getStats().droppedStaleCount).toBe(2);

    // Verify promise resolutions
    return Promise.all([
      r3Promise.then((res) => {
        expect(res.seq).toBe(3);
        expect(res.isStale).toBe(false);
        expect(res.html).toBe('<p>Content 3</p>');
      }),
      r1Promise.then((res) => {
        expect(res.seq).toBe(1);
        expect(res.isStale).toBe(true);
      }),
      r2Promise.then((res) => {
        expect(res.seq).toBe(2);
        expect(res.isStale).toBe(true);
      }),
    ]);
  });

  it('measures main-thread dispatch duration to be well under 16ms (one frame)', async () => {
    const coordinator = new RenderCoordinator();
    const result = await coordinator.render('# Header\n\nContent paragraph with $x=1$');

    expect(result.isStale).toBe(false);
    expect(result.html).toContain('Header');
    expect(coordinator.getStats().lastDispatchMs).toBeLessThan(16);
    coordinator.terminate();
  });

  it('handles worker errors gracefully without crashing', () => {
    let messageHandler: ((event: MessageEvent<any>) => void) | null = null;
    const mockWorker: IWorkerLike = {
      postMessage: vi.fn(),
      addEventListener: (type: string, listener: any) => {
        if (type === 'message') messageHandler = listener;
      },
      terminate: vi.fn(),
    };

    const coordinator = new RenderCoordinator(mockWorker);
    const renderPromise = coordinator.render('Bad input');

    messageHandler!({
      data: {
        type: 'renderResult',
        seq: 1,
        html: '',
        yaml: null,
        error: 'Simulated parser syntax failure',
      },
    } as MessageEvent);

    return renderPromise.then((res) => {
      expect(res.error).toBe('Simulated parser syntax failure');
      expect(res.isStale).toBe(false);
    });
  });

  it('processRenderRequest produces correct HTML and parses frontmatter and KaTeX', async () => {
    const request: MarkdownWorkerRequest = {
      type: 'render',
      seq: 42,
      content: '---\ntitle: Test Doc\n---\n\nFormula: $E=mc^2$\n\n> [!NOTE]\n> P-02 Worker verification',
      options: { breaks: false },
    };

    const response = await processRenderRequest(request);

    expect(response.type).toBe('renderResult');
    expect(response.seq).toBe(42);
    expect(response.yaml).toContain('title: Test Doc');
    expect(response.html).toContain('katex');
    expect(response.html).toContain('callout-note');
  });
});
