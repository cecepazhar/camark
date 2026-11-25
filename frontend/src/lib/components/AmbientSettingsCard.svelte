<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getAmbientStore } from '$lib/stores/ambient.svelte';
  import { getPro } from '$lib/stores/pro.svelte';

  let { defaultAccent = '#06b6d4', appName = 'CAMark' }: { defaultAccent?: string; appName?: string } = $props();

  const ambient = getAmbientStore();
  const pro = getPro();

  const PRESET_COLORS = [
    { name: 'Cyan (Default)', hex: defaultAccent },
    { name: 'Sky Blue', hex: '#0284c7' },
    { name: 'Violet', hex: '#8b5cf6' },
    { name: 'Emerald', hex: '#10b981' },
    { name: 'Amber Gold', hex: '#f59e0b' },
    { name: 'Rose', hex: '#f43f5e' },
    { name: 'Neon Purple', hex: '#d946ef' },
    { name: 'Pure White', hex: '#ffffff' }
  ];
</script>

<div class="p-6 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 shadow-sm space-y-6">
  <!-- Header: Toggle & Pro Status Indicator -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-2 mb-1">
        <h3 class="text-base font-semibold text-neutral-900 dark:text-white">
          Ambient Underglow & Chroma
        </h3>
        <span class="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-gradient-to-r from-amber-500 to-rose-500 text-white shadow-sm">
          PRO EXCLUSIVE
        </span>
        {#if !pro.isPro}
          <span class="text-xs text-neutral-400 italic">{t('ui.ambient.requiresPro')}</span>
        {/if}
      </div>
      <p class="text-xs text-neutral-500 dark:text-neutral-400 max-w-xl">
        {t('ui.ambient.desc')}
      </p>
    </div>

    <div class="flex items-center gap-3 shrink-0 self-start sm:self-center">
      <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">
        {ambient.config.enabled && pro.isPro ? 'Aktif' : 'Nonaktif'}
      </span>
      <button
        type="button"
        role="switch"
        aria-label="Toggle Ambient Underglow"
        aria-checked={ambient.config.enabled}
        disabled={!pro.isPro}
        onclick={() => ambient.setEnabled(!ambient.config.enabled)}
        class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none disabled:opacity-40 disabled:cursor-not-allowed {ambient.config.enabled && pro.isPro ? 'bg-cyan-600' : 'bg-neutral-300 dark:bg-neutral-700'}"
      >
        <span
          class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {ambient.config.enabled && pro.isPro ? 'translate-x-5' : 'translate-x-0'}"
        ></span>
      </button>
    </div>
  </div>

  {#if !pro.isPro}
    <!-- Locked state callout -->
    <div class="p-4 rounded-xl border border-amber-500/20 bg-amber-500/5 flex items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-500 shrink-0">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <div>
          <p class="text-xs font-semibold text-amber-600 dark:text-amber-400">{t('ui.ambient.exclusive')}</p>
          <p class="text-[11px] text-neutral-500 dark:text-neutral-400">{t('ui.ambient.unlock', { app: appName })}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if ambient.config.enabled && pro.isPro}
    <div class="pt-4 border-t border-neutral-200 dark:border-neutral-800 space-y-5">
      
      <!-- 1. Color Mode Selection -->
      <div class="space-y-2.5">
        <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          {t('ui.ambient.colorMode')}
        </span>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
          <button
            type="button"
            onclick={() => ambient.setMode('app-accent')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'app-accent' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full" style:background-color={defaultAccent}></span>
              <span class="text-xs font-semibold">App Brand</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">{t('ui.ambient.brandDesc', { app: appName })}</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('solid')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'solid' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full border border-black/20" style:background-color={ambient.config.customColor}></span>
              <span class="text-xs font-semibold">Solid Hex</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">{t('ui.ambient.solidDesc')}</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('rgb-cycle')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'rgb-cycle' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full bg-[linear-gradient(45deg,#06b6d4,#8b5cf6,#ec4899)]"></span>
              <span class="text-xs font-semibold">RGB Chroma</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">{t('ui.ambient.rgbDesc')}</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('aurora')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'aurora' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full bg-[radial-gradient(circle,#06b6d4,#8b5cf6)]"></span>
              <span class="text-xs font-semibold">Aurora Drift</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">{t('ui.ambient.auroraDesc')}</p>
          </button>
        </div>
      </div>

      <!-- Solid Color Picker & Presets (Shown when mode == 'solid') -->
      {#if ambient.config.mode === 'solid'}
        <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-950 space-y-3">
          <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
            {t('ui.ambient.palette')}
          </span>
          <div class="flex flex-wrap items-center gap-2">
            {#each PRESET_COLORS as preset}
              <button
                type="button"
                onclick={() => ambient.setCustomColor(preset.hex)}
                class="w-7 h-7 rounded-full border-2 transition-transform hover:scale-110 flex items-center justify-center {ambient.config.customColor === preset.hex ? 'border-white ring-2 ring-cyan-500 scale-110' : 'border-black/20'}"
                style:background-color={preset.hex}
                title={preset.name}
              >
                {#if ambient.config.customColor === preset.hex}
                  <svg class="w-3.5 h-3.5 {preset.hex === '#ffffff' ? 'text-black' : 'text-white'}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                {/if}
              </button>
            {/each}

            <!-- Native Color Input -->
            <div class="flex items-center gap-2 ml-2 pl-2 border-l border-neutral-300 dark:border-neutral-700">
              <input
                type="color"
                aria-label="Custom color picker"
                value={ambient.config.customColor}
                oninput={(e) => ambient.setCustomColor((e.target as HTMLInputElement).value)}
                class="w-8 h-8 rounded-lg cursor-pointer bg-transparent border-0"
              />
              <span class="text-xs font-mono font-semibold uppercase text-neutral-600 dark:text-neutral-300">
                {ambient.config.customColor}
              </span>
            </div>
          </div>
        </div>
      {/if}

      <!-- 2. Animation Effect Selection -->
      <div class="space-y-2.5">
        <span class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          {t('ui.ambient.animation')}
        </span>
        <div class="grid grid-cols-3 gap-2.5">
          <button
            type="button"
            onclick={() => ambient.setEffect('static')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'static' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <span class="text-xs">{t('ui.ambient.static')}</span>
          </button>

          <button
            type="button"
            onclick={() => ambient.setEffect('breathing')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'breathing' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <span class="text-xs">{t('ui.ambient.breathing')}</span>
          </button>

          <button
            type="button"
            onclick={() => ambient.setEffect('wave')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'wave' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <span class="text-xs">Wave / Flowing</span>
          </button>
        </div>
      </div>

      <!-- 3. Sliders: Brightness & Spread Blur Radius -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
        <div class="space-y-1.5">
          <div class="flex justify-between text-xs text-neutral-600 dark:text-neutral-400">
            <span>{t('ui.ambient.intensity')}</span>
            <span class="font-mono font-semibold">{Math.round(ambient.config.intensity * 100)}%</span>
          </div>
          <input
            type="range"
            min="0.1"
            max="1.0"
            step="0.05"
            aria-label={t('ui.ambient.intensity')}
            value={ambient.config.intensity}
            oninput={(e) => ambient.setIntensity(parseFloat((e.target as HTMLInputElement).value))}
            class="w-full accent-cyan-600"
          />
        </div>

        <div class="space-y-1.5">
          <div class="flex justify-between text-xs text-neutral-600 dark:text-neutral-400">
            <span>Spread Blur Radius</span>
            <span class="font-mono font-semibold">{ambient.config.blurRadius}px</span>
          </div>
          <input
            type="range"
            min="8"
            max="50"
            step="2"
            aria-label="Spread Blur Radius"
            value={ambient.config.blurRadius}
            oninput={(e) => ambient.setBlurRadius(parseInt((e.target as HTMLInputElement).value))}
            class="w-full accent-cyan-600"
          />
        </div>

        <div class="space-y-1.5">
          <div class="flex justify-between text-xs text-neutral-600 dark:text-neutral-400">
            <span>{t('ui.ambient.speed')}</span>
            <span class="font-mono font-semibold">{ambient.config.speedSec}s</span>
          </div>
          <input
            type="range"
            min="2"
            max="15"
            step="1"
            aria-label={t('ui.ambient.speed')}
            value={ambient.config.speedSec}
            oninput={(e) => ambient.setSpeedSec(parseFloat((e.target as HTMLInputElement).value))}
            class="w-full accent-cyan-600"
          />
        </div>
      </div>

      <!-- 4. Avatar Profile Halo Effect (PRO) -->
      <div class="space-y-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
                Avatar Profile Halo Effect (PRO)
              </span>
              <span class="text-[9px] font-bold px-1.5 py-0.2 rounded bg-cyan-500/10 text-cyan-500 border border-cyan-500/20 uppercase">
                Aura GPU
              </span>
            </div>
            <p class="text-[11px] text-neutral-500 dark:text-neutral-400">
              {t('ui.ambient.avatarDesc')}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <span class="text-xs text-neutral-500 dark:text-neutral-400">
              {ambient.config.avatarGlowEnabled ? 'Aktif' : 'Nonaktif'}
            </span>
            <button
              type="button"
              role="switch"
              aria-label="Toggle Avatar Glow Effect"
              aria-checked={ambient.config.avatarGlowEnabled}
              onclick={() => ambient.setAvatarGlowEnabled(!ambient.config.avatarGlowEnabled)}
              class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {ambient.config.avatarGlowEnabled ? 'bg-cyan-600' : 'bg-neutral-300 dark:bg-neutral-700'}"
            >
              <span
                class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {ambient.config.avatarGlowEnabled ? 'translate-x-4' : 'translate-x-0'}"
              ></span>
            </button>
          </div>
        </div>

        {#if ambient.config.avatarGlowEnabled}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-1">
            <!-- 1. Comet Beam -->
            <button
              type="button"
              onclick={() => ambient.setAvatarGlowEffect('comet-beam')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.avatarGlowEffect === 'comet-beam' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Conic Comet Beam</span>
                <!-- Mini Preview -->
                <div class="relative w-8 h-8 rounded-full bg-neutral-900 border border-neutral-700 flex items-center justify-center shrink-0 overflow-visible">
                  <div class="absolute -inset-1 rounded-full animate-spin [animation-duration:3s] [background:conic-gradient(from_0deg,transparent_0deg,transparent_240deg,#06b6d4_360deg)] [mask:radial-gradient(closest-side,transparent_65%,black_70%)] [-webkit-mask:radial-gradient(closest-side,transparent_65%,black_70%)]"></div>
                  <div class="w-4 h-4 rounded-full bg-cyan-500/20 text-cyan-400 flex items-center justify-center text-[8px] font-bold">1</div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.cometDesc')}
              </p>
            </button>

            <!-- 2. Dual Photons -->
            <button
              type="button"
              onclick={() => ambient.setAvatarGlowEffect('dual-photons')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.avatarGlowEffect === 'dual-photons' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Dual Photons</span>
                <!-- Mini Preview -->
                <div class="relative w-8 h-8 rounded-full bg-neutral-900 border border-neutral-700 flex items-center justify-center shrink-0 overflow-visible">
                  <div class="absolute -inset-1 rounded-full animate-spin [animation-duration:2.5s] [background:conic-gradient(from_0deg,transparent_0deg,transparent_140deg,#06b6d4_180deg,transparent_180deg,transparent_320deg,#c084fc_360deg)] [mask:radial-gradient(closest-side,transparent_65%,black_70%)] [-webkit-mask:radial-gradient(closest-side,transparent_65%,black_70%)]"></div>
                  <div class="w-4 h-4 rounded-full bg-purple-500/20 text-purple-400 flex items-center justify-center text-[8px] font-bold">2</div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.photonsDesc')}
              </p>
            </button>

            <!-- 3. Reactive Chroma Ring -->
            <button
              type="button"
              onclick={() => ambient.setAvatarGlowEffect('chroma-ring')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.avatarGlowEffect === 'chroma-ring' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Reactive Chroma</span>
                <!-- Mini Preview -->
                <div class="relative w-8 h-8 rounded-full bg-neutral-900 border border-neutral-700 flex items-center justify-center shrink-0 overflow-visible">
                  <div class="absolute -inset-1 rounded-full animate-spin [animation-duration:4s] [background:conic-gradient(from_0deg,#ff0055,#33cc33,#0099ff,#ff0055)] blur-[1px]"></div>
                  <div class="w-4 h-4 rounded-full bg-neutral-900 z-10 flex items-center justify-center text-[8px] font-bold text-neutral-300">3</div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.reactiveDesc')}
              </p>
            </button>
          </div>
        {/if}
      </div>

      <!-- 5. Work Area Card Outer Glow (PRO) -->
      <div class="space-y-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
                Work Area Card Outer Glow (PRO)
              </span>
              <span class="text-[9px] font-bold px-1.5 py-0.2 rounded bg-cyan-500/10 text-cyan-500 border border-cyan-500/20 uppercase">
                Card Aura
              </span>
            </div>
            <p class="text-[11px] text-neutral-500 dark:text-neutral-400">
              {t('ui.ambient.cardDesc')}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <span class="text-xs text-neutral-500 dark:text-neutral-400">
              {ambient.config.cardGlowEnabled ? 'Aktif' : 'Nonaktif'}
            </span>
            <button
              type="button"
              role="switch"
              aria-label="Toggle Card Glow Effect"
              aria-checked={ambient.config.cardGlowEnabled}
              onclick={() => ambient.setCardGlowEnabled(!ambient.config.cardGlowEnabled)}
              class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {ambient.config.cardGlowEnabled ? 'bg-cyan-600' : 'bg-neutral-300 dark:bg-neutral-700'}"
            >
              <span
                class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {ambient.config.cardGlowEnabled ? 'translate-x-4' : 'translate-x-0'}"
              ></span>
            </button>
          </div>
        </div>

        {#if ambient.config.cardGlowEnabled}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-1">
            <!-- 1. Diffused Halo -->
            <button
              type="button"
              onclick={() => ambient.setCardGlowStyle('diffused-halo')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.cardGlowStyle === 'diffused-halo' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Diffused Halo</span>
                <!-- Mini Preview -->
                <div class="relative w-10 h-7 bg-neutral-900 border border-neutral-800 rounded-br-sm overflow-visible flex items-end justify-end p-1">
                  <div class="absolute -top-1.5 -left-1.5 w-6 h-6 rounded-tl-lg bg-cyan-400/40 blur-[4px]"></div>
                  <div class="w-full h-full rounded-tl-md border-t border-l border-cyan-400/60 bg-neutral-800/80"></div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.diffusedDesc')}
              </p>
            </button>

            <!-- 2. Neon Hairline -->
            <button
              type="button"
              onclick={() => ambient.setCardGlowStyle('neon-border')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.cardGlowStyle === 'neon-border' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Neon Hairline</span>
                <!-- Mini Preview -->
                <div class="relative w-10 h-7 bg-neutral-900 border border-neutral-800 rounded-br-sm overflow-visible flex items-end justify-end p-1">
                  <div class="w-full h-full rounded-tl-md border-t-2 border-l-2 border-cyan-400 shadow-[0_0_8px_#06b6d4] bg-neutral-800/80"></div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.hairlineDesc')}
              </p>
            </button>

            <!-- 3. Chroma Border Beam -->
            <button
              type="button"
              onclick={() => ambient.setCardGlowStyle('chroma-beam')}
              class="p-3.5 rounded-xl border text-left transition-all relative flex flex-col justify-between gap-3 {ambient.config.cardGlowStyle === 'chroma-beam' ? 'border-cyan-500 bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/50' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold">Chroma Border Beam</span>
                <!-- Mini Preview -->
                <div class="relative w-10 h-7 bg-neutral-900 border border-neutral-800 rounded-br-sm overflow-hidden flex items-end justify-end p-1">
                  <div class="absolute -inset-2 bg-[conic-gradient(from_0deg,transparent_0deg,transparent_220deg,#06b6d4_360deg)] animate-spin [animation-duration:3s]"></div>
                  <div class="relative w-full h-full rounded-tl-md bg-neutral-900 border-t border-l border-cyan-400/40"></div>
                </div>
              </div>
              <p class="text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">
                {t('ui.ambient.beamDesc')}
              </p>
            </button>
          </div>
        {/if}
      </div>

    </div>
  {/if}
</div>
