<script lang="ts">
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import katex from 'katex';
  import { onMount, onDestroy } from 'svelte';
  import { getPro } from '$lib/stores/pro.svelte';

  interface Props {
    isOpen: boolean;
    rawMarkdown: string;
    docTitle: string;
    onClose: () => void;
  }

  let { isOpen, rawMarkdown, docTitle, onClose }: Props = $props();

  const pro = getPro();
  const effectiveOpen = $derived(isOpen && pro.isPro);

  let currentSlideIndex = $state(0);
  let slides = $derived.by(() => {
    // Split markdown by horizontal rule '---' or lines with only '---' or '***'
    const rawSlides = rawMarkdown
      .split(/\n(?:---|\*\*\*)\n/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    if (rawSlides.length === 0) {
      return ['# ' + docTitle + '\n\nDokumen kosong.'];
    }
    return rawSlides;
  });

  let totalSlides = $derived(slides.length);
  let renderedSlideHtml = $state('');

  $effect(() => {
    if (isOpen && slides.length > 0) {
      const current = slides[currentSlideIndex] || slides[0];
      renderCurrentSlide(current);
    }
  });

  async function renderCurrentSlide(slideMd: string) {
    try {
      // Math KaTeX
      let processed = slideMd.replace(/\$\$([\s\S]+?)\$\$/g, (_, math) => {
        try {
          return katex.renderToString(math, { displayMode: true, throwOnError: false });
        } catch {
          return `$$${math}$$`;
        }
      });
      processed = processed.replace(/\$([^\$\n]+?)\$/g, (_, math) => {
        try {
          return katex.renderToString(math, { displayMode: false, throwOnError: false });
        } catch {
          return `$${math}$`;
        }
      });

      const rawHtml = await marked.parse(processed, { gfm: true, breaks: true });
      renderedSlideHtml = DOMPurify.sanitize(rawHtml, {
        ADD_TAGS: ['span', 'div', 'svg', 'path'],
        ADD_ATTR: ['class', 'style'],
      });
    } catch (e) {
      renderedSlideHtml = `<p>Gagal merender slide: ${e}</p>`;
    }
  }

  function nextSlide() {
    if (currentSlideIndex < totalSlides - 1) {
      currentSlideIndex++;
    }
  }

  function prevSlide() {
    if (currentSlideIndex > 0) {
      currentSlideIndex--;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!effectiveOpen) return;
    if (e.key === 'ArrowRight' || e.key === ' ' || e.key === 'PageDown') {
      e.preventDefault();
      nextSlide();
    } else if (e.key === 'ArrowLeft' || e.key === 'PageUp') {
      e.preventDefault();
      prevSlide();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'Home') {
      e.preventDefault();
      currentSlideIndex = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      currentSlideIndex = totalSlides - 1;
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if effectiveOpen}
  <div
    class="fixed inset-0 z-50 bg-[#0A0A0C] text-[#EDEDED] flex flex-col justify-between p-6 sm:p-12 select-none font-sans animate-in fade-in zoom-in-95 duration-200"
  >
    <!-- Top Header Bar -->
    <header class="flex items-center justify-between text-xs text-neutral-400 border-b border-[#262626] pb-3 shrink-0">
      <div class="flex items-center gap-3">
        <span class="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-400 font-bold border border-cyan-500/30 uppercase text-[10px] tracking-wider">
          PRO PRESENTATION
        </span>
        <span class="font-medium text-white truncate max-w-md">{docTitle}</span>
      </div>

      <div class="flex items-center gap-3">
        <span class="font-mono text-xs text-neutral-400">
          Slide <strong class="text-white text-sm">{currentSlideIndex + 1}</strong> / {totalSlides}
        </span>
        <button
          type="button"
          onclick={onClose}
          class="px-2.5 py-1 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-white text-xs font-semibold transition-colors"
          title="Keluar (Esc)"
        >
          Tutup (Esc)
        </button>
      </div>
    </header>

    <!-- Main Presentation Slide Canvas -->
    <main
      class="flex-1 my-6 flex items-center justify-center p-8 sm:p-16 max-w-5xl mx-auto w-full bg-[#121217] border border-[#262626] rounded-2xl shadow-2xl overflow-y-auto relative bg-[linear-gradient(to_right,#ffffff05_1px,transparent_1px),linear-gradient(to_bottom,#ffffff05_1px,transparent_1px)] bg-[size:48px_48px]"
    >
      <div
        class="pro-presentation-content w-full text-center space-y-6 max-w-4xl"
      >
        {@html renderedSlideHtml}
      </div>
    </main>

    <!-- Footer Controls -->
    <footer class="flex items-center justify-between pt-2 border-t border-[#262626] text-xs shrink-0">
      <div class="flex items-center gap-2">
        <button
          type="button"
          disabled={currentSlideIndex === 0}
          onclick={prevSlide}
          class="px-4 py-2 rounded-xl bg-[#18181F] hover:bg-[#262626] text-white font-medium border border-[#262626] transition-colors disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <span>◀</span>
          <span>Sebelumnya</span>
        </button>
        <button
          type="button"
          disabled={currentSlideIndex === totalSlides - 1}
          onclick={nextSlide}
          class="px-5 py-2 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold transition-all disabled:opacity-30 disabled:cursor-not-allowed shadow-md shadow-cyan-600/30 flex items-center gap-2"
        >
          <span>Berikutnya</span>
          <span>▶</span>
        </button>
      </div>

      <!-- Slide Dots Navigation -->
      <div class="hidden md:flex items-center gap-1.5 max-w-md overflow-x-auto scrollbar-none px-2 py-1">
        {#each slides as _, idx}
          <button
            type="button"
            onclick={() => (currentSlideIndex = idx)}
            class="h-2 rounded-full transition-all {currentSlideIndex === idx ? 'w-6 bg-cyan-400' : 'w-2 bg-[#262626] hover:bg-neutral-600'}"
            title="Ke Slide {idx + 1}"
          ></button>
        {/each}
      </div>

      <div class="text-[11px] text-neutral-500">
        Navigasi: <kbd class="px-1.5 py-0.5 rounded bg-[#18181F] border border-[#262626]">←</kbd> <kbd class="px-1.5 py-0.5 rounded bg-[#18181F] border border-[#262626]">→</kbd> atau <kbd class="px-1.5 py-0.5 rounded bg-[#18181F] border border-[#262626]">Spasi</kbd>
      </div>
    </footer>
  </div>
{/if}

<style>
  :global(.pro-presentation-content h1) {
    font-size: 2.5rem;
    font-weight: 800;
    color: #ffffff;
    line-height: 1.2;
    margin-bottom: 1.5rem;
  }
  :global(.pro-presentation-content h2) {
    font-size: 2rem;
    font-weight: 700;
    color: #22d3ee;
    line-height: 1.3;
    margin-bottom: 1rem;
  }
  :global(.pro-presentation-content h3) {
    font-size: 1.5rem;
    font-weight: 600;
    color: #38bdf8;
    margin-bottom: 0.75rem;
  }
  :global(.pro-presentation-content p) {
    font-size: 1.25rem;
    line-height: 1.7;
    color: #d1d5db;
  }
  :global(.pro-presentation-content ul) {
    text-align: left;
    display: inline-block;
    font-size: 1.25rem;
    line-height: 2;
    color: #e5e7eb;
    margin: 1rem auto;
  }
  :global(.pro-presentation-content li) {
    margin-bottom: 0.5rem;
  }
  :global(.pro-presentation-content blockquote) {
    border-left: 4px solid #22d3ee;
    padding-left: 1.5rem;
    font-style: italic;
    color: #9ca3af;
    font-size: 1.25rem;
    margin: 1.5rem auto;
    text-align: left;
  }
  :global(.pro-presentation-content table) {
    margin: 1.5rem auto;
    border-collapse: collapse;
    font-size: 1.1rem;
  }
  :global(.pro-presentation-content th),
  :global(.pro-presentation-content td) {
    border: 1px solid #374151;
    padding: 0.75rem 1.25rem;
  }
  :global(.pro-presentation-content th) {
    background: rgba(34, 211, 238, 0.1);
    color: #22d3ee;
  }
</style>
