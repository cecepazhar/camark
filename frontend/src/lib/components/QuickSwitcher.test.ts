import { describe, it, expect, vi, afterEach, beforeAll } from 'vitest';
import { setLocale } from '$lib/i18n/index.svelte';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import QuickSwitcher from './QuickSwitcher.svelte';

// The assertions below use the Indonesian UI strings.
beforeAll(() => setLocale('id'));

describe('Quick Switcher Ctrl+O (F3-05 / S-01)', () => {
  afterEach(() => {
    cleanup();
  });

  const items = [
    { name: 'catatan-lama.md', path: '/workspace/catatan-lama.md' },
    { name: 'bab1-pendahuluan.md', path: '/workspace/skripsi/bab1-pendahuluan.md' },
    { name: 'arsip-tugas.md', path: '/workspace/kuliah/arsip-tugas.md' },
  ];

  const recent = ['/workspace/skripsi/bab1-pendahuluan.md'];

  it('ranks recent files at the top when query is empty', () => {
    const onPick = vi.fn();
    const onClose = vi.fn();
    const { getAllByRole } = render(QuickSwitcher, {
      props: { open: true, items, recent, onPick, onClose },
    });

    const buttons = getAllByRole('button');
    expect(buttons[0].textContent).toContain('bab1-pendahuluan.md');
  });

  it('filters fuzzily by name and path', async () => {
    const onPick = vi.fn();
    const onClose = vi.fn();
    const { getByPlaceholderText, getAllByRole } = render(QuickSwitcher, {
      props: { open: true, items, recent, onPick, onClose },
    });

    const input = getByPlaceholderText(/Cari catatan/i);
    await fireEvent.input(input, { target: { value: 'skrip' } });

    const buttons = getAllByRole('button');
    expect(buttons).toHaveLength(1);
    expect(buttons[0].textContent).toContain('bab1-pendahuluan.md');
  });

  it('picks top result on Enter keypress', async () => {
    const onPick = vi.fn();
    const onClose = vi.fn();
    const { getByPlaceholderText } = render(QuickSwitcher, {
      props: { open: true, items, recent, onPick, onClose },
    });

    const input = getByPlaceholderText(/Cari catatan/i);
    await fireEvent.input(input, { target: { value: 'lama' } });
    await fireEvent.keyDown(input, { key: 'Enter' });

    expect(onPick).toHaveBeenCalledWith(items[0]);
  });

  it('triggers onClose when Escape is pressed', async () => {
    const onPick = vi.fn();
    const onClose = vi.fn();
    const { getByPlaceholderText } = render(QuickSwitcher, {
      props: { open: true, items, recent, onPick, onClose },
    });

    const input = getByPlaceholderText(/Cari catatan/i);
    await fireEvent.keyDown(input, { key: 'Escape' });

    expect(onClose).toHaveBeenCalled();
  });
});
