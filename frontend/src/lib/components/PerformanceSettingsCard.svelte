<script lang="ts">
  // Settings → Performance. Every control is backed by `performance.json` through the
  // Rust prefs commands; memory figures come from the running process tree.
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPerformance, updatePerformancePrefs } from '$lib/stores/performance.svelte';
  import { clearWebCaches, getMemoryUsage, type MemoryUsage, type PerformancePrefs } from '$lib/api/performance';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { relaunch } from '@tauri-apps/plugin-process';

  const CARD =
    'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-xs dark:shadow-none text-neutral-900 dark:text-white';
  const ROW = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950 flex items-start justify-between gap-6';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const HINT = 'text-xs text-neutral-400 dark:text-neutral-500 mt-2';

  const LARGE_DOC_OPTIONS = [0, 10_000, 20_000, 50_000] as const;

  const perf = getPerformance();
  let saving = $state(false);
  let gpuChangedThisSession = $state(false);
  let memory = $state<MemoryUsage | null>(null);
  let memoryError = $state(false);
  let clearing = $state(false);

  async function save(patch: Partial<PerformancePrefs>) {
    if (saving) return;
    saving = true;
    try {
      await updatePerformancePrefs(patch);
      if ('gpuAcceleration' in patch) gpuChangedThisSession = !gpuChangedThisSession;
    } catch (e) {
      showToast(t('settings.performance.saveFailed', { error: String(e) }), 'error');
    } finally {
      saving = false;
    }
  }

  async function refreshMemory() {
    try {
      memory = await getMemoryUsage();
      memoryError = false;
    } catch {
      memoryError = true;
    }
  }

  async function clearCache() {
    clearing = true;
    try {
      const before = memory?.totalBytes ?? null;
      const supported = await clearWebCaches();
      if (!supported) {
        showToast(t('settings.performance.clearUnsupported'), 'info');
        return;
      }
      await refreshMemory();
      const after = memory?.totalBytes ?? null;
      showToast(
        before !== null && after !== null
          ? t('settings.performance.clearedWithDelta', { before: formatBytes(before), after: formatBytes(after) })
          : t('settings.performance.cleared'),
        'success'
      );
    } catch (e) {
      showToast(t('settings.performance.saveFailed', { error: String(e) }), 'error');
    } finally {
      clearing = false;
    }
  }

  function formatBytes(bytes: number): string {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function largeDocLabel(words: number): string {
    return words === 0
      ? t('settings.performance.largeDocOff')
      : t('settings.performance.largeDocWords', { count: words.toLocaleString() });
  }

  onMount(() => {
    void refreshMemory();
    const id = setInterval(() => void refreshMemory(), 5_000);
    return () => clearInterval(id);
  });
</script>

{#snippet toggle(checked: boolean, label: string, onToggle: () => void)}
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    disabled={saving}
    onclick={onToggle}
    class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 disabled:opacity-50 {checked
      ? 'bg-cyan-600'
      : 'bg-neutral-300 dark:bg-neutral-700'}"
  >
    <span
      class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow transition duration-200 {checked
        ? 'translate-x-5'
        : 'translate-x-0'}"
    ></span>
  </button>
{/snippet}

<div class="{CARD} space-y-5">
  <div>
    <h2 class="text-lg font-semibold">{t('settings.performance.title')}</h2>
    <p class="{MUTED} text-sm mt-0.5">{t('settings.performance.subtitle')}</p>
  </div>

  <!-- Live memory -->
  <div class={ROW}>
    <div class="min-w-0">
      <h3 class="text-sm font-semibold">{t('settings.performance.memoryTitle')}</h3>
      <p class="{MUTED} text-sm mt-1">{t('settings.performance.memoryDesc')}</p>
      <p class="text-2xl font-semibold mt-3 tabular-nums">
        {#if memory?.totalBytes != null}
          {formatBytes(memory.totalBytes)}
        {:else if memoryError || memory}
          <span class="text-base {MUTED}">{t('settings.performance.memoryUnavailable')}</span>
        {:else}
          <span class="text-base {MUTED}">…</span>
        {/if}
      </p>
      {#if memory}
        <p class={HINT}>{t('settings.performance.processCount', { count: memory.processCount })}</p>
      {/if}
    </div>
    <button
      type="button"
      onclick={clearCache}
      disabled={clearing}
      class="shrink-0 px-3.5 py-2 text-sm font-medium rounded-lg border border-neutral-300 dark:border-neutral-700 hover:border-cyan-500 hover:text-cyan-600 dark:hover:text-cyan-400 transition-colors disabled:opacity-50"
    >
      {clearing ? t('settings.performance.clearing') : t('settings.performance.clearCache')}
    </button>
  </div>

  <!-- Background memory saving -->
  <div class={ROW}>
    <div class="min-w-0">
      <h3 class="text-sm font-semibold">{t('settings.performance.backgroundTitle')}</h3>
      <p class="{MUTED} text-sm mt-1">{t('settings.performance.backgroundDesc')}</p>
      <p class={HINT}>{t('settings.performance.recommended')}</p>
    </div>
    {@render toggle(perf.prefs.backgroundMemorySaving, t('settings.performance.backgroundTitle'), () =>
      save({ backgroundMemorySaving: !perf.prefs.backgroundMemorySaving })
    )}
  </div>

  <!-- GPU acceleration -->
  <div class={ROW}>
    <div class="min-w-0">
      <h3 class="text-sm font-semibold">{t('settings.performance.gpuTitle')}</h3>
      <p class="{MUTED} text-sm mt-1">{t('settings.performance.gpuDesc')}</p>
      <p class={HINT}>{t('settings.performance.gpuHint')}</p>
      {#if gpuChangedThisSession}
        <button
          type="button"
          onclick={() => relaunch()}
          class="mt-3 px-3 py-1.5 text-xs font-semibold rounded-lg bg-cyan-600 text-white hover:bg-cyan-500 transition-colors"
        >
          {t('settings.performance.restartNow')}
        </button>
      {/if}
    </div>
    {@render toggle(perf.prefs.gpuAcceleration, t('settings.performance.gpuTitle'), () =>
      save({ gpuAcceleration: !perf.prefs.gpuAcceleration })
    )}
  </div>

  <!-- Large-document mode -->
  <div class={ROW}>
    <div class="min-w-0">
      <h3 class="text-sm font-semibold">{t('settings.performance.largeDocTitle')}</h3>
      <p class="{MUTED} text-sm mt-1">{t('settings.performance.largeDocDesc')}</p>
    </div>
    <select
      aria-label={t('settings.performance.largeDocTitle')}
      disabled={saving}
      value={perf.prefs.largeDocWords}
      onchange={(e) => save({ largeDocWords: Number((e.currentTarget as HTMLSelectElement).value) })}
      class="shrink-0 px-3 py-2 text-sm rounded-lg bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 focus:outline-none focus:border-cyan-500"
    >
      {#each LARGE_DOC_OPTIONS as words (words)}
        <option value={words}>{largeDocLabel(words)}</option>
      {/each}
    </select>
  </div>

  <!-- Low power mode -->
  <div class={ROW}>
    <div class="min-w-0">
      <h3 class="text-sm font-semibold">{t('settings.performance.lowPowerTitle')}</h3>
      <p class="{MUTED} text-sm mt-1">{t('settings.performance.lowPowerDesc')}</p>
    </div>
    {@render toggle(perf.prefs.lowPowerMode, t('settings.performance.lowPowerTitle'), () =>
      save({ lowPowerMode: !perf.prefs.lowPowerMode })
    )}
  </div>
</div>
