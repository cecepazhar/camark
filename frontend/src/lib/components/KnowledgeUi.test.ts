import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';
import QuickSwitcher from './QuickSwitcher.svelte';
import KnowledgePanel from './KnowledgePanel.svelte';

afterEach(() => cleanup());

const items = [
  { name: 'prd.md', path: '/v/prd.md' },
  { name: 'project-review.md', path: '/v/project-review.md' },
  { name: 'roadmap.md', path: '/v/notes/roadmap.md' },
];

describe('QuickSwitcher', () => {
  it('filters fuzzily and opens the top hit with Enter', async () => {
    const onPick = vi.fn();
    const { getByPlaceholderText, getAllByRole } = render(QuickSwitcher, {
      props: { open: true, items, onPick, onClose: vi.fn() },
    });
    const input = getByPlaceholderText(/Cari catatan/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'rdmp' } });
    await tick();
    const rows = getAllByRole('button');
    expect(rows[0].textContent).toContain('roadmap.md');
    await fireEvent.keyDown(input, { key: 'Enter' });
    expect(onPick).toHaveBeenCalledWith(items[2]);
  });

  it('offers to create a note when nothing matches', async () => {
    const onCreate = vi.fn();
    const { getByPlaceholderText, getByText } = render(QuickSwitcher, {
      props: { open: true, items, onPick: vi.fn(), onCreate, onClose: vi.fn() },
    });
    const input = getByPlaceholderText(/Cari catatan/);
    await fireEvent.input(input, { target: { value: 'zzqq' } });
    await tick();
    expect(getByText(/Enter untuk membuat "zzqq.md"/)).toBeTruthy();
    await fireEvent.keyDown(input, { key: 'Enter' });
    expect(onCreate).toHaveBeenCalledWith('zzqq');
  });

  it('closes on Escape and renders nothing when closed', async () => {
    const onClose = vi.fn();
    const { getByPlaceholderText } = render(QuickSwitcher, { props: { open: true, items, onPick: vi.fn(), onClose } });
    await fireEvent.keyDown(getByPlaceholderText(/Cari catatan/), { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
    const closed = render(QuickSwitcher, { props: { open: false, items, onPick: vi.fn(), onClose } });
    expect(closed.container.querySelector('input')).toBeNull();
  });
});

describe('KnowledgePanel', () => {
  const bl = [{ sourcePath: '/v/task.md', sourceName: 'task.md', line: 3, snippet: 'Dari [[prd]]' }];
  const mentions = [{ sourcePath: '/v/roadmap.md', sourceName: 'roadmap.md', line: 0, snippet: 'Membahas PRD' }];
  const tags = [{ tag: 'produk', paths: ['/v/prd.md', '/v/roadmap.md'] }];

  it('lists backlinks and opens the source at the right line', async () => {
    const onOpen = vi.fn();
    const { getByText } = render(KnowledgePanel, {
      props: { open: true, noteName: 'prd.md', backlinks: bl, mentions, tags, indexing: false, onOpen, onLinkMention: vi.fn(), onClose: vi.fn() },
    });
    expect(getByText('Backlinks (1)')).toBeTruthy();
    await fireEvent.click(getByText('task.md'));
    expect(onOpen).toHaveBeenCalledWith('/v/task.md', 3);
  });

  it('turns a mention into a link and browses tags', async () => {
    const onLinkMention = vi.fn();
    const onOpen = vi.fn();
    const { getByText } = render(KnowledgePanel, {
      props: { open: true, noteName: 'prd.md', backlinks: [], mentions, tags, indexing: false, onOpen, onLinkMention, onClose: vi.fn() },
    });
    await fireEvent.click(getByText('Mentions (1)'));
    await fireEvent.click(getByText('Jadikan [[link]]'));
    expect(onLinkMention).toHaveBeenCalledWith(mentions[0]);
    await fireEvent.click(getByText('Tags (1)'));
    await fireEvent.click(getByText('#produk'));
    await fireEvent.click(getByText('roadmap.md'));
    expect(onOpen).toHaveBeenCalledWith('/v/roadmap.md', 0);
  });
});
