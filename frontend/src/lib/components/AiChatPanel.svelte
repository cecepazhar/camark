<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getAiSettings, chatWithAi, type AiSettings } from '$lib/api/ai';
  import { showToast } from '$lib/stores/uiNotifications.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let inputPrompt = $state('');
  let isMinimized = $state(false);
  let isFullHeight = $state(false);
  let messages = $state<{ role: 'user' | 'assistant'; content: string }[]>([]);

  $effect(() => {
    if (messages.length === 0) {
      messages = [{ role: 'assistant', content: t('ai.greeting') }];
    } else if (messages.length === 1 && messages[0].role === 'assistant') {
      messages[0].content = t('ai.greeting');
    }
  });
  let isSending = $state(false);

  async function handleSend() {
    if (!inputPrompt.trim() || isSending) return;
    const prompt = inputPrompt.trim();
    messages.push({ role: 'user', content: prompt });
    inputPrompt = '';
    isSending = true;

    try {
      const response = await chatWithAi(prompt);
      messages.push({ role: 'assistant', content: response.message });
    } catch (e: any) {
      const errStr = typeof e === 'string' ? e : (e?.message || e?.code || t('ai.checkSettings'));
      showToast(errStr, 'error');
      messages.push({
        role: 'assistant',
        content: `${t('ai.errorMessage', { error: errStr })}`
      });
    } finally {
      isSending = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  {#if isMinimized}
    <!-- Floating collapsed pill so it never blocks the editor workspace -->
    <div class="fixed bottom-4 right-4 z-50 animate-in fade-in duration-150">
      <button
        onclick={() => (isMinimized = false)}
        class="flex items-center gap-2 px-3.5 py-2 rounded-full bg-rose-600 hover:bg-rose-500 text-white shadow-xl shadow-rose-600/30 font-medium text-xs border border-rose-400/30 transition-transform hover:scale-105 cursor-pointer"
      >
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
        <span>{t('ai.activeStatus')}</span>
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
        </svg>
      </button>
    </div>
  {:else}
    <!-- Floating non-destructive AI Card overlay in the bottom right corner -->
    <div
      class="fixed z-50 flex flex-col bg-neutral-900/95 backdrop-blur-xl border border-neutral-700/80 rounded-2xl shadow-2xl shadow-black/80 overflow-hidden transition-all duration-200 {isFullHeight
        ? 'top-[calc(3.5rem+env(safe-area-inset-top,0px))] bottom-3 right-3 w-[420px]'
        : 'bottom-4 right-4 w-[390px] h-[520px] max-h-[calc(100vh-4.5rem-env(safe-area-inset-top,0px))]'}"
    >
      <!-- Panel Header -->
      <div class="px-4 py-3 border-b border-neutral-800/80 bg-neutral-950/60 flex items-center justify-between select-none">
        <div class="flex items-center gap-2">
          <div class="w-2.5 h-2.5 rounded-full bg-rose-500 shadow-sm shadow-rose-500/50"></div>
          <h3 class="text-xs font-bold text-white tracking-wide flex items-center gap-1.5">
            <span>Hana AI</span>
            <span class="text-xs">🌸</span>
          </h3>
        </div>

        <div class="flex items-center gap-1">
          <!-- Minimize to compact bubble button -->
          <button
            type="button"
            onclick={() => (isMinimized = true)}
            class="p-1 text-neutral-400 hover:text-white rounded-lg hover:bg-neutral-800 transition-colors"
            title={t('ai.minimizeBubble')}
            aria-label={t('ai.minimize')}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <!-- Toggle full-height / floating card button -->
          <button
            type="button"
            onclick={() => (isFullHeight = !isFullHeight)}
            class="p-1 text-neutral-400 hover:text-white rounded-lg hover:bg-neutral-800 transition-colors"
            title={isFullHeight ? t('ai.floatingCardMode') : t('ai.fullScreenMode')}
            aria-label={t('ai.resize')}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              {#if isFullHeight}
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
              {:else}
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3" />
              {/if}
            </svg>
          </button>

          <!-- Close Panel button -->
          <button
            type="button"
            onclick={onClose}
            class="p-1 text-neutral-400 hover:text-rose-400 rounded-lg hover:bg-neutral-800 transition-colors"
            title={t('ai.closeAssistant')}
            aria-label={t('common.close')}
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Messages Stream -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3 text-xs leading-relaxed">
        {#each messages as msg}
          <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
            <div
              class="max-w-[88%] rounded-2xl px-3.5 py-2.5 {msg.role === 'user'
                ? 'bg-indigo-600 text-white rounded-br-none shadow-md shadow-indigo-600/20'
                : 'bg-neutral-800/90 text-neutral-200 rounded-bl-none border border-neutral-700/60'}"
            >
              {msg.content}
            </div>
          </div>
        {/each}
        {#if isSending}
          <div class="flex items-center gap-2 text-neutral-400 text-xs py-2">
            <svg class="w-3.5 h-3.5 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>{t('ai.processing')}</span>
          </div>
        {/if}
      </div>

      <!-- Input Bar -->
      <div class="p-3 border-t border-neutral-800/80 bg-neutral-950/80">
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleSend();
          }}
          class="flex gap-2"
        >
          <input
            type="text"
            bind:value={inputPrompt}
            placeholder={t('ai.inputPlaceholder')}
            class="flex-1 px-3.5 py-2 bg-neutral-900 border border-neutral-700/70 rounded-xl text-xs text-white placeholder-neutral-500 focus:outline-none focus:border-indigo-500 transition-colors"
          />
          <button
            type="submit"
            disabled={isSending || !inputPrompt.trim()}
            class="px-3.5 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white text-xs font-semibold rounded-xl transition-all shadow-md shadow-indigo-600/20 shrink-0"
          >
            {t('ai.send')}
          </button>
        </form>
      </div>
    </div>
  {/if}
{/if}
