<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { generateToc } from '$lib/utils/markdownTools';
  import { getPro } from '$lib/stores/pro.svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface Props {
    selectedText: string;
    onApplyResult: (result: string) => void;
    onClose: () => void;
  }

  let { selectedText, onApplyResult, onClose }: Props = $props();

  const pro = getPro();
  let isProcessing = $state(false);
  let activeAction = $state<string | null>(null);

  async function handleAction(actionKey: 'toc' | 'summary' | 'humanize' | 'table') {
    if (!selectedText.trim()) return;

    // Fast client-side TOC if selectedText has headings
    if (actionKey === 'toc') {
      const toc = generateToc(selectedText);
      onApplyResult(`${toc}\n\n${selectedText}`);
      showToast('Daftar Isi (TOC) berhasil dibuat!', 'success');
      onClose();
      return;
    }

    if (!pro.isPro) {
      showToast(t('pro.aiExclusive'), 'error');
      return;
    }

    isProcessing = true;
    activeAction = actionKey;
    try {
      const res = await invoke<string>('ai_copilot_action', {
        action: actionKey,
        selectedText: selectedText.trim(),
      });
      onApplyResult(res);
      showToast('AI Copilot berhasil memperbarui teks!', 'success');
      onClose();
    } catch (e: any) {
      const errStr = typeof e === 'string' ? e : (e?.message || 'Gagal memproses AI Action');
      showToast(`Gagal memproses AI Action: ${errStr}`, 'error');
    } finally {
      isProcessing = false;
      activeAction = null;
    }
  }
</script>

{#if selectedText.trim().length > 0}
  <div
    class="bg-[#18181F] border border-cyan-500/40 rounded-xl px-3 py-1.5 shadow-2xl flex items-center gap-2 select-none z-30 backdrop-blur-md animate-in fade-in slide-in-from-bottom-2 duration-150"
  >
    <div class="flex items-center gap-1.5 text-xs text-cyan-400 font-bold border-r border-[#262626] pr-2 shrink-0">
      <svg class="w-3.5 h-3.5 text-cyan-400 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2 2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z"></path>
        <rect x="4" y="8" width="16" height="12" rx="2"></rect>
      </svg>
      <span>AI Copilot</span>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-1 text-xs">
      <!-- TOC Generator -->
      <button
        type="button"
        disabled={isProcessing}
        onclick={() => handleAction('toc')}
        class="px-2 py-1 rounded-lg hover:bg-cyan-500/10 text-neutral-300 hover:text-cyan-300 transition-colors flex items-center gap-1 disabled:opacity-50"
        title="Buat Daftar Isi (TOC) dari heading yang dipilih"
      >
        <span>📑</span>
        <span>Daftar Isi</span>
      </button>

      <!-- Executive Summary -->
      <button
        type="button"
        disabled={isProcessing}
        onclick={() => handleAction('summary')}
        class="px-2 py-1 rounded-lg hover:bg-cyan-500/10 text-neutral-300 hover:text-cyan-300 transition-colors flex items-center gap-1 disabled:opacity-50"
        title="Ringkas teks terpilih menjadi poin eksekutif padat"
      >
        <span>⚡</span>
        <span>{activeAction === 'summary' ? 'Meringkas...' : 'Ringkas'}</span>
      </button>

      <!-- Grammar Humanizer -->
      <button
        type="button"
        disabled={isProcessing}
        onclick={() => handleAction('humanize')}
        class="px-2 py-1 rounded-lg hover:bg-cyan-500/10 text-neutral-300 hover:text-cyan-300 transition-colors flex items-center gap-1 disabled:opacity-50"
        title="Humanize gaya bahasa: natural, hindari filler AI robotik"
      >
        <span>✨</span>
        <span>{activeAction === 'humanize' ? 'Humanizing...' : 'Humanize'}</span>
      </button>

      <!-- Text to Table -->
      <button
        type="button"
        disabled={isProcessing}
        onclick={() => handleAction('table')}
        class="px-2 py-1 rounded-lg hover:bg-cyan-500/10 text-neutral-300 hover:text-cyan-300 transition-colors flex items-center gap-1 disabled:opacity-50"
        title="Ubah teks / daftar ini menjadi tabel Markdown rapi"
      >
        <span>📊</span>
        <span>{activeAction === 'table' ? 'Membuat Tabel...' : 'Tabelkan'}</span>
      </button>
    </div>

    <!-- Dismiss Button -->
    <button
      type="button"
      onclick={onClose}
      class="text-neutral-500 hover:text-neutral-300 p-1 rounded-md ml-1"
      title="Tutup Bar Aksi"
    >
      ✕
    </button>
  </div>
{/if}
